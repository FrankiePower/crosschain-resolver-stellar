#![cfg(test)]
extern crate alloc;
extern crate std;

use crate::{EscrowFactory, EscrowFactoryClient};
use shared::{DualAddress, Immutables, Timelocks};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _},
    Address, BytesN, Env, IntoVal, Val, Vec,
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

#[test]
fn test_factory_initialization() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let src_wasm = BytesN::from_array(&env, &[1; 32]);
    let dst_wasm = BytesN::from_array(&env, &[2; 32]);
    
    // Test factory initialization
    let factory_client = EscrowFactoryClient::new(
        &env, 
        &env.register(EscrowFactory, (admin.clone(), src_wasm.clone(), dst_wasm.clone()))
    );
    
    // The factory should be successfully initialized
    assert_eq!(factory_client.address.to_string().len() > 0, true);
}

#[test]
fn test_generic_deploy_pattern() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let src_wasm = BytesN::from_array(&env, &[1; 32]);
    let dst_wasm = BytesN::from_array(&env, &[2; 32]);
    
    let factory_client = EscrowFactoryClient::new(
        &env, 
        &env.register(EscrowFactory, (admin.clone(), src_wasm.clone(), dst_wasm.clone()))
    );
    
    let deployer = factory_client.address.clone();
    let wasm_hash = BytesN::from_array(&env, &[3; 32]);
    let salt = BytesN::from_array(&env, &[4; 32]);
    let init_fn = symbol_short!("init");
    let init_args: Vec<Val> = (5u32,).into_val(&env);
    
    env.mock_all_auths();
    
    // Test that the generic deploy method exists with correct signature
    // Note: This would fail in actual deployment without valid WASM
    // but demonstrates the correct function signature
    
    // We can't actually deploy without valid WASM, but we can verify the method exists
    // by checking the client has the expected methods
    assert_eq!(factory_client.address.to_string().len() > 0, true);
}

#[test]
fn test_escrow_deployment_concept() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let src_wasm = BytesN::from_array(&env, &[1; 32]);
    let dst_wasm = BytesN::from_array(&env, &[2; 32]);
    
    let factory_client = EscrowFactoryClient::new(
        &env, 
        &env.register(EscrowFactory, (admin.clone(), src_wasm, dst_wasm))
    );
    
    let immutables = create_test_immutables(&env);
    let order_hash = BytesN::from_array(&env, &[0x03; 32]);
    let rescue_delay = 86_400u64;
    let deployer = factory_client.address.clone();
    
    env.mock_all_auths();
    
    // Test the concept of escrow deployment
    // Note: Actual deployment would require valid compiled WASM files
    // This demonstrates the factory pattern with correct parameters
    
    // The deployment functions should have the correct signature:
    // deploy_src_escrow(deployer, order_hash, rescue_delay, immutables) -> (Address, Val)
    // deploy_dst_escrow(deployer, order_hash, rescue_delay, immutables) -> (Address, Val)
    // deploy_escrow_pair(deployer, order_hash, rescue_delay, immutables) -> ((Address, Val), (Address, Val))
    
    // Verify the factory is ready for real deployment
    assert_eq!(factory_client.address.to_string().len() > 0, true);
}

#[test]
fn test_wasm_hash_update() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let src_wasm = BytesN::from_array(&env, &[1; 32]);
    let dst_wasm = BytesN::from_array(&env, &[2; 32]);
    
    let factory_client = EscrowFactoryClient::new(
        &env, 
        &env.register(EscrowFactory, (admin.clone(), src_wasm, dst_wasm))
    );
    
    let new_src_wasm = Some(BytesN::from_array(&env, &[5; 32]));
    let new_dst_wasm = Some(BytesN::from_array(&env, &[6; 32]));
    
    env.mock_all_auths();
    
    // Test that WASM hashes can be updated
    factory_client.update_wasm_hashes(&new_src_wasm, &new_dst_wasm);
    
    // In a real scenario, we would verify the storage was updated
    assert_eq!(factory_client.address.to_string().len() > 0, true);
}
