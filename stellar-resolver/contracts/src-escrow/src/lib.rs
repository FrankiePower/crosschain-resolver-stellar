#![no_std]

// Source Chain Escrow Contract
// Handles the source side of cross-chain atomic swaps

mod srcescrow;

// Re-export the contract
pub use srcescrow::SrcEscrow;

#[cfg(test)]
mod test;
