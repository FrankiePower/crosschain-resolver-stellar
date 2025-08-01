import {z} from 'zod'
import Sdk from '@1inch/cross-chain-sdk'
import * as process from 'node:process'

// Chain type enumeration for EVM vs Stellar
export enum ChainType {
    EVM = 'evm',
    STELLAR = 'stellar'
}

const bool = z
    .string()
    .transform((v) => v.toLowerCase() === 'true')
    .pipe(z.boolean())

// Extended config schema with optional Stellar properties
const ConfigSchema = z.object({
    // Existing EVM configuration
    SRC_CHAIN_RPC: z.string().url(),
    DST_CHAIN_RPC: z.string().url(),
    SRC_CHAIN_CREATE_FORK: bool.default('true'),
    DST_CHAIN_CREATE_FORK: bool.default('true'),
    // Optional Stellar configuration
    STELLAR_NETWORK_PASSPHRASE: z.string().optional(),
    STELLAR_HORIZON_URL: z.string().url().optional(),
    STELLAR_SOROBAN_RPC_URL: z.string().url().optional(),
    STELLAR_ESCROW_FACTORY: z.string().optional(),
    STELLAR_PRIVATE_KEY: z.string().optional(),
    STELLAR_USDC_ADDRESS: z.string().optional()
})

const fromEnv = ConfigSchema.parse(process.env)

// Base chain configuration interface
interface BaseChainConfig {
    type: ChainType
    url: string
    ownerPrivateKey: string
    tokens: {
        [symbol: string]: {
            address: string
            donor?: string
        }
    }
}

// EVM-specific configuration (has Limit Order Protocol)
export interface EVMChainConfig extends BaseChainConfig {
    type: ChainType.EVM
    chainId: number
    createFork: boolean
    limitOrderProtocol: string
    wrappedNative: string
}

// Stellar-specific configuration (has escrow factory system)
export interface StellarChainConfig extends BaseChainConfig {
    type: ChainType.STELLAR
    chainId: number
    networkPassphrase: string
    horizonUrl: string
    sorobanRpcUrl: string
    escrowFactory: string
    // Note: No limitOrderProtocol - uses escrow factory instead
    // Note: No wrappedNative - uses XLM directly
}

// Union type for all supported chains
export type ChainConfig = EVMChainConfig | StellarChainConfig

// Helper functions for type checking
export function isEVMChain(chain: ChainConfig): chain is EVMChainConfig {
    return chain.type === ChainType.EVM
}

export function isStellarChain(chain: ChainConfig): chain is StellarChainConfig {
    return chain.type === ChainType.STELLAR
}

export const config = {
    chain: {
        // Source chain (Ethereum) - EVM with Limit Order Protocol
        source: {
            type: ChainType.EVM,
            chainId: Sdk.NetworkEnum.ETHEREUM,
            url: fromEnv.SRC_CHAIN_RPC,
            createFork: fromEnv.SRC_CHAIN_CREATE_FORK,
            limitOrderProtocol: '0x111111125421ca6dc452d289314280a0f8842a65',
            wrappedNative: '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2',
            ownerPrivateKey: '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
            tokens: {
                USDC: {
                    address: '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
                    donor: '0xd54F23BE482D9A58676590fCa79c8E43087f92fB'
                }
            }
        } as EVMChainConfig,
        
        // Destination chain (BSC) - EVM with Limit Order Protocol  
        destination: {
            type: ChainType.EVM,
            chainId: Sdk.NetworkEnum.BINANCE,
            url: fromEnv.DST_CHAIN_RPC,
            createFork: fromEnv.DST_CHAIN_CREATE_FORK,
            limitOrderProtocol: '0x111111125421ca6dc452d289314280a0f8842a65',
            wrappedNative: '0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c',
            ownerPrivateKey: '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
            tokens: {
                USDC: {
                    address: '0x8965349fb649a33a30cbfda057d8ec2c48abe2a2',
                    donor: '0x4188663a85C92EEa35b5AD3AA5cA7CeB237C6fe9'
                }
            }
        } as EVMChainConfig,

        // Stellar destination chain configuration - LIVE DEPLOYMENT
        stellar: {
            type: ChainType.STELLAR,
            chainId: 999999, // Custom numeric ID for 1inch SDK compatibility
            url: fromEnv.STELLAR_SOROBAN_RPC_URL || 'https://soroban-testnet.stellar.org',
            networkPassphrase: fromEnv.STELLAR_NETWORK_PASSPHRASE || 'Test SDF Network ; September 2015',
            horizonUrl: fromEnv.STELLAR_HORIZON_URL || 'https://horizon-testnet.stellar.org',
            sorobanRpcUrl: fromEnv.STELLAR_SOROBAN_RPC_URL || 'https://soroban-testnet.stellar.org',
            // 🎯 LATEST FACTORY CONTRACT v3 - With authorization fix for fund_escrow
            escrowFactory: fromEnv.STELLAR_ESCROW_FACTORY || 'CBB3ONF3Q5LXIAATDL7PXBCEWIBJTD75SWVP2EYHHC2FD6UNNJ5ENCJD',
            // 🔑 Test Stellar private key for resolver operations
            ownerPrivateKey: fromEnv.STELLAR_PRIVATE_KEY || 'SAHN2KFIGYCYNZ6CIAWJEEWXF2QKJKMMJZZ5GNBUN2U6QYWB6ZNR2HVV',
            tokens: {
                USDC: {
                    // EVM-compatible address for 1inch SDK compatibility only - actual Stellar settlement uses real contract
                    address: fromEnv.STELLAR_USDC_ADDRESS || '0x0000000000000000000000000000000000000001'
                },
                XLM: {
                    address: 'native'
                }
            }
        } as StellarChainConfig
    }
} as const

// Backward compatibility type - existing code continues to work
export type LegacyChainConfig = (typeof config.chain)['source' | 'destination']
