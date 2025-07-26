#![cfg(test)]

use super::*;
use crate::timelock::{timelocks, Stage, Timelocks};
use crate::types::TimeLockError;
use soroban_sdk::Env;

#[test]
fn test_timelock_basic_functionality() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    env.as_contract(&contract_id, || {
        let mut timelocks = Timelocks {
            deployed_at: 1000,
            src_withdrawal: 100,
            src_public_withdrawal: 200,
            src_cancellation: 300,
            src_public_cancellation: 400,
            dst_withdrawal: 150,
            dst_public_withdrawal: 250,
            dst_cancellation: 350,
        };

        // Test set_deployed_at
        timelocks::set_deployed_at(&env, &mut timelocks, 2000);
        assert_eq!(timelocks.deployed_at, 2000);

        // Test get for different stages
        assert_eq!(timelocks::get(&timelocks, Stage::SrcWithdrawal).unwrap(), 2100);
        assert_eq!(timelocks::get(&timelocks, Stage::SrcPublicWithdrawal).unwrap(), 2200);
        assert_eq!(timelocks::get(&timelocks, Stage::DstWithdrawal).unwrap(), 2150);

        // Test rescue_start
        assert_eq!(timelocks::rescue_start(&timelocks, 500).unwrap(), 2500);
    });
}

#[test]
fn test_timelock_overflow_errors() {
    let timelocks = Timelocks {
        deployed_at: u64::MAX - 50,
        src_withdrawal: 100,
        src_public_withdrawal: 200,
        src_cancellation: 300,
        src_public_cancellation: 400,
        dst_withdrawal: 150,
        dst_public_withdrawal: 250,
        dst_cancellation: 350,
    };

    // Test rescue_start overflow
    assert_eq!(
        timelocks::rescue_start(&timelocks, 100),
        Err(TimeLockError::RescueStartOverflow)
    );

    // Test timelock value overflow
    assert_eq!(
        timelocks::get(&timelocks, Stage::SrcWithdrawal),
        Err(TimeLockError::TimelockValueOverflow)
    );
}

#[test]
fn test_timelock_validation_deployment_timestamp() {
    let timelocks = Timelocks {
        deployed_at: 0,
        src_withdrawal: 100,
        src_public_withdrawal: 200,
        src_cancellation: 300,
        src_public_cancellation: 400,
        dst_withdrawal: 150,
        dst_public_withdrawal: 250,
        dst_cancellation: 350,
    };

    assert_eq!(
        timelocks::validate_timelocks(&timelocks),
        Err(TimeLockError::DeploymentTimestampNotSet)
    );
}

#[test]
fn test_timelock_validation_source_chain_ordering() {
    let timelocks = Timelocks {
        deployed_at: 1000,
        src_withdrawal: 300,  // Invalid: should be < src_public_withdrawal
        src_public_withdrawal: 200,
        src_cancellation: 400,
        src_public_cancellation: 500,
        dst_withdrawal: 150,
        dst_public_withdrawal: 250,
        dst_cancellation: 350,
    };

    assert_eq!(
        timelocks::validate_timelocks(&timelocks),
        Err(TimeLockError::InvalidSourceChainTimelockOrdering)
    );
}

#[test]
fn test_timelock_validation_destination_chain_ordering() {
    let timelocks = Timelocks {
        deployed_at: 1000,
        src_withdrawal: 100,
        src_public_withdrawal: 200,
        src_cancellation: 300,
        src_public_cancellation: 400,
        dst_withdrawal: 350,  // Invalid: should be < dst_public_withdrawal
        dst_public_withdrawal: 250,
        dst_cancellation: 400,
    };

    assert_eq!(
        timelocks::validate_timelocks(&timelocks),
        Err(TimeLockError::InvalidDestinationChainTimelockOrdering)
    );
}

#[test]
fn test_timelock_validation_offset_too_large() {
    // Since the validation checks if values > u32::MAX, but the fields are already u32,
    // this check will never trigger in the current implementation.
    // The validation should pass for u32::MAX values as they're valid u32 values.
    let timelocks_with_max_values = Timelocks {
        deployed_at: 1000,
        src_withdrawal: 100,
        src_public_withdrawal: 200,
        src_cancellation: 300,
        src_public_cancellation: u32::MAX,
        dst_withdrawal: 150,
        dst_public_withdrawal: 250,
        dst_cancellation: u32::MAX,
    };

    // This should pass since u32::MAX is a valid u32 value
    assert!(timelocks::validate_timelocks(&timelocks_with_max_values).is_ok());
}

#[test]
fn test_timelock_validation_valid_configuration() {
    let timelocks = Timelocks {
        deployed_at: 1000,
        src_withdrawal: 100,
        src_public_withdrawal: 200,
        src_cancellation: 300,
        src_public_cancellation: 400,
        dst_withdrawal: 150,
        dst_public_withdrawal: 250,
        dst_cancellation: 350,
    };

    assert!(timelocks::validate_timelocks(&timelocks).is_ok());
}

#[test]
fn test_timelock_storage_functions() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    env.as_contract(&contract_id, || {
        let timelocks = Timelocks {
            deployed_at: 1000,
            src_withdrawal: 100,
            src_public_withdrawal: 200,
            src_cancellation: 300,
            src_public_cancellation: 400,
            dst_withdrawal: 150,
            dst_public_withdrawal: 250,
            dst_cancellation: 350,
        };

        // Test store and retrieve
        timelocks::store_timelocks(&env, &timelocks);
        let retrieved = timelocks::get_timelocks(&env);
        
        assert!(retrieved.is_some());
        let retrieved_timelocks = retrieved.unwrap();
        assert_eq!(retrieved_timelocks.deployed_at, 1000);
        assert_eq!(retrieved_timelocks.src_withdrawal, 100);
    });
}

#[test]
fn test_all_stage_variants() {
    let timelocks = Timelocks {
        deployed_at: 1000,
        src_withdrawal: 100,
        src_public_withdrawal: 200,
        src_cancellation: 300,
        src_public_cancellation: 400,
        dst_withdrawal: 150,
        dst_public_withdrawal: 250,
        dst_cancellation: 350,
    };

    // Test all stage variants return correct values
    assert_eq!(timelocks::get(&timelocks, Stage::SrcWithdrawal).unwrap(), 1100);
    assert_eq!(timelocks::get(&timelocks, Stage::SrcPublicWithdrawal).unwrap(), 1200);
    assert_eq!(timelocks::get(&timelocks, Stage::SrcCancellation).unwrap(), 1300);
    assert_eq!(timelocks::get(&timelocks, Stage::SrcPublicCancellation).unwrap(), 1400);
    assert_eq!(timelocks::get(&timelocks, Stage::DstWithdrawal).unwrap(), 1150);
    assert_eq!(timelocks::get(&timelocks, Stage::DstPublicWithdrawal).unwrap(), 1250);
    assert_eq!(timelocks::get(&timelocks, Stage::DstCancellation).unwrap(), 1350);
}
