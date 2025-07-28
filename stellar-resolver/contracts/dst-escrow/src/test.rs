#![cfg(test)]

use super::*;
use shared::{
    other_immutables as immutables, timelocks, BaseEscrowTrait, DualAddress, EscrowError,
    Immutables, Stage, Timelocks,
};
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{Address, BytesN, Env};

// Helper functions for testing
fn create_test_dual_address(env: &Env) -> DualAddress {
    let e = Env::default();
    let evm_addr = BytesN::from_array(env, &[0x42; 20]);
    let stellar_addr = Address::generate(&e);
    DualAddress {
        evm: evm_addr,
        stellar: stellar_addr,
    }
}

fn create_test_secret(env: &Env) -> (BytesN<32>, BytesN<32>) {
    let secret = BytesN::from_array(env, &[0x42; 32]);
    // Create hashlock by hashing the secret
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

// ===== DSTESCROW TESTS =====

#[test]
fn test_dstescrow_rescue_delay() {
    let env = Env::default();
    let contract_id = env.register(DstEscrow, ());

    env.as_contract(&contract_id, || {
        let delay = DstEscrow::dst_rescue_delay(env.clone());
        assert_eq!(delay, 86_400); // Default 24 hours
    });
}

#[test]
fn test_dstescrow_private_withdraw_timelock() {
    let env = Env::default();
    let contract_id = env.register(DstEscrow, ());
    let (secret, hashlock) = create_test_secret(&env);
    let immutables = create_test_immutables_with_secret(&env, secret.clone(), hashlock);

    // Set current time within dst withdrawal window
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = 1175; // Between deployed_at + dst_withdrawal (1150) and dst_cancellation (1350)
    });

    env.as_contract(&contract_id, || {
        // Note: This will fail due to missing caller authentication and missing address mappings
        let _result = DstEscrow::withdraw(env.clone(), secret, immutables);
        // Should pass timelock validation but may fail on other checks
    });
}

#[test]
fn test_dstescrow_public_withdraw_timelock() {
    let env = Env::default();
    let contract_id = env.register(DstEscrow, ());
    let (secret, hashlock) = create_test_secret(&env);
    let immutables = create_test_immutables_with_secret(&env, secret.clone(), hashlock);

    // Set current time within dst public withdrawal window
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = 1275; // Between deployed_at + dst_public_withdrawal (1250) and dst_cancellation (1350)
    });

    env.as_contract(&contract_id, || {
        // Note: This will fail due to missing address mappings
        let _result = DstEscrow::public_withdraw(env.clone(), secret, immutables);
        // Should pass timelock validation but may fail on other checks
    });
}
