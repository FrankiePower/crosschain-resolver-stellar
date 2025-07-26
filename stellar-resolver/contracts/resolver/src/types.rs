use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TimeLockError {
    RescueStartOverflow = 1,
    TimelockValueOverflow = 2,
    DeploymentTimestampNotSet = 3,
    InvalidSourceChainTimelockOrdering = 4,
    InvalidDestinationChainTimelockOrdering = 5,
    TimelockOffsetTooLarge = 6,
}