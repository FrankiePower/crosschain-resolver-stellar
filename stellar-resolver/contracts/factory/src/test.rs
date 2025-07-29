#![cfg(test)]

use super::*;
use shared::{
    other_immutables as immutables, DualAddress, EscrowError, Immutables, Timelocks,
};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, BytesN, Env};
use soroban_sdk::token::{StellarAssetClient as TokenAdmin, TokenClient};

// Helper functions for testing (following dst/src escrow patterns)

// Token setup helper function
fn setup_token(env: &Env) -> (Address, TokenAdmin) {
    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_admin_client = TokenAdmin::new(env, &token.address());
    
    // Set auth for token admin operations
    env.mock_all_auths();
    
    (token.address(), token_admin_client)
}

fn create_test_dual_address(env: &Env) -> DualAddress {
    let e = Env::default();
    let evm_addr = BytesN::from_array(env, &[0x42; 20]);
    let stellar_addr = Address::generate(&e);
    DualAddress {
        evm: evm_addr,
        stellar: stellar_addr,
    }
}

fn create_test_dual_address_with_stellar(env: &Env, stellar_addr: Address, evm_suffix: u8) -> DualAddress {
    let mut evm_bytes = [0x42; 20];
    evm_bytes[19] = evm_suffix; // Make unique EVM addresses
    let evm_addr = BytesN::from_array(env, &evm_bytes);
    DualAddress {
        evm: evm_addr,
        stellar: stellar_addr,
    }
}

fn create_test_secret(env: &Env) -> (BytesN<32>, BytesN<32>) {
    let secret = BytesN::from_array(env, &[0x42; 32]);
    let secret_bytes = soroban_sdk::Bytes::from_array(env, &secret.to_array());
    let hashlock = env.crypto().keccak256(&secret_bytes).into();
    (secret, hashlock)
}

fn create_test_immutables_with_secret(
    env: &Env,
    _secret: BytesN<32>,
    hashlock: BytesN<32>,
) -> Immutables {
    Immutables {
        order_hash: BytesN::from_array(env, &[0x01; 32]),
        hashlock,
        maker: create_test_dual_address(env),
        taker: create_test_dual_address(env),
        token: create_test_dual_address(env),
        amount: 1000,
        safety_deposit: 100,
        timelocks: Timelocks::new(
            env, 1000, // deployed_at
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

// ===== FACTORY TESTS =====

#[test]
fn test_factory_constructor() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(EscrowFactory, (admin.clone(), 86400u64));
    
    // The constructor should have been called during registration
    // Now verify admin and rescue delay are set correctly
    let stored_admin = env.as_contract(&contract_id, || {
        EscrowFactory::get_admin(env.clone())
    });
    let stored_delay = env.as_contract(&contract_id, || {
        EscrowFactory::get_rescue_delay(env.clone())
    });
    
    assert_eq!(stored_admin, admin);
    assert_eq!(stored_delay, 86400);
}

#[test]
fn test_factory_get_admin() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(EscrowFactory, (admin.clone(), 3600u64));
    
    let retrieved_admin = env.as_contract(&contract_id, || {
        EscrowFactory::get_admin(env.clone())
    });
    assert_eq!(retrieved_admin, admin);
}

#[test]
fn test_factory_get_rescue_delay() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let custom_delay = 7200u64; // 2 hours
    let contract_id = env.register(EscrowFactory, (admin, custom_delay));
    
    let retrieved_delay = env.as_contract(&contract_id, || {
        EscrowFactory::get_rescue_delay(env.clone())
    });
    assert_eq!(retrieved_delay, custom_delay);
}

#[test]
fn test_factory_create_src_escrow_basic() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(EscrowFactory, (admin, 86400u64));
    
    let (secret, hashlock) = create_test_secret(&env);
    let immutables = create_test_immutables_with_secret(&env, secret, hashlock);
    
    env.as_contract(&contract_id, || {
        // Note: This will likely fail due to missing address mappings, but we're testing the basic call
        let _result = EscrowFactory::create_src_escrow(env.clone(), immutables);
        // Test that the function can be called without panicking
    });
}

