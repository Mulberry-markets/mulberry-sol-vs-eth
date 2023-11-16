use anchor_lang::prelude::*;

#[error_code]
pub enum QuickBetsErrors {
    #[msg("Invalid oracle.")]
    InvalidOracle,
    #[msg("Invalid admin, You are not the admin of this market.")]
    InvalidAdmin,
    #[msg("Invalid side.")]
    InvalidSide,
    #[msg("The house wallet doesn't have enough funds.")]
    HouseBankrupt,
    #[msg("You already have a bet on the other side.")]
    AlreadyBet,
    #[msg("Anticipation period ending too soon")]
    AnticipationTimeTooSoon,
    #[msg("betting period ending too soon")]
    BettingTimeTooSoon,
    #[msg("Bet already settled")]
    BetAlreadySettled,
    #[msg("You are not on the winning side")]
    NotOnWinningSide,
    #[msg("Bet not settled")]
    BetNotSettled,
    #[msg("Bet already claimed")]
    AlreadyClaimed,
    #[msg("Game is inactive")]
    BettingInactive,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("No space left")]
    NoSpaceLeft,
    #[msg("No bet found")]
    NoBetFound,
    #[msg("Not all bets are claimed")]
    BetsNotClaimed,
    #[msg("Vault not empty")]
    VaultNotEmpty,
    #[msg("too early to close the game")]
    GameNotClosed,
    #[msg("Game vault mismatch")]
    GameVaultMismatch,
    #[msg("Max user bet exceeded")]
    MaxUserBetExceeded,
    #[msg("Your size isn't size")]
    InvalidSize,
    #[msg("Oracle didn't update in 10 seconds or more")]
    StaleOracle,
    #[msg("Game in progress")]
    GameInProgress,
    #[msg("Not eligible for a spin yet")]
    NotEligible,
    #[msg("Reward not claimed yet")]
    RewardNotClaimed,
    #[msg("Insufficient balance")]
    InsufficientBalance
}