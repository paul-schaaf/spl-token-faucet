use solana_program::program_error::ProgramError;
use solana_program::program_option::COption;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

use crate::error::FaucetError;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Faucet {
    pub is_initialized: bool,
    pub admin: COption<Pubkey>,
    pub mint: Pubkey,
    pub amount: u64,
}

impl Sealed for Faucet {}

impl IsInitialized for Faucet {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Faucet {
    const LEN: usize = 77;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < Faucet::LEN {
            return Err(FaucetError::IncorrectInitializationData.into());
        }
        let src = array_ref![src, 0, Faucet::LEN];
        let (is_initialized, admin, amount, mint) = array_refs![src, 1, 36, 8, 32];

        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(FaucetError::IncorrectInitializationData.into()),
        };
        Ok(Self {
            is_initialized,
            admin: unpack_coption_key(admin)?,
            amount: u64::from_le_bytes(*amount),
            mint: Pubkey::new_from_array(*mint),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Faucet::LEN];
        let (is_initialized_dst, admin_dst, amount_dst, mint_dst) =
            mut_array_refs!(dst, 1, 36, 8, 32);
        let &Faucet {
            is_initialized,
            ref admin,
            ref mint,
            amount,
        } = self;

        pack_coption_key(admin, admin_dst);
        is_initialized_dst[0] = is_initialized as u8;
        *amount_dst = amount.to_le_bytes();
        *mint_dst = mint.to_bytes();
    }
}

// Helpers
fn pack_coption_key(src: &COption<Pubkey>, dst: &mut [u8; 36]) {
    let (tag, body) = mut_array_refs![dst, 4, 32];
    match src {
        COption::Some(key) => {
            *tag = [1, 0, 0, 0];
            body.copy_from_slice(key.as_ref());
        }
        COption::None => {
            *tag = [0; 4];
        }
    }
}
fn unpack_coption_key(src: &[u8; 36]) -> Result<COption<Pubkey>, ProgramError> {
    let (tag, body) = array_refs![src, 4, 32];
    match *tag {
        [0, 0, 0, 0] => Ok(COption::None),
        [1, 0, 0, 0] => Ok(COption::Some(Pubkey::new_from_array(*body))),
        _ => Err(ProgramError::InvalidAccountData),
    }
}
