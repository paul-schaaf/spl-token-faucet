//! Instruction types

use crate::error::{EscrowError, EscrowError::InvalidInstruction};
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

pub enum EscrowInstruction {
    /// Starts the trade by creating and populating an escrow account and transferring ownership of the given temp token account to the PDA
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person initializing the escrow
    /// 1. `[writable]` Temporary token account that should be created prior to this instruction and owned by the initializer
    /// 2. `[]` The initializer's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The escrow account, it will hold all necessary info about the trade.
    /// This acc SHOULD be created atomically in the same tx as calling `InitEscrow`. Otherwise another party can acquire ownership of the uninitialized account.
    /// 4. `[]` The token program TODO: should this get passed as an input or as an account?
    /// 5. `[optional]` The account of the only person the initializer accepts for the deal, if left empty, anyone can take the other side of the escrow
    InitEscrow {
        /// the amount the initializer expects to be paid in the other token, as a u64 because that's the max possible supply of a token
        amount: u64,
    },
    /// Accepts a trade
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person taking the trade
    /// 1. `[writable]` Taker's temporary token account that will be closed
    /// 2. `[writable]` The taker's token account for the token they will receive should the trade go through
    /// 3. `[writable]` The PDA's temp token account to get tokens from and eventually close
    /// 4. `[writable]` The creator's main account to send their rent fees to
    /// 5. `[writable]` The creator's token account that will receive tokens
    /// 6. `[writable]` The escrow account holding the escrow info
    /// 7. `[]` The token program TODO: should this get passed as an input or as an account?
    /// 8. `[]` The PDA account
    Exchange {
        /// the amount the taker expects to be paid in the other token, as a u64 because that's the max possible supply of a token
        amount: u64,
    },
    /// TODO
    Cancel,
}

impl EscrowInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitEscrow {
                amount: unpack_amount(rest)?,
            },
            1 => Self::Exchange {
                amount: unpack_amount(rest)?,
            },
            _ => return Err(EscrowError::InvalidInstruction.into()),
        })
    }
}

fn unpack_amount(input: &[u8]) -> Result<u64, EscrowError> {
    let amount = input
        .get(..8)
        .and_then(|slice| slice.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or(InvalidInstruction)?;
    Ok(amount)
}
