use std::collections::HashMap;
use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token;
use anchor_spl::token::{Token, TokenAccount};

use crate::consts::{ETH_ORACLE, GLOBAL_AUTH_SEED, GLOBAL_STATE_SEED, MARGIN_OF_ERROR, SOL_ORACLE};
use crate::quick_bets_errors::QuickBetsErrors;
use crate::state::{Game, GameStatus, GlobalAuth, GlobalState, User};
use crate::utils::{get_price_from_pyth, transfer_tokens};

pub fn handle_resolve_game(ctx: Context<ResolveBet>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let global_state = &mut ctx.accounts.global_state;


    global_state.confirm_crank_admin(&ctx.accounts.signer)?;

    require!(!game.is_settled, QuickBetsErrors::BetAlreadySettled);
    msg!("anticipation start : {}", game.anticipating_start);
    msg!("current time: {} ", Clock::get()?.unix_timestamp);

    if game.anticipating_start + global_state.anticipation_time
        > Clock::get()?.unix_timestamp as u64 + MARGIN_OF_ERROR
    {
        return Err(QuickBetsErrors::AnticipationTimeTooSoon.into());
    }

    if Pubkey::from_str(SOL_ORACLE).unwrap() != *ctx.accounts.sol_feed.key {
        return Err(QuickBetsErrors::InvalidOracle.into());
    }

    if Pubkey::from_str(ETH_ORACLE).unwrap() != *ctx.accounts.eth_feed.key {
        return Err(QuickBetsErrors::InvalidOracle.into());
    }

    let sol_price = get_price_from_pyth(ctx.accounts.sol_feed.clone())?;
    let eth_price = get_price_from_pyth(ctx.accounts.eth_feed.clone())?;

    game.final_sol_price = sol_price;
    game.final_eth_price = eth_price;
    game.is_settled = true;
    game.anticipating_end = Clock::get()?.unix_timestamp as u64;

    ctx.accounts
        .global_state
        .modify_game_record(game.key(), GameStatus::Resolved);

    let pool_size = game.sol_bet_size + game.eth_bet_size;
    let winners_multiplier = if game.get_winner() == 0 {
        pool_size as f64 / game.sol_bet_size as f64
    } else if game.get_winner() == 1 {
        pool_size as f64 / game.eth_bet_size as f64
    } else {
        0.0
    };

    let amount_owed_to_winners = game.get_amount_owed_to_winners(winners_multiplier);
    msg!("Amount owed to winners: {}", amount_owed_to_winners);
    msg!("Amount in game vault: {}", ctx.accounts.game_vault.amount);
    let won_by_house = ctx.accounts.game_vault.amount - amount_owed_to_winners;

    if won_by_house > 0 {
        let bump = *ctx.bumps.get("global_auth_pda").unwrap();
        let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
        transfer_tokens(
            ctx.accounts.game_vault.to_account_info(),
            ctx.accounts.house_wallet.to_account_info(),
            ctx.accounts.global_auth_pda.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            won_by_house,
            Some(seeds),
        )?;
    }

    let account_infos = [
        ctx.accounts.player_account_1.to_account_info(),
        ctx.accounts.player_account_2.to_account_info(),
        ctx.accounts.player_account_3.to_account_info(),
        ctx.accounts.player_account_4.to_account_info(),
        ctx.accounts.player_account_5.to_account_info(),
        ctx.accounts.player_account_6.to_account_info(),
        ctx.accounts.player_account_7.to_account_info(),
    ];


    let mut total_payouts = 0;
    let mut amount_to_pay: HashMap<Pubkey, u64> = HashMap::new();


    for  (i, account) in account_infos.iter().enumerate() {
        if account.key == ctx.accounts.signer.key {
            continue;
        }

        let user_bet = if let Some(user_bet) = game.get_user_bet(account.key()) {
            user_bet
        } else {
            continue;
        };
        if user_bet.claimed {
            continue;
        }

        msg!("adding user record, amount: {}, side: {}", user_bet.amount, user_bet.side == game.get_winner());
        match i {
            0 => {

                ctx.accounts.player_user_account_1.add_bet_record(user_bet.amount, user_bet.side == game.get_winner());
            }
            1 => {
                ctx.accounts.player_user_account_2.add_bet_record(user_bet.amount, user_bet.side == game.get_winner());
            }
            2 => {
                ctx.accounts.player_user_account_3.add_bet_record(user_bet.amount, user_bet.side == game.get_winner());
            }
            3 => {
                ctx.accounts.player_user_account_4.add_bet_record(user_bet.amount, user_bet.side == game.get_winner());
            }
            4 => {
                ctx.accounts.player_user_account_5.add_bet_record(user_bet.amount, user_bet.side == game.get_winner());
            }
            5 => {
                ctx.accounts.player_user_account_6.add_bet_record(user_bet.amount, user_bet.side == game.get_winner());
            }
            6 => {
                ctx.accounts.player_user_account_7.add_bet_record(user_bet.amount, user_bet.side == game.get_winner());
            }
            _ => {}
        }
        let payout_amount = game.calculate_winning_amount(user_bet.amount, user_bet.side);
        game.mark_bet_claimed(*account.key)?;
        total_payouts += payout_amount;
        if payout_amount > 0 {
            amount_to_pay.insert(account.key(), payout_amount);
        }
    }

    if total_payouts == 0 {
        let cpi_accounts = token::CloseAccount {
            account: ctx.accounts.signer_wsol_account.to_account_info(),
            destination: ctx.accounts.signer.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::close_account(cpi_ctx)?;
        return Ok(());
    }
    let bump = *ctx.bumps.get("global_auth_pda").unwrap();
    let seeds: &[&[&[u8]]] = &[&[GLOBAL_AUTH_SEED, &[bump]]];
    transfer_tokens(
        ctx.accounts.game_vault.to_account_info(),
        ctx.accounts.signer_wsol_account.to_account_info(),
        ctx.accounts.global_auth_pda.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        total_payouts,
        Some(seeds),
    )?;

    let cpi_accounts = token::CloseAccount {
        account: ctx.accounts.signer_wsol_account.to_account_info(),
        destination: ctx.accounts.signer.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::close_account(cpi_ctx)?;

    for (key, value) in amount_to_pay {
        // get key account from the accounts array
        let mut user = ctx.accounts.signer_wsol_account.to_account_info();
        for account in ctx.accounts.to_account_infos() {
            if account.key() == key {
                user = account
            }
        }
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.signer.to_account_info().clone(),
                to: user.clone(),
            },
        );
        system_program::transfer(cpi_context, value)?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ResolveBet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub game: Box<Account<'info, Game>>,
    #[account(mut)]
    pub signer_wsol_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [GLOBAL_STATE_SEED], bump)]
    pub global_state: Account<'info, GlobalState>,
    /// CHECK: Checking this manually in the instruction
    pub sol_feed: AccountInfo<'info>,
    /// CHECK: Checking this manually in the instruction
    pub eth_feed: AccountInfo<'info>,
    #[account(mut, constraint = game.game_vault == game_vault.key())]
    pub game_vault: Account<'info, TokenAccount>,

    #[account(mut,
    seeds = [GLOBAL_AUTH_SEED],
    bump)]
    pub global_auth_pda: Box<Account<'info, GlobalAuth>>,
    #[account(mut, constraint = global_state.house_wallet == house_wallet.key())]
    pub house_wallet: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    // These accounts are technically optional
    /// CHECK: will be checked manually in the instruction
    #[account(mut)]
    pub player_account_1: AccountInfo<'info>,
    #[account(mut)]
    pub player_user_account_1: Box<Account<'info, User>>,
    /// CHECK: will be checked manually in the instruction
    #[account(mut)]
    pub player_account_2: AccountInfo<'info>,
    #[account(mut)]
    pub player_user_account_2: Box<Account<'info, User>>,
    /// CHECK: will be checked manually in the instruction
    #[account(mut)]
    pub player_account_3: AccountInfo<'info>,

    #[account(mut)]
    pub player_user_account_3: Box<Account<'info, User>>,
    /// CHECK: will be checked manually in the instruction
    #[account(mut)]
    pub player_account_4: AccountInfo<'info>,
    #[account(mut)]
    pub player_user_account_4: Box<Account<'info, User>>,
    /// CHECK: will be checked manually in the instruction
    #[account(mut)]
    pub player_account_5: AccountInfo<'info>,
    #[account(mut)]
    pub player_user_account_5: Box<Account<'info, User>>,
    /// CHECK: will be checked manually in the instruction
    #[account(mut)]
    pub player_account_6: AccountInfo<'info>,
    #[account(mut)]
    pub player_user_account_6: Box<Account<'info, User>>,
    /// CHECK: will be checked manually in the instruction
    #[account(mut)]
    pub player_account_7: AccountInfo<'info>,
    #[account(mut)]
    pub player_user_account_7: Box<Account<'info, User>>,
}
