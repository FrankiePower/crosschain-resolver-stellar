#![cfg(test)]
extern crate alloc;
extern crate std;

use crate::{EscrowFactory, EscrowFactoryClient, EscrowStage, EscrowType};
use shared::{DualAddress, Immutables, Timelocks, EscrowError};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _},
    Address, BytesN, Env,
};

// Helper function to create test immutables
fn create_test_immutables(env: &Env) -> Immutables {
    let evm_addr = BytesN::from_array(env, &[0x42; 20]);
    let stellar_addr = Address::generate(env);
    
    let dual_address = DualAddress {
        evm: evm_addr,
        stellar: stellar_addr,
    };
    
    Immutables {
        order_hash: BytesN::from_array(env, &[0x01; 32]),
        hashlock: BytesN::from_array(env, &[0x02; 32]),
        maker: dual_address.clone(),
        taker: dual_address.clone(),
        token: dual_address,
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

fn create_factory_client(env: &Env) -> EscrowFactoryClient {
    let admin = Address::generate(env);
    let rescue_delay = 86_400u64;
    
    EscrowFactoryClient::new(
        env, 
        &env.register(EscrowFactory, (admin, rescue_delay))
    )
}

#[test]
fn test_factory_initialization() {
    let env = Env::default();
    let factory_client = create_factory_client(&env);
    
    let admin = factory_client.get_admin();
    let rescue_delay = factory_client.get_rescue_delay();
    
    assert_eq!(rescue_delay, 86_400u64);
    assert_eq!(admin.to_string().len() > 0, true);
}

#[test]
fn test_create_src_escrow() {
    let env = Env::default();
    let factory_client = create_factory_client(&env);
    
    let immutables = create_test_immutables(&env);
    
    // Create source escrow
    let result = factory_client.create_src_escrow(immutables.clone());
    assert!(result.is_ok());
    
    let factory_address = result.unwrap();
    assert_eq!(factory_address, factory_client.address);
    
    // Verify escrow was stored correctly
    let escrow_id = shared::other_immutables::hash(&env, immutables.clone()).unwrap();
    let stage = factory_client.get_escrow_stage(escrow_id.clone());
    assert_eq!(stage, EscrowStage::Created);
    
    let (escrow_type, stored_immutables) = factory_client.get_escrow_state(escrow_id.clone()).unwrap();
    assert_eq!(escrow_type, EscrowType::Source);
    assert_eq!(stored_immutables.amount, immutables.amount);
    assert_eq!(stored_immutables.safety_deposit, immutables.safety_deposit);
}

#[test]
fn test_create_dst_escrow() {
    let env = Env::default();
    let factory_client = create_factory_client(&env);
    
    let immutables = create_test_immutables(&env);
    
    // Create destination escrow
    let result = factory_client.create_dst_escrow(immutables.clone());
    assert!(result.is_ok());
    
    let factory_address = result.unwrap();
    assert_eq!(factory_address, factory_client.address);
    
    // Verify escrow was stored correctly
    let escrow_id = shared::other_immutables::hash(&env, immutables.clone()).unwrap();
    let stage = factory_client.get_escrow_stage(escrow_id.clone());
    assert_eq!(stage, EscrowStage::Created);
    
    let (escrow_type, stored_immutables) = factory_client.get_escrow_state(escrow_id.clone()).unwrap();
    assert_eq!(escrow_type, EscrowType::Destination);
    assert_eq!(stored_immutables.amount, immutables.amount);
}

#[test]
fn test_create_duplicate_escrow_fails() {
    let env = Env::default();
    let factory_client = create_factory_client(&env);
    
    let immutables = create_test_immutables(&env);
    
    // Create first escrow - should succeed
    let result1 = factory_client.create_src_escrow(immutables.clone());
    assert!(result1.is_ok());
    
    // Try to create same escrow again - should fail
    let result2 = factory_client.create_src_escrow(immutables.clone());
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), EscrowError::InvalidImmutables);
}

