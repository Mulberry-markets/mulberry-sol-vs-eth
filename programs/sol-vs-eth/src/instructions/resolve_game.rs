use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{ETH_ORACLE, GLOBAL_AUTH_SEED, GLOBAL_STATE_SEED, SOL_ORACLE};
use crate::sol_vs_eth_errors::SolVsEthErr;
use crate::state::{Game, GameStatus, GlobalAuth, GlobalState};
use crate::utils::{get_price_from_pyth, transfer_tokens};

pub fn handle_resolve_game(ctx: Context<ResolveBet>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let global_state = &mut ctx.accounts.global_state;

    global_state.confirm_crank_admin(&ctx.accounts.signer)?;

    require!(!game.is_settled , SolVsEthErr::BetAlreadySettled);

    msg!("anticipation start : {}", game.anticipating_start);

    if game.anticipating_start + global_state.anticipation_time > Clock::get()?.unix_timestamp as u64 {
        return Err(SolVsEthErr::AnticipationTimeTooSoon.into());
    }

    if Pubkey::from_str(SOL_ORACLE).unwrap() != *ctx.accounts.sol_feed.key {
        return Err(SolVsEthErr::InvalidOracle.into());
    }

    if Pubkey::from_str(ETH_ORACLE).unwrap() != *ctx.accounts.eth_feed.key {
        return Err(SolVsEthErr::InvalidOracle.into());
    }


    let sol_price = get_price_from_pyth(ctx.accounts.sol_feed.clone())?;
    let eth_price = get_price_from_pyth(ctx.accounts.eth_feed.clone())?;

    game.final_sol_price = sol_price;
    game.final_eth_price = eth_price;
    game.is_settled = true;
    game.anticipating_end = Clock::get()?.unix_timestamp as u64;

    ctx.accounts.global_state.modify_game_record(game.key(), GameStatus::Resolved);

    if game.get_winner() == 2 {
        msg!("Draw, refunding the house's bet");
        let bump = *ctx.bumps.get("global_auth_pda").unwrap();
        let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
        transfer_tokens(
            ctx.accounts.game_vault.to_account_info(),
            ctx.accounts.house_wallet.to_account_info(),
            ctx.accounts.global_auth_pda.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            game.house_bet_amount,
            Some(seeds),
        )?;
    }

    if game.get_winner() == game.house_bet_side {
        let total_pool_size = game.sol_bet_size + game.eth_bet_size;
        let total_sol_bets = game.sol_bet_size;
        let total_eth_bets = game.eth_bet_size;
        let mut winning_amount = 0;
        if game.house_bet_side == 0 {
            // this means the user bet on sol, and sol won
            let user_pool_share = game.house_bet_amount as f64 / total_sol_bets as f64;
            winning_amount = (total_pool_size as f64 * user_pool_share) as u64;
        } else {
            // this means the user bet on eth, and eth won
            let user_pool_share = game.house_bet_amount as f64 / total_eth_bets as f64;
            winning_amount = (total_pool_size as f64 * user_pool_share) as u64;
        }

        let bump = *ctx.bumps.get("global_auth_pda").unwrap();
        let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
        transfer_tokens(
            ctx.accounts.game_vault.to_account_info(),
            ctx.accounts.house_wallet.to_account_info(),
            ctx.accounts.global_auth_pda.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            winning_amount,
            Some(seeds),
        )?;
    }

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