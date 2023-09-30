use std::mem::size_of;

use anchor_lang::prelude::*;

use crate::state::{Game, GlobalState, UserBetAccount};
use crate::consts::{GLOBAL_STATE_SEED};
use crate::sol_vs_eth_errors::SolVsEthErr;

pub fn handle_create_user_game_account(ctx: Context<CreateUserGameAccount>) -> Result<()> {
    let game = &ctx.accounts.game;
    let global_state = &ctx.accounts.global_state;

    require!(game.betting_active(global_state.betting_time)?, SolVsEthErr::BettingInactive);

    ctx.accounts.user_game_account.amount = 0;
    ctx.accounts.user_game_account.side = u8::MAX;
    ctx.accounts.user_game_account.claimed = false;
    Ok(())
}


#[derive(Accounts)]
pub struct CreateUserGameAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub game: Account<'info, Game>,
    #[account(init, seeds = [game.key().as_ref(), signer.key().as_ref()], bump, payer = signer, space = size_of::< UserBetAccount > () + 12)]
    pub user_game_account: Account<'info, UserBetAccount>,
    #[account(mut, seeds = [GLOBAL_STATE_SEED], bump)]
    pub global_state: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>,
}