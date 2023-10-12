use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{GLOBAL_AUTH_SEED, GLOBAL_STATE_SEED};
use crate::consts::ADMIN_WALLETS;
use crate::quick_bets_errors::QuickBetsErrors;
use crate::state::{GlobalAuth, GlobalState};
use crate::utils::transfer_tokens;

pub fn handle_withdraw_funds(ctx : Context<WithdrawFunds>, amount : u64) -> Result<()>{
    let signer = &ctx.accounts.signer;


    if signer.key.to_string() != ADMIN_WALLETS {
        return err!(QuickBetsErrors::Unauthorized);
    };


    let bump = *ctx.bumps.get("global_auth_pda").unwrap();
    let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
    transfer_tokens(
        ctx.accounts.house_wallet.to_account_info(),
        ctx.accounts.receiver.to_account_info(),
        ctx.accounts.global_auth_pda.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        amount,
        Some(seeds),
    )?;



    Ok(())
}


#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    #[account(mut)]
    pub signer : Signer<'info>,
    #[account(mut, constraint = receiver.mint == house_wallet.mint)]
    pub receiver : Account<'info, TokenAccount>,
    #[account(mut,
    seeds = [GLOBAL_AUTH_SEED],
    bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,
    #[account(mut, seeds = [GLOBAL_STATE_SEED], bump)]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut, constraint = global_state.house_wallet == house_wallet.key())]
    pub house_wallet: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}