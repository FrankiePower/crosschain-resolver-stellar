#![no_std]

/// Cross-chain escrow factory for deploying src-escrow and dst-escrow contracts
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Symbol, Val, Vec, IntoVal};
use shared::{DualAddress, Immutables};

#[contract]
pub struct EscrowFactory;

const ADMIN: Symbol = symbol_short!("admin");
const SRC_WASM: Symbol = symbol_short!("src_wasm");
const DST_WASM: Symbol = symbol_short!("dst_wasm");

#[contractimpl]
impl EscrowFactory {
    /// Initialize factory with admin and escrow contract WASM hashes
    pub fn __constructor(
        env: Env, 
        admin: Address,
        src_escrow_wasm: BytesN<32>,
        dst_escrow_wasm: BytesN<32>
    ) {
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&SRC_WASM, &src_escrow_wasm);
        env.storage().instance().set(&DST_WASM, &dst_escrow_wasm);
    }

    /// Deploy source chain escrow contract with proper initialization
    pub fn deploy_src_escrow(
        env: Env,
        deployer: Address,
        order_hash: BytesN<32>,
        rescue_delay: u64,
        immutables: Immutables,
    ) -> (Address, Val) {
        // Skip authorization if deployer is the current contract
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        }

        let src_wasm: BytesN<32> = env.storage().instance().get(&SRC_WASM).unwrap();
        
        // Use order_hash as salt for deterministic address
        let salt = order_hash;

        // Deploy the contract using the uploaded Wasm with given hash
        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy(src_wasm);

        // Initialize the contract after deployment
        let factory_addr = env.current_contract_address();
        let init_args: Vec<Val> = (factory_addr, rescue_delay, immutables).into_val(&env);
        
        let res: Val = env.invoke_contract(&deployed_address, &symbol_short!("init"), init_args);

        (deployed_address, res)
    }

    /// Deploy destination chain escrow contract with proper initialization
    pub fn deploy_dst_escrow(
        env: Env,
        deployer: Address,
        order_hash: BytesN<32>,
        rescue_delay: u64,
        immutables: Immutables,
    ) -> (Address, Val) {
        // Skip authorization if deployer is the current contract
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        }

        let dst_wasm: BytesN<32> = env.storage().instance().get(&DST_WASM).unwrap();
        
        // Use order_hash as salt for deterministic address
        let salt = order_hash;

        // Deploy the contract using the uploaded Wasm with given hash
        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy(dst_wasm);

        // Initialize the contract after deployment
        let factory_addr = env.current_contract_address();
        let init_args: Vec<Val> = (factory_addr, rescue_delay, immutables).into_val(&env);
        
        let res: Val = env.invoke_contract(&deployed_address, &symbol_short!("init"), init_args);

        (deployed_address, res)
    }

    /// Deploy both escrow contracts atomically for a cross-chain swap
    pub fn deploy_escrow_pair(
        env: Env,
        deployer: Address,
        order_hash: BytesN<32>,
        rescue_delay: u64,
        immutables: Immutables,
    ) -> ((Address, Val), (Address, Val)) {
        let src_result = Self::deploy_src_escrow(
            env.clone(), 
            deployer.clone(), 
            order_hash.clone(), 
            rescue_delay, 
            immutables.clone()
        );
        let dst_result = Self::deploy_dst_escrow(
            env.clone(), 
            deployer, 
            order_hash, 
            rescue_delay, 
            immutables
        );
        
        (src_result, dst_result)
    }

    /// Generic deploy method following Soroban pattern
    pub fn deploy(
        env: Env,
        deployer: Address,
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
    ) -> (Address, Val) {
        // Skip authorization if deployer is the current contract
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        }

        // Deploy the contract using the uploaded Wasm with given hash
        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy(wasm_hash);

        // Invoke the init function with the given arguments
        let res: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);
        
        (deployed_address, res)
    }

    /// Update WASM hashes (admin only)
    pub fn update_wasm_hashes(
        env: Env,
        src_escrow_wasm: Option<BytesN<32>>,
        dst_escrow_wasm: Option<BytesN<32>>
    ) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        if let Some(src_wasm) = src_escrow_wasm {
            env.storage().instance().set(&SRC_WASM, &src_wasm);
        }
        if let Some(dst_wasm) = dst_escrow_wasm {
            env.storage().instance().set(&DST_WASM, &dst_wasm);
        }
    }
}

mod test;