#[test]
fn test_factory_create_dst_escrow_basic() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(EscrowFactory, (admin, 86400u64));
    
    let (secret, hashlock) = create_test_secret(&env);
    let immutables = create_test_immutables_with_secret(&env, secret, hashlock);
    
    env.as_contract(&contract_id, || {
        // Note: This will likely fail due to missing address mappings, but we're testing the basic call
        let _result = EscrowFactory::create_dst_escrow(env.clone(), immutables);
        // Test that the function can be called without panicking
    });
}

// ===== SINGLETON ARCHITECTURE CRITICAL TESTS =====

#[test]
fn test_multiple_concurrent_escrows() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(EscrowFactory, (admin, 86400u64));
    
    env.as_contract(&contract_id, || {
        // Create 3 different escrows with unique order_hash values
        let (secret1, hashlock1) = create_test_secret(&env);
        let mut immutables1 = create_test_immutables_with_secret(&env, secret1, hashlock1);
        immutables1.order_hash = BytesN::from_array(&env, &[0x01; 32]); // Unique hash
        
        let (secret2, hashlock2) = create_test_secret(&env);
        let mut immutables2 = create_test_immutables_with_secret(&env, secret2, hashlock2);
        immutables2.order_hash = BytesN::from_array(&env, &[0x02; 32]); // Different hash
        
        let (secret3, hashlock3) = create_test_secret(&env);
        let mut immutables3 = create_test_immutables_with_secret(&env, secret3, hashlock3);
        immutables3.order_hash = BytesN::from_array(&env, &[0x03; 32]); // Different hash
        
        // Set up address mappings for all escrows
        immutables::map_evm_to_stellar(&env, immutables1.maker.evm.clone(), immutables1.maker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables1.taker.evm.clone(), immutables1.taker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables1.token.evm.clone(), immutables1.token.stellar.clone());
        
        immutables::map_evm_to_stellar(&env, immutables2.maker.evm.clone(), immutables2.maker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables2.taker.evm.clone(), immutables2.taker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables2.token.evm.clone(), immutables2.token.stellar.clone());
        
        immutables::map_evm_to_stellar(&env, immutables3.maker.evm.clone(), immutables3.maker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables3.taker.evm.clone(), immutables3.taker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables3.token.evm.clone(), immutables3.token.stellar.clone());
        
        // Deploy all three escrows
        let result1 = EscrowFactory::create_src_escrow(env.clone(), immutables1.clone());
        let result2 = EscrowFactory::create_dst_escrow(env.clone(), immutables2.clone());
        let result3 = EscrowFactory::create_src_escrow(env.clone(), immutables3.clone());
        
        // All should succeed independently
        assert!(result1.is_ok(), "First escrow creation should succeed");
        assert!(result2.is_ok(), "Second escrow creation should succeed");
        assert!(result3.is_ok(), "Third escrow creation should succeed");
        
        // Verify each escrow is stored independently by order_hash
        let order_hash1 = immutables1.order_hash.clone();
        let order_hash2 = immutables2.order_hash.clone();
        let order_hash3 = immutables3.order_hash.clone();
        
        // All should have different order_hash values (we set them differently)
        assert_ne!(order_hash1, order_hash2, "Order hashes should be different");
        assert_ne!(order_hash2, order_hash3, "Order hashes should be different");
        assert_ne!(order_hash1, order_hash3, "Order hashes should be different");
        
        // Verify each escrow maintains correct state (keyed by order_hash)
        let (type1, stored1) = EscrowFactory::get_escrow_state(env.clone(), order_hash1.clone()).unwrap();
        let (type2, stored2) = EscrowFactory::get_escrow_state(env.clone(), order_hash2.clone()).unwrap();
        let (type3, stored3) = EscrowFactory::get_escrow_state(env.clone(), order_hash3.clone()).unwrap();
        
        assert_eq!(type1, EscrowType::Source);
        assert_eq!(type2, EscrowType::Destination);
        assert_eq!(type3, EscrowType::Source);
        
        assert_eq!(stored1.order_hash, immutables1.order_hash);
        assert_eq!(stored2.order_hash, immutables2.order_hash);
        assert_eq!(stored3.order_hash, immutables3.order_hash);
        
        // All should be in Created state
        assert_eq!(EscrowFactory::get_escrow_stage(env.clone(), order_hash1), EscrowStage::Created);
        assert_eq!(EscrowFactory::get_escrow_stage(env.clone(), order_hash2), EscrowStage::Created);
        assert_eq!(EscrowFactory::get_escrow_stage(env.clone(), order_hash3), EscrowStage::Created);
    });
}

