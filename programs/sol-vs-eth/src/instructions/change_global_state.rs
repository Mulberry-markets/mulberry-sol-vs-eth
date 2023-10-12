use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::consts::ADMIN_WALLETS;
use crate::state::GlobalState;

pub fn handle_change_global_state(
    ctx: Context<ChangeGlobalState>,
    betting_fees: u64,
    max_house_match: u64,
    betting_period: u64,
    anticipation_period: u64,
    max_user_bet: u64,
    min_multiplier: f64,
    max_house_bet_size : u64
) -> Result<()> {
    let global_state = &mut ctx.accounts.global_state;

    global_state.betting_fees = betting_fees;
    global_state.max_house_match = max_house_match;
    global_state.anticipation_time = anticipation_period;

    global_state.betting_time = betting_period;

    global_state.max_user_bet = max_user_bet;
    
    global_state.min_multiplier = min_multiplier;

    global_state.max_house_bet_size = max_house_bet_size;
    
    
    global_state.crank_admin = ctx.accounts.new_crank_admin.key();
    Ok(())
}

#[derive(Accounts)]
pub struct ChangeGlobalState<'info> {
    #[account(mut, address = Pubkey::from_str(ADMIN_WALLETS).unwrap())]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    /// CHECK: can techincally be any account
    pub new_crank_admin: AccountInfo<'info>,
}
