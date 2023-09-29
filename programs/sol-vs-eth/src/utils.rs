use std::str::FromStr;
use anchor_lang::prelude::*;
use anchor_spl::token;
use pyth_sdk_solana::load_price_feed_from_account_info;
use crate::consts::ADMIN_WALLETS;
use crate::sol_vs_eth_errors::SolVsEthErr;

pub fn get_price_from_pyth(oracle_address: AccountInfo) -> Result<u64> {
    let sol_price_feed = load_price_feed_from_account_info(&oracle_address).unwrap();

    let sol_price = sol_price_feed.get_price_no_older_than(Clock::get()?.unix_timestamp, 10).unwrap();
    let sol_price_lamports = sol_price.price;

    Ok(sol_price_lamports as u64)
}


pub fn confirm_admin(signer_address: &Signer) -> Result<()> {
    // ! NOTE: this is going to be temporary before the market goes full on permissionless
    let market_admin_addresses: Vec<Pubkey> = ADMIN_WALLETS
        .iter()
        .map(|address| Pubkey::from_str(address).unwrap())
        .collect();

    if !market_admin_addresses.contains(&signer_address.key()) {
        return Err(SolVsEthErr::InvalidAdmin.into());
    }
    msg!("Admin confirmed");
    Ok(())
}

pub fn transfer_tokens<'a>(
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
    amount: u64,
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let cpi_accounts = token::Transfer {
        from,
        to,
        authority,
    };
    let cpi_program = token_program;

    let cpi_ctx = match seeds {
        Some(s) => CpiContext::new_with_signer(cpi_program, cpi_accounts, s),
        None => CpiContext::new(cpi_program, cpi_accounts),
    };

    token::transfer(cpi_ctx, amount)?;
    Ok(())
}