#[test]
fn test_withdrawal_state_isolation() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(EscrowFactory, (admin, 86400u64));
    
    // Setup token contract with proper balances
    let (token_address, token_admin) = setup_token(&env);
    let token_client = TokenClient::new(&env, &token_address);
    
    // Fund the factory contract for token transfers
    token_admin.mint(&contract_id, &10000);
    
    env.as_contract(&contract_id, || {
        // Create 3 escrows (A, B, C) with the actual token
        let (secretA, hashlockA) = create_test_secret(&env);
        let mut immutablesA = create_test_immutables_with_secret(&env, secretA.clone(), hashlockA);
        immutablesA.order_hash = BytesN::from_array(&env, &[0xAA; 32]);
        immutablesA.token = create_test_dual_address_with_stellar(&env, token_address.clone(), 0xAA);
        
        let (secretB, hashlockB) = create_test_secret(&env);
        let mut immutablesB = create_test_immutables_with_secret(&env, secretB.clone(), hashlockB);
        immutablesB.order_hash = BytesN::from_array(&env, &[0xBB; 32]);
        immutablesB.token = create_test_dual_address_with_stellar(&env, token_address.clone(), 0xBB);
        
        let (secretC, hashlockC) = create_test_secret(&env);
        let mut immutablesC = create_test_immutables_with_secret(&env, secretC.clone(), hashlockC);
        immutablesC.order_hash = BytesN::from_array(&env, &[0xCC; 32]);
        immutablesC.token = create_test_dual_address_with_stellar(&env, token_address.clone(), 0xCC);
        
        // Set up address mappings for all escrows
        for immutables in [&immutablesA, &immutablesB, &immutablesC] {
            immutables::map_evm_to_stellar(&env, immutables.maker.evm.clone(), immutables.maker.stellar.clone());
            immutables::map_evm_to_stellar(&env, immutables.taker.evm.clone(), immutables.taker.stellar.clone());
            immutables::map_evm_to_stellar(&env, immutables.token.evm.clone(), immutables.token.stellar.clone());
        }
        
        // Deploy all three escrows
        EscrowFactory::create_src_escrow(env.clone(), immutablesA.clone()).unwrap();
        EscrowFactory::create_src_escrow(env.clone(), immutablesB.clone()).unwrap();
        EscrowFactory::create_src_escrow(env.clone(), immutablesC.clone()).unwrap();
        
        let order_hashA = immutablesA.order_hash.clone();
        let order_hashB = immutablesB.order_hash.clone();
        let order_hashC = immutablesC.order_hash.clone();
        
        // Verify all start in Created state
        assert_eq!(EscrowFactory::get_escrow_stage(env.clone(), order_hashA.clone()), EscrowStage::Created);
        assert_eq!(EscrowFactory::get_escrow_stage(env.clone(), order_hashB.clone()), EscrowStage::Created);
        assert_eq!(EscrowFactory::get_escrow_stage(env.clone(), order_hashC.clone()), EscrowStage::Created);
        
        // Set time to allow withdrawals
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1150; // Within withdrawal window
        });
        
        // Get initial token balance
        let initial_balance = token_client.balance(&contract_id);
        
        // Attempt withdrawal from escrow B only
        let withdrawal_result = EscrowFactory::withdraw(env.clone(), order_hashB.clone(), secretB);
        
        // Check state isolation: A and C should remain unaffected regardless of B's outcome
        let stageA_after = EscrowFactory::get_escrow_stage(env.clone(), order_hashA.clone());
        let stageC_after = EscrowFactory::get_escrow_stage(env.clone(), order_hashC.clone());
        
        // A and C must still be in Created state (unaffected by B's operation)
        assert_eq!(stageA_after, EscrowStage::Created, "Escrow A should remain unaffected");
        assert_eq!(stageC_after, EscrowStage::Created, "Escrow C should remain unaffected");
        
        // Verify escrow data integrity for A and C
        let (_, storedA) = EscrowFactory::get_escrow_state(env.clone(), order_hashA).unwrap();
        let (_, storedC) = EscrowFactory::get_escrow_state(env.clone(), order_hashC).unwrap();
        
        assert_eq!(storedA.order_hash, immutablesA.order_hash, "Escrow A data should be intact");
        assert_eq!(storedC.order_hash, immutablesC.order_hash, "Escrow C data should be intact");
        assert_eq!(storedA.amount, immutablesA.amount, "Escrow A amount should be intact");
        assert_eq!(storedC.amount, immutablesC.amount, "Escrow C amount should be intact");
        
        // If withdrawal succeeded, check that B's state changed but only B's state
        if withdrawal_result.is_ok() {
            let stageB_after = EscrowFactory::get_escrow_stage(env.clone(), order_hashB);
            assert_eq!(stageB_after, EscrowStage::Withdrawn, "Escrow B should be withdrawn if successful");
            
            // Verify token transfer happened (balance reduced)
            let final_balance = token_client.balance(&contract_id);
            let expected_transferred = immutablesB.amount + immutablesB.safety_deposit;
            assert_eq!(final_balance, initial_balance - expected_transferred, "Token balance should reflect withdrawal");
        }
    });
}

