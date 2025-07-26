#![cfg(test)]

use super::*;
use crate::timelock::{timelocks, Stage, Timelocks};
use crate::immutables::{immutables, DualAddress, Immutables};
use crate::types::TimeLockError;
use soroban_sdk::{Env, Address, BytesN};
use soroban_sdk::testutils::Address as _;

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

// ===== IMMUTABLES TESTS =====

fn create_test_dual_address(env: &Env) -> DualAddress {

    let e = Env::default();
    let evm_addr = BytesN::from_array(env, &[0x42; 20]);
    let stellar_addr = Address::generate(&e);
    DualAddress {
        evm: evm_addr,
        stellar: stellar_addr,
    }
}

fn create_test_immutables(env: &Env) -> Immutables {
    Immutables {
        order_hash: BytesN::from_array(env, &[0x01; 32]),
        hashlock: BytesN::from_array(env, &[0x02; 32]),
        maker: create_test_dual_address(env),
        taker: create_test_dual_address(env),
        token: create_test_dual_address(env),
        amount: 1000,
        safety_deposit: 100,
        timelocks: Timelocks {
            deployed_at: 1000,
            src_withdrawal: 100,
            src_public_withdrawal: 200,
            src_cancellation: 300,
            src_public_cancellation: 400,
            dst_withdrawal: 150,
            dst_public_withdrawal: 250,
            dst_cancellation: 350,
        },
    }
}

#[test]
fn test_immutables_validation_valid() {
    let env = Env::default();
    let immutables = create_test_immutables(&env);
    
    assert!(immutables::validate_amounts(&immutables).is_ok());
}

#[test]
fn test_immutables_validation_zero_amount() {
    let env = Env::default();
    let mut immutables = create_test_immutables(&env);
    immutables.amount = 0;
    
    assert_eq!(
        immutables::validate_amounts(&immutables),
        Err(TimeLockError::DeploymentTimestampNotSet)
    );
}

#[test]
fn test_immutables_validation_negative_amount() {
    let env = Env::default();
    let mut immutables = create_test_immutables(&env);
    immutables.amount = -100;
    
    assert_eq!(
        immutables::validate_amounts(&immutables),
        Err(TimeLockError::DeploymentTimestampNotSet)
    );
}

#[test]
fn test_immutables_validation_negative_safety_deposit() {
    let env = Env::default();
    let mut immutables = create_test_immutables(&env);
    immutables.safety_deposit = -50;
    
    assert_eq!(
        immutables::validate_amounts(&immutables),
        Err(TimeLockError::DeploymentTimestampNotSet)
    );
}

#[test]
fn test_immutables_hash_generation() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    env.as_contract(&contract_id, || {
        let immutables = create_test_immutables(&env);
        
        // Test that hash generation succeeds
        let hash_result = immutables::hash(&env, &immutables);
        assert!(hash_result.is_ok());
        
        let hash = hash_result.unwrap();
        // Hash should be 32 bytes
        assert_eq!(hash.to_array().len(), 32);
        
        // Test that same immutables produce same hash
        let hash2 = immutables::hash(&env, &immutables).unwrap();
        assert_eq!(hash, hash2);
    });
}

#[test]
fn test_immutables_hash_different_for_different_data() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    env.as_contract(&contract_id, || {
        let immutables1 = create_test_immutables(&env);
        let mut immutables2 = create_test_immutables(&env);
        immutables2.amount = 2000; // Different amount
        
        let hash1 = immutables::hash(&env, &immutables1).unwrap();
        let hash2 = immutables::hash(&env, &immutables2).unwrap();
        
        // Different immutables should produce different hashes
        assert_ne!(hash1, hash2);
    });
}

#[test]
fn test_immutables_hash_validation_failure() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    env.as_contract(&contract_id, || {
        let mut immutables = create_test_immutables(&env);
        immutables.amount = -100; // Invalid amount
        
        let hash_result = immutables::hash(&env, &immutables);
        assert_eq!(hash_result, Err(TimeLockError::DeploymentTimestampNotSet));
    });
}

#[test]
fn test_evm_to_stellar_address_mapping() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    env.as_contract(&contract_id, || {
        let evm_addr = BytesN::from_array(&env, &[0xab; 20]);
        let stellar_addr = Address::generate(&env);
        
        // Test mapping
        immutables::map_evm_to_stellar(&env, evm_addr.clone(), stellar_addr.clone());
        
        // Test retrieval
        let retrieved = immutables::get_stellar_addr(&env, &evm_addr);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), stellar_addr);
    });
}

#[test]
fn test_evm_to_stellar_address_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    env.as_contract(&contract_id, || {
        let evm_addr = BytesN::from_array(&env, &[0xcd; 20]);
        
        // Test retrieval of non-existent mapping
        let retrieved = immutables::get_stellar_addr(&env, &evm_addr);
        assert!(retrieved.is_none());
    });
}

#[test]
fn test_immutables_storage_and_retrieval() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    env.as_contract(&contract_id, || {
        let immutables = create_test_immutables(&env);
        
        // Test storage
        let store_result = immutables::store_immutables(&env, &immutables);
        assert!(store_result.is_ok());
        
        // Test retrieval
        let retrieved = immutables::get_immutables(&env);
        assert!(retrieved.is_some());
        
        let retrieved_immutables = retrieved.unwrap();
        assert_eq!(retrieved_immutables.amount, immutables.amount);
        assert_eq!(retrieved_immutables.safety_deposit, immutables.safety_deposit);
        assert_eq!(retrieved_immutables.order_hash, immutables.order_hash);
    });
}

#[test]
fn test_immutables_storage_validation_failure() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    env.as_contract(&contract_id, || {
        let mut immutables = create_test_immutables(&env);
        immutables.amount = -100; // Invalid amount
        
        let store_result = immutables::store_immutables(&env, &immutables);
        assert_eq!(store_result, Err(TimeLockError::DeploymentTimestampNotSet));
    });
}

#[test]
fn test_immutables_retrieval_when_empty() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    
    env.as_contract(&contract_id, || {
        // Test retrieval when nothing is stored
        let retrieved = immutables::get_immutables(&env);
        assert!(retrieved.is_none());
    });
}

#[test]
fn test_dual_address_structure() {
    let env = Env::default();
    let evm_addr = BytesN::from_array(&env, &[0xef; 20]);
    let stellar_addr = Address::generate(&env);
    
    let dual_addr = DualAddress {
        evm: evm_addr.clone(),
        stellar: stellar_addr.clone(),
    };
    
    assert_eq!(dual_addr.evm, evm_addr);
    assert_eq!(dual_addr.stellar, stellar_addr);
}

#[test]
fn test_immutables_with_zero_safety_deposit() {
    let env = Env::default();
    let mut immutables = create_test_immutables(&env);
    immutables.safety_deposit = 0; // Zero safety deposit should be valid
    
    assert!(immutables::validate_amounts(&immutables).is_ok());
}
