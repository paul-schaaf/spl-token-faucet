use solana_sdk::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

use crate::error::EscrowError;

pub struct Escrow {
    is_initialized: bool,
    initializer_pubkey: Pubkey,
    sending_token_account_pubkey: Pubkey,
    receiving_token_account_pubkey: Pubkey,
    expected_amount: u64,
}

impl Sealed for Escrow {}

impl IsInitialized for Escrow {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Escrow {
    const LEN: usize = 105;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < Escrow::LEN {
            return Err(EscrowError::MissingInitializationData.into());
        }
        let src = array_ref![src, 0, Escrow::LEN];
        let (
            is_initialized,
            initializer_pubkey,
            sending_token_account_pubkey,
            receiving_token_account_pubkey,
            expected_amount,
        ) = array_refs![src, 1, 32, 32, 32, 8];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Escrow {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            sending_token_account_pubkey: Pubkey::new_from_array(*sending_token_account_pubkey),
            receiving_token_account_pubkey: Pubkey::new_from_array(*receiving_token_account_pubkey),
            expected_amount: u64::from_le_bytes(*expected_amount),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Escrow::LEN];
        let (
            is_initialized_dst,
            initializer_pubkey_dst,
            sending_token_account_pubkey_dst,
            receiving_token_account_pubkey_dst,
            expected_amount_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 8];

        let Escrow {
            is_initialized,
            initializer_pubkey,
            sending_token_account_pubkey,
            receiving_token_account_pubkey,
            expected_amount,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        sending_token_account_pubkey_dst.copy_from_slice(sending_token_account_pubkey.as_ref());
        receiving_token_account_pubkey_dst.copy_from_slice(receiving_token_account_pubkey.as_ref());
        *expected_amount_dst = expected_amount.to_le_bytes();
    }
}
