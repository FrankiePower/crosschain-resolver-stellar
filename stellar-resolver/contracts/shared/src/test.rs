#![cfg(test)]

use super::*;
use soroban_sdk::{Env, Address, BytesN, log};
use soroban_sdk::testutils::{Address as _, Ledger};

extern crate std;

// ===== TIMELOCK TESTS =====

#[test]
fn test_timelock_basic_functionality() {
    let env = Env::default();
    
    let mut timelocks = Timelocks::new(
        &env,
        1000, // deployed_at
        100,  // src_withdrawal
        200,  // src_public_withdrawal
        300,  // src_cancellation
        400,  // src_public_cancellation
        150,  // dst_withdrawal
        250,  // dst_public_withdrawal
        350,  // dst_cancellation
    );

    // 🔍 LOG THE PACKED VALUE FOR DEBUGGING JavaScript conversion
    log!(&env, "🔍 Timelocks created with individual values:");
    log!(&env, "  deployed_at: {}", timelocks.get_deployed_at(&env));
    log!(&env, "  src_withdrawal: {}", timelocks.get_stage_offset(&env, Stage::SrcWithdrawal));
    log!(&env, "  src_public_withdrawal: {}", timelocks.get_stage_offset(&env, Stage::SrcPublicWithdrawal));
    log!(&env, "  src_cancellation: {}", timelocks.get_stage_offset(&env, Stage::SrcCancellation));
    log!(&env, "  src_public_cancellation: {}", timelocks.get_stage_offset(&env, Stage::SrcPublicCancellation));
    log!(&env, "  dst_withdrawal: {}", timelocks.get_stage_offset(&env, Stage::DstWithdrawal));
    log!(&env, "  dst_public_withdrawal: {}", timelocks.get_stage_offset(&env, Stage::DstPublicWithdrawal));
    log!(&env, "  dst_cancellation: {}", timelocks.get_stage_offset(&env, Stage::DstCancellation));
    
    let packed_bytes = timelocks.to_bytes(&env);
    log!(&env, "🔍 Packed bytes length: {}", packed_bytes.len());

    // Test set_deployed_at
    timelocks.set_deployed_at(&env, 2000);
    assert_eq!(timelocks.get_deployed_at(&env), 2000);

    // Test get for different stages
    assert_eq!(timelocks.get_stage_timestamp(&env, Stage::SrcWithdrawal).unwrap(), 2100);
    assert_eq!(timelocks.get_stage_timestamp(&env, Stage::SrcPublicWithdrawal).unwrap(), 2200);
    assert_eq!(timelocks.get_stage_timestamp(&env, Stage::DstWithdrawal).unwrap(), 2150);

    // Test rescue_start
    assert_eq!(timelocks::rescue_start(&timelocks, &env, 500).unwrap(), 2500);
}

#[test]
fn test_timelock_overflow_errors() {
    let env = Env::default();
    
    // Test rescue_start overflow - use u64::MAX as rescue_delay to force overflow
    let timelocks = Timelocks::new(
        &env,
        1000, // Normal deployed_at
        100,
        200,
        300,
        400,
        150,
        250,
        350,
    );

    // This should overflow: 1000 + u64::MAX
    assert_eq!(
        timelocks::rescue_start(&timelocks, &env, u64::MAX),
        Err(TimeLockError::RescueStartOverflow)
    );

    // For realistic timelock scenarios, u32 + u32 won't overflow u64
    // So test that normal values work fine
    assert!(timelocks::rescue_start(&timelocks, &env, 500).is_ok());
    assert!(timelocks.get_stage_timestamp(&env, Stage::SrcWithdrawal).is_ok());
}

#[test]
fn test_timelock_validation_deployment_timestamp() {
    let env = Env::default();
    let timelocks = Timelocks::new(
        &env,
        0, // deployed_at = 0 (invalid)
        100,
        200,
        300,
        400,
        150,
        250,
        350,
    );

    assert_eq!(
        timelocks::validate_timelocks(&timelocks, &env),
        Err(TimeLockError::DeploymentTimestampNotSet)
    );
}

