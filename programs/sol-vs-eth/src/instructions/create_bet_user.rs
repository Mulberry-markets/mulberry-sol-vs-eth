use std::mem::size_of;

use anchor_lang::prelude::*;

use crate::state::{Bet, GlobalState, UserBetAccount};
use crate::consts::{GLOBAL_STATE_SEED};
use crate::sol_vs_eth_errors::SolVsEthErr;

pub fn handle_create_bet_user(ctx: Context<CreateBetUser>) -> Result<()> {
    let bet = &ctx.accounts.bet;
    let global_state = &ctx.accounts.global_state;

    require!(bet.betting_active(global_state.betting_time)?, SolVsEthErr::BettingInactive);

    ctx.accounts.user_bet_account.amount = 0;
    ctx.accounts.user_bet_account.side = u8::MAX;
    ctx.accounts.user_bet_account.claimed = false;
    Ok(())
}


#[derive(Accounts)]
pub struct CreateBetUser<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    #[account(init, seeds = [bet.key().as_ref(), signer.key().as_ref()], bump, payer = signer, space = size_of::< UserBetAccount > () + 12)]
    pub user_bet_account: Account<'info, UserBetAccount>,
    #[account(mut, seeds = [GLOBAL_STATE_SEED], bump)]
    pub global_state: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>,
}