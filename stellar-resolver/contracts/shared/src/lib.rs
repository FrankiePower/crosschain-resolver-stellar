#![no_std]

// Shared library for cross-chain atomic swap contracts
// Contains common types, timelock logic, immutables, and base escrow functionality

pub mod timelock;
pub mod types;
pub mod immutables;
pub mod baseescrow;

// Re-export commonly used types for easier imports
pub use types::*;
pub use timelock::{timelocks, Stage, Timelocks};
pub use immutables::{immutables as other_immutables, DualAddress, Immutables};
pub use baseescrow::{BaseEscrowTrait, Error as EscrowError, only_taker, only_valid_secret, only_before, only_after, uni_transfer};

#[cfg(test)]
mod test;
