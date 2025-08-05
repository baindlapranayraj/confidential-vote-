use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("The computation was aborted")]
    AbortedComputation,
    #[msg("Cluster not set")]
    ClusterNotSet,
}


// Events
#[event]
pub struct VoteEvent {
    pub timestamp: i64,
}

#[event]
pub struct RevealResultEvent {
    pub output: bool,
}
