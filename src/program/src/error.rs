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
pub enum EscrowError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    /// Missing initialization data
    #[error("Missing Initialization Data")]
    MissingInitializationData,
    /// Expected funds mismatch
    #[error("Expected Funds Mismatch")]
    ExpectedFundsMismatch,
    /// Unknown account
    #[error("Unknown Account")]
    UnknownAccount,
    /// Amount overflow
    #[error("Amount Overflow")]
    AmountOverflow
}

impl solana_program::program_error::PrintProgramError for EscrowError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            Self::ExpectedFundsMismatch => info!("Error: Expected funds mismatch"),
            Self::InvalidInstruction => info!("Error: Invalid Instruction"),
            Self::MissingInitializationData => info!("Error: Missing initialization data"),
            Self::UnknownAccount => info!("Error: Unknown account"),
            Self::AmountOverflow => info!("Error: Amount overflow")
        }
    }
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for EscrowError {
    fn type_of() -> &'static str {
        "EscrowError"
    }
}
