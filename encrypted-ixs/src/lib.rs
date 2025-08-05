use arcis_imports::*;

#[encrypted] // Marks this module as containing encrypted computations
mod circuits {
    // Privacy-preserving functions go here in this module
    // This code runs across multiple parties, not on Solana

    use arcis_imports::*;

    // taking this from program solana pda account, its a slice from the poll_account vote_state field
    pub struct VoteState {
        yes: u64,
        no: u64,
    }

    // Inputs from the client side
    pub struct UserVote {
        vote: bool,
    }

    // encrypted instruction
    #[instruction]
    pub fn init_vote_state(mxe: Mxe) -> Enc<Mxe, VoteState> {
        // mxe is Multi-Party Execution Environment

        let vote_state = VoteState { yes: 0, no: 0 }; // Initializes vote struct

        mxe.from_arcis(vote_state) // Returns Cipher Text to callback functuion of init_vote_state
    }

    #[instruction]
    pub fn vote(
        vote_ctxt: Enc<Shared, UserVote>, // user input from the clinet
        vote_states_ctx: Enc<Mxe, VoteState>,
    ) -> Enc<Mxe, VoteState> {
        // Decrypt
        let user_vote = vote_ctxt.to_arcis();
        let mut vote_state = vote_states_ctx.to_arcis(); // main

        if user_vote.vote {
            vote_state.yes = vote_state.yes + 1;
        } else {
            vote_state.no = vote_state.no + 1;
        }

        // Encrypts
        vote_states_ctx.owner.from_arcis(vote_state)
    }

    #[instruction] // revelaing the final results
    pub fn final_result(vote_state_ctxt: Enc<Mxe, VoteState>) -> bool {
        let vote_poll_state = vote_state_ctxt.to_arcis();
        (vote_poll_state.yes > vote_poll_state.no).reveal()
    }

}

// ==================== Learnings ====================
// - from_arcis(): Encrypts data for MPC computation
// - to_arcis(): Decrypts data for computation (temporarily)
// - reveal(): Reveals final result (only what you want public)
//
//
//  Type Matching
//  -  Enc<Shared, T> → Argument::EncryptedBool() taking data from the user
//  -  Enc<Mxe, T> → Argument::Account() (from blockchain)
//  -  u64 → Argument::PlaintextU64() --> nonce
//  -  u128 → Argument::PlaintextU128() --> nonce
//  -  mxe: Mxe → Provided automatically by Arcium
//
