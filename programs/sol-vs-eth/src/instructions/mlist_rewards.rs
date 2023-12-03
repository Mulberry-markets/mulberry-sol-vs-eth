use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::consts::REDEEMER_WALLET;
use crate::quick_bets_errors::QuickBetsErrors;

pub fn handle_init_airdrop(ctx: Context<InitAirdrop>) -> Result<()> {
    let airdrop = &mut ctx.accounts.airdrop;
    airdrop.amount = 0;
    airdrop.start_time = 0;
    airdrop.current_airdrop = 0;
    Ok(())
}

pub fn handle_reset_airdrop(ctx: Context<ResetAirdrop>, amount: u64) -> Result<()> {
    let airdrop = &mut ctx.accounts.airdrop;
    airdrop.amount = amount;
    airdrop.start_time = Clock::get()?.unix_timestamp as u64;
    airdrop.current_airdrop += 1;
    Ok(())
}

pub fn handle_create_airdrop_account(
    ctx: Context<CreateAirdropAccount>,
    discord_id: u64,
    mlists: u16,
) -> Result<()> {
    ctx.accounts.user_airdrop_account.last_claimed = 0;
    ctx.accounts.user_airdrop_account.mlists_count = mlists;
    Ok(())
}

pub fn handle_change_mlists_count(
    ctx: Context<ChangeMlistsCount>,
    discordId: u64,
    mlist_count: u16,
) -> Result<()> {
    if ctx.accounts.admin.key().to_string() != REDEEMER_WALLET {
        return Err(QuickBetsErrors::Unauthorized.into());
    }
    ctx.accounts.user_airdrop_account.mlists_count = mlist_count;
    Ok(())
}

#[derive(Accounts)]
#[instruction(discord_id: u64)]
pub struct ChangeMlistsCount<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut, seeds = [b"airdrop_account",  discord_id.to_le_bytes().as_slice()], bump)]
    pub user_airdrop_account: Account<'info, UserAirdropAccount>,
}

pub fn handle_claim_reward(ctx: Context<ClaimAirdrop>, discord_id: u64) -> Result<()> {
    let airdrop = &mut ctx.accounts.airdrop;
    let user_airdrop_account = &mut ctx.accounts.user_airdrop_account;
    if ctx.accounts.admin.key().to_string() != REDEEMER_WALLET {
        return Err(QuickBetsErrors::Unauthorized.into());
    }

    if user_airdrop_account.mlists_count == 0 {
        return Err(QuickBetsErrors::NoMlists.into());
    }

    require!(
        user_airdrop_account.last_claimed < airdrop.current_airdrop,
        QuickBetsErrors::RewardAlreadyClaimed
    );

    user_airdrop_account.last_claimed = airdrop.current_airdrop;

    let cpi_context = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: ctx.accounts.admin.to_account_info().clone(),
            to: ctx.accounts.receiver.to_account_info().clone(),
        },
    );
    system_program::transfer(
        cpi_context,
        airdrop.amount * user_airdrop_account.mlists_count as u64,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct InitAirdrop<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init, seeds=[b"airdrop"],bump, payer = signer, space = 24 + 8)]
    pub airdrop: Account<'info, Airdrop>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResetAirdrop<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub airdrop: Account<'info, Airdrop>,
}

#[derive(Accounts)]
#[instruction(discord_id: u64)]
pub struct CreateAirdropAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init, seeds=[b"airdrop_account", discord_id.to_le_bytes().as_slice()],bump, payer = signer, space = 20)]
    pub user_airdrop_account: Account<'info, UserAirdropAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(discord_id: u64)]
pub struct ClaimAirdrop<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub receiver: Signer<'info>,
    #[account(mut)]
    pub airdrop: Account<'info, Airdrop>,
    #[account(mut, seeds = [b"airdrop_account",  discord_id.to_le_bytes().as_slice()], bump)]
    pub user_airdrop_account: Account<'info, UserAirdropAccount>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Airdrop {
    pub amount: u64,
    pub start_time: u64,
    pub current_airdrop: u16,
}

#[account]
pub struct UserAirdropAccount {
    pub last_claimed: u16,
    pub mlists_count: u16,
}
