# Stellar Cross-Chain Escrow Factory

A Soroban smart contract system for enabling cross-chain atomic swaps between EVM chains and Stellar.

## Project Structure

```text
.
├── contracts/
│   ├── factory/          # Main factory contract (manages all escrows)
│   ├── src-escrow/       # Source escrow logic
│   ├── dst-escrow/       # Destination escrow logic  
│   └── shared/           # Common libraries and types
├── Cargo.toml
└── README.md
```

## Architecture

Unlike EVM chains that use minimal proxies, Stellar contracts manage escrow states internally:

- **Factory Contract**: Single contract that manages all escrow operations
- **Internal State Management**: Each escrow identified by `order_hash` 
- **Cross-Chain Support**: Maps EVM addresses to Stellar addresses
- **Timelock System**: Preserves hashlock and timelock functionality for atomic swaps

## Contract Deployment

### Prerequisites
- Stellar CLI installed and configured
- Identity configured (e.g., "franky")
- Testnet account with funding

### Build Contracts

Build all contracts from workspace root:
```bash
cd stellar-resolver
stellar contract build
```

**Expected Output:**
```
✅ Build Complete
   Wasm File: target/wasm32v1-none/release/dst_escrow.wasm (7 functions)
   Wasm File: target/wasm32v1-none/release/src_escrow.wasm (6 functions)  
   Wasm File: target/wasm32v1-none/release/factory.wasm (11 functions)
```

### Deploy Factory Contract

The factory contract contains all escrow functionality:

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/factory.wasm \
  --source franky \
  --network testnet \
  --alias crosschain_factory \
  -- \
  --admin GBL4FMN3MPLPA2IS7T2K5VAGGVT4WJWJ24YXYFAHIFOGGCVEM6WVVAQA \
  --rescue_delay 86400
```

**Important:** Constructor arguments are required:
- `admin`: Admin address for factory management
- `rescue_delay`: Emergency rescue delay in seconds (86400 = 24 hours)

### Deployment Result

**Current Testnet Deployment:**
- **Contract ID**: `CBY2HYKT4A66AR4RGCWILHDDSOT2DU7LHZ32IQD4GGET6X6EMPPQ4FYA`
- **Network**: Stellar Testnet
- **Admin**: `GBL4FMN3MPLPA2IS7T2K5VAGGVT4WJWJ24YXYFAHIFOGGCVEM6WVVAQA`
- **Explorer**: https://stellar.expert/explorer/testnet/contract/CBY2HYKT4A66AR4RGCWILHDDSOT2DU7LHZ32IQD4GGET6X6EMPPQ4FYA

### Verify Deployment

Test contract functionality:

```bash
# Check admin address
stellar contract invoke \
  --id CBY2HYKT4A66AR4RGCWILHDDSOT2DU7LHZ32IQD4GGET6X6EMPPQ4FYA \
  --source franky \
  --network testnet \
  -- get_admin

# Expected: "GBL4FMN3MPLPA2IS7T2K5VAGGVT4WJWJ24YXYFAHIFOGGCVEM6WVVAQA"

# Check rescue delay
stellar contract invoke \
  --id CBY2HYKT4A66AR4RGCWILHDDSOT2DU7LHZ32IQD4GGET6X6EMPPQ4FYA \
  --source franky \
  --network testnet \
  -- get_rescue_delay

# Expected: 86400
```

## Available Functions

The deployed factory contract provides:

### Core Escrow Operations
- `create_src_escrow(immutables)` - Create source chain escrow
- `create_dst_escrow(immutables)` - Create destination chain escrow  
- `withdraw(order_hash, secret)` - Withdraw funds with secret
- `cancel(order_hash, immutables)` - Cancel escrow operation

### Management Functions  
- `get_admin()` - Get factory admin address
- `get_rescue_delay()` - Get rescue delay setting
- `get_escrow_state(order_hash)` - Get escrow state by order hash
- `get_escrow_stage(order_hash)` - Get escrow stage (Created/Withdrawn/Cancelled)
- `rescue_funds(order_hash, immutables)` - Emergency rescue function

## Common Issues & Solutions

### Build Errors
```bash
# If you get "can't find crate for 'core'" error:
rustup target add wasm32v1-none
# For older Rust versions:
rustup target add wasm32-unknown-unknown
```

### Deployment Errors
```bash
# Error: "Missing argument admin"
# Solution: Include constructor arguments after --
stellar contract deploy --wasm factory.wasm --source franky --network testnet \
  -- --admin YOUR_ADDRESS --rescue_delay 86400
```

### Initial Deploy Error (First Attempt)
```bash
# This failed because constructor args were missing:
stellar contract deploy --wasm factory.wasm --source franky --network testnet --alias crosschain_factory

# Error: "Missing argument admin"
# Fix: Add constructor arguments after the -- separator
```

### Integration Notes
- Factory manages escrows internally (no separate contract instances)
- Each escrow identified by unique `order_hash`
- Cross-chain address mapping required for EVM ↔ Stellar integration
- Timelock system maintains atomic swap guarantees

## Development Identity

**Franky Identity:**
- **Public Key**: `GBL4FMN3MPLPA2IS7T2K5VAGGVT4WJWJ24YXYFAHIFOGGCVEM6WVVAQA`
- **Network**: Testnet
- **Usage**: Contract deployment and testing

## Cross-Chain Integration

This factory is designed for integration with 1inch Fusion+ to enable EVM ↔ Stellar swaps:

### EVM → Stellar Flow
1. User creates order on EVM chain (Ethereum)
2. Resolver fills order, creating source escrow
3. **Handoff Point**: EVM immutables → Stellar factory `create_dst_escrow()`
4. Factory creates destination escrow on Stellar
5. User withdraws from Stellar using secret
6. Resolver withdraws from EVM using revealed secret

### Key Integration Points
- **Source Chain**: Standard EVM with 1inch Fusion+ SDK
- **Destination Chain**: This Stellar factory contract
- **Data Bridge**: Convert 1inch SDK types to Stellar contract parameters
- **Address Mapping**: EVM addresses ↔ Stellar addresses via factory mapping system

## Testing Commands

Quick verification commands for deployed contract:

```bash
# Set contract alias for easier reference
FACTORY_ID="CBY2HYKT4A66AR4RGCWILHDDSOT2DU7LHZ32IQD4GGET6X6EMPPQ4FYA"

# Test basic functionality
stellar contract invoke --id $FACTORY_ID --source franky --network testnet -- get_admin
stellar contract invoke --id $FACTORY_ID --source franky --network testnet -- get_rescue_delay

# For actual escrow operations, you'll need proper immutables data structure
# See contracts/factory/src/test.rs for examples
```