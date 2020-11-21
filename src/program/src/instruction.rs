use std::convert::TryInto;
use std::mem::size_of;

use crate::error::FaucetError;
use solana_program::program_error::ProgramError;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum FaucetInstruction {
    /// Initializes a faucet and transfers mint authority to the PDA
    ///
    /// 0. `[signer]` Current mint authority
    /// 1. `[]` New mint authority - Program Derived Address
    /// 2. `[writable]` Token Mint Account
    /// 3. `[writable]` Faucet Account
    /// 4. `[]` The SPL Token program
    /// 5. `[optional]` Admin Account
    InitFaucet {
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
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(FaucetError::InvalidInstruction)?;
                Self::InitFaucet { amount }
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

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::InitFaucet { amount } => {
                buf.push(0);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::MintTokens { amount } => {
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::CloseFaucet => {
                buf.push(2);
            }
        }

        buf
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_instruction_unpacking() {
        // 1 tag, 1 admin, 8 amount
        let check = FaucetInstruction::unpack(&[0, 7, 3, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(FaucetInstruction::InitFaucet { amount: 775 }, check);
        // 1 tag,  8 amount
        let check = FaucetInstruction::unpack(&[1, 7, 3, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(FaucetInstruction::MintTokens { amount: 775 }, check);

        // 1 tag
        let check = FaucetInstruction::unpack(&[2]).unwrap();
        assert_eq!(FaucetInstruction::CloseFaucet, check);
    }

    #[test]
    fn test_instruction_packing() {
        let check = FaucetInstruction::InitFaucet { amount: 900 };

        let packed = check.pack();
        let mut expect = vec![0];
        expect.extend_from_slice(&u64::to_le_bytes(900));
        assert_eq!(packed, expect);

        let check = FaucetInstruction::MintTokens { amount: 900 };

        let packed = check.pack();
        let mut expect = vec![1];
        expect.extend_from_slice(&u64::to_le_bytes(900));
        assert_eq!(packed, expect);

        let check = FaucetInstruction::CloseFaucet;

        let packed = check.pack();
        assert_eq!(packed, vec![2]);
    }
}
