use std::mem::size_of;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use anchor_spl::token::TokenAccount;

use crate::consts::{GLOBAL_AUTH_SEED, GLOBAL_STATE_SEED};
use crate::state::{Game, GlobalAuth, GlobalState};

pub fn handle_start_game(ctx: Context<StartGame>) -> Result<()> {
    ctx.accounts.global_state.confirm_crank_admin(&ctx.accounts.signer)?;

    let game = &mut ctx.accounts.game;
    game.betting_start = Clock::get()?.unix_timestamp as u64;
    game.eth_bet_size = 0;
    game.sol_bet_size = 0;
    game.game_vault = ctx.accounts.game_vault.key();


    Ok(())
}


#[derive(Accounts)]
pub struct StartGame<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, payer = signer, space = size_of::< Game > () + 12)]
    pub game: Account<'info, Game>,

    #[account(mut, seeds = [GLOBAL_AUTH_SEED], bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,

    #[account(mut, seeds = [GLOBAL_STATE_SEED], bump)]
    pub global_state: Account<'info, GlobalState>,

    #[account(constraint = global_state.betting_currency == betting_token.key())]
    pub betting_token: Account<'info, Mint>,

    #[account(init, payer = signer, token::mint = betting_token, token::authority = global_auth_pda)]
    pub game_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}