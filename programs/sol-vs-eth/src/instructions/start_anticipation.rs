use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::consts::{ETH_ORACLE, GLOBAL_STATE_SEED, SOL_ORACLE};
use crate::sol_vs_eth_errors::SolVsEthErr;
use crate::state::{Game, GameStatus, GlobalState};
use crate::utils::get_price_from_pyth;

pub fn handle_start_anticipation(ctx: Context<StartAnticipation>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let global_state = &mut ctx.accounts.global_state;

    global_state.confirm_crank_admin(&ctx.accounts.signer)?;
    msg!("betting start : {}", game.betting_start + global_state.betting_time);
    msg!("current time : {}", Clock::get()?.unix_timestamp as u64);
    if game.betting_start + global_state.betting_time > Clock::get()?.unix_timestamp as u64 {
        return Err(SolVsEthErr::BettingTimeTooSoon.into());
    }

    if Pubkey::from_str(SOL_ORACLE).unwrap() != *ctx.accounts.sol_feed.key {
        return Err(SolVsEthErr::InvalidOracle.into());
    }

    if Pubkey::from_str(ETH_ORACLE).unwrap() != *ctx.accounts.eth_feed.key {
        return Err(SolVsEthErr::InvalidOracle.into());
    }

    let sol_price = get_price_from_pyth(ctx.accounts.sol_feed.clone())?;
    let eth_price = get_price_from_pyth(ctx.accounts.eth_feed.clone())?;
    msg!("Sol price: {}", sol_price);
    msg!("Eth price: {}", eth_price);
    game.initial_sol_price = sol_price;
    game.initial_eth_price = eth_price;

    game.anticipating_start = Clock::get()?.unix_timestamp as u64;

    ctx.accounts.global_state.modify_game_record(game.key(), GameStatus::Anticipation);

    Ok(())
}

#[derive(Accounts)]
pub struct StartAnticipation<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub game: Box<Account<'info, Game>>,
    #[account(mut, seeds = [GLOBAL_STATE_SEED], bump)]
    pub global_state: Account<'info, GlobalState>,
    /// CHECK: Checking this manually in the instruction
    pub sol_feed: AccountInfo<'info>,
    /// CHECK: Checking this manually in the instruction
    pub eth_feed: AccountInfo<'info>,
}
