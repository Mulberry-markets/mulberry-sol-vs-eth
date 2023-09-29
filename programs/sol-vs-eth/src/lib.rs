use anchor_lang::prelude::*;

use instructions::*;

mod state;
mod instructions;
mod consts;
mod sol_vs_eth_errors;
mod utils;


declare_id!("AHX2zXNjyVNBWMyMUcpPtcxiuWG44ZzEmPdKe4z8KnSC");

#[program]
mod sol_vs_eth {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        handle_initialize(ctx)
    }

    pub fn start_betting(ctx: Context<StartBetting>) -> Result<()> {
        handle_start_betting(ctx)
    }

    pub fn place_bet(ctx: Context<PlaceBet>, bet_size: u64, side: u8) -> Result<()> {
        handle_place_bet(ctx, bet_size, side)
    }

    pub fn start_anticipation(ctx: Context<StartAnticipation>) -> Result<()> {
        handle_start_anticipation(ctx)
    }

    pub fn resolve_bet(ctx: Context<ResolveBet>) -> Result<()> {
        handle_resolve_bet(ctx)
    }

    pub fn claim_win(ctx: Context<ClaimWin>) -> Result<()> {
        handle_claim_win(ctx)
    }

    pub fn create_user_bet_account(ctx: Context<CreateBetUser>) -> Result<()> {
        handle_create_bet_user(ctx)
    }
}

