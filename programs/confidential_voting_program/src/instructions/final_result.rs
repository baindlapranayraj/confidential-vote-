#![allow(unexpected_cfgs)]

use crate::{error::ErrorCode, state::PollAccount, COMP_DEF_OFFSET_INIT_REVEAL, ID};
use anchor_lang::{prelude::*, solana_program::sysvar};
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::ID_CONST;

#[init_computation_definition_accounts("final_result", payer)]
#[derive(Accounts)]
pub struct InitRevealResultCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,

    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    /// Can't check it here as it's not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[callback_accounts("final_result", payer)]
#[derive(Accounts)]
pub struct InitRevealResultCallback<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_REVEAL)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,

    #[account(
        address = sysvar::instructions::ID,
    )]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,

    pub arcium_program: Program<'info, Arcium>,
}

#[queue_computation_accounts("final_result", signer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, id: u32)]
pub struct RevealResults<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"poll", signer.key().as_ref(), id.to_le_bytes().as_ref()],
        bump = poll_account.bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    // accounts req for confidetial instructions
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
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_REVEAL)
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

    #[account(
        address = poll_account.authority.key()
    )]
    /// CHECK: author_poll, checked by the account constraint
    pub author_poll: UncheckedAccount<'info>,

    // instruction req progrms
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}
