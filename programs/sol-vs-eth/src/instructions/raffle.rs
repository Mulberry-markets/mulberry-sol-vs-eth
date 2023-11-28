use anchor_lang::prelude::*;

use crate::consts::{ADMIN_WALLETS, USER_ACCOUNT_SEED};
use crate::quick_bets_errors::QuickBetsErrors;
use crate::state::{User, UserItemAccount};

pub fn handle_create_raffle(
    ctx: Context<CreateRaffleItem>,
    raffle_id: u8,
    price: u16,
    total_quantity: u16,
    limit_per_user: u16,
) -> Result<()> {
    if ctx.accounts.admin.key.to_string() != ADMIN_WALLETS {
        return Err(QuickBetsErrors::Unauthorized.into());
    }

    let item = &mut ctx.accounts.item.load_init()?;

    item.item_id = raffle_id as u16;
    item.price = price;
    item.total_quantity = total_quantity;
    item.quantity_left = total_quantity;
    item.limit_per_user = limit_per_user;
    Ok(())
}

pub fn handle_buy_raffle_tickets(
    ctx: Context<BuyRaffleTickets>,
    raffle_id: u8,
    discord_id: u64,
    tickets_amount: u16,
) -> Result<()> {
    let item = &mut ctx.accounts.item.load_mut()?;
    if ctx.accounts.user_account.total_points < item.price * tickets_amount {
        return Err(QuickBetsErrors::InsufficientBalance.into());
    }

    if item.quantity_left < tickets_amount {
        return Err(QuickBetsErrors::SoldOut.into());
    }
    if ctx.accounts.user_item_account.total_bought as u16 >= item.limit_per_user
        && item.limit_per_user != 0
    {
        return Err(QuickBetsErrors::LimitReached.into());
    }

    ctx.accounts.user_item_account.total_bought += tickets_amount as u8;
    ctx.accounts.user_item_account.total_spent += (item.price * tickets_amount) as u8;
    item.quantity_left -= tickets_amount;
    item.add_tickets(tickets_amount as u64, discord_id);
    ctx.accounts.user_account.total_points -= item.price * tickets_amount;

    msg!(
        "discord_id: {}, raffle_id: {}, price: {}, amount: {}",
        discord_id,
        raffle_id,
        item.price,
        tickets_amount
    );
    Ok(())
}

pub fn handle_create_raffle_account(
    ctx: Context<CreateRaffleAccount>,
    discord_id: u64,
) -> Result<()> {
    let user_item_account = &mut ctx.accounts.user_item_account;
    user_item_account.total_bought = 0;
    user_item_account.total_spent = 0;
    Ok(())
}

pub fn handle_close_raffle(
    ctx : Context<CloseRaffle>,
) -> Result<()> {
    let item = &mut ctx.accounts.item.load_mut()?;
    for i in 0..item.entrants.len() {
        msg!("discord_id: {}, tickets_bought: {}", item.entrants[i].discord_id, item.entrants[i].tickets_bought);
    }

    Ok(())
}

#[derive(Accounts)]
pub struct CloseRaffle<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, close = admin)]
    pub item: AccountLoader<'info, RaffleItem>,
}

#[derive(Accounts)]
#[instruction(raffle_id: u8, discord_id: u64, tickets_amount: u16)]
pub struct BuyRaffleTickets<'info> {
    pub buyer: Signer<'info>,
    #[account(mut, seeds = [USER_ACCOUNT_SEED, buyer.key.as_ref()], bump)]
    pub user_account: Account<'info, User>,
    #[account(mut, seeds = [discord_id.to_le_bytes().as_slice(), item.key().as_ref()], bump )]
    pub user_item_account: Account<'info, UserItemAccount>,
    #[account(mut)]
    pub item: AccountLoader<'info, RaffleItem>,
}

#[derive(Accounts)]
pub struct CreateRaffleItem<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(zero)]
    pub item: AccountLoader<'info, RaffleItem>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction( discord_id : u64)]
pub struct CreateRaffleAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub item: AccountLoader<'info, RaffleItem>,
    #[account(init, seeds = [discord_id.to_le_bytes().as_slice(), item.key().as_ref()], bump , payer = signer, space = 6 + 8)]
    pub user_item_account: Account<'info, UserItemAccount>,
    pub system_program: Program<'info, System>,
}

#[account(zero_copy)]
pub struct RaffleItem {
    pub item_id: u16,
    pub price: u16,
    pub total_quantity: u16,
    pub quantity_left: u16,
    pub limit_per_user: u16,
    pub _padding: [u8; 6],
    pub entrants: [Entry; 200],
}

#[zero_copy]
pub struct Entry {
    pub discord_id: u64,
    pub tickets_bought: u64,
}

impl RaffleItem {
    pub fn add_tickets(&mut self, tickets_count: u64, discord_id: u64) {
        for i in 0..self.entrants.len() {
            if self.entrants[i].discord_id == discord_id {
                self.entrants[i].tickets_bought += tickets_count;
                return;
            } else if self.entrants[i].discord_id == 0 {
                self.entrants[i].discord_id = discord_id;
                self.entrants[i].tickets_bought = tickets_count;
                return;
            }
        }
    }
}
