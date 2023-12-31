use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{GLOBAL_AUTH_SEED, USER_ACCOUNT_SEED};
use crate::quick_bets_errors::QuickBetsErrors;
use crate::state::{Game, GlobalAuth, User};
use crate::utils::transfer_tokens;

pub fn handle_claim_win(ctx: Context<ClaimWin>) -> Result<()> {
    let game = &mut ctx.accounts.game;

    require!(game.is_settled, QuickBetsErrors::BetNotSettled);
    let user_bet = if let Some(user_bet) = game.get_user_bet(ctx.accounts.receiver.key()) {
        user_bet
    } else {
        return err!(QuickBetsErrors::NoBetFound);
    };

    if user_bet.claimed {
        msg!("You have already claimed your bet");
        return Ok(());
    }
    // require!(!user_bet.claimed, QuickBetsErrors::AlreadyClaimed);
    let user_bet_size = user_bet.amount;

    // adding user streak
    ctx.accounts
        .user_account
        .add_bet_record(user_bet_size, user_bet.side == game.get_winner());


    game.mark_bet_claimed(ctx.accounts.owner.key())?;

    if game.get_winner() != 2 && user_bet.side != game.get_winner() {
        msg!("You are not on the winning side");

        return Ok(());
    }


    if game.get_winner() == 2 {
        msg!("Draw, returning your bet");
        // Market resolved with a draw, return the user's bet
        let bump = *ctx.bumps.get("global_auth_pda").unwrap();
        let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
        transfer_tokens(
            ctx.accounts.game_vault.to_account_info(),
            ctx.accounts.receiver.to_account_info(),
            ctx.accounts.global_auth_pda.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            user_bet_size,
            Some(seeds),
        )?;
        return Ok(());
    }
    let total_pool_size = game.sol_bet_size + game.eth_bet_size;
    let total_sol_bets = game.sol_bet_size;
    let total_eth_bets = game.eth_bet_size;
    let mut winning_amount = 0;
    if user_bet.side == 0 {
        // this means the user bet on sol, and sol won
        let user_pool_share = user_bet_size as f64 / total_sol_bets as f64;
        winning_amount = (total_pool_size as f64 * user_pool_share) as u64;
    } else {
        // this means the user bet on eth, and eth won
        let user_pool_share = user_bet_size as f64 / total_eth_bets as f64;
        winning_amount = (total_pool_size as f64 * user_pool_share) as u64;
    }

    // transfer the winning amount to the user
    let bump = *ctx.bumps.get("global_auth_pda").unwrap();
    let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
    transfer_tokens(
        ctx.accounts.game_vault.to_account_info(),
        ctx.accounts.receiver.to_account_info(),
        ctx.accounts.global_auth_pda.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        winning_amount,
        Some(seeds),
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct ClaimWin<'info> {
    #[account(mut)]
    pub game: Box<Account<'info, Game>>,
    #[account(mut,
    seeds = [GLOBAL_AUTH_SEED],
    bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,
    #[account(mut, constraint = game.game_vault == game_vault.key())]
    pub game_vault: Account<'info, TokenAccount>,

    /// CHECK: Account doesn't matter in itself, receiver is the bet identifier.
    pub owner: AccountInfo<'info>,
    #[account(mut, seeds = [USER_ACCOUNT_SEED, owner.key.as_ref()], bump)]
    pub user_account: Account<'info, User>,
    #[account(mut, token::authority = owner)]
    pub receiver: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
