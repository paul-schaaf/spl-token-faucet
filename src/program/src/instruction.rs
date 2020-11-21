use std::convert::TryInto;

use crate::error::FaucetError;
use solana_program::program_error::ProgramError;
use solana_program::program_option::COption;
use solana_program::pubkey::Pubkey;

pub enum FaucetInstruction {
    /// Initializes a faucet and transfers mint authority to the PDA
    ///
    /// 0. `[signer]` Current mint authority
    /// 1. `[]` New mint authority - Program Derived Address
    /// 2. `[writable]` Token Mint Account
    /// 3. `[writable]` Faucet Account
    /// 4. `[]` The SPL Token program
    InitFaucet {
        /// an admin may mint any amount of tokens per ix
        admin: COption<Pubkey>,
        /// all other accounts may only mint this amount per ix
        amount: u64,
    },
    /// Mints Tokens
    ///
    /// 0. `[]` The mint authority - Program Derived Address
    /// 1. `[writable]` Token Mint Account
    /// 2. `[writable]` Destination Account
    /// 3. `[]` The SPL Token Program
    /// 4. `[optional/signer]` Admin Account
    MintTokens { amount: u64 },
    /// Closes the faucet, can only be done if the faucet has an admin key
    ///
    /// 0. `[signer]` Admin account
    /// 1. `[writable]` Destination account for rent
    /// 2. `[writable]` Faucet account
    CloseFaucet,
}

impl FaucetInstruction {
    /// Unpacks a byte buffer into a [FaucetInstruction](enum.FaucetInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(FaucetError::InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (admin, rest) = unpack_pubkey_option(rest)?;
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(FaucetError::InvalidInstruction)?;
                Self::InitFaucet { admin, amount }
            }
            1 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(FaucetError::InvalidInstruction)?;
                Self::MintTokens { amount }
            }
            2 => Self::CloseFaucet,
            _ => return Err(FaucetError::InvalidInstruction.into()),
        })
    }
}

fn unpack_pubkey_option(input: &[u8]) -> Result<(COption<Pubkey>, &[u8]), ProgramError> {
    match input.split_first() {
        Option::Some((&0, rest)) => Ok((COption::None, rest)),
        Option::Some((&1, rest)) if rest.len() >= 32 => {
            let (key, rest) = rest.split_at(32);
            let pk = Pubkey::new(key);
            Ok((COption::Some(pk), rest))
        }
        _ => Err(FaucetError::InvalidInstruction.into()),
    }
}
