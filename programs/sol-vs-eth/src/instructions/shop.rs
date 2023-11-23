use anchor_lang::prelude::*;

use crate::consts::{ADMIN_WALLETS, USER_ACCOUNT_SEED};
use crate::quick_bets_errors::QuickBetsErrors;
use crate::state::User;

pub fn handle_buy_item(ctx: Context<BuyItem>,edition: u8, item_id: u8, discord_id : u64) -> Result<()> {
    let item = &mut ctx.accounts.item;
    if ctx.accounts.user_account.total_points < item.price as u16 {
        return Err(QuickBetsErrors::InsufficientBalance.into());
    }

    if item.quantity_left == 0  {
        return Err(QuickBetsErrors::SoldOut.into());
    }

    item.quantity_left -= 1;
    ctx.accounts.user_account.total_points -= item.price as u16;


    msg!("discord_id: {}, item_id: {}, price: {}", discord_id, item_id, item.price);
    Ok(())
}

pub fn handle_list_item(
    ctx: Context<ListItem>,
    item_id: u8,
    price: u8,
    total_quantity: u8,
    quantity_left: u8,
    edition: u8,
) -> Result<()> {
    if ctx.accounts.admin.key.to_string() != ADMIN_WALLETS {
        return Err(QuickBetsErrors::Unauthorized.into());
    }
    ctx.accounts.item.item_id = item_id;
    ctx.accounts.item.price = price;
    ctx.accounts.item.total_quantity = total_quantity;
    ctx.accounts.item.quantity_left = quantity_left;
    ctx.accounts.item.edition = edition;
    Ok(())
}

pub fn handle_add_balance(ctx: Context<AddBalance>, amount: u16) -> Result<()> {
    return Ok(());
    // ctx.accounts.user_account.total_points += amount;
    // Ok(())
}

pub fn handle_change_price(ctx: Context<ChangePrice>, new_price: u8, edition: u8, item_id: u8) -> Result<()> {
    if ctx.accounts.admin.key.to_string() != ADMIN_WALLETS {
        return Err(QuickBetsErrors::Unauthorized.into());
    }
    ctx.accounts.item.price = new_price;
    Ok(())
}

#[derive(Accounts)]
pub struct AddBalance<'info> {
    #[account(mut)]
    pub user_account: Account<'info, User>,
}

#[derive(Accounts)]
#[instruction(edition: u8,item_id: u8, discord_id: u64)]
pub struct BuyItem<'info> {
    pub buyer: Signer<'info>,
    #[account(mut, seeds = [USER_ACCOUNT_SEED, buyer.key.as_ref()], bump)]
    pub user_account: Account<'info, User>,
    #[account(mut, seeds = [&[item_id, edition]], bump)]
    pub item: Account<'info, ShopItem>,
}

#[derive(Accounts)]
#[instruction(item_id: u8, price: u8, total_quantity: u8, quantity_left: u8, edition: u8)]
pub struct ListItem<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, seeds = [&[item_id, edition]], bump, payer= admin, space= 2 + 2+ 2+2 + 8)]
    pub item: Account<'info, ShopItem>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(new_price: u8, edition: u8, item_id: u8)]
pub struct ChangePrice<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [&[item_id, edition]], bump)]
    pub item: Account<'info, ShopItem>,
}


#[account]
pub struct ShopItem {
    item_id: u8,
    price: u8,
    total_quantity: u8,
    quantity_left: u8,
    edition: u8,
}
