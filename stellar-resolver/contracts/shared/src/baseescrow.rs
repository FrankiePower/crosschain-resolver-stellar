use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror, Address, BytesN, Env, symbol_short, token,
};
use crate::immutables::{DualAddress, Immutables, immutables};
use crate::timelock::timelocks;

// Storage keys
#[contracttype]
pub enum DataKey {
    RescueDelay,
    Factory,
    Immutables,
}

// Errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Error {
    InvalidCaller = 1,
    InvalidImmutables = 2,
    InvalidSecret = 3,
    InvalidTime = 4,
    NativeTokenSendingFailure = 5,
    AddressMappingMissing = 6,
    TimeLockError = 7,
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
pub fn only_taker(env: &Env, immutables: &Immutables) -> Result<(), Error> {
    let _stellar_taker = immutables::get_stellar_addr(env, &immutables.taker.evm)
        .ok_or(Error::AddressMappingMissing)?;
    // TODO: Add proper caller authentication
    // if caller != stellar_taker {
        // return Err(Error::InvalidCaller);
        Ok(())
    }


pub fn only_valid_secret(env: &Env, secret: &BytesN<32>, immutables: &Immutables) -> Result<(), Error> {
    use soroban_sdk::Bytes;
    let secret_bytes = Bytes::from_array(env, &secret.to_array());
    let computed_hash = env.crypto().keccak256(&secret_bytes);
    let computed_hash_bytes: BytesN<32> = computed_hash.into();
    if computed_hash_bytes != immutables.hashlock {
        return Err(Error::InvalidSecret);
    }
    Ok(())
}

pub fn only_after(env: &Env, start: u64) -> Result<(), Error> {
    if env.ledger().timestamp() < start {
        return Err(Error::InvalidTime);
    }
    Ok(())
}

pub fn only_before(env: &Env, stop: u64) -> Result<(), Error> {
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
        let rescue_start = timelocks::rescue_start(&immutables.timelocks, &env, Self::rescue_delay(env.clone())).map_err(|_| Error::TimeLockError)?;
        only_after(&env, rescue_start)?;
        let stellar_token = immutables::get_stellar_addr(&env, &token.evm)
            .ok_or(Error::AddressMappingMissing)?;
        if stellar_token != immutables.token.stellar {
            return Err(Error::InvalidImmutables);
        }
        let stellar_taker = immutables::get_stellar_addr(&env, &immutables.taker.evm)
            .ok_or(Error::AddressMappingMissing)?;
        uni_transfer(&env, &stellar_token, &stellar_taker, amount)?;
        env.events().publish((symbol_short!("FundsSave"), stellar_token), amount);
        Ok(())
    }

    fn initialize(env: Env, factory: Address, rescue_delay: u64, immutables: Immutables) {
        validate_immutables(&env, &immutables).expect("Invalid immutables");
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
    timelocks::validate_timelocks(&immutables.timelocks, env).map_err(|_| Error::TimeLockError)?;
    // Hash validation (to be extended by derived contracts)
    if immutables::hash(env, immutables).map_err(|_| Error::TimeLockError)? != immutables.order_hash {
        return Err(Error::InvalidImmutables);
    }
    Ok(())
}

// Token transfer
pub fn uni_transfer(env: &Env, token: &Address, to: &Address, amount: i128) -> Result<(), Error> {
    if amount <= 0 {
        return Ok(());
    }
    // For token contracts (not native tokens)
    {
        let token_client = token::Client::new(env, token);
        token_client.transfer(&env.current_contract_address(), to, &amount);
    }
    Ok(())
}