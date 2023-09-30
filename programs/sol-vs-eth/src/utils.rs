use anchor_lang::prelude::*;
use anchor_spl::token;
use pyth_sdk_solana::load_price_feed_from_account_info;

pub fn get_price_from_pyth(oracle_address: AccountInfo) -> Result<u64> {
    let price_feed = load_price_feed_from_account_info(&oracle_address).unwrap();

    let price = price_feed.get_price_no_older_than(Clock::get()?.unix_timestamp, 10).unwrap();
    // let price = price_feed.get_price_unchecked();

    let price_lots = price.price;

    Ok(price_lots as u64)
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
