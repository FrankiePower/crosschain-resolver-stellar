# 1inch Fusion+ Cross-Chain Resolver for Stellar

## Overview

This project implements **1inch Fusion+ for cross-chain atomic swaps** between EVM and Stellar networks. It adapts 1inch's intent-based atomic swap mechanism to handle the architectural differences between EVM smart contracts and Stellar's contract environment.

## Architecture

### Core Concept: Dual Address System

Since EVM and Stellar use different address formats, users provide **separate receiving addresses** for each chain:

- **EVM Address**: `0x70997970c51812dc3a010c7d01b50e0d17dc79c8` (20 bytes)
- **Stellar Address**: `GAGDEHLKL52PLPPW5DSGUP5TAKS2KUFJ7SY2QIBAMWD5YJZI7QR5Y33V` (32 bytes)

**Key Point**: Users specify where they want to receive funds on each chain, and balance verification checks their designated receiving addresses.

## Cross-Chain Atomic Swap Flow

### Phase 1: Order Creation & Address Specification

**User Intent**: Swap EVM tokens for Stellar tokens

```javascript
User provides:
├─ EVM Address: 0x742d35Cc... (where they want to send from)
├─ Stellar Address: GCAXA5L4T7G2E7R... (where they want to receive)
├─ Amount: 100 USDC → 99 XLM
└─ Secret: random_32_bytes (for atomic commitment)
```

### Phase 2: Escrow Deployment

**EVM Chain (Source)**
```solidity
1. Deploy EVM escrow contract:
   - order_hash (unique identifier)
   - hashlock = keccak256(secret)
   - maker EVM address (user's sending address)
   - taker EVM address (resolver's address)
   - 100 USDC locked in escrow
```

**Stellar Chain (Destination)**  
```rust
2. Factory.create_dst_escrow():
   - Same order_hash (key link!)
   - Same hashlock
   - DualAddress mapping:
     * evm: user's EVM address
     * stellar: user's STELLAR receiving address
   - Amount: 99 XLM equivalent
```

### Phase 3: Resolver Funding

**Critical Step**: Resolver funds the Stellar escrow with actual XLM

```rust
3. Factory.fund_escrow():
   - order_hash: links to created escrow
   - from: resolver's Stellar address
   - amount: 99 XLM (in stroops)
   
   // XLM moves from resolver → factory contract
```

### Phase 4: Atomic Execution

**Secret Revelation & Claims**

```javascript
4a. User withdraws from EVM escrow:
    - Calls withdraw(secret) on EVM
    - Reveals secret publicly on blockchain
    - Receives resolver's tokens on EVM

4b. User claims from Stellar escrow:  
    - Calls Factory.withdraw(order_hash, revealed_secret)
    - Factory validates secret matches hashlock
    - XLM transferred to user's Stellar receiving address
```

### Phase 5: Balance Verification

**Key Testing Point**: Check user's designated receiving addresses

```javascript
Balance checks:
├─ EVM: Check user's EVM address balance (should decrease)
├─ Stellar: Check user's STELLAR receiving address (should increase)
└─ NOT resolver addresses - they're just facilitators
```

## Address Flow Example

```
Alice wants to swap EVM USDC → Stellar XLM

Alice specifies:
├─ EVM address: 0xAlice123... (her EVM wallet)  
└─ Stellar address: GCAlice456... (her Stellar wallet)

Resolver facilitates:
├─ EVM: Resolver address 0xResolver789...
└─ Stellar: Resolver address GAResolver012...

Fund flows:
1. Alice's EVM → EVM Escrow (100 USDC)
2. Resolver's Stellar → Stellar Factory (99 XLM) 
3. EVM Escrow → Resolver's EVM (100 USDC, after secret reveal)
4. Stellar Factory → Alice's Stellar (99 XLM, after secret reveal)

Final result:
✅ Alice: Lost 100 USDC on EVM, Gained 99 XLM on Stellar
✅ Resolver: Gained 100 USDC on EVM, Lost 99 XLM on Stellar
✅ Atomic swap completed across chains
```

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

Perfect! The bindings are now working correctly. I've
  successfully:

  1. Generated TypeScript bindings for the Stellar contract using
   stellar contract bindings typescript
  2. Built the bindings package with npm install && npm run build
  3. Integrated the bindings into the test project by:
    - Adding the bindings as a local dependency in package.json
    - Importing the Client class (not Contract) and proper types
    - Using the correct constructor parameters for the Client
    - Converting data to the proper format using u256 and i128
  types
    - Calling contract methods using the generated API:
  stellarContract.create_dst_escrow({immutables: 
  stellarImmutables})

  The test is now successfully calling the Stellar contract
  method create_dst_escrow without type errors. The bindings
  provide a clean, typed interface to interact with your deployed
   Stellar contract, automatically handling the conversion
  between JavaScript/TypeScript types and Stellar's native types.

  The key learnings from this integration:
  - Use Client class, not Contract
  - Import proper types like u256, i128 from
  @stellar/stellar-sdk/contract
  - Pass data in the correct format expected by the contract spec
  - Use the generated method signatures for type safety

  Your Stellar contract bindings are now ready for use in the
  cross-chain resolver example!