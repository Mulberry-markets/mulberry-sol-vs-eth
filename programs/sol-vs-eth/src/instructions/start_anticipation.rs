use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::consts::{ETH_ORACLE, GLOBAL_STATE_SEED, SOL_ORACLE};
use crate::sol_vs_eth_errors::SolVsEthErr;
use crate::state::{Bet, GlobalState};
use crate::utils::get_price_from_pyth;

pub fn handle_start_anticipation(ctx: Context<StartAnticipation>) -> Result<()> {
    let bet = &mut ctx.accounts.bet;
    let global_state = &mut ctx.accounts.global_state;

    global_state.confirm_crank_admin(&ctx.accounts.signer)?;

    if bet.betting_start + global_state.betting_time > Clock::get()?.unix_timestamp as u64 {
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
    bet.init_sol_price = sol_price;
    bet.init_eth_price = eth_price;

    bet.anticipating_start = Clock::get()?.unix_timestamp as u64;

    Ok(())
}

#[derive(Accounts)]
pub struct StartAnticipation<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    #[account(mut, seeds = [GLOBAL_STATE_SEED], bump)]
    pub global_state: Account<'info, GlobalState>,
    /// CHECK: Checking this manually in the instruction
    pub sol_feed: AccountInfo<'info>,
    /// CHECK: Checking this manually in the instruction
    pub eth_feed: AccountInfo<'info>,
}
