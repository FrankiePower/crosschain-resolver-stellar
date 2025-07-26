use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, Map};

use crate::timelock::Timelocks;
use crate::types::TimeLockError;

// Dual-address struct for cross-chain compatibility
#[contracttype]
#[derive(Clone)]
pub struct DualAddress {
    pub evm: BytesN<20>,    // EVM 20-byte address
    pub stellar: Address,   // Stellar 32-byte address
}

// Updated Immutables struct
#[contracttype]
#[derive(Clone)]
pub struct Immutables {
    pub order_hash: BytesN<32>,     // keccak256 hash of fields or contract address
    pub hashlock: BytesN<32>,       // keccak256 hash of secret
    pub maker: DualAddress,         // Maker addresses (EVM and Stellar)
    pub taker: DualAddress,         // Taker addresses
    pub token: DualAddress,         // Token contract addresses
    pub amount: i128,               // Token amount (signed, validated positive)
    pub safety_deposit: i128,       // Safety deposit (signed, validated positive)
    pub timelocks: Timelocks,       // Time constraints
}

// Storage for address mappings
#[contracttype]
pub enum DataKey {
    AddressMap, // Map<BytesN<20>, Address> for EVM-to-Stellar address mapping
    ImmutablesData, // Key for storing Immutables struct
}

// Immutables module
pub mod immutables {
    use super::*;

    /// Validates that amounts are positive before processing
    pub fn validate_amounts(immutables: &Immutables) -> Result<(), TimeLockError> {
        if immutables.amount <= 0 {
            return Err(TimeLockError::DeploymentTimestampNotSet); // Using closest available error
        }
        if immutables.safety_deposit < 0 {
            return Err(TimeLockError::DeploymentTimestampNotSet); // Using closest available error
        }
        Ok(())
    }

    /// Computes keccak256 hash of Immutables to match Solidity
    pub fn hash(env: &Env, immutables: &Immutables) -> Result<BytesN<32>, TimeLockError> {
        // Validate amounts before processing
        validate_amounts(immutables)?;

        let mut bytes = Bytes::new(env);
        
        // orderHash: 32 bytes
        bytes.extend_from_array(&immutables.order_hash.to_array());
        
        // hashlock: 32 bytes
        bytes.extend_from_array(&immutables.hashlock.to_array());
        
        // maker: 20-byte EVM address, padded to 32 bytes
        let mut maker_padded = [0u8; 32];
        maker_padded[12..32].copy_from_slice(&immutables.maker.evm.to_array());
        bytes.extend_from_array(&maker_padded);
        
        // taker: 20-byte EVM address, padded to 32 bytes
        let mut taker_padded = [0u8; 32];
        taker_padded[12..32].copy_from_slice(&immutables.taker.evm.to_array());
        bytes.extend_from_array(&taker_padded);
        
        // token: 20-byte EVM address, padded to 32 bytes
        let mut token_padded = [0u8; 32];
        token_padded[12..32].copy_from_slice(&immutables.token.evm.to_array());
        bytes.extend_from_array(&token_padded);
        
        // amount: i128 as 16-byte big-endian (validated positive)
        let amount_bytes = (immutables.amount as u128).to_be_bytes();
        let mut amount_padded = [0u8; 32];
        amount_padded[16..32].copy_from_slice(&amount_bytes);
        bytes.extend_from_array(&amount_padded);
        
        // safety_deposit: i128 as 16-byte big-endian (validated non-negative)
        let deposit_bytes = (immutables.safety_deposit as u128).to_be_bytes();
        let mut deposit_padded = [0u8; 32];
        deposit_padded[16..32].copy_from_slice(&deposit_bytes);
        bytes.extend_from_array(&deposit_padded);
        
        // timelocks: 36 bytes packed (all 7 fields included)
        let timelocks_bytes = pack_timelocks(&immutables.timelocks);
        bytes.extend_from_array(&timelocks_bytes);

        Ok(env.crypto().keccak256(&bytes).into())
    }

    /// Packs Timelocks to 36 bytes to include all 7 timelock fields
    fn pack_timelocks(timelocks: &Timelocks) -> [u8; 36] {
        let mut bytes = [0u8; 36];
        
        // deployed_at: u64 (8 bytes, offset 0)
        bytes[0..8].copy_from_slice(&timelocks.deployed_at.to_be_bytes());
        
        // src_withdrawal: u32 (4 bytes, offset 8)
        bytes[8..12].copy_from_slice(&timelocks.src_withdrawal.to_be_bytes());
        
        // src_public_withdrawal: u32 (4 bytes, offset 12)
        bytes[12..16].copy_from_slice(&timelocks.src_public_withdrawal.to_be_bytes());
        
        // src_cancellation: u32 (4 bytes, offset 16)
        bytes[16..20].copy_from_slice(&timelocks.src_cancellation.to_be_bytes());
        
        // src_public_cancellation: u32 (4 bytes, offset 20)
        bytes[20..24].copy_from_slice(&timelocks.src_public_cancellation.to_be_bytes());
        
        // dst_withdrawal: u32 (4 bytes, offset 24)
        bytes[24..28].copy_from_slice(&timelocks.dst_withdrawal.to_be_bytes());
        
        // dst_public_withdrawal: u32 (4 bytes, offset 28)
        bytes[28..32].copy_from_slice(&timelocks.dst_public_withdrawal.to_be_bytes());
        
        // dst_cancellation: u32 (4 bytes, offset 32) - now included!
        bytes[32..36].copy_from_slice(&timelocks.dst_cancellation.to_be_bytes());
        
        bytes
    }

    /// Maps EVM address to Stellar address
    pub fn map_evm_to_stellar(env: &Env, evm_addr: BytesN<20>, stellar_addr: Address) {
        let mut map: Map<BytesN<20>, Address> = env
            .storage()
            .persistent()
            .get(&DataKey::AddressMap)
            .unwrap_or(Map::new(env));
        map.set(evm_addr, stellar_addr);
        env.storage().persistent().set(&DataKey::AddressMap, &map);
    }

    /// Gets Stellar address for EVM address
    pub fn get_stellar_addr(env: &Env, evm_addr: &BytesN<20>) -> Option<Address> {
        let map: Map<BytesN<20>, Address> = env
            .storage()
            .persistent()
            .get(&DataKey::AddressMap)
            .unwrap_or(Map::new(env));
        map.get(evm_addr.clone())
    }

    /// Stores Immutables in persistent storage
    pub fn store_immutables(env: &Env, immutables: &Immutables) -> Result<(), TimeLockError> {
        validate_amounts(immutables)?;
        env.storage().persistent().set(&DataKey::ImmutablesData, immutables);
        Ok(())
    }

    /// Retrieves Immutables from persistent storage
    pub fn get_immutables(env: &Env) -> Option<Immutables> {
        env.storage().persistent().get(&DataKey::ImmutablesData)
    }
}