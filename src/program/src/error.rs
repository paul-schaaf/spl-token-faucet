//! Error types

use num_derive::FromPrimitive;
use thiserror::Error;

use solana_program::{decode_error::DecodeError, program_error::ProgramError};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum FaucetError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Incorrect initialization data
    #[error("Incorrect Initialization Data")]
    IncorrectInitializationData,
    /// Not Rent Excempt
    #[error("Account Not Rent Exempt")]
    AccountNotRentExempt,
    /// Account Already In Use
    #[error("Account Already In Use")]
    AccountAlreadyInUse,
    /// Requesting Too Many Tokens
    #[error("Requesting Too Many Tokens")]
    RequestingTooManyTokens,
    /// Non Admin Closure Attempt
    #[error("Non Admin Closure Attempt")]
    NonAdminClosureAttempt,
    /// Non Closable Faucet Closure Attempt
    #[error("Non Closable Faucet Closure Attempt")]
    NonClosableFaucetClosureAttempt,
    /// Overflow
    #[error("Overflow")]
    Overflow,
    /// Invalid Mint
    #[error("Invalid Mint")]
    InvalidMint,
    /// Incorrect Mint Authority
    #[error("Incorrect Mint Authority")]
    IncorrectMintAuthority,
}

impl From<FaucetError> for ProgramError {
    fn from(e: FaucetError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for FaucetError {
    fn type_of() -> &'static str {
        "FaucetError"
    }
}
