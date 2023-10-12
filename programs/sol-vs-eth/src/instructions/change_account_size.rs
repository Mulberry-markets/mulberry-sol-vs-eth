use crate::consts::ADMIN_WALLETS;
use crate::quick_bets_errors::QuickBetsErrors;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;

pub fn handle_change_account_size(ctx: Context<ChangeAccountSize>, new_size: u64) -> Result<()> {
    let signer = &ctx.accounts.signer;
    let system_program = &ctx.accounts.system_program;
    let account_to_resize = &mut ctx.accounts.account_to_resize;

    if signer.key.to_string() != ADMIN_WALLETS {
        return Err(QuickBetsErrors::Unauthorized.into());
    }

    require!(
        account_to_resize.data_len() < new_size as usize,
        QuickBetsErrors::InvalidSize
    );

    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size as usize);
    let lamports_diff = new_minimum_balance.saturating_sub(account_to_resize.lamports());

    invoke(
        &system_instruction::transfer(signer.key, account_to_resize.key, lamports_diff),
        &[
            signer.to_account_info().clone(),
            account_to_resize.to_account_info().clone(),
            system_program.to_account_info().clone(),
        ],
    )?;

    account_to_resize.realloc(new_size as usize, false)?;

    Ok(())
}

#[derive(Accounts)]
pub struct ChangeAccountSize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: Doesn't matter what account we use here, we are only increasing the size anyway
    #[account(mut)]
    pub account_to_resize: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
