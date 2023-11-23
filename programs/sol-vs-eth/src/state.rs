use crate::quick_bets_errors::QuickBetsErrors;

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

    // storing the latest 5 games, including the current active game.
    pub game_records: [GameRecord; 5],

    pub to_close: Pubkey,

    // the minimum multiplier for users bets
    pub min_multiplier: f64,

    // a theoretical limit to how much the house is willing to bet even if it's a partial match
    pub max_house_bet_size: u64,

    // the maximum bet size from a user
    pub max_user_bet: u64,
}

impl GlobalState {
    pub fn confirm_crank_admin(&self, signer_address: &Signer) -> Result<()> {
        if self.crank_admin != signer_address.key() {
            return Err(QuickBetsErrors::InvalidAdmin.into());
        }
        msg!("Admin confirmed");
        Ok(())
    }

    pub fn add_game_record(&mut self, game_address: Pubkey) {
        let mut game_records = self.game_records.clone();
        game_records.rotate_right(1);
        self.to_close = game_records[0].game_address;
        game_records[0] = GameRecord {
            game_address,
            status: GameStatus::Betting,
        };
        self.game_records = game_records;
    }

    pub fn modify_game_record(&mut self, game_address: Pubkey, status: GameStatus) {
        for game_record in self.game_records.iter_mut() {
            if game_record.game_address == game_address {
                game_record.status = status;
                return;
            }
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct GameRecord {
    pub game_address: Pubkey,
    pub status: GameStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq)]
pub enum GameStatus {
    #[default]
    Betting,
    Anticipation,
    Resolved,
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

    // a max of 25 users can bet on each side.
    pub user_bets: [UserBet; 20],
}

impl Game {
    /// 0 meaning that sol had won,
    /// 1 meaning that eth had won
    /// 2 meaning that it's a draw
    pub fn get_winner(&self) -> u8 {
        let sol_change = (self.final_sol_price as i64 - self.initial_sol_price as i64) as f64
            / self.initial_sol_price as f64;
        let eth_change = (self.final_eth_price as i64 - self.initial_eth_price as i64) as f64
            / self.initial_eth_price as f64;

        if sol_change == eth_change {
            2
        } else if eth_change > sol_change {
            1
        } else {
            0
        }
    }

    pub fn calculate_winning_amount(&self, amount: u64, side: u8) -> u64 {
        let total_pool_size = self.sol_bet_size + self.eth_bet_size;
        let total_sol_bets = self.sol_bet_size;
        let total_eth_bets = self.eth_bet_size;
        let game_winner = self.get_winner();
        if side != game_winner && game_winner != 2 {
            return 0;
        } else if game_winner == 2 {
            return amount;
        }
        if side == game_winner && side == 0 {
            // this means the user bet on sol, and sol won
            let user_pool_share = amount as f64 / total_sol_bets as f64;
            (total_pool_size as f64 * user_pool_share) as u64
        } else if side == game_winner && side == 1 {
            // this means the user bet on eth, and eth won
            let user_pool_share = amount as f64 / total_eth_bets as f64;
            (total_pool_size as f64 * user_pool_share) as u64
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

    pub fn add_user_bet(&mut self, owner: Pubkey, amount: u64, side: u8) -> Result<u64> {
        // look if they have a bet on there already
        for user_bet_slot in self.user_bets.iter_mut() {
            if user_bet_slot.owner == owner {
                if user_bet_slot.side != side {
                    return Err(QuickBetsErrors::AlreadyBet.into());
                }
                user_bet_slot.amount += amount;
                return Ok(user_bet_slot.amount);
            }
        }
        // User got no bets, look for empty slot
        for user_bet_slot in self.user_bets.iter_mut() {
            if user_bet_slot.owner == Pubkey::default() {
                user_bet_slot.owner = owner;
                user_bet_slot.amount = amount;
                user_bet_slot.claimed = false;
                user_bet_slot.side = side;
                return Ok(user_bet_slot.amount);
            }
        }
        Err(QuickBetsErrors::NoSpaceLeft.into())
    }

    pub fn get_user_bet(&self, owner: Pubkey) -> Option<UserBet> {
        for user_bet_slot in self.user_bets.iter() {
            if user_bet_slot.owner == owner {
                return Some(user_bet_slot.clone());
            }
        }
        None
    }

    pub fn mark_bet_claimed(&mut self, owner: Pubkey) -> Result<()> {
        for user_bet_slot in self.user_bets.iter_mut() {
            if user_bet_slot.owner == owner {
                user_bet_slot.claimed = true;
                return Ok(());
            }
        }
        Err(QuickBetsErrors::NoBetFound.into())
    }

    pub fn check_all_bets_claimed(&self) -> bool {
        let winning_side = self.get_winner();

        for user_bet_slot in self.user_bets.iter() {
            if !user_bet_slot.claimed
                && user_bet_slot.owner != Pubkey::default()
                && user_bet_slot.side == winning_side
            {
                return false;
            }
        }
        true
    }

    pub fn get_amount_owed_to_winners(&self, winner_multiplier: f64) -> u64 {
        let winning_side = self.get_winner();
        let mut amount = 0;
        if winning_side == 2 {
            for user_bet_slot in self.user_bets.iter() {
                amount += user_bet_slot.amount;
            }
            return amount;
        }

        for user_bet_slot in self.user_bets.iter() {
            if user_bet_slot.side == winning_side {
                amount += (user_bet_slot.amount as f64 * winner_multiplier) as u64;
            }
        }
        amount
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct UserBet {
    pub amount: u64,
    pub claimed: bool,
    pub side: u8,
    pub owner: Pubkey,
}

#[account]
pub struct User {
    pub last_spin: u64,
    pub reward: u16,
    pub claimed: bool,
    pub volume_24h: u64,
    pub current_win_streak: u8,
    pub current_lose_streak: u8,
    pub last_game_bet_size: u64,
    pub total_points: u16,
}

impl User {
    pub fn check_spin_eligible(&self) -> Result<()> {
        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        if !(current_time > self.last_spin + 60 * 60 * 24 && self.volume_24h >= 5 * 1e6 as u64) {
            return err!(QuickBetsErrors::NotEligible);
        }
        if !self.claimed && self.last_spin != 0 {
            return err!(QuickBetsErrors::RewardNotClaimed);
        }

        Ok(())
    }

    pub fn register_spin(&mut self, reward: u16) {
        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        self.last_spin = current_time;
        self.reward += reward;
        self.claimed = false;
        self.volume_24h = 0;
    }

    pub fn add_volume(&mut self, volume: u64) {
        // if they had 24 hour since their last reward, reset the volume
        let current_time = Clock::get().unwrap().unix_timestamp as u64;
        if current_time > self.last_spin + 60 * 60 * 24 && !self.claimed {
            self.volume_24h += volume;
        }
    }

    pub fn add_bet_record(&mut self, bet_size: u64, win: bool) {
        msg!("adding bet record: {}, {}", bet_size, win);
        self.last_game_bet_size = bet_size;
        if win {
            self.current_win_streak += 1;
            self.current_lose_streak = 0;
            if self.current_win_streak == 3
                || self.current_win_streak == 5
                || self.current_win_streak == 7
                || self.current_win_streak > 7
            {
                if self.last_game_bet_size >= (0.25 * 1e9) as u64 && bet_size >= (0.25 * 1e9) as u64
                {
                    self.total_points += 2;
                } else {
                    self.total_points += 1;
                }
            }
        } else if !win {
            self.current_lose_streak += 1;
            self.current_win_streak = 0;
            if self.current_lose_streak == 3
                || self.current_lose_streak == 5
                || self.current_lose_streak == 7
                || self.current_lose_streak > 7
            {
                if self.last_game_bet_size >= (0.25 * 1e9) as u64 && bet_size >= (0.25 * 1e9) as u64
                {
                    self.total_points += 2;
                } else {
                    self.total_points += 1;
                }
            }
        }
    }
}
