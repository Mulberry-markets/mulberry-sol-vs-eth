use anchor_lang::prelude::*;

use crate::instructions::*;
use crate::state::GlobalState;

mod consts;
mod instructions;
mod quick_bets_errors;
mod state;
mod utils;

declare_id!("6gbfSJq7YN6To7TqYhHpZRh22P33MTtc47EAUr9fEYKM");

#[program]
mod mulberry_quick_bets {
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

    #[allow(clippy::too_many_arguments)]
    pub fn change_global_state(
        ctx: Context<ChangeGlobalState>,
        betting_fees: u64,
        max_house_match: u64,
        betting_period: u64,
        anticipation_period: u64,
        max_user_bet: u64,
        max_house_bet_size: u64,
        min_multiplier: f64,
    ) -> Result<()> {
        handle_change_global_state(
            ctx,
            betting_fees,
            max_house_match,
            betting_period,
            anticipation_period,
            max_user_bet,
            min_multiplier,
            max_house_bet_size,
        )
    }

    pub fn close_game(ctx: Context<CloseGame>) -> Result<()> {
        handle_close_game(ctx)
    }

    pub fn clean_game_records(ctx: Context<CleanGameRecords>) -> Result<()> {
        ctx.accounts
            .global_state
            .confirm_crank_admin(&ctx.accounts.signer)?;
        for _ in 0..5 {
            ctx.accounts.global_state.add_game_record(Pubkey::default());
        }

        ctx.accounts.global_state.to_close = Pubkey::default();
        Ok(())
    }

    pub fn change_account_size(ctx: Context<ChangeAccountSize>, new_size: u64) -> Result<()> {
        handle_change_account_size(ctx, new_size)
    }

    pub fn withdraw_funds(ctx: Context<WithdrawFunds>, amount: u64) -> Result<()> {
        handle_withdraw_funds(ctx, amount)
    }

    // pub fn use_spin(ctx: Context<UseSpin>, result: u16) -> Result<()> {
    //     handle_use_spin(ctx, result)
    // }

    // pub fn claim_spin_reward(ctx: Context<ClaimSpinReward>) -> Result<()> {
    //     handle_claim_spin_reward(ctx)
    // }

    pub fn create_user_spin_account(ctx: Context<CreateUserSpinAccount>) -> Result<()> {
        handle_create_user_spin_account(ctx)
    }

    // pub fn close_user_spin_account(ctx: Context<CloseUserSpinAccount>) -> Result<()> {
    //     handle_close_user_spin_account(ctx)
    // }

    pub fn buy_item(
        ctx: Context<BuyItem>,
        edition: u8,
        item_id: u8,
        discord_id: u64,
    ) -> Result<()> {
        handle_buy_item(ctx, edition, item_id, discord_id)
    }

    pub fn add_balance(ctx: Context<AddBalance>, amount: u16) -> Result<()> {
        handle_add_balance(ctx, amount)
    }

    pub fn list_item(
        ctx: Context<ListItem>,
        item_id: u8,
        price: u8,
        total_quantity: u8,
        quantity_left: u8,
        edition: u8,
        limit_per_user: u8
    ) -> Result<()> {
        handle_list_item(ctx, item_id, price, total_quantity, quantity_left, edition, limit_per_user)
    }

    pub fn change_price(ctx: Context<ChangePrice>, new_price: u8, edition: u8, item_id: u8) -> Result<()> {
        handle_change_price(ctx, new_price, edition, item_id)
    }

    pub fn init_airdrop(ctx: Context<InitAirdrop>) -> Result<()> {
        handle_init_airdrop(ctx)
    }

    pub fn reset_airdrop(ctx: Context<ResetAirdrop>, amount : u64) -> Result<()> {
        handle_reset_airdrop(ctx, amount)
    }

    pub fn create_airdrop_account(ctx: Context<CreateAirdropAccount>, discord_id: u64) -> Result<()> {
        handle_create_airdrop_account(ctx, discord_id)
    }

    pub fn claim_airdrop(ctx: Context<ClaimAirdrop>, discord_id: u64) -> Result<()> {
        handle_claim_reward(ctx, discord_id)
    }

    pub fn create_item_account(ctx: Context<CreateItemAccount>, item_id: u8,
                               edition: u8,) -> Result<()> {
        handle_create_item_account(ctx, item_id, edition)
    }

    pub fn change_limit_per_user(ctx: Context<ChangeLimitPerUser>, limit_per_user: u8,
                                 edition: u8,
                                 item_id: u8,) -> Result<()> {
        handle_change_limit_per_user(ctx, limit_per_user, edition, item_id)
    }

    pub fn restock(ctx: Context<RestockItems>,amount: u8, edition: u8, item_id: u8) -> Result<()> {
        handle_restock(ctx, amount, edition, item_id)
    }

}

#[derive(Accounts)]
pub struct CleanGameRecords<'info> {
    signer: Signer<'info>,
    #[account(mut)]
    global_state: Account<'info, GlobalState>,
}
