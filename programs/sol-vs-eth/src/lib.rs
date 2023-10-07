use anchor_lang::prelude::*;

use instructions::*;

use crate::state::{ GlobalState};

mod state;
mod instructions;
mod consts;
mod sol_vs_eth_errors;
mod utils;


declare_id!("64Gkr29K1xh9WuKDTLVpHSGi5hqKrFoZDuSvAmJZxHgD");

#[program]
mod sol_vs_eth {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        handle_initialize(ctx)
    }

    pub fn start_game(ctx: Context<StartGame>) -> Result<()> {
        handle_start_game(ctx)
    }

    pub fn place_bet(ctx: Context<PlaceBet>, bet_size: u64, side: u8) -> Result<()> {
        handle_place_bet(ctx, bet_size, side)
    }

    pub fn start_anticipation(ctx: Context<StartAnticipation>) -> Result<()> {
        handle_start_anticipation(ctx)
    }

    pub fn resolve_game(ctx: Context<ResolveBet>) -> Result<()> {
        handle_resolve_game(ctx)
    }

    pub fn claim_win(ctx: Context<ClaimWin>) -> Result<()> {
        handle_claim_win(ctx)
    }

    // pub fn create_user_game_account(ctx: Context<CreateUserGameAccount>) -> Result<()> {
    //     handle_create_user_game_account(ctx)
    // }

    pub fn change_global_state(ctx: Context<ChangeGlobalState>, betting_fees: u64, max_house_match: u64, betting_period: u64, anticipation_period: u64) -> Result<()> {
        handle_change_global_state(ctx, betting_fees, max_house_match, betting_period, anticipation_period)
    }

    pub fn close_game(ctx: Context<CloseGame>) -> Result<()> {
        handle_close_game(ctx)
    }

    pub fn clean_game_records(ctx: Context<CleanGameRecords>) -> Result<()> {
        ctx.accounts.global_state.confirm_crank_admin(&ctx.accounts.signer)?;
        for _ in 0..5 {
            ctx.accounts.global_state.add_game_record(Pubkey::default());
        }

        ctx.accounts.global_state.to_close = Pubkey::default();
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CleanGameRecords<'info> {
    signer: Signer<'info>,
    #[account(mut)]
    global_state: Account<'info, GlobalState>,
}
