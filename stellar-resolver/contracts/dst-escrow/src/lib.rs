#![no_std]

// Destination Chain Escrow Contract
// Handles the destination side of cross-chain atomic swaps

mod dstescrow;

// Re-export the contract
pub use dstescrow::DstEscrow;

#[cfg(test)]
mod test;
