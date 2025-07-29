# crosschain-resolver-stellar

Cross-Chain Atomic Swap Technical Architecture Flow

  Based on the factory contract implementation, here's
  how the cross-chain swap will work:

  Phase 1: Setup & Deployment

  EVM Chain (Source)                 Stellar Chain
  (Destination)
       |                                      |
  1. Maker creates order_hash        1. Factory receives
   cross-chain
     (unique identifier)                deployment
  request
       |                                      |
  2. Deploy EVM escrow with:         2.
  Factory.create_dst_escrow()
     - order_hash                       - Same
  order_hash (key!)
     - hashlock (secret hash)           - Maps
  EVM→Stellar addresses
     - maker/taker EVM addresses        - Stores in
  persistent storage
     - token amount + safety deposit    - Returns
  factory address
       |                                      |
  3. EVM escrow locks funds          3. Stellar factory
  ready to receive

  Phase 2: Cross-Chain State Synchronization

  Key Insight: order_hash is the canonical cross-chain 
  identifier

  EVM Escrow Contract               Stellar Factory 
  Contract
       |                                   |
  State: { order_hash: 0xABC... }   Storage:
  EscrowDataKey::EscrowState(0xABC...)
         funds locked                      →
  (EscrowType::Destination, immutables)

                  Both contracts share the SAME
  order_hash
                  This ensures cross-chain consistency

  Phase 3: Atomic Swap Execution

  Happy Path - Successful Swap:

  1. Taker discovers secret on EVM chain:
     EVM: taker calls withdraw(secret) → reveals secret

  2. Cross-chain secret propagation:
     → secret is now public on EVM blockchain
     → Anyone can read it from transaction data

  3. Taker claims on Stellar:
     Factory.withdraw(order_hash, secret)
     ├─ Validates: only_taker() + timing + secret
     ├─ Gets escrow: get_escrow_state(order_hash) 
     ├─ Transfers: token + safety_deposit to taker
     └─ Updates: EscrowStage → Withdrawn

  Failure Path - Cancellation:

  After timelock expires:

  EVM Chain:                        Stellar Chain:
  Maker calls cancel()
  Factory.cancel(order_hash)
  ├─ Validates maker + timing       ├─ Validates maker +
   timing  
  ├─ Returns funds to maker         ├─ Returns funds to 
  maker
  └─ State: Cancelled              └─ State: Cancelled

  Phase 4: State Management Architecture

  Singleton Factory Pattern:

  // Single factory manages multiple escrows
  pub enum EscrowDataKey {
      EscrowState(BytesN<32>),    // order_hash → (type,
   immutables)  
      EscrowStage(BytesN<32>),    // order_hash → 
  Created/Withdrawn/Cancelled
  }

  // Each escrow isolated by order_hash
  Factory Contract Storage:
  ├─ EscrowState(0xABC...) → (Source, immutables_A)
  ├─ EscrowState(0xDEF...) → (Destination, immutables_B)

  ├─ EscrowStage(0xABC...) → Created
  └─ EscrowStage(0xDEF...) → Withdrawn

  Phase 5: Cross-Chain Address Resolution

  // Bridges EVM ↔ Stellar address spaces
  DualAddress {
      evm: BytesN<20>,        // 0x742d35Cc6aF4B3... 
      stellar: Address,       // GCAXA5L4T7G2E7R...
  }

  // Factory stores mappings
  immutables::map_evm_to_stellar(env, evm_addr,
  stellar_addr);

  // Runtime resolution  
  let stellar_token = immutables::get_stellar_addr(env,
  &immutables.token.evm)
      .ok_or(Error::AddressMappingMissing)?;

  Phase 6: Security & Atomicity Guarantees

  Cross-Chain Consistency:

  Guarantee: Same order_hash = Same swap across chains
  ├─ EVM escrow uses order_hash as identifier
  ├─ Stellar factory uses order_hash as storage key  
  ├─ Prevents duplicate deployments: ❌ 
  InvalidImmutables
  └─ Links both sides of atomic swap

  Atomicity via Cryptographic Commitment:

  Secret-based atomicity:
  1. secret is known only to maker initially
  2. hashlock = keccak256(secret) locks both sides
  3. Revealing secret on ANY chain enables claim on BOTH
   chains
  4. Time locks ensure eventual consistency (cancel if
  stuck)

  Phase 7: Complete Flow Example

  Alice (EVM) ↔ Bob (Stellar) Token Swap

  1. Setup:
     order_hash = 0x123...
     secret = random_32_bytes  
     hashlock = keccak256(secret)

  2. Deploy:
     EVM: create_escrow(order_hash, hashlock, alice→bob,
   1000_USDC)
     Stellar: Factory.create_dst_escrow(order_hash, 
  hashlock, bob→alice, 2000_XLM)

  3. Execution:
     Bob → EVM.withdraw(secret)     // Reveals secret,
  gets 1000 USDC
     Alice → Factory.withdraw(order_hash, secret)  // 
  Gets 2000 XLM

  4. Result: 
     ✅ Alice: 1000 USDC → 2000 XLM
     ✅ Bob: 2000 XLM → 1000 USDC  
     ✅ Atomic swap completed

  Key Architectural Insights:

  1. order_hash is the cross-chain bridge - ensures both
   contracts reference the same logical swap
  2. Factory singleton pattern - one contract manages
  many swaps efficiently
  3. Dual address mapping - seamlessly bridges
  EVM/Stellar address spaces
  4. Cryptographic atomicity - secret revelation makes
  both sides claimable
  5. State isolation - each swap's state is completely
  independent
  6. Time-based safety - locks prevent indefinite
  hanging states

  This architecture achieves true atomic swaps across
  EVM and Stellar networks using the factory as a
  stateful coordinator on the Stellar side.