#[test]
fn test_withdraw_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    
    let factory_client = create_factory_client(&env);
    let immutables = create_test_immutables(&env);
    
    // Create escrow
    let result = factory_client.create_src_escrow(immutables.clone());
    assert!(result.is_ok());
    
    let escrow_id = shared::other_immutables::hash(&env, immutables.clone()).unwrap();
    let secret = immutables.hashlock; // Use hashlock as secret for testing
    
    // Withdraw should succeed
    let withdraw_result = factory_client.withdraw(escrow_id.clone(), &secret);
    assert!(withdraw_result.is_ok());
    
    // Verify stage updated
    let stage = factory_client.get_escrow_stage(escrow_id.clone());
    assert_eq!(stage, EscrowStage::Withdrawn);
    
    // Try to withdraw again - should fail
    let withdraw_result2 = factory_client.withdraw(escrow_id.clone(), secret);
    assert!(withdraw_result2.is_err());
    assert_eq!(withdraw_result2.unwrap_err(), EscrowError::InvalidTime);
}

#[test]
fn test_cancel_escrow() {
    let env = Env::default();
    env.mock_all_auths();
    
    let factory_client = create_factory_client(&env);
    let immutables = create_test_immutables(&env);
    
    // Create escrow
    let result = factory_client.create_src_escrow(immutables.clone());
    assert!(result.is_ok());
    
    let escrow_id = shared::other_immutables::hash(&env, immutables.clone()).unwrap();
    
    // Cancel should succeed (mocked auth)
    let cancel_result = factory_client.cancel(escrow_id.clone());
    assert!(cancel_result.is_ok());
    
    // Verify stage updated
    let stage = factory_client.get_escrow_stage(escrow_id.clone());
    assert_eq!(stage, EscrowStage::Cancelled);
}

#[test]
fn test_rescue_funds() {
    let env = Env::default();
    env.mock_all_auths();
    
    let factory_client = create_factory_client(&env);
    let immutables = create_test_immutables(&env);
    
    // Create escrow
    let result = factory_client.create_src_escrow(immutables.clone());
    assert!(result.is_ok());
    
    let escrow_id = shared::other_immutables::hash(&env, immutables.clone()).unwrap();
    
    // Rescue funds should succeed (mocked auth and timing)
    let rescue_result = factory_client.rescue_funds(escrow_id, 500i128);
    assert!(rescue_result.is_ok());
}

#[test]
fn test_multiple_escrows() {
    let env = Env::default();
    let factory_client = create_factory_client(&env);
    
    // Create multiple different escrows
    let mut immutables1 = create_test_immutables(&env);
    immutables1.amount = 1000;
    
    let mut immutables2 = create_test_immutables(&env);
    immutables2.amount = 2000;
    immutables2.order_hash = BytesN::from_array(&env, &[0x02; 32]); // Different hash
    
    // Create both escrows
    let result1 = factory_client.create_src_escrow(immutables1.clone());
    let result2 = factory_client.create_dst_escrow(immutables2.clone());
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    
    // Verify both are stored correctly
    let escrow_id1 = shared::other_immutables::hash(&env, &immutables1).unwrap();
    let escrow_id2 = shared::other_immutables::hash(&env, &immutables2).unwrap();
    
    let (type1, stored1) = factory_client.get_escrow_state(escrow_id1).unwrap();
    let (type2, stored2) = factory_client.get_escrow_state(escrow_id2).unwrap();
    
    assert_eq!(type1, EscrowType::Source);
    assert_eq!(type2, EscrowType::Destination);
    assert_eq!(stored1.amount, 1000);
    assert_eq!(stored2.amount, 2000);
}

#[test]
fn test_nonexistent_escrow() {
    let env = Env::default();
    let factory_client = create_factory_client(&env);
    
    let fake_escrow_id = BytesN::from_array(&env, &[0xFF; 32]);
    
    // Try to get nonexistent escrow - should fail
    let result = factory_client.get_escrow_state(&fake_escrow_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), EscrowError::InvalidImmutables);
    
    // Stage should default to Created
    let stage = factory_client.get_escrow_stage(&fake_escrow_id);
    assert_eq!(stage, EscrowStage::Created);
}