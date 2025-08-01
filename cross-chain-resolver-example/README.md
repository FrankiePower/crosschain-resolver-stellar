# cross-chain-resolver-example

## Overview

This example demonstrates cross-chain atomic swaps between Ethereum (EVM) and Stellar networks using the 1inch Cross-Chain SDK. The system enables users to swap ETH/USDC on Ethereum for XLM/USDC on Stellar through a resolver-based bridge.

## Installation

Install example deps

```shell
pnpm install
```

Install [foundry](https://book.getfoundry.sh/getting-started/installation)

```shell
curl -L https://foundry.paradigm.xyz | bash
```

Install contract deps

```shell
forge install
```

## Running

### EVM→Stellar Cross-Chain Tests

The tests now support EVM to Stellar cross-chain swaps. To run:

```shell
SRC_CHAIN_RPC=ETH_FORK_URL DST_CHAIN_RPC=BNB_FORK_URL pnpm test
```

Note: DST_CHAIN_RPC is still required for 1inch SDK compatibility, but actual destination settlement happens on Stellar testnet.

### Public rpc

| Chain    | Url                          |
|----------|------------------------------|
| Ethereum | https://eth.merkle.io        |
| BSC      | wss://bsc-rpc.publicnode.com |

## Stellar Configuration

### Generate Stellar Test Keypair

If you need to generate a new Stellar test keypair for development:

```bash
node -e "const StellarSdk = require('@stellar/stellar-sdk'); const keypair = StellarSdk.Keypair.random(); console.log('Public key:', keypair.publicKey()); console.log('Secret key:', keypair.secret());"
```

### Fund Test Account

After generating a new keypair, fund it using Stellar's Friendbot:

```bash
# Replace PUBLIC_KEY with your generated public key
curl "https://friendbot.stellar.org?addr=PUBLIC_KEY"
```

Example:
```bash
curl "https://friendbot.stellar.org?addr=GAGDEHLKL52PLPPW5DSGUP5TAKS2KUFJ7SY2QIBAMWD5YJZI7QR5Y33V"
```

### Current Test Keypair

**⚠️ FOR TESTING ONLY - DO NOT USE IN PRODUCTION**

- **Public Key**: `GAGDEHLKL52PLPPW5DSGUP5TAKS2KUFJ7SY2QIBAMWD5YJZI7QR5Y33V`
- **Secret Key**: `SAHN2KFIGYCYNZ6CIAWJEEWXF2QKJKMMJZZ5GNBUN2U6QYWB6ZNR2HVV`

This keypair is used as the resolver's Stellar account for:
- Calling `fund_escrow()` to deposit XLM to contracts via SAC
- Signing Stellar transactions 
- Acting as the taker (resolver) in destination chain operations
- Cross-chain bridge funding operations

### Stellar Network Details

- **Network**: Stellar Testnet
- **Horizon URL**: `https://horizon-testnet.stellar.org`
- **Soroban RPC URL**: `https://soroban-testnet.stellar.org`
- **Network Passphrase**: `Test SDF Network ; September 2015`
- **Factory Contract**: `CBB3ONF3Q5LXIAATDL7PXBCEWIBJTD75SWVP2EYHHC2FD6UNNJ5ENCJD` (v3 - with authorization fix)
- **XLM SAC Contract**: `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`

### Cross-Chain Bridge Architecture

**EVM → Stellar Flow:**
1. User creates limit order on Ethereum (EVM chain)
2. Resolver fills order and creates source escrow 
3. Resolver calls `create_dst_escrow()` on Stellar factory
4. **NEW**: Resolver calls `fund_escrow()` to deposit XLM via SAC
5. User calls `withdraw()` with secret to claim XLM from Stellar
6. Resolver uses revealed secret to claim tokens from EVM source escrow

**Key Components:**
- **1inch SDK**: Handles EVM-side order creation and fulfillment
- **Stellar Factory**: Manages destination escrows and XLM custody
- **XLM SAC**: Stellar Asset Contract for native XLM token operations
- **Address Mapping**: EVM hex addresses ↔ Stellar addresses

## Test accounts

### Available Accounts

```
(0) 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266" Owner of EscrowFactory
(1) 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8" User
(2) 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a: "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC" Resolver
```
