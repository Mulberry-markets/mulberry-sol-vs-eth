use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{ETH_ORACLE, GLOBAL_AUTH_SEED, GLOBAL_STATE_SEED, MARGIN_OF_ERROR, SOL_ORACLE};
use crate::quick_bets_errors::QuickBetsErrors;
use crate::state::{Game, GameStatus, GlobalAuth, GlobalState};
use crate::utils::{get_price_from_pyth, transfer_tokens};

pub fn handle_start_anticipation(ctx: Context<StartAnticipation>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let global_state = &mut ctx.accounts.global_state;

    global_state.confirm_crank_admin(&ctx.accounts.signer)?;

    msg!("anticipation start : {}", game.betting_start);
    msg!("current time: {} ", Clock::get()?.unix_timestamp);
    // the + 1 at the end is the game allowing a margin of error of 1 second,
    // most games should still end at the exact time

    if game.betting_start + global_state.betting_time > Clock::get()?.unix_timestamp as u64 + MARGIN_OF_ERROR {
        return Err(QuickBetsErrors::BettingTimeTooSoon.into());
    }

    if Pubkey::from_str(SOL_ORACLE).unwrap() != *ctx.accounts.sol_feed.key {
        return Err(QuickBetsErrors::InvalidOracle.into());
    }

    if Pubkey::from_str(ETH_ORACLE).unwrap() != *ctx.accounts.eth_feed.key {
        return Err(QuickBetsErrors::InvalidOracle.into());
    }

    let sol_price = get_price_from_pyth(ctx.accounts.sol_feed.clone())?;
    let eth_price = get_price_from_pyth(ctx.accounts.eth_feed.clone())?;
    msg!("Sol price: {}", sol_price);
    msg!("Eth price: {}", eth_price);
    game.initial_sol_price = sol_price;
    game.initial_eth_price = eth_price;

    game.anticipating_start = Clock::get()?.unix_timestamp as u64;
    let min_for_sol_payout = game.sol_bet_size as f64 * global_state.min_multiplier;
    let min_for_eth_payout = game.eth_bet_size as f64 * global_state.min_multiplier;
    let pool_size = game.sol_bet_size + game.eth_bet_size;

    let mut matched_amount = 0_f64;
    if min_for_sol_payout > pool_size as f64 {
        matched_amount = min_for_sol_payout - pool_size as f64;
        game.eth_bet_size += matched_amount as u64;
    }
    if min_for_eth_payout > pool_size as f64 {
        matched_amount = min_for_eth_payout - pool_size as f64;
        game.sol_bet_size += matched_amount as u64;
    }

    matched_amount = std::cmp::min(
        matched_amount as u64,
        global_state.max_house_bet_size - game.house_bet_amount,
    ) as f64;

    ctx.accounts
        .global_state
        .modify_game_record(game.key(), GameStatus::Anticipation);

    if matched_amount == 0_f64 {
        msg!("odds are good enough already, no need to match");
        return Ok(());
    }

    let bump = *ctx.bumps.get("global_auth_pda").unwrap();
    let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
    transfer_tokens(
        ctx.accounts.house_wallet.to_account_info(),
        ctx.accounts.game_vault.to_account_info(),
        ctx.accounts.global_auth_pda.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        matched_amount as u64,
        Some(seeds),
    )?;

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

    #[account(mut, constraint = global_state.house_wallet == house_wallet.key())]
    pub house_wallet: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub game_vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: Checking this manually in the instruction
    pub sol_feed: AccountInfo<'info>,
    /// CHECK: Checking this manually in the instruction
    pub eth_feed: AccountInfo<'info>,
    #[account(mut,
    seeds = [GLOBAL_AUTH_SEED],
    bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,
    pub token_program: Program<'info, Token>,
}
