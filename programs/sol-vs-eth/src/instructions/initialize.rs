use std::mem::size_of;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::consts::{GLOBAL_AUTH_SEED, GLOBAL_STATE_SEED};
use crate::state::{GlobalAuth, GlobalState};

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;

    global_state.crank_admin = ctx.accounts.signer.key();

    // initial fee of 500(5%)
    global_state.betting_fees = 500;
    // max bet amount of 0.5 sol
    global_state.max_house_match = (0.5 * 1e6_f64) as u64;

    global_state.paused = false;

    global_state.house_wallet = ctx.accounts.house_wallet.key();

    global_state.anticipation_time = 60;

    global_state.betting_time = 60;

    global_state.betting_currency = ctx.accounts.betting_currency.key();


    Ok(())
}


#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init,
    seeds = [GLOBAL_STATE_SEED], bump,
    payer = signer,
    space = size_of::< GlobalState > () + 12)]
    pub global_state: Box<Account<'info, GlobalState>>,

    #[account(init,
    seeds = [GLOBAL_AUTH_SEED],
    bump,
    payer = signer,
    space = size_of::< GlobalAuth > () + 12)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,


    pub betting_currency : Account<'info, Mint>,
    #[account(init, payer = signer, token::mint = betting_currency, token::authority = global_auth_pda)]
    pub house_wallet: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}