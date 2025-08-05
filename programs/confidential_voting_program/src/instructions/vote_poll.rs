#![allow(unexpected_cfgs)]

use crate::{error::ErrorCode, state::PollAccount, COMP_DEF_OFFSET_INIT_VOTE, ID};
use anchor_lang::{prelude::*, solana_program::sysvar};
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::ID_CONST;

#[init_computation_definition_accounts("vote", signer)]
#[derive(Accounts)]
pub struct InitVoteCompDef<'info> {
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

    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[callback_accounts("vote", payer)]
#[derive(Accounts)]
pub struct InitVoteCallback<'info> {
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
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_VOTE)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(
        address = sysvar::instructions::ID,
    )]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,

    pub arcium_program: Program<'info, Arcium>,
}

// Vote Instruction Accounts
#[queue_computation_accounts("init_vote_state", signer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _id: u32)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"poll",
            poll_account.authority.key().to_bytes().as_ref(),
            poll_account.id.to_be_bytes().as_ref()
        ],
        bump = poll_account.bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    // Some Common Queue instruction accounts
    #[account(
            address = derive_mxe_pda!()
        )]
    pub mxe_account: Account<'info, MXEAccount>,

    #[account(
        mut,
        address = derive_mempool_pda!()
    )]
    /// CHECK: mempool_account, checked by the arcium program
    pub mempool_account: UncheckedAccount<'info>,

    #[account(
        mut,
        address = derive_execpool_pda!()
    )]
    /// CHECK: executing_pool, checked by the arcium program
    pub executing_pool: UncheckedAccount<'info>,

    #[account(
        mut,
        address = derive_comp_pda!(computation_offset)
    )]
    /// CHECK: computation_account, checked by the arcium program.
    pub computation_account: UncheckedAccount<'info>,

    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_VOTE)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account)
    )]
    pub cluster_account: Account<'info, Cluster>,

    #[account(
        mut,
        address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS,
    )]
    pub pool_account: Account<'info, FeePool>,

    #[account(
        address = ARCIUM_CLOCK_ACCOUNT_ADDRESS,
    )]
    pub clock_account: Account<'info, ClockAccount>,

    // instruction required program accounts
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}