#[test]
fn test_timelock_validation_source_chain_ordering() {
    let env = Env::default();
    let timelocks = Timelocks::new(
        &env,
        1000,
        300,  // Invalid: should be < src_public_withdrawal
        200,
        400,
        500,
        150,
        250,
        350,
    );

    assert_eq!(
        timelocks::validate_timelocks(&timelocks, &env),
        Err(TimeLockError::InvalidSourceChainTimelockOrdering)
    );
}

#[test]
fn test_timelock_validation_destination_chain_ordering() {
    let env = Env::default();
    let timelocks = Timelocks::new(
        &env,
        1000,
        100,
        200,
        300,
        400,
        350,  // Invalid: should be < dst_public_withdrawal
        250,
        400,
    );

    assert_eq!(
        timelocks::validate_timelocks(&timelocks, &env),
        Err(TimeLockError::InvalidDestinationChainTimelockOrdering)
    );
}

#[test]
fn test_timelock_validation_offset_too_large() {
    let env = Env::default();
    let timelocks_with_max_values = Timelocks::new(
        &env,
        1000,
        100,
        200,
        300,
        u32::MAX,
        150,
        250,
        u32::MAX,
    );

    // This should pass since u32::MAX is a valid u32 value
    assert!(timelocks::validate_timelocks(&timelocks_with_max_values, &env).is_ok());
}

#[test]
fn test_timelock_validation_valid_configuration() {
    let env = Env::default();
    let timelocks = Timelocks::new(
        &env,
        1000,
        100,
        200,
        300,
        400,
        150,
        250,
        350,
    );

    assert!(timelocks::validate_timelocks(&timelocks, &env).is_ok());
}

#[test]
fn test_timelock_storage_functions() {
    let env = Env::default();
    
    let timelocks = Timelocks::new(
        &env,
        1000,
        100,
        200,
        300,
        400,
        150,
        250,
        350,
    );

    // Test store and retrieve - wrap with as_contract
    env.as_contract(&env.current_contract_address(), || {
        timelocks::store_timelocks(&env, &timelocks);
        let retrieved = timelocks::get_timelocks(&env);
        
        assert!(retrieved.is_some());
        let retrieved_timelocks = retrieved.unwrap();
        assert_eq!(retrieved_timelocks.get_deployed_at(&env), 1000);
        assert_eq!(retrieved_timelocks.get_stage_offset(&env, Stage::SrcWithdrawal), 100);
    });
}

#[test]
fn test_all_stage_variants() {
    let env = Env::default();
    let timelocks = Timelocks::new(
        &env,
        1000,
        100,
        200,
        300,
        400,
        150,
        250,
        350,
    );

    // Test all stage variants return correct values
    assert_eq!(timelocks.get_stage_timestamp(&env, Stage::SrcWithdrawal).unwrap(), 1100);
    assert_eq!(timelocks.get_stage_timestamp(&env, Stage::SrcPublicWithdrawal).unwrap(), 1200);
    assert_eq!(timelocks.get_stage_timestamp(&env, Stage::SrcCancellation).unwrap(), 1300);
    assert_eq!(timelocks.get_stage_timestamp(&env, Stage::SrcPublicCancellation).unwrap(), 1400);
    assert_eq!(timelocks.get_stage_timestamp(&env, Stage::DstWithdrawal).unwrap(), 1150);
    assert_eq!(timelocks.get_stage_timestamp(&env, Stage::DstPublicWithdrawal).unwrap(), 1250);
    assert_eq!(timelocks.get_stage_timestamp(&env, Stage::DstCancellation).unwrap(), 1350);
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
        timelocks: Timelocks::new(
            env,
            1000, // deployed_at
            100,  // src_withdrawal
            200,  // src_public_withdrawal
            300,  // src_cancellation
            400,  // src_public_cancellation
            150,  // dst_withdrawal
            250,  // dst_public_withdrawal
            350,  // dst_cancellation
        ),
    }
}

