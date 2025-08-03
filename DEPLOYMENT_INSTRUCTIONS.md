# Deployment Instructions - SAC Admin Architecture

## Problem Solved: Classic Asset Trustline Requirements

**Issue**: Soroban contracts cannot directly hold classic Stellar assets (like classic USDC) due to trustline requirements.

**Solution**: SAC Admin architecture using custom tokens with factory contract as administrator.

## Complete Deployment Commands

### 1. Generate Custom USDC Issuer
```bash
stellar keys generate --network testnet stellarUSDC2
stellar keys fund stellarUSDC2 --network testnet
stellar keys address stellarUSDC2
# Result: GBWLGSKYMM52ZNPH4NJRA4HSTZFHXA6EYVEEEPRF2L32JP6ZDN4MZS33
```

### 2. Deploy Custom SAC for SUSDC
```bash
stellar contract asset deploy --source stellarUSDC2 --network testnet --asset SUSDC2:GBWLGSKYMM52ZNPH4NJRA4HSTZFHXA6EYVEEEPRF2L32JP6ZDN4MZS33
# Result: CCDLGXP5T3VZZ2SJ3TXLQC2KNELSU3NP3B6CXTEY3CC2TBXCEGZTRVS7
```

### 3. Build Updated Factory Contract
```bash
cd stellar-resolver
stellar contract build
```

**Key Changes in Factory Contract (`factory/src/lib.rs`)**:
```rust
// OLD: Transfer existing tokens (failed due to trustlines)
let token_client = token::Client::new(&env, &stellar_token);
token_client.transfer(&from, &env.current_contract_address(), &amount);

// NEW: Mint tokens to factory (escrow holds them)
let token_client = token::StellarAssetClient::new(&env, &stellar_token);
token_client.mint(&env.current_contract_address(), &amount);
```

### 4. Deploy Factory v5 with SAC Admin Powers
```bash
stellar contract deploy --wasm target/wasm32v1-none/release/factory.wasm --source franky --network testnet --alias crosschain_factory_v5 -- --admin GAGDEHLKL52PLPPW5DSGUP5TAKS2KUFJ7SY2QIBAMWD5YJZI7QR5Y33V --rescue_delay 86400
# Result: CDOQ3UNUOZVPGUF3VIO4XMOTN5LVMBOAECRMIV3C7PEBDMB4D6GQRUNM
```

### 5. Create Final SAC with Franky as Issuer (for permissions)
```bash
stellar contract asset deploy --source franky --network testnet --asset SUSDC3:GBL4FMN3MPLPA2IS7T2K5VAGGVT4WJWJ24YXYFAHIFOGGCVEM6WVVAQA
# Result: CCLWDDFJLRCZK3JCVUJVCYEK5EQQFPOQDAPNQWNQFGLKZW2QP6YNWT5G
```

### 6. Set Factory as SAC Administrator
```bash
stellar contract invoke --source franky --network testnet --id CCLWDDFJLRCZK3JCVUJVCYEK5EQQFPOQDAPNQWNQFGLKZW2QP6YNWT5G -- set_admin --new_admin CDOQ3UNUOZVPGUF3VIO4XMOTN5LVMBOAECRMIV3C7PEBDMB4D6GQRUNM
# Result: ✅ Factory now has SAC admin powers
```

### 7. Fix SDK Compatibility Issue

**Problem**: "Bad union switch: 4" error after testnet p23 upgrade

**Solution**: Update to SDK v14.0.0-rc.3

**Main Package** (`evm-cross-chain-resolver/package.json`):
```json
"@stellar/stellar-sdk": "14.0.0-rc.3"
```

**Bindings Package** (`stellar-bindings/factory/package.json`):
```json
"@stellar/stellar-sdk": "14.0.0-rc.3"
```

### 8. Generate New TypeScript Bindings
```bash
stellar contract bindings typescript --contract-id CDOQ3UNUOZVPGUF3VIO4XMOTN5LVMBOAECRMIV3C7PEBDMB4D6GQRUNM --output-dir ../evm-cross-chain-resolver/stellar-bindings/factory --overwrite --network testnet
```

**Fix DataKey conflicts in generated bindings**:
```typescript
// Change duplicate DataKey types to unique names
export type TimelockDataKey = {tag: "Timelocks", values: void};
export type ImmutablesDataKey = {tag: "AddressMap", values: void} | {tag: "ImmutablesData", values: void};
export type BaseEscrowDataKey = {tag: "RescueDelay", values: void} | {tag: "Factory", values: void} | {tag: "Immutables", values: void};
```

### 9. Rebuild Bindings with Updated SDK
```bash
cd evm-cross-chain-resolver/stellar-bindings/factory
npm install && npm run build
```

### 10. Update Test Configuration

**Config Updates** (`tests/config.ts`):
```typescript
// Update factory contract
escrowFactory: 'CDOQ3UNUOZVPGUF3VIO4XMOTN5LVMBOAECRMIV3C7PEBDMB4D6GQRUNM',

// Update SAC token  
stellarContractId: 'CCLWDDFJLRCZK3JCVUJVCYEK5EQQFPOQDAPNQWNQFGLKZW2QP6YNWT5G'
```

**Test Updates** (`tests/main.spec.ts`):
```typescript
// Add trustline creation for resolver
const susdc3Asset = new StellarSdk.Asset('SUSDC3', 'GBL4FMN3MPLPA2IS7T2K5VAGGVT4WJWJ24YXYFAHIFOGGCVEM6WVVAQA')
// ... trustline creation code
```

## Architecture Summary

### SAC Admin Flow
1. **Factory mints tokens** to itself (acts as escrow)
2. **User withdraws** with secret → tokens transfer from factory to user
3. **No trustlines required** for minting (SAC admin bypass)
4. **Atomic guarantees maintained** through secret revelation

### Cross-Chain Flow
1. **EVM**: User locks 100 USDC → Resolver fills order
2. **Stellar**: Factory mints 99 SUSDC3 to itself (escrow)
3. **User reveals secret** on Stellar → gets 99 SUSDC3
4. **Resolver uses revealed secret** on EVM → gets 100 USDC

## Deployed Contracts

- **Factory v5**: `CDOQ3UNUOZVPGUF3VIO4XMOTN5LVMBOAECRMIV3C7PEBDMB4D6GQRUNM`
- **SUSDC3 SAC**: `CCLWDDFJLRCZK3JCVUJVCYEK5EQQFPOQDAPNQWNQFGLKZW2QP6YNWT5G`
- **SUSDC3 Issuer**: `GBL4FMN3MPLPA2IS7T2K5VAGGVT4WJWJ24YXYFAHIFOGGCVEM6WVVAQA`

## Key Breakthrough

This architecture solves the fundamental limitation that **"Soroban contracts cannot directly hold classic Stellar assets"** by using SAC admin powers to mint tokens directly to the contract, bypassing trustline requirements while maintaining atomic swap guarantees.