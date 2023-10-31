use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::GLOBAL_AUTH_SEED;
use crate::quick_bets_errors::QuickBetsErrors;
use crate::state::{Game, GlobalAuth, GlobalState};
use crate::utils::transfer_tokens;
use std::str::FromStr;


pub fn handle_place_bet(ctx: Context<PlaceBet>, bet_size: u64, side: u8) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let global_state = &ctx.accounts.global_state;
    let payer = &ctx.accounts.payer;


    require!(game.betting_active(global_state.betting_time)?, QuickBetsErrors::BettingInactive);


    // check if there's any bet on the other side, if not, then match it upto the max_house_match.
    let match_bet = match side {
        0 => global_state.max_house_match > game.house_bet_amount,
        1 => global_state.max_house_match > game.house_bet_amount,
        _ => {
            msg!("Invalid side");
            return Err(QuickBetsErrors::InvalidSide.into());
        }
    };
    let house_match_left = global_state.max_house_match - game.house_bet_amount;
    if match_bet {
        let matched_amount = std::cmp::min(bet_size, house_match_left);
        if matched_amount > ctx.accounts.house_wallet.amount {
            msg!("Not enough funds to match bet");
            return Err(QuickBetsErrors::HouseBankrupt.into());
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
        game.house_bet_amount += matched_amount;
        if side == 0 {
            game.eth_bet_size += matched_amount;
            // house is betting the opposite side
        } else {
            game.sol_bet_size += matched_amount;
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

    // update the game state
    if side == 0 {
        game.sol_bet_size += bet_size;
    } else {
        game.eth_bet_size += bet_size;
    }
    msg!("user bet size : {}", bet_size);
    let total_user_bet = game.add_user_bet(payer.key(), ctx.accounts.signer.key(), bet_size, side)?;
    msg!("total user bet : {}", total_user_bet);
    require!(total_user_bet <= global_state.max_user_bet, QuickBetsErrors::MaxUserBetExceeded);
    Ok(())
}


#[derive(Accounts)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub payer: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub game: Box<Account<'info, Game>>,

    #[account(mut,
    seeds = [GLOBAL_AUTH_SEED],
    bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,

    #[account(mut)]
    pub game_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = global_state.house_wallet == house_wallet.key())]
    pub house_wallet: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = Pubkey::from_str("GnVThWobQUHgr99r1ihvPbnUK5YXTMSCXFQP74XSuT67").unwrap())]
    pub fees_wallet : Box<Account<'info, TokenAccount>>,
    pub global_state: Box<Account<'info, GlobalState>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}