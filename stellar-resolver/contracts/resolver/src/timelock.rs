use crate::types::TimeLockError;

use soroban_sdk::{contracttype, Env};

// Define the Stage Enum to represent different timelock periods
#[contracttype]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Stage {
    SrcWithdrawal = 0,  // Period when only taker can withdraw on source chain
    SrcPublicWithdrawal = 1,  // Period when anyone with secret can withdraw on source chain
    SrcCancellation = 2,      // Period when only taker can cancel on source chain
    SrcPublicCancellation = 3,// Period when anyone can cancel on source chain
    DstWithdrawal = 4,        // Period when only maker can withdraw on destination chain
    DstPublicWithdrawal = 5,  // Period when anyone with secret can withdraw on destination chain
    DstCancellation = 6,      // Period when only maker can cancel on destination chain
}

// Define the Timelocks struct to store deployment timestamp and period offsets
#[contracttype]
#[derive(Clone, Debug)]
pub struct Timelocks {
    pub deployed_at: u64,              // Unix timestamp of contract deployment
    pub src_withdrawal: u32,           // Offset (seconds) from deployed_at for source chain withdrawal
    pub src_public_withdrawal: u32,    // Offset for source chain public withdrawal
    pub src_cancellation: u32,         // Offset for source chain cancellation
    pub src_public_cancellation: u32,  // Offset for source chain public cancellation
    pub dst_withdrawal: u32,           // Offset for destination chain withdrawal
    pub dst_public_withdrawal: u32,    // Offset for destination chain public withdrawal
    pub dst_cancellation: u32,         // Offset for destination chain cancellation
}

// Define storage keys for persistent storage
// Used to store and retrieve Timelocks struct in Soroban's ledger
#[contracttype]
pub enum DataKey {
    Timelocks, // Key for storing the Timelocks struct
}

// Timelocks module containing functions to manage timelock data
pub mod timelocks {
    use super::*;

    /// Sets the deployment timestamp in the Timelocks struct
    /// Equivalent to Solidity's setDeployedAt; stores updated struct in persistent storage
    /// # Arguments
    /// * `env` - Soroban environment for storage access
    /// * `timelocks` - Mutable reference to the Timelocks struct to update
    /// * `value` - New deployment timestamp (Unix timestamp)
    pub fn set_deployed_at(env: &Env, timelocks: &mut Timelocks, value: u64) {
        timelocks.deployed_at = value;
        // Persist the updated Timelocks struct to storage
        env.storage().persistent().set(&DataKey::Timelocks, timelocks);
    }

    /// Calculates the start of the rescue period
    /// Equivalent to Solidity's rescueStart; adds rescue_delay to deployed_at
    /// # Arguments
    /// * `timelocks` - Reference to the Timelocks struct
    /// * `rescue_delay` - Delay (seconds) after which funds can be rescued
    /// # Returns
    /// * `Result<u64, TimeLockError>` - The rescue period start time or an error if overflow occurs
    pub fn rescue_start(timelocks: &Timelocks, rescue_delay: u64) -> Result<u64, TimeLockError> {
        timelocks
            .deployed_at
            .checked_add(rescue_delay)
            .ok_or(TimeLockError::RescueStartOverflow)
    }

    /// Retrieves the absolute timestamp for a given stage
    /// Equivalent to Solidity's get; adds stage-specific offset to deployed_at
    /// # Arguments
    /// * `timelocks` - Reference to the Timelocks struct
    /// * `stage` - The timelock stage (e.g., SrcWithdrawal)
    /// # Returns
    /// * `Result<u64, TimeLockError>` - The stage's start time or an error if overflow occurs
    pub fn get(timelocks: &Timelocks, stage: Stage) -> Result<u64, TimeLockError> {
        // Select the appropriate offset based on the stage
        let offset = match stage {
            Stage::SrcWithdrawal => timelocks.src_withdrawal,
            Stage::SrcPublicWithdrawal => timelocks.src_public_withdrawal,
            Stage::SrcCancellation => timelocks.src_cancellation,
            Stage::SrcPublicCancellation => timelocks.src_public_cancellation,
            Stage::DstWithdrawal => timelocks.dst_withdrawal,
            Stage::DstPublicWithdrawal => timelocks.dst_public_withdrawal,
            Stage::DstCancellation => timelocks.dst_cancellation,
        };
        // Add offset to deployed_at, checking for overflow
        timelocks
            .deployed_at
            .checked_add(offset as u64)
            .ok_or(TimeLockError::TimelockValueOverflow)
    }

    /// Validates the timelocks to ensure correct ordering and non-zero periods
    /// Ensures logical progression of periods (e.g., withdrawal before cancellation)
    /// # Arguments
    /// * `timelocks` - Reference to the Timelocks struct
    /// # Returns
    /// * `Result<(), TimeLockError>` - Ok if valid, or an error if ordering is invalid
    pub fn validate_timelocks(timelocks: &Timelocks) -> Result<(), TimeLockError> {
        // Ensure deployed_at is set
        if timelocks.deployed_at == 0 {
            return Err(TimeLockError::DeploymentTimestampNotSet);
        }

        // Ensure source chain timelocks are in logical order
        if timelocks.src_withdrawal >= timelocks.src_public_withdrawal
            || timelocks.src_public_withdrawal >= timelocks.src_cancellation
            || timelocks.src_cancellation >= timelocks.src_public_cancellation
        {
            return Err(TimeLockError::InvalidSourceChainTimelockOrdering);
        }

        // Ensure destination chain timelocks are in logical order
        if timelocks.dst_withdrawal >= timelocks.dst_public_withdrawal
            || timelocks.dst_public_withdrawal >= timelocks.dst_cancellation
        {
            return Err(TimeLockError::InvalidDestinationChainTimelockOrdering);
        }

        // Note: Offset validation for u32::MAX is implicit since fields are u32 type

        Ok(())
    }

    /// Stores the Timelocks struct in persistent storage
    /// # Arguments
    /// * `env` - Soroban environment for storage access
    /// * `timelocks` - Reference to the Timelocks struct to store
    pub fn store_timelocks(env: &Env, timelocks: &Timelocks) {
        env.storage().persistent().set(&DataKey::Timelocks, timelocks);
    }

    /// Retrieves the Timelocks struct from persistent storage
    /// # Arguments
    /// * `env` - Soroban environment for storage access
    /// # Returns
    /// * `Option<Timelocks>` - The stored Timelocks struct, or None if not found
    pub fn get_timelocks(env: &Env) -> Option<Timelocks> {
        env.storage().persistent().get(&DataKey::Timelocks)
    }
}