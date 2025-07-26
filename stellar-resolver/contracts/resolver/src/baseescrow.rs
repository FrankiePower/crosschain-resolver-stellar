use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, Symbol, symbol_short, token,
};
use crate::immutables::{DualAddress, Immutables, immutables};
use crate::timelocks::{self, Timelocks};

// Import token interface
soroban_sdk::contractimport!(file = "../token.wasm");

// Storage keys
#[contracttype]
pub enum DataKey {
    RescueDelay,
    Factory,
    Immutables,
}

// Errors
#[contracttype]
pub enum Error {
    InvalidCaller,
    InvalidImmutables,
    InvalidSecret,
    InvalidTime,
    NativeTokenSendingFailure,
    AddressMappingMissing,
}

// BaseEscrow trait
pub trait BaseEscrowTrait {
    fn rescue_delay(env: Env) -> u64;
    fn factory(env: Env) -> Address;
    fn withdraw(env: Env, secret: BytesN<32>, immutables: Immutables) -> Result<(), Error>;
    fn cancel(env: Env, immutables: Immutables) -> Result<(), Error>;
    fn rescue_funds(env: Env, token: DualAddress, amount: i128, immutables: Immutables) -> Result<(), Error>;
    fn initialize(env: Env, factory: Address, rescue_delay: u64, immutables: Immutables);
}

// Modifier helpers
fn only_taker(env: &Env, immutables: &Immutables) -> Result<(), Error> {
    let stellar_taker = immutables::get_stellar_addr(env, &immutables.taker.evm)
        .ok_or(Error::AddressMappingMissing)?;
    if env.invoker() != stellar_taker {
        return Err(Error::InvalidCaller);
    }
    Ok(())
}

fn only_valid_secret(env: &Env, secret: &BytesN<32>, immutables: &Immutables) -> Result<(), Error> {
    let computed_hash = env.crypto().keccak256(secret);
    if computed_hash.to_bytes() != immutables.hashlock {
        return Err(Error::InvalidSecret);
    }
    Ok(())
}

fn only_after(env: &Env, start: u64) -> Result<(), Error> {
    if env.ledger().timestamp() < start {
        return Err(Error::InvalidTime);
    }
    Ok(())
}

fn only_before(env: &Env, stop: u64) -> Result<(), Error> {
    if env.ledger().timestamp() >= stop {
        return Err(Error::InvalidTime);
    }
    Ok(())
}

// BaseEscrow contract
#[contract]
pub struct BaseEscrow;

// Partial implementation
#[contractimpl]
impl BaseEscrowTrait for BaseEscrow {
    fn rescue_delay(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::RescueDelay)
            .unwrap_or(86_400)
    }

    fn factory(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Factory)
            .unwrap_or_else(|| panic!("Factory not set"))
    }

    fn rescue_funds(env: Env, token: DualAddress, amount: i128, immutables: Immutables) -> Result<(), Error> {
        only_taker(&env, &immutables)?;
        validate_immutables(&env, &immutables)?;
        let rescue_start = timelocks::rescue_start(&immutables.timelocks, Self::rescue_delay(env))?;
        only_after(&env, rescue_start)?;
        let stellar_token = immutables::get_stellar_addr(env, &token.evm)
            .ok_or(Error::AddressMappingMissing)?;
        if stellar_token != immutables.token.stellar {
            return Err(Error::InvalidImmutables);
        }
        let stellar_taker = immutables::get_stellar_addr(env, &immutables.taker.evm)
            .ok_or(Error::AddressMappingMissing)?;
        uni_transfer(&env, &stellar_token, &stellar_taker, amount)?;
        env.events().publish((symbol_short!("FundsRescued"), stellar_token), amount);
        Ok(())
    }

    fn initialize(env: Env, factory: Address, rescue_delay: u64, immutables: Immutables) {
        validate_immutables(&env, &immutables)?;
        // Map EVM addresses to Stellar addresses (provided by bridge or relayer)
        immutables::map_evm_to_stellar(&env, immutables.maker.evm.clone(), immutables.maker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables.taker.evm.clone(), immutables.taker.stellar.clone());
        immutables::map_evm_to_stellar(&env, immutables.token.evm.clone(), immutables.token.stellar.clone());

        env.storage().persistent().set(&DataKey::Factory, &factory);
        env.storage().persistent().set(&DataKey::RescueDelay, &rescue_delay);
        env.storage().persistent().set(&DataKey::Immutables, &immutables);
        timelocks::store_timelocks(&env, &immutables.timelocks);
    }

    fn withdraw(_env: Env, _secret: BytesN<32>, _immutables: Immutables) -> Result<(), Error> {
        panic!("withdraw must be implemented by derived contract")
    }

    fn cancel(_env: Env, _immutables: Immutables) -> Result<(), Error> {
        panic!("cancel must be implemented by derived contract")
    }
}

// Abstract validate_immutables
fn validate_immutables(env: &Env, immutables: &Immutables) -> Result<(), Error> {
    // Basic validation
    if immutables.amount <= 0 || immutables.safety_deposit <= 0 {
        return Err(Error::InvalidImmutables);
    }
    timelocks::validate_timelocks(&immutables.timelocks)?;
    // Hash validation (to be extended by derived contracts)
    if immutables::hash(env, immutables) != immutables.order_hash {
        return Err(Error::InvalidImmutables);
    }
    Ok(())
}

// Token transfer
fn uni_transfer(env: &Env, token: &Address, to: &Address, amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Ok(());
    }
    if token == &Address::zero() {
        return Err(Error::NativeTokenSendingFailure);
    } else {
        let token_client = token::Client::new(env, token);
        token_client.transfer(&env.current_contract_address(), to, &amount);
    }
    Ok(())
}