use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{ETH_ORACLE, SOL_ORACLE, GLOBAL_STATE_SEED, GLOBAL_AUTH_SEED};
use crate::sol_vs_eth_errors::SolVsEthErr;
use crate::state::{Bet, GlobalAuth, GlobalState};
use crate::utils::{get_price_from_pyth, transfer_tokens};

pub fn handle_resolve_bet(ctx: Context<ResolveBet>) -> Result<()> {
    let bet = &mut ctx.accounts.bet;
    let global_state = &mut ctx.accounts.global_state;

    global_state.confirm_crank_admin(&ctx.accounts.signer)?;


    require!(!bet.is_settled , SolVsEthErr::BetAlreadySettled);

    msg!("anticipation start : {}", bet.anticipating_start);

    if Clock::get()?.unix_timestamp as u64 <= bet.anticipating_start + global_state.anticipation_time {
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

    bet.final_sol_price = sol_price;
    bet.final_eth_price = eth_price;
    bet.is_settled = true;
    bet.anticipating_end = Clock::get()?.unix_timestamp as u64;


    if bet.get_winner() == bet.house_bet_side || bet.get_winner() == 2 {
        msg!("House did win");
        msg!("transferring the house's bet");
        let bump = *ctx.bumps.get("global_auth_pda").unwrap();
        let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
        transfer_tokens(
            ctx.accounts.betting_vault.to_account_info(),
            ctx.accounts.house_wallet.to_account_info(),
            ctx.accounts.global_auth_pda.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            bet.house_bet_amount,
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
    pub bet: Account<'info, Bet>,
    #[account(mut, seeds = [GLOBAL_STATE_SEED], bump)]
    pub global_state: Account<'info, GlobalState>,
    /// CHECK: Checking this manually in the instruction
    pub sol_feed: AccountInfo<'info>,
    /// CHECK: Checking this manually in the instruction
    pub eth_feed: AccountInfo<'info>,
    #[account(mut, constraint = bet.bet_vault == betting_vault.key())]
    pub betting_vault: Account<'info, TokenAccount>,

    #[account(mut,
    seeds = [GLOBAL_AUTH_SEED],
    bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = global_state.house_wallet == house_wallet.key())]
    pub house_wallet: Account<'info, TokenAccount>,
}