#![allow(unexpected_cfgs)] // STOP SHOUTING !!!!!!!!
#![allow(deprecated)]

use anchor_lang::{prelude::*, solana_program::sysvar};
use arcium_anchor::prelude::*;
pub mod error;
pub mod instructions;
pub mod state;

use error::{ErrorCode, RevealResultEvent, VoteEvent};

use arcium_client::idl::arcium::types::CallbackAccount;
use instructions::*;
use state::*;

pub const COMP_DEF_OFFSET_INIT_VOTE_STATE: u32 = comp_def_offset("init_vote_state");
pub const COMP_DEF_OFFSET_INIT_VOTE: u32 = comp_def_offset("vote");
pub const COMP_DEF_OFFSET_INIT_REVEAL: u32 = comp_def_offset("final_result");

declare_id!("5zvvrGs2BiWwnh9WpfRVKE4pWkfWn2mpqgC1rz5FrWgQ");

// keep init and call_back functions at here

#[arcium_program]
pub mod confidential_voting_program {

    use super::*;

    pub fn init_vote_state_comp_def(ctx: Context<InitVoteStatesCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;

        Ok(())
    }

    pub fn create_poll(
        ctx: Context<CreatePoll>,
        computation_offset: u64, // a unique identifier for this specific computation instance
        id: u32,
        question: String,
        nonce: u128,
    ) -> Result<()> {
        // Initializing the PollAccount
        ctx.accounts.poll_account.set_inner(PollAccount {
            bump: ctx.bumps.poll_account,
            vote_state: [[0; 32]; 2],
            id,
            authority: ctx.accounts.signer.key(),
            nonce, // number used once
            question,
        });

        let args = vec![Argument::PlaintextU128(nonce)];

        // This code is scheduling our encrypted instruction to run on the Arcium network.
        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![CallbackAccount {
                pubkey: ctx.accounts.poll_account.key(), // this where callback should go
                is_writable: true,
            }],
            None, // url webhook for external notifications
        )?;

        // ================ The Flow: What Happens Next ================
        //   This code runs → Computation gets queued in the mempool
        //   MPC cluster picks it up → Your encrypted instruction runs across multiple parties
        //   Computation completes → Arcium automatically calls your callback instruction
        //   Callback receives result → Your poll account gets updated with the encrypted result

        Ok(())
    }

    #[arcium_callback(encrypted_ix = "init_vote_state")]
    pub fn init_vote_state_callback(
        ctx: Context<InitVoteStatesCallback>,
        output: ComputationOutputs<InitVoteStateOutput>,
    ) -> Result<()> {
        let o = match output {
            ComputationOutputs::Success(InitVoteStateOutput { field_0 }) => field_0,

            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        ctx.accounts.poll_account.nonce = o.nonce;
        ctx.accounts.poll_account.vote_state = o.ciphertexts;

        Ok(())
    }

    pub fn init_vote_comp_def(ctx: Context<InitVoteCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;

        Ok(())
    }

    pub fn vote(
        _ctx: Context<Vote>,
        computation_offset: u64,
        _id: u32,

        vote_encryption_pubkey: [u8; 32], // encrypted ix publickey
        vote_nonce: u128,                 // new nonce

        vote: [u8; 32], // actual user vote here (true or false in encrypted bytes)
    ) -> Result<()> {
        let args = vec![
            Argument::ArcisPubkey(vote_encryption_pubkey),
            Argument::PlaintextU128(_ctx.accounts.poll_account.nonce),
            Argument::PlaintextU128(vote_nonce),
            Argument::Account(_ctx.accounts.poll_account.key(), 8 + 1, 32 * 2),
            Argument::EncryptedBool(vote),
        ];

        queue_computation(
            _ctx.accounts,
            computation_offset,
            args,
            vec![CallbackAccount {
                pubkey: _ctx.accounts.poll_account.key(),
                is_writable: true,
            }],
            None,
        )?;

        Ok(())
    }

    #[arcium_callback(encrypted_ix = "vote")]
    pub fn vote_callback(
        ctx: Context<InitVoteCallback>,
        output: ComputationOutputs<VoteOutput>,
    ) -> Result<()> {
        let o = match output {
            ComputationOutputs::Success(VoteOutput { field_0 }) => field_0,
            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        ctx.accounts.poll_account.nonce = o.nonce;
        ctx.accounts.poll_account.vote_state = o.ciphertexts;

        let clock = Clock::get()?;

        emit!(VoteEvent {
            timestamp: clock.unix_timestamp.into()
        });

        Ok(())
    }

    pub fn init_reveal_result(ctx: Context<InitRevealResultCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, true, 0, None, None)?;
        Ok(())
    }

    pub fn final_result(
        ctx: Context<RevealResults>,
        computation_offset: u64,
        id: u32,
    ) -> Result<()> {
        require!(
            ctx.accounts.poll_account.authority == ctx.accounts.signer.key(),
            ErrorCode::InvalidAuthority
        );

        msg!("Revealing voting result for poll with id {}", id);

        let args = vec![
            Argument::PlaintextU128(ctx.accounts.poll_account.nonce),
            Argument::Account(
                ctx.accounts.poll_account.key(),
                // Offset calculation: 8 bytes (discriminator) + 1 byte (bump)
                8 + 1,
                32 * 2, // 2 encrypted vote counters (yes/no), 32 bytes each
            ),
        ];

        queue_computation(ctx.accounts, computation_offset, args, vec![], None)?;

        Ok(())
    }

    #[arcium_callback(encrypted_ix = "final_result")]
    pub fn final_result_callback(
        ctx: Context<InitRevealResultCallback>,
        output: ComputationOutputs<FinalResultOutput>,
    ) -> Result<()> {
        let o = match output {
            ComputationOutputs::Success(FinalResultOutput { field_0 }) => field_0,

            _ => return Err(ErrorCode::AbortedComputation.into()),
        };

        emit!(RevealResultEvent { output: o });

        Ok(())
    }
}

// This struct is used to initialize a computation definition
#[init_computation_definition_accounts("init_vote_state", signer)] // arcium will impl traits
#[derive(Accounts)]
pub struct InitVoteStatesCompDef<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut, address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>, // A PDA that manages cryptographic operations

    #[account(mut)]
    /// CHECK: This will be checked inside of the instruction logic
    ///  Will be used later to verify and execute our encrypted instruction
    pub comp_def_account: UncheckedAccount<'info>, // computation definition account, our instruction will init and stores the metadata of our encryption ix's.

    // Programs required for instruction
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[callback_accounts("init_vote_state", payer)]
#[derive(Accounts)]
pub struct InitVoteStatesCallback<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
                 b"poll",       
                 poll_account.authority.key().to_bytes().as_ref(),
                 poll_account.id.to_be_bytes().as_ref(),
             ],
            bump = poll_account.bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_VOTE_STATE)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(
        address = sysvar::instructions::ID,
    )]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,

    pub arcium_program: Program<'info, Arcium>,
}
