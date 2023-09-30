use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::consts::GLOBAL_AUTH_SEED;
use crate::sol_vs_eth_errors::SolVsEthErr;
use crate::state::{Game, GlobalAuth, GlobalState, UserBetAccount};
use crate::utils::transfer_tokens;

pub fn handle_place_bet(ctx: Context<PlaceBet>, bet_size: u64, side: u8) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let global_state = &ctx.accounts.global_state;
    let payer = &ctx.accounts.payer;
    let use_game_account = &mut ctx.accounts.user_game_account;


    require!(game.betting_active(global_state.betting_time)?, SolVsEthErr::BettingInactive);


    // Users can only bet one one side.
    if use_game_account.side != side && use_game_account.side != u8::MAX {
        msg!("You already have a bet on the other side");
        return Err(SolVsEthErr::AlreadyBet.into());
    }
    use_game_account.side = side;

    // check if there's any bet on the other side, if not, then match it upto the max_house_match.
    let match_bet = match side {
        0 => game.eth_bet_size == 0,
        1 => game.sol_bet_size == 0,
        _ => {
            msg!("Invalid side");
            return Err(SolVsEthErr::InvalidSide.into());
        }
    };

    if match_bet {
        let matched_amount = std::cmp::min(bet_size, global_state.max_house_match);
        if matched_amount > payer.amount {
            msg!("Not enough funds");
            return Err(SolVsEthErr::HouseBankrupt.into());
        }

        let bump = *ctx.bumps.get("global_auth_pda").unwrap();
        let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
        transfer_tokens(
            ctx.accounts.house_wallet.to_account_info(),
            ctx.accounts.game_vault.to_account_info(),
            ctx.accounts.global_auth_pda.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            matched_amount,
            Some(seeds),
        )?;
        game.house_bet_amount = matched_amount;
        if side == 0 {
            game.eth_bet_size = matched_amount;
            // house is betting the opposite side
            game.house_bet_side = 1;
        } else {
            game.sol_bet_size = matched_amount;
            game.house_bet_side = 0;
        }
    }


    // transfer the user bet to the vault
    transfer_tokens(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.game_vault.to_account_info(),
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        bet_size,
        None,
    )?;

    // transfer the fees to the house wallet
    let fee = (bet_size as f64 * (global_state.betting_fees as f64 / 100.0 / 100.0)) as u64;
    transfer_tokens(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.house_wallet.to_account_info(),
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        fee,
        None,
    )?;


    Ok(())
}


#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Account<'info, TokenAccount>,
    pub game: Account<'info, Game>,
    pub betting_token: Account<'info, Mint>,

    #[account(mut, seeds = [game.key().as_ref(), signer.key().as_ref()], bump)]
    pub user_game_account: Account<'info, UserBetAccount>,

    #[account(mut,
    seeds = [GLOBAL_AUTH_SEED],
    bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,

    #[account(mut, constraint = game.game_vault == game_vault.key())]
    pub game_vault: Account<'info, TokenAccount>,

    #[account(mut, constraint = global_state.house_wallet == house_wallet.key())]
    pub house_wallet: Account<'info, TokenAccount>,

    pub global_state: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}