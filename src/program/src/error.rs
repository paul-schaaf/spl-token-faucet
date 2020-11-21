//! Error types

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use thiserror::Error;

use solana_program::{
    decode_error::DecodeError,
    info,
    program_error::{PrintProgramError, ProgramError},
};

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
}

impl solana_program::program_error::PrintProgramError for FaucetError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            Self::InvalidInstruction => info!("Error: Invalid Instruction"),
            Self::IncorrectInitializationData => info!("Error: Incorrect initialization data"),
            Self::AccountNotRentExempt => info!("Error: Account Not Rent Exempt"),
            Self::AccountAlreadyInUse => info!("Error: Account Already In Use"),
        }
    }
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
