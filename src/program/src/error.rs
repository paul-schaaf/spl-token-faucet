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
    /// Missing initialization data
    #[error("Missing Initialization Data")]
    MissingInitializationData,
}

impl solana_program::program_error::PrintProgramError for FaucetError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            Self::InvalidInstruction => info!("Error: Invalid Instruction"),
            Self::MissingInitializationData => info!("Error: Missing initialization data"),
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
