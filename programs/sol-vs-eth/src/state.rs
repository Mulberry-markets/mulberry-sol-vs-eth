use crate::sol_vs_eth_errors::SolVsEthErr;

use super::*;

#[account]
pub struct GlobalState {
    /// The base fee that's going to be charged on all bets on the 1 minute markets.
    /// every 100 is a 1% fee.
    pub betting_fees: u64,

    /// house is going to be matching the first bet on the 1 minute markets.
    /// max_house_match is the maximum amount of lamports that the house is going to match.
    pub max_house_match: u64,

    // admin responsible for cranking the program, initializing and finalizing bets.
    pub crank_admin: Pubkey,

    /// a temporary security measure, to pause the program in case of a bug so no more markets are created.
    pub paused: bool,

    // the central house wallet that will receive the fees and place matching bets
    pub house_wallet: Pubkey,

    // time in seconds to how long the anticipation phase will last
    pub anticipation_time: u64,

    // time in seconds to how long the betting phase will last
    pub betting_time: u64,

    pub betting_currency: Pubkey,

}

impl GlobalState {
    pub fn confirm_crank_admin(&self, signer_address: &Signer) -> Result<()> {
        if self.crank_admin != signer_address.key() {
            return Err(SolVsEthErr::InvalidAdmin.into());
        }
        msg!("Admin confirmed");
        Ok(())
    }
}


#[account]
#[derive(Default)]
pub struct GlobalAuth {}


#[account]
#[derive(Default)]
pub struct Game {
    // initial eth and sol prices, from pyth
    pub initial_eth_price: u64,
    pub initial_sol_price: u64,

    // the size of the bet on each side
    pub eth_bet_size: u64,
    pub sol_bet_size: u64,

    // final price of eth and sol, from pyth, will be used for the resolution
    pub final_eth_price: u64,
    pub final_sol_price: u64,

    // marked to true if the results are out already
    pub is_settled: bool,

    // betting start time and anticipation start and end time.
    pub betting_start: u64,
    pub anticipating_start: u64,
    pub anticipating_end: u64,

    // the vault to store the collatoral for each bet.
    pub game_vault: Pubkey,

    // Amount and side that house matched
    pub house_bet_side: u8,
    pub house_bet_amount: u64,
    
}

impl Game {
    /// 0 meaning that sol had won,
    /// 1 meaning that eth had won
    /// 2 meaning that it's a draw
    pub fn get_winner(&self) -> u8 {
        let sol_change = (self.final_sol_price - self.initial_sol_price) as f64 / self.initial_sol_price as f64;
        let eth_change = (self.final_eth_price - self.initial_eth_price) as f64 / self.initial_eth_price as f64;

        if sol_change == eth_change {
            2
        } else if eth_change > sol_change {
            1
        } else {
            0
        }
    }

    pub fn betting_active(&self, duration: u64) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp as u64;
        if current_time > self.betting_start + duration {
            return Ok(false);
        }
        Ok(true)
    }

}


// #[account]
// pub struct User {
//     pub owner : Pubkey,
//     pub credit : u64,
// }

#[account]
pub struct UserBetAccount {
    pub amount: u64,
    pub side: u8,
    pub claimed: bool,
}
