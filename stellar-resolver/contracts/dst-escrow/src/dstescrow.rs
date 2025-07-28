use shared::{
    only_after, only_before, only_taker, only_valid_secret, other_immutables as immutables,
    timelocks, uni_transfer, BaseEscrowTrait, DualAddress, EscrowError as Error, Immutables, Stage,
};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Symbol};

#[contract]
pub struct DstEscrow;

#[contractimpl]
impl DstEscrow {
    /// Public initialization function called after deployment
    pub fn init(env: Env, factory: Address, rescue_delay: u64, immutables: Immutables) -> Symbol {
        // Use the BaseEscrow initialize method
        <shared::baseescrow::BaseEscrow as BaseEscrowTrait>::initialize(env, factory, rescue_delay, immutables);
        symbol_short!("init_ok")
    }

    /// 🔄 PUBLIC WITHDRAWAL: Anyone can withdraw with secret (lines 48-54 in Solidity)
    pub fn public_withdraw(
        env: Env,
        secret: BytesN<32>,
        immutables: Immutables,
    ) -> Result<(), Error> {
        only_valid_secret(&env, &secret, &immutables)?;
        dst_validate_immutables(&env, &immutables)?;

        // Different timelock: DstPublicWithdrawal → DstCancellation window
        let public_start = timelocks::get(&immutables.timelocks, &env, Stage::DstPublicWithdrawal)
            .map_err(|_| Error::TimeLockError)?;
        let public_end = timelocks::get(&immutables.timelocks, &env, Stage::DstCancellation)
            .map_err(|_| Error::TimeLockError)?;
        only_after(&env, public_start)?;
        only_before(&env, public_end)?;

        _dst_withdraw(&env, secret, &immutables)?;
        Ok(())
    }
    fn get_rescue_delay(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&shared::baseescrow::DataKey::RescueDelay)
            .unwrap_or(86_400)
    }

    fn get_factory(env: Env) -> Address {
        env.storage()
            .persistent()
            .get(&shared::baseescrow::DataKey::Factory)
            .unwrap_or_else(|| panic!("Factory not set"))
    }

    // 🔄 PRIVATE WITHDRAWAL: Only taker can withdraw (lines 34-41 in Solidity)
    pub fn withdraw(env: Env, secret: BytesN<32>, immutables: Immutables) -> Result<(), Error> {
        only_taker(&env, &immutables)?;
        only_valid_secret(&env, &secret, &immutables)?;
        dst_validate_immutables(&env, &immutables)?;

        // Different timelock: DstWithdrawal → DstCancellation window
        let withdraw_start = timelocks::get(&immutables.timelocks, &env, Stage::DstWithdrawal)
            .map_err(|_| Error::TimeLockError)?;
        let withdraw_end = timelocks::get(&immutables.timelocks, &env, Stage::DstCancellation)
            .map_err(|_| Error::TimeLockError)?;
        only_after(&env, withdraw_start)?;
        only_before(&env, withdraw_end)?;

        _dst_withdraw(&env, secret, &immutables)?;
        Ok(())
    }

    pub fn cancel(env: Env, immutables: Immutables) -> Result<(), Error> {
        only_taker(&env, &immutables)?;
        dst_validate_immutables(&env, &immutables)?;

        // Can only cancel AFTER DstCancellation time (line 65 in Solidity)
        let cancel_time = timelocks::get(&immutables.timelocks, &env, Stage::DstCancellation)
            .map_err(|_| Error::TimeLockError)?;
        only_after(&env, cancel_time)?;

        let stellar_token = immutables::get_stellar_addr(&env, &immutables.token.evm)
            .ok_or(Error::AddressMappingMissing)?;
        let stellar_taker = immutables::get_stellar_addr(&env, &immutables.taker.evm)
            .ok_or(Error::AddressMappingMissing)?;

        // ✅ CRITICAL: In cancel, funds go back to TAKER (resolver), not maker (lines 67-68)
        uni_transfer(&env, &stellar_token, &stellar_taker, immutables.amount)?;
        // Safety deposit to taker (simplified for now)
        uni_transfer(
            &env,
            &stellar_token,
            &stellar_taker,
            immutables.safety_deposit,
        )?;

        env.events()
            .publish((symbol_short!("Cancelled"),), immutables.amount);
        Ok(())
    }


    pub fn dst_rescue_delay(env: Env) -> u64 {
        Self::get_rescue_delay(env)
    }

    pub fn dst_factory(env: Env) -> Address {
        Self::get_factory(env)
    }
}

// ✅ CRITICAL: Funds go to MAKER (user), not taker (lines 82, 93 in Solidity)
fn _dst_withdraw(env: &Env, secret: BytesN<32>, immutables: &Immutables) -> Result<(), Error> {
    let stellar_token = immutables::get_stellar_addr(env, &immutables.token.evm)
        .ok_or(Error::AddressMappingMissing)?;
    let stellar_maker = immutables::get_stellar_addr(env, &immutables.maker.evm)
        .ok_or(Error::AddressMappingMissing)?;

    // 🎯 KEY DIFFERENCE: Funds go to MAKER (user gets their swapped tokens)
    uni_transfer(env, &stellar_token, &stellar_maker, immutables.amount)?;
    // Safety deposit goes to caller (incentive for revealing secret)
    // For now, send to maker (simplified)
    uni_transfer(
        env,
        &stellar_token,
        &stellar_maker,
        immutables.safety_deposit,
    )?;

    env.events()
        .publish((symbol_short!("Withdraw"), secret), immutables.amount);
    Ok(())
}

fn dst_validate_immutables(env: &Env, immutables: &Immutables) -> Result<(), Error> {
    if immutables.amount <= 0 || immutables.safety_deposit <= 0 {
        return Err(Error::InvalidImmutables);
    }
    timelocks::validate_timelocks(&immutables.timelocks, env).map_err(|_| Error::TimeLockError)?;
    if immutables::hash(env, immutables).map_err(|_| Error::TimeLockError)? != immutables.order_hash
    {
        return Err(Error::InvalidImmutables);
    }
    if immutables::get_stellar_addr(env, &immutables.maker.evm).is_none()
        || immutables::get_stellar_addr(env, &immutables.taker.evm).is_none()
        || immutables::get_stellar_addr(env, &immutables.token.evm).is_none()
    {
        return Err(Error::AddressMappingMissing);
    }
    Ok(())
}
