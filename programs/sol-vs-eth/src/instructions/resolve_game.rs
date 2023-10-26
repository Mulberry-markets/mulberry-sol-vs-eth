use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{ETH_ORACLE, GLOBAL_AUTH_SEED, GLOBAL_STATE_SEED, SOL_ORACLE};
use crate::quick_bets_errors::QuickBetsErrors;
use crate::state::{Game, GameStatus, GlobalAuth, GlobalState};
use crate::utils::{get_price_from_pyth, transfer_tokens};

pub fn handle_resolve_game(ctx: Context<ResolveBet>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let global_state = &mut ctx.accounts.global_state;

    global_state.confirm_crank_admin(&ctx.accounts.signer)?;

    require!(!game.is_settled, QuickBetsErrors::BetAlreadySettled);
    msg!("anticipation start : {}",game.anticipating_start);
    msg!("current time: {} ", Clock::get()?.unix_timestamp);
    if game.anticipating_start + global_state.anticipation_time
        > Clock::get()?.unix_timestamp as u64
    {
        return Err(QuickBetsErrors::AnticipationTimeTooSoon.into());
    }

    if Pubkey::from_str(SOL_ORACLE).unwrap() != *ctx.accounts.sol_feed.key {
        return Err(QuickBetsErrors::InvalidOracle.into());
    }

    if Pubkey::from_str(ETH_ORACLE).unwrap() != *ctx.accounts.eth_feed.key {
        return Err(QuickBetsErrors::InvalidOracle.into());
    }

    let sol_price = get_price_from_pyth(ctx.accounts.sol_feed.clone())?;
    let eth_price = get_price_from_pyth(ctx.accounts.eth_feed.clone())?;

    game.final_sol_price = sol_price;
    game.final_eth_price = eth_price;
    game.is_settled = true;
    game.anticipating_end = Clock::get()?.unix_timestamp as u64;

    ctx.accounts
        .global_state
        .modify_game_record(game.key(), GameStatus::Resolved);

    let pool_size = game.sol_bet_size + game.eth_bet_size;
    let winners_multiplier = if game.get_winner() == 0 {
        pool_size as f64 / game.sol_bet_size as f64
    } else if game.get_winner() == 1 {
        pool_size as f64 / game.eth_bet_size as f64
    } else {
        0.0
    };


    let amount_owed_to_winners = game.get_amount_owed_to_winners(winners_multiplier);
    msg!("Amount owed to winners: {}", amount_owed_to_winners);
    msg!("Amount in game vault: {}", ctx.accounts.game_vault.amount);
    let won_by_house = ctx.accounts.game_vault.amount - amount_owed_to_winners;

    if won_by_house == 0 {
        return Ok(());
    }
    let bump = *ctx.bumps.get("global_auth_pda").unwrap();
    let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
    transfer_tokens(
        ctx.accounts.game_vault.to_account_info(),
        ctx.accounts.house_wallet.to_account_info(),
        ctx.accounts.global_auth_pda.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        won_by_house,
        Some(seeds),
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct ResolveBet<'info> {
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
    #[account(mut, constraint = game.game_vault == game_vault.key())]
    pub game_vault: Account<'info, TokenAccount>,

    #[account(mut,
    seeds = [GLOBAL_AUTH_SEED],
    bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,
    #[account(mut, constraint = global_state.house_wallet == house_wallet.key())]
    pub house_wallet: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