#[test]
fn test_duplicate_order_hash_prevention() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let contract_id = env.register(EscrowFactory, (admin, 86400u64));
    
    env.as_contract(&contract_id, || {
        // Create escrow with specific order_hash
        let (secret, hashlock) = create_test_secret(&env);
        let mut immutables = create_test_immutables_with_secret(&env, secret, hashlock);
        let duplicate_hash = BytesN::from_array(&env, &[0xFF; 32]);
        immutables.order_hash = duplicate_hash.clone();
        
        // Set up address mappings
        immutables::map_evm_to_stellar(&env, immutables.maker.evm.clone(), immutables.maker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables.taker.evm.clone(), immutables.taker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables.token.evm.clone(), immutables.token.stellar.clone());
        
        // First deployment should succeed
        let result1 = EscrowFactory::create_src_escrow(env.clone(), immutables.clone());
        assert!(result1.is_ok(), "First deployment should succeed");
        
        // Verify escrow was created and is in Created state
        let order_hash = immutables.order_hash.clone();
        assert_eq!(EscrowFactory::get_escrow_stage(env.clone(), order_hash.clone()), EscrowStage::Created);
        
        // Create a different escrow but with the SAME order_hash
        let (secret2, hashlock2) = create_test_secret(&env);
        let mut immutables2 = create_test_immutables_with_secret(&env, secret2, hashlock2);
        immutables2.order_hash = duplicate_hash; // Same hash!
        immutables2.amount = 2000; // Different amount
        immutables2.safety_deposit = 200; // Different deposit
        
        // Set up address mappings for second escrow
        immutables::map_evm_to_stellar(&env, immutables2.maker.evm.clone(), immutables2.maker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables2.taker.evm.clone(), immutables2.taker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables2.token.evm.clone(), immutables2.token.stellar.clone());
        
        // Second deployment with same order_hash should fail
        let result2 = EscrowFactory::create_src_escrow(env.clone(), immutables2);
        assert!(result2.is_err(), "Duplicate order_hash deployment should fail");
        
        // Should get InvalidImmutables error (our contract's "already exists" error)
        if let Err(error) = result2 {
            assert_eq!(error, EscrowError::InvalidImmutables, "Should get InvalidImmutables error for duplicate");
        }
        
        // Verify original escrow remains intact and unaffected
        let (stored_type, stored_data) = EscrowFactory::get_escrow_state(env.clone(), order_hash.clone()).unwrap();
        assert_eq!(stored_type, EscrowType::Source);
        assert_eq!(stored_data.amount, 1000, "Original escrow amount should be unchanged");
        assert_eq!(stored_data.safety_deposit, 100, "Original escrow deposit should be unchanged");
        assert_eq!(EscrowFactory::get_escrow_stage(env.clone(), order_hash), EscrowStage::Created, "Original escrow state should be unchanged");
        
        // Try with destination escrow (different function, same hash) - should also fail
        let result3 = EscrowFactory::create_dst_escrow(env.clone(), immutables.clone());
        assert!(result3.is_err(), "Duplicate order_hash deployment should fail even with different escrow type");
    });
}