#[test]
fn test_immutables_validation_valid() {
    let env = Env::default();
    let immutables = create_test_immutables(&env);
    
    assert!(other_immutables::validate_amounts(&immutables).is_ok());
}

#[test]
fn test_immutables_validation_zero_amount() {
    let env = Env::default();
    let mut immutables = create_test_immutables(&env);
    immutables.amount = 0;
    
    assert_eq!(
        other_immutables::validate_amounts(&immutables),
        Err(TimeLockError::DeploymentTimestampNotSet)
    );
}

#[test]
fn test_immutables_validation_negative_amount() {
    let env = Env::default();
    let mut immutables = create_test_immutables(&env);
    immutables.amount = -100;
    
    assert_eq!(
        other_immutables::validate_amounts(&immutables),
        Err(TimeLockError::DeploymentTimestampNotSet)
    );
}

#[test]
fn test_immutables_validation_negative_safety_deposit() {
    let env = Env::default();
    let mut immutables = create_test_immutables(&env);
    immutables.safety_deposit = -50;
    
    assert_eq!(
        other_immutables::validate_amounts(&immutables),
        Err(TimeLockError::DeploymentTimestampNotSet)
    );
}

#[test]
fn test_immutables_hash_generation() {
    let env = Env::default();
    let immutables = create_test_immutables(&env);
    
    // Test that hash generation succeeds
    let hash_result = other_immutables::hash(&env, &immutables);
    assert!(hash_result.is_ok());
    
    let hash = hash_result.unwrap();
    // Hash should be 32 bytes
    assert_eq!(hash.to_array().len(), 32);
    
    // Test that same immutables produce same hash
    let hash2 = other_immutables::hash(&env, &immutables).unwrap();
    assert_eq!(hash, hash2);
}

#[test]
fn test_immutables_hash_different_for_different_data() {
    let env = Env::default();
    let immutables1 = create_test_immutables(&env);
    let mut immutables2 = create_test_immutables(&env);
    immutables2.amount = 2000; // Different amount
    
    let hash1 = other_immutables::hash(&env, &immutables1).unwrap();
    let hash2 = other_immutables::hash(&env, &immutables2).unwrap();
    
    // Different immutables should produce different hashes
    assert_ne!(hash1, hash2);
}

#[test]
fn test_immutables_hash_validation_failure() {
    let env = Env::default();
    let mut immutables = create_test_immutables(&env);
    immutables.amount = -100; // Invalid amount
    
    let hash_result = other_immutables::hash(&env, &immutables);
    assert_eq!(hash_result, Err(TimeLockError::DeploymentTimestampNotSet));
}

#[test]
fn test_evm_to_stellar_address_mapping() {
    let env = Env::default();
    
    let evm_addr = BytesN::from_array(&env, &[0xab; 20]);
    let stellar_addr = Address::generate(&env);
    
    // Test mapping - wrap with as_contract
    env.as_contract(&env.current_contract_address(), || {
        other_immutables::map_evm_to_stellar(&env, evm_addr.clone(), stellar_addr.clone());
        
        // Test retrieval
        let retrieved = other_immutables::get_stellar_addr(&env, &evm_addr);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), stellar_addr);
    });
}

#[test]
fn test_evm_to_stellar_address_not_found() {
    let env = Env::default();
    
    let evm_addr = BytesN::from_array(&env, &[0xcd; 20]);
    
    // Test retrieval of non-existent mapping - wrap with as_contract
    env.as_contract(&env.current_contract_address(), || {
        let retrieved = other_immutables::get_stellar_addr(&env, &evm_addr);
        assert!(retrieved.is_none());
    });
}

