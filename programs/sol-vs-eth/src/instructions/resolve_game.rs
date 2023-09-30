use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{ETH_ORACLE, SOL_ORACLE, GLOBAL_STATE_SEED, GLOBAL_AUTH_SEED};
use crate::sol_vs_eth_errors::SolVsEthErr;
use crate::state::{Game, GlobalAuth, GlobalState};
use crate::utils::{get_price_from_pyth, transfer_tokens};

pub fn handle_resolve_game(ctx: Context<ResolveBet>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let global_state = &mut ctx.accounts.global_state;

    global_state.confirm_crank_admin(&ctx.accounts.signer)?;


    require!(!game.is_settled , SolVsEthErr::BetAlreadySettled);

    msg!("anticipation start : {}", game.anticipating_start);

    if Clock::get()?.unix_timestamp as u64 <= game.anticipating_start + global_state.anticipation_time {
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


    if game.get_winner() == game.house_bet_side || game.get_winner() == 2 {
        msg!("House did win");
        msg!("transferring the house's bet");
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

    Ok(())
}


#[derive(Accounts)]
pub struct ResolveBet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub game: Account<'info, Game>,
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