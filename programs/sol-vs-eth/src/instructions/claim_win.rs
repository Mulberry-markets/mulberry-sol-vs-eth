use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::GLOBAL_AUTH_SEED;
use crate::sol_vs_eth_errors::SolVsEthErr;
use crate::state::{Bet, GlobalAuth, GlobalState, UserBetAccount};
use crate::utils::transfer_tokens;

pub fn handle_claim_win(ctx: Context<ClaimWin>) -> Result<()> {
    let bet = &mut ctx.accounts.bet;
    let user_bet_account = &mut ctx.accounts.user_bet_account;


    require!(bet.is_settled, SolVsEthErr::BetNotSettled);
    require!(!user_bet_account.claimed, SolVsEthErr::AlreadyClaimed);


    if bet.get_winner() != 2 && user_bet_account.side != bet.get_winner() {
        msg!("You are not on the winning side");
        return Err(SolVsEthErr::NotOnWinningSide.into());
    }

    let user_bet_size = user_bet_account.amount;
    if bet.get_winner() == 2 {
        msg!("Draw, returning your bet");
        // Market resolved with a draw, return the user's bet
        let bump = *ctx.bumps.get("global_auth_pda").unwrap();
        let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
        transfer_tokens(
            ctx.accounts.betting_vault.to_account_info(),
            ctx.accounts.receiver.to_account_info(),
            ctx.accounts.global_auth_pda.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            user_bet_size,
            Some(seeds),
        )?;
        user_bet_account.claimed = true;
        return Ok(());
    }
    let total_sol_bets = bet.sol_bet_size;
    let total_eth_bets = bet.eth_bet_size;
    let mut winning_amount = 0;
    if user_bet_account.side == 0 {
        // this means the user bet on sol, and sol won
        let user_pool_share = user_bet_size as f64 / total_sol_bets as f64;
        winning_amount = (total_eth_bets as f64 * user_pool_share) as u64;
    } else {
        // this means the user bet on eth, and eth won
        let user_pool_share = user_bet_size as f64 / total_eth_bets as f64;
        winning_amount = (total_sol_bets as f64 * user_pool_share) as u64;
    }

    // transfer the winning amount to the user
    let bump = *ctx.bumps.get("global_auth_pda").unwrap();
    let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
    transfer_tokens(
        ctx.accounts.betting_vault.to_account_info(),
        ctx.accounts.receiver.to_account_info(),
        ctx.accounts.global_auth_pda.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        winning_amount,
        Some(seeds),
    )?;

    user_bet_account.claimed = true;


    Ok(())
}

#[derive(Accounts)]
pub struct ClaimWin<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub bet: Account<'info, Bet>,
    #[account(mut, seeds = [bet.key().as_ref(), signer.key().as_ref()], bump)]
    pub user_bet_account: Account<'info, UserBetAccount>,
    #[account(mut,
    seeds = [GLOBAL_AUTH_SEED],
    bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,
    #[account(mut, constraint = bet.bet_vault == betting_vault.key())]
    pub betting_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>,
    pub global_state: Account<'info, GlobalState>,
    pub token_program: Program<'info, Token>,
}