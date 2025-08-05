use anchor_lang::prelude::*;

/// Represents a confidential poll with encrypted vote tallies.
#[account]
#[derive(InitSpace)]
pub struct PollAccount {
    pub bump: u8,
    /// PDA bump seed

    /// Encrypted vote counters: [yes_count, no_count] as 32-byte ciphertexts
    pub vote_state: [[u8; 32]; 2],

    /// Unique identifier for this poll
    pub id: u32,

    /// Public key of the poll creator (only they can reveal results)
    pub authority: Pubkey, 

    /// Cryptographic nonce for the encrypted vote counters
    pub nonce: u128,

    /// The poll question (max 50 characters)
    #[max_len(50)]
    pub question: String,
}
