use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::consts::{REDEEMER_WALLET, USER_ACCOUNT_SEED};
use crate::quick_bets_errors::QuickBetsErrors;
use crate::state::User;

pub fn handle_deduct_balance(ctx: Context<DeductBalance>, price : u8, item_id : u8) -> Result<()> {

    msg!("Buying item with id : {}", item_id);
    if ctx.accounts.signer.key
        != &Pubkey::from_str(REDEEMER_WALLET).unwrap()
    {
        return Err(QuickBetsErrors::Unauthorized.into());
    }

    if ctx.accounts.user_account.total_points < price as u16 {
        return Err(QuickBetsErrors::InsufficientBalance.into());
    }

    ctx.accounts.user_account.total_points -= price as u16;

    msg!("Bought item {} for {} points", item_id, price);

    Ok(())
}

pub fn handle_add_balance(ctx: Context<AddBalance>, amount : u16) -> Result<()> {
    ctx.accounts.user_account.total_points += amount;
    Ok(())
}
#[derive(Accounts)]
pub struct AddBalance<'info> {
    #[account(mut)]
    pub user_account: Account<'info, User>,
}



#[derive(Accounts)]
pub struct DeductBalance<'info> {
    pub signer: Signer<'info>,
    pub buyer : Signer<'info>,
    #[account(mut, seeds = [USER_ACCOUNT_SEED, buyer.key.as_ref()], bump)]
    pub user_account: Account<'info, User>,
}
