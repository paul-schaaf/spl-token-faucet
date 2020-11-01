use std::str::FromStr;

use solana_sdk::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    info,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};

use crate::{instruction::EscrowInstruction, state::Escrow};

pub struct Processor;

const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = EscrowInstruction::unpack(input)?;

        match instruction {
            EscrowInstruction::InitEscrow { amount } => {
                info!("Instruction: InitEscrow");
                return Self::process_init_escrow(accounts, amount, program_id);
            }
            EscrowInstruction::Exchange { amount } => {
                info!("Instruction: Exchange");
                return Self::process_exchange(accounts, amount, program_id);
            }
            EscrowInstruction::Cancel => (),
        };
        Ok(())
    }

    pub fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let temp_token_account = next_account_info(account_info_iter)?;
        if *temp_token_account.owner != Pubkey::from_str(TOKEN_PROGRAM_ID).unwrap() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // TODO: check that temp token account is owned (in token program jargon) by the initializer

        let received_token_account = next_account_info(account_info_iter)?;
        if *received_token_account.owner != Pubkey::from_str(TOKEN_PROGRAM_ID).unwrap() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let escrow_account = next_account_info(account_info_iter)?;

        if escrow_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;
        if escrow_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        escrow_info.is_initialized = true;
        escrow_info.initializer_pubkey = *initializer.key;
        escrow_info.sending_token_account_pubkey = *temp_token_account.key;
        escrow_info.receiving_token_account_pubkey = *received_token_account.key;
        escrow_info.expected_amount = amount;

        Escrow::pack(escrow_info, &mut escrow_account.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_exchange(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let taker = next_account_info(account_info_iter)?;

        if !taker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let temp_token_account = next_account_info(account_info_iter)?;
        if *temp_token_account.owner != Pubkey::from_str(TOKEN_PROGRAM_ID).unwrap() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // TODO: check that temp token account is owned (in token program jargon) by the initializer

        let received_token_account = next_account_info(account_info_iter)?;
        if *received_token_account.owner != Pubkey::from_str(TOKEN_PROGRAM_ID).unwrap() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let escrow_account = next_account_info(account_info_iter)?;

        if escrow_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let escrow_info = Escrow::unpack(&escrow_account.data.borrow())?;

        Ok(())
    }
}
