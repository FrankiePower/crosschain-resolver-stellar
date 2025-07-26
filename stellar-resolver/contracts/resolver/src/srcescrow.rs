use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, Symbol,
symbol_short,
};
use crate::base_escrow::{BaseEscrowTrait, Error, only_taker,
only_valid_secret, only_before, only_after, uni_transfer};
use crate::immutables::{Immutables, DualAddress, immutables};
use crate::timelocks::{self, Stage, Timelocks};

#[contract]
pub struct SrcEscrow;

// Helper function for maker check
fn only_maker(env: &Env, immutables: &Immutables) -> Result<(), Error> {
    let stellar_maker = immutables::get_stellar_addr(env,
&immutables.maker.evm)
        .ok_or(Error::AddressMappingMissing)?;
    if env.invoker() != stellar_maker {
        return Err(Error::InvalidCaller);
    }
    Ok(())
}

#[contractimpl]
impl BaseEscrowTrait for SrcEscrow {
    fn rescue_delay(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&crate::base_escrow::DataKey::RescueDelay)
            .unwrap_or(86_400)
    }

    fn factory(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&crate::base_escrow::DataKey::Factory)
            .unwrap_or_else(|| panic!("Factory not set"))
    }

    fn withdraw(env: Env, secret: BytesN<32>, immutables: Immutables) ->
 Result<(), Error> {
        only_taker(&env, &immutables)?;
        only_valid_secret(&env, &secret, &immutables)?;
        validate_immutables(&env, &immutables)?;
        let withdraw_deadline = timelocks::get(&immutables.timelocks,
Stage::SrcPublicWithdrawal)?;
        only_before(&env, withdraw_deadline)?;

        // Get addresses
        let stellar_token = immutables::get_stellar_addr(&env,
&immutables.token.evm)
            .ok_or(Error::AddressMappingMissing)?;
        let stellar_taker = immutables::get_stellar_addr(&env,
&immutables.taker.evm)
            .ok_or(Error::AddressMappingMissing)?;

        // ✅ FIX: Funds to taker, safety deposit to caller (incentive 
for revealing secret)
        uni_transfer(&env, &stellar_token, &stellar_taker,
immutables.amount)?;
        uni_transfer(&env, &stellar_token, &env.invoker(),
immutables.safety_deposit)?;

        env.events().publish((symbol_short!("Withdrawal"), secret),
immutables.amount);
        Ok(())
    }

    fn cancel(env: Env, immutables: Immutables) -> Result<(), Error> {
        only_maker(&env, &immutables)?;
        validate_immutables(&env, &immutables)?;
        let cancel_time = timelocks::get(&immutables.timelocks,
Stage::SrcCancellation)?;
        only_after(&env, cancel_time)?;

        // Get addresses
        let stellar_token = immutables::get_stellar_addr(&env,
&immutables.token.evm)
            .ok_or(Error::AddressMappingMissing)?;
        let stellar_maker = immutables::get_stellar_addr(&env,
&immutables.maker.evm)
            .ok_or(Error::AddressMappingMissing)?;

        // ✅ FIX: Funds back to maker, safety deposit to caller 
(incentive for cleanup)
        uni_transfer(&env, &stellar_token, &stellar_maker,
immutables.amount)?;
        uni_transfer(&env, &stellar_token, &env.invoker(),
immutables.safety_deposit)?;

        env.events().publish((symbol_short!("EscrowCancelled"),),
immutables.amount);
        Ok(())
    }

    fn rescue_funds(env: Env, token: DualAddress, amount: i128,
immutables: Immutables) -> Result<(), Error> {
        // Delegate to BaseEscrow implementation
        <crate::base_escrow::BaseEscrow as
BaseEscrowTrait>::rescue_funds(env, token, amount, immutables)
    }

    fn initialize(env: Env, factory: Address, rescue_delay: u64,
immutables: Immutables) {
        // Delegate to BaseEscrow implementation
        <crate::base_escrow::BaseEscrow as
BaseEscrowTrait>::initialize(env, factory, rescue_delay, immutables)
    }
}

// Specific validate_immutables for SrcEscrow
fn validate_immutables(env: &Env, immutables: &Immutables) -> Result<(),
 Error> {
    if immutables.amount <= 0 || immutables.safety_deposit <= 0 {
        return Err(Error::InvalidImmutables);
    }
    timelocks::validate_timelocks(&immutables.timelocks)?;
    if immutables::hash(env, immutables) != immutables.order_hash {
        return Err(Error::InvalidImmutables);
    }
    // Verify Stellar addresses exist
    if immutables::get_stellar_addr(env,
&immutables.maker.evm).is_none() ||
       immutables::get_stellar_addr(env,
&immutables.taker.evm).is_none() ||
       immutables::get_stellar_addr(env,
&immutables.token.evm).is_none() {
        return Err(Error::AddressMappingMissing);
    }
    Ok(())
}