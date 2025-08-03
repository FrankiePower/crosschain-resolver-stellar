import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}


export const networks = {
  testnet: {
    networkPassphrase: "Test SDF Network ; September 2015",
    contractId: "CDOQ3UNUOZVPGUF3VIO4XMOTN5LVMBOAECRMIV3C7PEBDMB4D6GQRUNM",
  }
} as const

export type EscrowDataKey = {tag: "EscrowState", values: readonly [Buffer]} | {tag: "EscrowStage", values: readonly [Buffer]};

export type EscrowType = {tag: "Source", values: void} | {tag: "Destination", values: void};

export type EscrowStage = {tag: "Created", values: void} | {tag: "Withdrawn", values: void} | {tag: "Cancelled", values: void} | {tag: "Rescued", values: void};

/**
 * Timelock stages - must match Solidity exactly
 */
export enum Stage {
  SrcWithdrawal = 0,
  SrcPublicWithdrawal = 1,
  SrcCancellation = 2,
  SrcPublicCancellation = 3,
  DstWithdrawal = 4,
  DstPublicWithdrawal = 5,
  DstCancellation = 6,
}


/**
 * Timelocks - packed into single U256 value to match Solidity exactly
 * This MUST match the Solidity TimelocksLib.sol bit packing exactly
 */
export interface Timelocks {
  /**
 * Single U256 value containing all timelock data
 * Packed exactly like Solidity's uint256 timelocks
 */
packed_value: u256;
}

/**
 * Storage key for timelock data
 */
export type TimelockDataKey = {tag: "Timelocks", values: void};

export const TimeLockError = {
  1: {message:"RescueStartOverflow"},
  2: {message:"TimelockValueOverflow"},
  3: {message:"DeploymentTimestampNotSet"},
  4: {message:"InvalidSourceChainTimelockOrdering"},
  5: {message:"InvalidDestinationChainTimelockOrdering"},
  6: {message:"TimelockOffsetTooLarge"}
}


export interface DualAddress {
  evm: Buffer;
  stellar: string;
}


export interface Immutables {
  amount: i128;
  hashlock: Buffer;
  maker: DualAddress;
  order_hash: Buffer;
  safety_deposit: i128;
  taker: DualAddress;
  timelocks: Timelocks;
  token: DualAddress;
}

export type ImmutablesDataKey = {tag: "AddressMap", values: void} | {tag: "ImmutablesData", values: void};

export type BaseEscrowDataKey = {tag: "RescueDelay", values: void} | {tag: "Factory", values: void} | {tag: "Immutables", values: void};

export const Errors = {
  1: {message:"InvalidCaller"},
  2: {message:"InvalidImmutables"},
  3: {message:"InvalidSecret"},
  4: {message:"InvalidTime"},
  5: {message:"NativeTokenSendingFailure"},
  6: {message:"AddressMappingMissing"},
  7: {message:"TimeLockError"}
}

