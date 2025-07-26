use soroban_sdk::{contracttype, Env, U256};
use crate::types::TimeLockError;

/// Timelock stages - must match Solidity exactly
#[contracttype]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Stage {
    SrcWithdrawal = 0,
    SrcPublicWithdrawal = 1,
    SrcCancellation = 2,
    SrcPublicCancellation = 3,
    DstWithdrawal = 4,
    DstPublicWithdrawal = 5,
    DstCancellation = 6,
}

/// Timelocks - packed into single U256 value to match Solidity exactly
/// This MUST match the Solidity TimelocksLib.sol bit packing exactly
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Timelocks {
    /// Single U256 value containing all timelock data
    /// Packed exactly like Solidity's uint256 timelocks
    pub packed_value: U256,
}

impl Timelocks {
    /// Constants matching Solidity TimelocksLib.sol
    const DEPLOYED_AT_OFFSET: u32 = 224;
    
    /// Create new Timelocks with proper bit packing
    pub fn new(
        env: &Env,
        deployed_at: u32,
        src_withdrawal: u32,
        src_public_withdrawal: u32,
        src_cancellation: u32,
        src_public_cancellation: u32,
        dst_withdrawal: u32,
        dst_public_withdrawal: u32,
        dst_cancellation: u32,
    ) -> Self {
        let mut packed = U256::from_u32(env, 0);
        
        // Pack deployed_at (bits 224-255)
        let deployed_at_shifted = U256::from_u32(env, deployed_at).shl(Self::DEPLOYED_AT_OFFSET);
        packed = packed.add(&deployed_at_shifted);
        
        // Pack each timelock (32 bits each)
        let src_withdrawal_shifted = U256::from_u32(env, src_withdrawal).shl(192);
        packed = packed.add(&src_withdrawal_shifted);
        
        let src_public_withdrawal_shifted = U256::from_u32(env, src_public_withdrawal).shl(160);
        packed = packed.add(&src_public_withdrawal_shifted);
        
        let src_cancellation_shifted = U256::from_u32(env, src_cancellation).shl(128);
        packed = packed.add(&src_cancellation_shifted);
        
        let src_public_cancellation_shifted = U256::from_u32(env, src_public_cancellation).shl(96);
        packed = packed.add(&src_public_cancellation_shifted);
        
        let dst_withdrawal_shifted = U256::from_u32(env, dst_withdrawal).shl(64);
        packed = packed.add(&dst_withdrawal_shifted);
        
        let dst_public_withdrawal_shifted = U256::from_u32(env, dst_public_withdrawal).shl(32);
        packed = packed.add(&dst_public_withdrawal_shifted);
        
        let dst_cancellation_shifted = U256::from_u32(env, dst_cancellation);
        packed = packed.add(&dst_cancellation_shifted);
        
        Self { packed_value: packed }
    }
    
    /// Set deployed_at timestamp
    pub fn set_deployed_at(&mut self, env: &Env, value: u32) {
        // Create mask to clear deployed_at bits (invert the deployed_at mask)
        let deployed_at_mask = U256::from_u32(env, 0xffffffff).shl(Self::DEPLOYED_AT_OFFSET);
        let clear_mask = deployed_at_mask; // In real implementation, this would be bitwise NOT
        
        // Clear the deployed_at bits and set new value
        let new_deployed_at = U256::from_u32(env, value).shl(Self::DEPLOYED_AT_OFFSET);
        
        // For simplicity, we'll reconstruct the entire value
        // In production, you'd use proper bitwise operations
        let current_bytes = self.packed_value.to_be_bytes();
        let mut bytes_array = [0u8; 32];
        
        // Copy bytes from Soroban Bytes to array
        for i in 0..32 {
            bytes_array[i] = current_bytes.get(i as u32).unwrap_or(0);
        }
        
        // Update deployed_at in the first 4 bytes (big-endian)
        bytes_array[0..4].copy_from_slice(&value.to_be_bytes());
        
        // Convert back to U256
        let updated_bytes = soroban_sdk::Bytes::from_array(env, &bytes_array);
        self.packed_value = U256::from_be_bytes(env, &updated_bytes);
    }
    
    /// Get deployed_at timestamp (bits 224-255)
    pub fn get_deployed_at(&self, env: &Env) -> u32 {
        let bytes = self.packed_value.to_be_bytes();
        let mut deployed_at_bytes = [0u8; 4];
        
        // Extract first 4 bytes (deployed_at is in bits 224-255)
        for i in 0..4 {
            deployed_at_bytes[i] = bytes.get(i as u32).unwrap_or(0);
        }
        
        u32::from_be_bytes(deployed_at_bytes)
    }
    
