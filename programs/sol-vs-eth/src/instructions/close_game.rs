use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{GLOBAL_AUTH_SEED, GLOBAL_STATE_SEED};
use crate::sol_vs_eth_errors::SolVsEthErr;
use crate::state::{Game, GlobalAuth, GlobalState};

pub fn handle_close_game(ctx: Context<CloseGame>) -> Result<()> {
    ctx.accounts.global_state.confirm_crank_admin(&ctx.accounts.signer)?;

    let game = &mut ctx.accounts.game;


    for i in ctx.accounts.global_state.game_records.iter() {
        if i.game_address == game.key() {
            return err!(SolVsEthErr::GameNotClosed);
        }
    }

    ctx.accounts.global_state.to_close = Pubkey::default();


    require!(game.check_all_bets_claimed(), SolVsEthErr::BetsNotClaimed);

    if ctx.accounts.game_vault.amount != 0 {
        return err!(SolVsEthErr::VaultNotEmpty);
    }

    let cpi_accounts = token::CloseAccount {
        account: ctx.accounts.game_vault.to_account_info(),
        destination: ctx.accounts.signer.to_account_info(),
        authority: ctx.accounts.global_auth_pda.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let bump = *ctx.bumps.get("global_auth_pda").unwrap();
    let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);

    token::close_account(cpi_ctx)?;

    Ok(())
}


#[derive(Accounts)]
pub struct CloseGame<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, close = signer)]
    pub game: Box<Account<'info, Game>>,

    #[account(mut, seeds = [GLOBAL_AUTH_SEED], bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,

    #[account(mut, seeds = [GLOBAL_STATE_SEED], bump)]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub game_vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}