export interface Client {
  /**
   * Construct and simulate a create_src_escrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Create source chain escrow - stores immutables and returns factory address
   */
  create_src_escrow: ({immutables}: {immutables: Immutables}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a create_dst_escrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Create destination chain escrow - stores immutables and accepts XLM funding
   */
  create_dst_escrow: ({immutables}: {immutables: Immutables}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a fund_escrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Fund escrow with tokens (resolver deposits funds for user withdrawal)
   */
  fund_escrow: ({order_hash, from, amount}: {order_hash: Buffer, from: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Withdraw from escrow using secret (order_hash is the key)
   */
  withdraw: ({order_hash, secret}: {order_hash: Buffer, secret: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cancel transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Cancel escrow (maker only, after timelock) - order_hash is the key
   */
  cancel: ({order_hash}: {order_hash: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a rescue_funds transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Rescue funds (taker only, after rescue delay) - order_hash is the key
   */
  rescue_funds: ({order_hash, amount}: {order_hash: Buffer, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_escrow_state transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get escrow state by order_hash
   */
  get_escrow_state: ({order_hash}: {order_hash: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<readonly [EscrowType, Immutables]>>>

  /**
   * Construct and simulate a get_escrow_stage transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Get escrow stage by order_hash
   */
  get_escrow_stage: ({order_hash}: {order_hash: Buffer}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<EscrowStage>>

  /**
   * Construct and simulate a get_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Admin functions
   */
  get_admin: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_rescue_delay transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_rescue_delay: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u64>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, rescue_delay}: {admin: string, rescue_delay: u64},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({admin, rescue_delay}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAgAAAAAAAAAAAAAADUVzY3Jvd0RhdGFLZXkAAAAAAAACAAAAAQAAAAAAAAALRXNjcm93U3RhdGUAAAAAAQAAA+4AAAAgAAAAAQAAAAAAAAALRXNjcm93U3RhZ2UAAAAAAQAAA+4AAAAg",
        "AAAAAgAAAAAAAAAAAAAACkVzY3Jvd1R5cGUAAAAAAAIAAAAAAAAAAAAAAAZTb3VyY2UAAAAAAAAAAAAAAAAAC0Rlc3RpbmF0aW9uAA==",
        "AAAAAgAAAAAAAAAAAAAAC0VzY3Jvd1N0YWdlAAAAAAQAAAAAAAAAAAAAAAdDcmVhdGVkAAAAAAAAAAAAAAAACVdpdGhkcmF3bgAAAAAAAAAAAAAAAAAACUNhbmNlbGxlZAAAAAAAAAAAAAAAAAAAB1Jlc2N1ZWQA",
        "AAAAAAAAADZJbml0aWFsaXplIGZhY3Rvcnkgd2l0aCBhZG1pbiBhbmQgZGVmYXVsdCByZXNjdWUgZGVsYXkAAAAAAA1fX2NvbnN0cnVjdG9yAAAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAxyZXNjdWVfZGVsYXkAAAAGAAAAAA==",
        "AAAAAAAAAEpDcmVhdGUgc291cmNlIGNoYWluIGVzY3JvdyAtIHN0b3JlcyBpbW11dGFibGVzIGFuZCByZXR1cm5zIGZhY3RvcnkgYWRkcmVzcwAAAAAAEWNyZWF0ZV9zcmNfZXNjcm93AAAAAAAAAQAAAAAAAAAKaW1tdXRhYmxlcwAAAAAH0AAAAApJbW11dGFibGVzAAAAAAABAAAD6QAAABMAAAAD",
        "AAAAAAAAAEtDcmVhdGUgZGVzdGluYXRpb24gY2hhaW4gZXNjcm93IC0gc3RvcmVzIGltbXV0YWJsZXMgYW5kIGFjY2VwdHMgWExNIGZ1bmRpbmcAAAAAEWNyZWF0ZV9kc3RfZXNjcm93AAAAAAAAAQAAAAAAAAAKaW1tdXRhYmxlcwAAAAAH0AAAAApJbW11dGFibGVzAAAAAAABAAAD6QAAABMAAAAD",
        "AAAAAAAAAEVGdW5kIGVzY3JvdyB3aXRoIHRva2VucyAocmVzb2x2ZXIgZGVwb3NpdHMgZnVuZHMgZm9yIHVzZXIgd2l0aGRyYXdhbCkAAAAAAAALZnVuZF9lc2Nyb3cAAAAAAwAAAAAAAAAKb3JkZXJfaGFzaAAAAAAD7gAAACAAAAAAAAAABGZyb20AAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAAAw==",
        "AAAAAAAAADlXaXRoZHJhdyBmcm9tIGVzY3JvdyB1c2luZyBzZWNyZXQgKG9yZGVyX2hhc2ggaXMgdGhlIGtleSkAAAAAAAAId2l0aGRyYXcAAAACAAAAAAAAAApvcmRlcl9oYXNoAAAAAAPuAAAAIAAAAAAAAAAGc2VjcmV0AAAAAAPuAAAAIAAAAAEAAAPpAAAD7QAAAAAAAAAD",
        "AAAAAAAAAEJDYW5jZWwgZXNjcm93IChtYWtlciBvbmx5LCBhZnRlciB0aW1lbG9jaykgLSBvcmRlcl9oYXNoIGlzIHRoZSBrZXkAAAAAAAZjYW5jZWwAAAAAAAEAAAAAAAAACm9yZGVyX2hhc2gAAAAAA+4AAAAgAAAAAQAAA+kAAAPtAAAAAAAAAAM=",
        "AAAAAAAAAEVSZXNjdWUgZnVuZHMgKHRha2VyIG9ubHksIGFmdGVyIHJlc2N1ZSBkZWxheSkgLSBvcmRlcl9oYXNoIGlzIHRoZSBrZXkAAAAAAAAMcmVzY3VlX2Z1bmRzAAAAAgAAAAAAAAAKb3JkZXJfaGFzaAAAAAAD7gAAACAAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAAD",
        "AAAAAAAAAB5HZXQgZXNjcm93IHN0YXRlIGJ5IG9yZGVyX2hhc2gAAAAAABBnZXRfZXNjcm93X3N0YXRlAAAAAQAAAAAAAAAKb3JkZXJfaGFzaAAAAAAD7gAAACAAAAABAAAD6QAAA+0AAAACAAAH0AAAAApFc2Nyb3dUeXBlAAAAAAfQAAAACkltbXV0YWJsZXMAAAAAAAM=",
        "AAAAAAAAAB5HZXQgZXNjcm93IHN0YWdlIGJ5IG9yZGVyX2hhc2gAAAAAABBnZXRfZXNjcm93X3N0YWdlAAAAAQAAAAAAAAAKb3JkZXJfaGFzaAAAAAAD7gAAACAAAAABAAAH0AAAAAtFc2Nyb3dTdGFnZQA=",
        "AAAAAAAAAA9BZG1pbiBmdW5jdGlvbnMAAAAACWdldF9hZG1pbgAAAAAAAAAAAAABAAAAEw==",
        "AAAAAAAAAAAAAAAQZ2V0X3Jlc2N1ZV9kZWxheQAAAAAAAAABAAAABg==",
        "AAAAAwAAAC1UaW1lbG9jayBzdGFnZXMgLSBtdXN0IG1hdGNoIFNvbGlkaXR5IGV4YWN0bHkAAAAAAAAAAAAABVN0YWdlAAAAAAAABwAAAAAAAAANU3JjV2l0aGRyYXdhbAAAAAAAAAAAAAAAAAAAE1NyY1B1YmxpY1dpdGhkcmF3YWwAAAAAAQAAAAAAAAAPU3JjQ2FuY2VsbGF0aW9uAAAAAAIAAAAAAAAAFVNyY1B1YmxpY0NhbmNlbGxhdGlvbgAAAAAAAAMAAAAAAAAADURzdFdpdGhkcmF3YWwAAAAAAAAEAAAAAAAAABNEc3RQdWJsaWNXaXRoZHJhd2FsAAAAAAUAAAAAAAAAD0RzdENhbmNlbGxhdGlvbgAAAAAG",
        "AAAAAQAAAIVUaW1lbG9ja3MgLSBwYWNrZWQgaW50byBzaW5nbGUgVTI1NiB2YWx1ZSB0byBtYXRjaCBTb2xpZGl0eSBleGFjdGx5ClRoaXMgTVVTVCBtYXRjaCB0aGUgU29saWRpdHkgVGltZWxvY2tzTGliLnNvbCBiaXQgcGFja2luZyBleGFjdGx5AAAAAAAAAAAAAAlUaW1lbG9ja3MAAAAAAAABAAAAX1NpbmdsZSBVMjU2IHZhbHVlIGNvbnRhaW5pbmcgYWxsIHRpbWVsb2NrIGRhdGEKUGFja2VkIGV4YWN0bHkgbGlrZSBTb2xpZGl0eSdzIHVpbnQyNTYgdGltZWxvY2tzAAAAAAxwYWNrZWRfdmFsdWUAAAAM",
        "AAAAAgAAAB1TdG9yYWdlIGtleSBmb3IgdGltZWxvY2sgZGF0YQAAAAAAAAAAAAAHRGF0YUtleQAAAAABAAAAAAAAAAAAAAAJVGltZWxvY2tzAAAA",
        "AAAABAAAAAAAAAAAAAAADVRpbWVMb2NrRXJyb3IAAAAAAAAGAAAAAAAAABNSZXNjdWVTdGFydE92ZXJmbG93AAAAAAEAAAAAAAAAFVRpbWVsb2NrVmFsdWVPdmVyZmxvdwAAAAAAAAIAAAAAAAAAGURlcGxveW1lbnRUaW1lc3RhbXBOb3RTZXQAAAAAAAADAAAAAAAAACJJbnZhbGlkU291cmNlQ2hhaW5UaW1lbG9ja09yZGVyaW5nAAAAAAAEAAAAAAAAACdJbnZhbGlkRGVzdGluYXRpb25DaGFpblRpbWVsb2NrT3JkZXJpbmcAAAAABQAAAAAAAAAWVGltZWxvY2tPZmZzZXRUb29MYXJnZQAAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAAC0R1YWxBZGRyZXNzAAAAAAIAAAAAAAAAA2V2bQAAAAPuAAAAFAAAAAAAAAAHc3RlbGxhcgAAAAAT",
        "AAAAAQAAAAAAAAAAAAAACkltbXV0YWJsZXMAAAAAAAgAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAIaGFzaGxvY2sAAAPuAAAAIAAAAAAAAAAFbWFrZXIAAAAAAAfQAAAAC0R1YWxBZGRyZXNzAAAAAAAAAAAKb3JkZXJfaGFzaAAAAAAD7gAAACAAAAAAAAAADnNhZmV0eV9kZXBvc2l0AAAAAAALAAAAAAAAAAV0YWtlcgAAAAAAB9AAAAALRHVhbEFkZHJlc3MAAAAAAAAAAAl0aW1lbG9ja3MAAAAAAAfQAAAACVRpbWVsb2NrcwAAAAAAAAAAAAAFdG9rZW4AAAAAAAfQAAAAC0R1YWxBZGRyZXNzAA==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAgAAAAAAAAAAAAAACkFkZHJlc3NNYXAAAAAAAAAAAAAAAAAADkltbXV0YWJsZXNEYXRhAAA=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAwAAAAAAAAAAAAAAC1Jlc2N1ZURlbGF5AAAAAAAAAAAAAAAAB0ZhY3RvcnkAAAAAAAAAAAAAAAAKSW1tdXRhYmxlcwAA",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAABwAAAAAAAAANSW52YWxpZENhbGxlcgAAAAAAAAEAAAAAAAAAEUludmFsaWRJbW11dGFibGVzAAAAAAAAAgAAAAAAAAANSW52YWxpZFNlY3JldAAAAAAAAAMAAAAAAAAAC0ludmFsaWRUaW1lAAAAAAQAAAAAAAAAGU5hdGl2ZVRva2VuU2VuZGluZ0ZhaWx1cmUAAAAAAAAFAAAAAAAAABVBZGRyZXNzTWFwcGluZ01pc3NpbmcAAAAAAAAGAAAAAAAAAA1UaW1lTG9ja0Vycm9yAAAAAAAABw==" ]),
      options
    )
  }
  public readonly fromJSON = {
    create_src_escrow: this.txFromJSON<Result<string>>,
        create_dst_escrow: this.txFromJSON<Result<string>>,
        fund_escrow: this.txFromJSON<Result<void>>,
        withdraw: this.txFromJSON<Result<void>>,
        cancel: this.txFromJSON<Result<void>>,
        rescue_funds: this.txFromJSON<Result<void>>,
        get_escrow_state: this.txFromJSON<Result<readonly [EscrowType, Immutables]>>,
        get_escrow_stage: this.txFromJSON<EscrowStage>,
        get_admin: this.txFromJSON<string>,
        get_rescue_delay: this.txFromJSON<u64>
  }
}