    /// Get raw timelock offset for a stage (not absolute timestamp)
    pub fn get_stage_offset(&self, env: &Env, stage: Stage) -> u32 {
        let bytes = self.packed_value.to_be_bytes();
        let start_idx = match stage {
            Stage::SrcWithdrawal => 4,
            Stage::SrcPublicWithdrawal => 8,
            Stage::SrcCancellation => 12,
            Stage::SrcPublicCancellation => 16,
            Stage::DstWithdrawal => 20,
            Stage::DstPublicWithdrawal => 24,
            Stage::DstCancellation => 28,
        };
        
        let mut offset_bytes = [0u8; 4];
        for i in 0..4 {
            offset_bytes[i] = bytes.get((start_idx + i) as u32).unwrap_or(0);
        }
        
        u32::from_be_bytes(offset_bytes)
    }
    
    /// Get absolute timestamp for a stage (deployed_at + offset)
    pub fn get_stage_timestamp(&self, env: &Env, stage: Stage) -> Result<u64, TimeLockError> {
        let deployed_at = self.get_deployed_at(env) as u64;
        let offset = self.get_stage_offset(env, stage) as u64;
        
        deployed_at
            .checked_add(offset)
            .ok_or(TimeLockError::TimelockValueOverflow)
    }
    
    /// Convert to 32-byte array for EVM-compatible hashing
    pub fn to_bytes(&self, env: &Env) -> [u8; 32] {
        let bytes = self.packed_value.to_be_bytes();
        let mut result = [0u8; 32];
        
        for i in 0..32 {
            result[i] = bytes.get(i as u32).unwrap_or(0);
        }
        
        result
    }
    
    /// Create from 32-byte array (for cross-chain compatibility)
    pub fn from_bytes(env: &Env, bytes: [u8; 32]) -> Self {
        let soroban_bytes = soroban_sdk::Bytes::from_array(env, &bytes);
        Self {
            packed_value: U256::from_be_bytes(env, &soroban_bytes),
        }
    }
}

/// Timelocks module - functions for working with timelock data
pub mod timelocks {
    use super::*;
    
    /// Storage key for timelock data
    #[contracttype]
    pub enum DataKey {
        Timelocks,
    }
    
    /// Set deployed_at timestamp and persist to storage
    pub fn set_deployed_at(env: &Env, timelocks: &mut Timelocks, value: u64) {
        // Note: deployed_at is u32 in the packed format (matching Solidity)
        timelocks.set_deployed_at(env, value as u32);
        env.storage().persistent().set(&DataKey::Timelocks, timelocks);
    }
    
    /// Calculate rescue start time
    pub fn rescue_start(timelocks: &Timelocks, env: &Env, rescue_delay: u64) -> Result<u64, TimeLockError> {
        let deployed_at = timelocks.get_deployed_at(env) as u64;
        deployed_at
            .checked_add(rescue_delay)
            .ok_or(TimeLockError::RescueStartOverflow)
    }
    
    /// Get absolute timestamp for a stage
    pub fn get(timelocks: &Timelocks, env: &Env, stage: Stage) -> Result<u64, TimeLockError> {
        timelocks.get_stage_timestamp(env, stage)
    }
    
    /// Validate timelock ordering and constraints
    pub fn validate_timelocks(timelocks: &Timelocks, env: &Env) -> Result<(), TimeLockError> {
        // Ensure deployed_at is set
        if timelocks.get_deployed_at(env) == 0 {
            return Err(TimeLockError::DeploymentTimestampNotSet);
        }
        
        // Validate source chain ordering
        let src_withdrawal = timelocks.get_stage_offset(env, Stage::SrcWithdrawal);
        let src_public_withdrawal = timelocks.get_stage_offset(env, Stage::SrcPublicWithdrawal);
        let src_cancellation = timelocks.get_stage_offset(env, Stage::SrcCancellation);
        let src_public_cancellation = timelocks.get_stage_offset(env, Stage::SrcPublicCancellation);
        
        if src_withdrawal >= src_public_withdrawal
            || src_public_withdrawal >= src_cancellation
            || src_cancellation >= src_public_cancellation
        {
            return Err(TimeLockError::InvalidSourceChainTimelockOrdering);
        }
        
        // Validate destination chain ordering
        let dst_withdrawal = timelocks.get_stage_offset(env, Stage::DstWithdrawal);
        let dst_public_withdrawal = timelocks.get_stage_offset(env, Stage::DstPublicWithdrawal);
        let dst_cancellation = timelocks.get_stage_offset(env, Stage::DstCancellation);
        
        if dst_withdrawal >= dst_public_withdrawal
            || dst_public_withdrawal >= dst_cancellation
        {
            return Err(TimeLockError::InvalidDestinationChainTimelockOrdering);
        }
        
        Ok(())
    }
    
    /// Store timelocks in persistent storage
    pub fn store_timelocks(env: &Env, timelocks: &Timelocks) {
        env.storage().persistent().set(&DataKey::Timelocks, timelocks);
    }
    
    /// Retrieve timelocks from persistent storage
    pub fn get_timelocks(env: &Env) -> Option<Timelocks> {
        env.storage().persistent().get(&DataKey::Timelocks)
    }
}