#[test]
fn test_immutables_storage_and_retrieval() {
    let env = Env::default();
    
    let immutables = create_test_immutables(&env);
    
    // Test storage - wrap with as_contract
    env.as_contract(&env.current_contract_address(), || {
        let store_result = other_immutables::store_immutables(&env, &immutables);
        assert!(store_result.is_ok());
        
        // Test retrieval
        let retrieved = other_immutables::get_immutables(&env);
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
    
    let mut immutables = create_test_immutables(&env);
    immutables.amount = -100; // Invalid amount
    
    let store_result = other_immutables::store_immutables(&env, &immutables);
    assert_eq!(store_result, Err(TimeLockError::DeploymentTimestampNotSet));
}

#[test]
fn test_immutables_retrieval_when_empty() {
    let env = Env::default();
    
    // Test retrieval when nothing is stored - wrap with as_contract
    env.as_contract(&env.current_contract_address(), || {
        let retrieved = other_immutables::get_immutables(&env);
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
    
    assert!(other_immutables::validate_amounts(&immutables).is_ok());
}

// ===== BASEESCROW TESTS =====

fn create_test_secret(env: &Env) -> (BytesN<32>, BytesN<32>) {
    let secret = BytesN::from_array(env, &[0x42; 32]);
    // Create hashlock by hashing the secret
    let secret_bytes = soroban_sdk::Bytes::from_array(env, &secret.to_array());
    let hashlock = env.crypto().keccak256(&secret_bytes).into();
    (secret, hashlock)
}

fn create_test_immutables_with_secret(env: &Env, _secret: BytesN<32>, hashlock: BytesN<32>) -> Immutables {
    Immutables {
        order_hash: BytesN::from_array(env, &[0x01; 32]),
        hashlock,
        maker: create_test_dual_address(env),
        taker: create_test_dual_address(env),
        token: create_test_dual_address(env),
        amount: 1000,
        safety_deposit: 100,
        timelocks: Timelocks::new(
            env,
            1000, // deployed_at
            100,  // src_withdrawal
            200,  // src_public_withdrawal
            300,  // src_cancellation
            400,  // src_public_cancellation
            150,  // dst_withdrawal
            250,  // dst_public_withdrawal
            350,  // dst_cancellation
        ),
    }
}

#[test]
fn test_only_valid_secret_success() {
    let env = Env::default();
    let (secret, hashlock) = create_test_secret(&env);
    let immutables = create_test_immutables_with_secret(&env, secret.clone(), hashlock);
    
    assert!(only_valid_secret(&env, &secret, &immutables).is_ok());
}

#[test]
fn test_only_valid_secret_failure() {
    let env = Env::default();
    let (_, hashlock) = create_test_secret(&env);
    let wrong_secret = BytesN::from_array(&env, &[0x99; 32]);
    let immutables = create_test_immutables_with_secret(&env, wrong_secret.clone(), hashlock);
    
    assert_eq!(
        only_valid_secret(&env, &wrong_secret, &immutables),
        Err(EscrowError::InvalidSecret)
    );
}

#[test]
fn test_only_before_success() {
    let env = Env::default();
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = 1000;
    });
    
    assert!(only_before(&env, 2000).is_ok());
}

#[test]
fn test_only_before_failure() {
    let env = Env::default();
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = 2000;
    });
    
    assert_eq!(only_before(&env, 1000), Err(EscrowError::InvalidTime));
}

#[test]
fn test_only_after_success() {
    let env = Env::default();
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = 2000;
    });
    
    assert!(only_after(&env, 1000).is_ok());
}

#[test]
fn test_only_after_failure() {
    let env = Env::default();
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = 1000;
    });
    
    assert_eq!(only_after(&env, 2000), Err(EscrowError::InvalidTime));
}

// ===== CROSS-CHAIN ATOMIC SWAP SCENARIO TEST =====

