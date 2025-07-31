#![no_std]

/// Cross-chain escrow factory that manages multiple escrow states internally
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Symbol, contracttype};
use shared::{
    Immutables, EscrowError as Error, only_taker, only_valid_secret, only_before, only_after, uni_transfer,
    other_immutables as immutables, timelocks, Stage
};

#[contract]
pub struct EscrowFactory;

// Storage keys for factory configuration
const ADMIN: Symbol = symbol_short!("admin");
const RESCUE_DELAY: Symbol = symbol_short!("rsc_delay");

// Storage key type for individual escrow states
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowDataKey {
    // Escrow state keyed by order_hash (cross-chain consistency)
    EscrowState(BytesN<32>),
    // Escrow stage tracking keyed by order_hash
    EscrowStage(BytesN<32>),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowType {
    Source,
    Destination,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowStage {
    Created,
    Withdrawn,
    Cancelled,
    Rescued,
}

#[contractimpl]
impl EscrowFactory {
    /// Initialize factory with admin and default rescue delay
    pub fn __constructor(env: Env, admin: Address, rescue_delay: u64) {
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&RESCUE_DELAY, &rescue_delay);
    }

    /// Create source chain escrow - stores immutables and returns factory address
    pub fn create_src_escrow(
        env: Env,
        immutables: Immutables,
    ) -> Result<Address, Error> {
        // Basic validation (amounts and timelocks) - like create_dst_escrow
        Self::validate_basic_immutables(&env, &immutables)?;
        
        // Use order_hash as the unique escrow identifier (cross-chain consistency)
        let order_hash = immutables.order_hash.clone();
        
        // Check if escrow already exists (prevent duplicate order_hash)
        if env.storage().persistent().has(&EscrowDataKey::EscrowState(order_hash.clone())) {
            return Err(Error::InvalidImmutables); // Already exists
        }
        
        // Store escrow data keyed by order_hash
        env.storage().persistent().set(&EscrowDataKey::EscrowState(order_hash.clone()), &(EscrowType::Source, immutables.clone()));
        env.storage().persistent().set(&EscrowDataKey::EscrowStage(order_hash.clone()), &EscrowStage::Created);
        
        // Map addresses for cross-chain resolution (CREATE FIRST, like create_dst_escrow)
        immutables::map_evm_to_stellar(&env, immutables.maker.evm.clone(), immutables.maker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables.taker.evm.clone(), immutables.taker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables.token.evm.clone(), immutables.token.stellar.clone());
        
        // Now validate that address mappings were created successfully (like create_dst_escrow)
        Self::validate_address_mappings(&env, &immutables)?;
        
        // Store timelocks
        timelocks::store_timelocks(&env, &immutables.timelocks);
        
        env.events().publish((symbol_short!("SrcCreate"), order_hash), immutables.amount);
        Ok(env.current_contract_address())
    }

    /// Create destination chain escrow - stores immutables and returns factory address
    pub fn create_dst_escrow(
        env: Env,
        immutables: Immutables,
    ) -> Result<Address, Error> {
        // Basic validation (amounts and timelocks)
        Self::validate_basic_immutables(&env, &immutables)?;
        
        // Use order_hash as the unique escrow identifier (cross-chain consistency)
        let order_hash = immutables.order_hash.clone();
        
        // Check if escrow already exists (prevent duplicate order_hash)
        if env.storage().persistent().has(&EscrowDataKey::EscrowState(order_hash.clone())) {
            return Err(Error::InvalidImmutables); // Already exists
        }
        
        // Store escrow data keyed by order_hash
        env.storage().persistent().set(&EscrowDataKey::EscrowState(order_hash.clone()), &(EscrowType::Destination, immutables.clone()));
        env.storage().persistent().set(&EscrowDataKey::EscrowStage(order_hash.clone()), &EscrowStage::Created);
        
        // Map addresses for cross-chain resolution
        immutables::map_evm_to_stellar(&env, immutables.maker.evm.clone(), immutables.maker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables.taker.evm.clone(), immutables.taker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables.token.evm.clone(), immutables.token.stellar.clone());
        
        // Now validate that address mappings were created successfully
        Self::validate_address_mappings(&env, &immutables)?;
        
        // Store timelocks
        timelocks::store_timelocks(&env, &immutables.timelocks);
        
        env.events().publish((symbol_short!("DstCreate"), order_hash), immutables.amount);
        Ok(env.current_contract_address())
    }

    /// Withdraw from escrow using secret (order_hash is the key)
    pub fn withdraw(env: Env, order_hash: BytesN<32>, secret: BytesN<32>) -> Result<(), Error> {
        let (escrow_type, immutables) = Self::get_escrow_state(env.clone(), order_hash.clone())?;
        
        // Check current stage
        let stage: EscrowStage = env.storage().persistent().get(&EscrowDataKey::EscrowStage(order_hash.clone())).unwrap_or(EscrowStage::Created);
        if stage != EscrowStage::Created {
            return Err(Error::InvalidTime);
        }
        
        // Validate secret and timing based on escrow type
        only_taker(&env, &immutables)?;
        only_valid_secret(&env, &secret, &immutables)?;
        
        match escrow_type {
            EscrowType::Source => {
                let withdraw_deadline = timelocks::get(&immutables.timelocks, &env, Stage::SrcPublicWithdrawal).map_err(|_| Error::TimeLockError)?;
                only_before(&env, withdraw_deadline)?;
            },
            EscrowType::Destination => {
                let withdraw_deadline = timelocks::get(&immutables.timelocks, &env, Stage::DstPublicWithdrawal).map_err(|_| Error::TimeLockError)?;
                only_before(&env, withdraw_deadline)?;
            }
        }
        
        // Get addresses
        let stellar_token = immutables::get_stellar_addr(&env, &immutables.token.evm)
            .ok_or(Error::AddressMappingMissing)?;
        let stellar_taker = immutables::get_stellar_addr(&env, &immutables.taker.evm)
            .ok_or(Error::AddressMappingMissing)?;
        
        // Transfer funds to taker
        uni_transfer(&env, &stellar_token, &stellar_taker, immutables.amount)?;
        uni_transfer(&env, &stellar_token, &stellar_taker, immutables.safety_deposit)?;
        
        // Update stage
        env.storage().persistent().set(&EscrowDataKey::EscrowStage(order_hash.clone()), &EscrowStage::Withdrawn);
        
        env.events().publish((symbol_short!("Withdraw"), secret), immutables.amount);
        Ok(())
    }

    /// Cancel escrow (maker only, after timelock) - order_hash is the key
    pub fn cancel(env: Env, order_hash: BytesN<32>) -> Result<(), Error> {
        let (escrow_type, immutables) = Self::get_escrow_state(env.clone(), order_hash.clone())?;
        
        // Check current stage
        let stage: EscrowStage = env.storage().persistent().get(&EscrowDataKey::EscrowStage(order_hash.clone())).unwrap_or(EscrowStage::Created);
        if stage != EscrowStage::Created {
            return Err(Error::InvalidTime);
        }
        
        // Validate maker and timing based on escrow type
        Self::only_maker(&env, &immutables)?;
        
        match escrow_type {
            EscrowType::Source => {
                let cancel_time = timelocks::get(&immutables.timelocks, &env, Stage::SrcCancellation).map_err(|_| Error::TimeLockError)?;
                only_after(&env, cancel_time)?;
            },
            EscrowType::Destination => {
                let cancel_time = timelocks::get(&immutables.timelocks, &env, Stage::DstCancellation).map_err(|_| Error::TimeLockError)?;
                only_after(&env, cancel_time)?;
            }
        }
        
        // Get addresses
        let stellar_token = immutables::get_stellar_addr(&env, &immutables.token.evm)
            .ok_or(Error::AddressMappingMissing)?;
        let stellar_maker = immutables::get_stellar_addr(&env, &immutables.maker.evm)
            .ok_or(Error::AddressMappingMissing)?;
        
        // Return funds to maker
        uni_transfer(&env, &stellar_token, &stellar_maker, immutables.amount)?;
        uni_transfer(&env, &stellar_token, &stellar_maker, immutables.safety_deposit)?;
        
        // Update stage
        env.storage().persistent().set(&EscrowDataKey::EscrowStage(order_hash.clone()), &EscrowStage::Cancelled);
        
        env.events().publish((symbol_short!("Cancelled"),), immutables.amount);
        Ok(())
    }

    /// Rescue funds (taker only, after rescue delay) - order_hash is the key
    pub fn rescue_funds(env: Env, order_hash: BytesN<32>, amount: i128) -> Result<(), Error> {
        let (_, immutables) = Self::get_escrow_state(env.clone(), order_hash.clone())?;
        
        only_taker(&env, &immutables)?;
        
        let rescue_delay: u64 = env.storage().instance().get(&RESCUE_DELAY).unwrap_or(86_400);
        let rescue_start = timelocks::rescue_start(&immutables.timelocks, &env, rescue_delay).map_err(|_| Error::TimeLockError)?;
        only_after(&env, rescue_start)?;
        
        let stellar_token = immutables::get_stellar_addr(&env, &immutables.token.evm)
            .ok_or(Error::AddressMappingMissing)?;
        let stellar_taker = immutables::get_stellar_addr(&env, &immutables.taker.evm)
            .ok_or(Error::AddressMappingMissing)?;
        
        uni_transfer(&env, &stellar_token, &stellar_taker, amount)?;
        env.events().publish((symbol_short!("FundsSave"), stellar_token), amount);
        Ok(())
    }

    /// Get escrow state by order_hash
    pub fn get_escrow_state(env: Env, order_hash: BytesN<32>) -> Result<(EscrowType, Immutables), Error> {
        env.storage().persistent().get(&EscrowDataKey::EscrowState(order_hash))
            .ok_or(Error::InvalidImmutables)
    }

    /// Get escrow stage by order_hash
    pub fn get_escrow_stage(env: Env, order_hash: BytesN<32>) -> EscrowStage {
        env.storage().persistent().get(&EscrowDataKey::EscrowStage(order_hash))
            .unwrap_or(EscrowStage::Created)
    }

    /// Admin functions
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN).unwrap()
    }

    pub fn get_rescue_delay(env: Env) -> u64 {
        env.storage().instance().get(&RESCUE_DELAY).unwrap_or(86_400)
    }

    /// Helper functions
    fn only_maker(env: &Env, immutables: &Immutables) -> Result<(), Error> {
        let _stellar_maker = immutables::get_stellar_addr(env, &immutables.maker.evm)
            .ok_or(Error::AddressMappingMissing)?;
        // TODO: Add proper caller authentication
        Ok(())
    }

    fn validate_src_immutables(env: &Env, immutables: &Immutables) -> Result<(), Error> {
        if immutables.amount <= 0 || immutables.safety_deposit <= 0 {
            return Err(Error::InvalidImmutables);
        }
        timelocks::validate_timelocks(&immutables.timelocks, env).map_err(|_| Error::TimeLockError)?;
        // Verify Stellar addresses exist
        if immutables::get_stellar_addr(env, &immutables.maker.evm).is_none() ||
           immutables::get_stellar_addr(env, &immutables.taker.evm).is_none() ||
           immutables::get_stellar_addr(env, &immutables.token.evm).is_none() {
            return Err(Error::AddressMappingMissing);
        }
        Ok(())
    }

    fn validate_basic_immutables(env: &Env, immutables: &Immutables) -> Result<(), Error> {
        if immutables.amount <= 0 || immutables.safety_deposit <= 0 {
            return Err(Error::InvalidImmutables);
        }
        timelocks::validate_timelocks(&immutables.timelocks, env).map_err(|_| Error::TimeLockError)?;
        Ok(())
    }

    fn validate_address_mappings(env: &Env, immutables: &Immutables) -> Result<(), Error> {
        // Verify Stellar addresses exist (should be available after mapping creation)
        if immutables::get_stellar_addr(env, &immutables.maker.evm).is_none() ||
           immutables::get_stellar_addr(env, &immutables.taker.evm).is_none() ||
           immutables::get_stellar_addr(env, &immutables.token.evm).is_none() {
            return Err(Error::AddressMappingMissing);
        }
        Ok(())
    }

    fn validate_dst_immutables(env: &Env, immutables: &Immutables) -> Result<(), Error> {
        Self::validate_basic_immutables(env, immutables)?;
        Self::validate_address_mappings(env, immutables)?;
        Ok(())
    }
}

mod test;
