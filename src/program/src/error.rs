//! Error types

use num_derive::FromPrimitive;
use thiserror::Error;

use solana_program::{decode_error::DecodeError, program_error::ProgramError};

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