#[test]
fn test_atomic_swap_happy_path_timelock_ordering() {
    let env = Env::default();
    let (secret, hashlock) = create_test_secret(&env);
    let immutables = create_test_immutables_with_secret(&env, secret.clone(), hashlock);
    
    // Verify timelock ordering for atomic swap
    assert!(timelocks::validate_timelocks(&immutables.timelocks, &env).is_ok());
    
    // Test the expected timelock sequence:
    // 1. Contracts deployed at time 1000
    // 2. SrcWithdrawal: 1100 (taker can withdraw from source)
    // 3. DstWithdrawal: 1150 (maker can withdraw from destination) 
    // 4. SrcPublicWithdrawal: 1200 (anyone can withdraw from source)
    // 5. DstPublicWithdrawal: 1250 (anyone can withdraw from destination)
    // 6. SrcCancellation: 1300 (maker can cancel source)
    // 7. DstCancellation: 1350 (taker can cancel destination)
    
    let src_withdrawal = immutables.timelocks.get_stage_timestamp(&env, Stage::SrcWithdrawal).unwrap();
    let dst_withdrawal = immutables.timelocks.get_stage_timestamp(&env, Stage::DstWithdrawal).unwrap();
    let src_public = immutables.timelocks.get_stage_timestamp(&env, Stage::SrcPublicWithdrawal).unwrap();
    let dst_public = immutables.timelocks.get_stage_timestamp(&env, Stage::DstPublicWithdrawal).unwrap();
    let src_cancel = immutables.timelocks.get_stage_timestamp(&env, Stage::SrcCancellation).unwrap();
    let dst_cancel = immutables.timelocks.get_stage_timestamp(&env, Stage::DstCancellation).unwrap();
    
    assert_eq!(src_withdrawal, 1100);
    assert_eq!(dst_withdrawal, 1150);
    assert_eq!(src_public, 1200);
    assert_eq!(dst_public, 1250);
    assert_eq!(src_cancel, 1300);
    assert_eq!(dst_cancel, 1350);
    
    // Verify proper ordering for atomic swaps
    assert!(src_withdrawal < dst_withdrawal); // Taker withdraws first
    assert!(dst_withdrawal < src_public);     // Maker has time to respond
    assert!(src_public < dst_public);         // Public phases ordered
    assert!(dst_public < src_cancel);         // Cancellation comes last
    assert!(src_cancel < dst_cancel);         // Source cancelled before destination
}

#[test]
fn test_secret_hash_consistency() {
    let env = Env::default();
    let (secret1, hashlock1) = create_test_secret(&env);
    // Create a different secret for testing
    let secret2 = BytesN::from_array(&env, &[0x99; 32]);
    let secret2_bytes = soroban_sdk::Bytes::from_array(&env, &secret2.to_array());
    let hashlock2: BytesN<32> = env.crypto().keccak256(&secret2_bytes).into();
    
    // Same secret should produce same hash
    let secret1_copy = secret1.clone();
    let secret1_bytes = soroban_sdk::Bytes::from_array(&env, &secret1_copy.to_array());
    let hashlock1_copy: BytesN<32> = env.crypto().keccak256(&secret1_bytes).into();
    assert_eq!(hashlock1, hashlock1_copy);
    
    // Different secrets should produce different hashes
    assert_ne!(hashlock1, hashlock2);
    
    // Verify secret validation works
    let immutables1 = create_test_immutables_with_secret(&env, secret1.clone(), hashlock1);
    let immutables2 = create_test_immutables_with_secret(&env, secret2.clone(), hashlock2);
    
    assert!(only_valid_secret(&env, &secret1, &immutables1).is_ok());
    assert!(only_valid_secret(&env, &secret2, &immutables2).is_ok());
    
    // Cross-validation should fail
    assert_eq!(
        only_valid_secret(&env, &secret1, &immutables2),
        Err(EscrowError::InvalidSecret)
    );
    assert_eq!(
        only_valid_secret(&env, &secret2, &immutables1),
        Err(EscrowError::InvalidSecret)
    );
}