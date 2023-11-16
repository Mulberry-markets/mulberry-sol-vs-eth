use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::consts::USER_ACCOUNT_SEED;
use crate::quick_bets_errors::QuickBetsErrors;
use crate::state::User;

// Result will be sent from an authority account from the backend
pub fn handle_use_spin(ctx: Context<UseSpin>, result: u16) -> Result<()> {
    if ctx.accounts.signer.key
        != &Pubkey::from_str("F25WqA7xPLZboJ9Ydad4Z9wGrw25gawkK9jai8nbRsr7").unwrap()
    {
        return Err(QuickBetsErrors::Unauthorized.into());
    }

    if result > 10_000 {
        return Err(QuickBetsErrors::InvalidSize.into());
    }
    ctx.accounts.user_spin_account.check_spin_eligible()?;
    ctx.accounts.user_spin_account.register_spin(result);


    Ok(())
}

pub fn handle_claim_spin_reward(ctx: Context<ClaimSpinReward>) -> Result<()> {
    ctx.accounts.user_spin_account.claimed = true;
    Ok(())
}

pub fn handle_create_user_spin_account(_ctx: Context<CreateUserSpinAccount>) -> Result<()> {
    Ok(())
}

pub fn handle_close_user_spin_account(_ctx: Context<CloseUserSpinAccount>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct UseSpin<'info> {
    pub signer: Signer<'info>,
    /// CHECK: a check isn't needed for this
    pub user: AccountInfo<'info>,
    #[account(mut, seeds = [USER_ACCOUNT_SEED, user.key.as_ref()], bump)]
    pub user_spin_account: Account<'info, User>,
}

#[derive(Accounts)]
pub struct ClaimSpinReward<'info> {
    pub signer: Signer<'info>,
    #[account(mut, seeds = [USER_ACCOUNT_SEED, signer.key.as_ref()], bump)]
    pub user_spin_account: Account<'info, User>,
}

#[derive(Accounts)]
pub struct CreateUserSpinAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(init,seeds = [USER_ACCOUNT_SEED, signer.key.as_ref()], bump,payer = signer, space = 100)]
    pub user_spin_account: Account<'info, User>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseUserSpinAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut,close = signer)]
    pub user_spin_account: Account<'info, User>,
    pub system_program: Program<'info, System>,
}
