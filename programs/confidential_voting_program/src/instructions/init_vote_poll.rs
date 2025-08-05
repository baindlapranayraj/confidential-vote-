use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
use arcium_client::idl::arcium::ID_CONST;
use crate::error::ErrorCode;

use crate::{state::PollAccount, COMP_DEF_OFFSET_INIT_VOTE_STATE};
use crate::ID;

#[queue_computation_accounts("init_vote_state", signer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, id: u32)]
pub struct CreatePoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer, 
        space = 8 + PollAccount::INIT_SPACE,
        seeds = [
                 b"poll",
                 signer.key().to_bytes().as_ref(),
                 id.to_le_bytes().as_ref()
            ],
         bump,
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
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_VOTE_STATE)
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


// Well these are kinda common account for every queue instruction
//
// 1)  mxe_account - Multi-Party Execution Environment account that coordinates the confidential computation
// 2)  mempool_account - Memory pool for queuing computations
// 3)  executing_pool - Pool of nodes that will execute the computation
// 4)  computation_account - Specific account for this computation instance
// 5)  comp_def_account - Computation definition that specifies what to compute
// 6)  cluster_account - Cluster of nodes participating in MPC
// 7)  pool_account - Fee pool for paying computation costs
// 8)  clock_account - For timing and coordination

