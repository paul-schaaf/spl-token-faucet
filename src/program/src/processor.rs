use crate::{error::EscrowError, instruction::EscrowInstruction, state::Escrow};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    info,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};
use spl_token::{
    error::TokenError,
    instruction::{set_authority as set_token_authority, AuthorityType as TokenAccountAuthority},
    state::Account as TokenAccount,
};
use std::str::FromStr;

const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

pub struct Processor;
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

        let temp_token_account_info = TokenAccount::unpack(&temp_token_account.data.borrow())?;
        if temp_token_account_info.owner != *initializer.key {
            return Err(TokenError::OwnerMismatch.into());
        }

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

        let (pda, _nonce) = Pubkey::find_program_address(&[b"escrow"], program_id);

        let token_program = next_account_info(account_info_iter)?;
        let owner_change_ix = set_token_authority(
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            TokenAccountAuthority::AccountOwner,
            initializer.key,
            &[&initializer.key],
        )?;

        info!("Calling the token program to transfer token account ownership");
        invoke(
            &owner_change_ix,
            &[
                temp_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    pub fn process_exchange(
        accounts: &[AccountInfo],
        amount_expected_by_taker: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let taker = next_account_info(account_info_iter)?;

        if !taker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let takers_temp_token_account = next_account_info(account_info_iter)?;
        if *takers_temp_token_account.owner != Pubkey::from_str(TOKEN_PROGRAM_ID).unwrap() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let takers_temp_token_account_info =
            TokenAccount::unpack(&takers_temp_token_account.data.borrow())?;
        if takers_temp_token_account_info.owner != *taker.key {
            return Err(TokenError::OwnerMismatch.into());
        }

        let takers_received_token_account = next_account_info(account_info_iter)?;
        if *takers_received_token_account.owner != Pubkey::from_str(TOKEN_PROGRAM_ID).unwrap() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let pdas_temp_token_account = next_account_info(account_info_iter)?;
        let pdas_temp_token_account_info =
            TokenAccount::unpack(&pdas_temp_token_account.data.borrow())?;
        let (pda, nonce) = Pubkey::find_program_address(&[b"escrow"], program_id);

        if pdas_temp_token_account_info.owner != pda {
            return Err(TokenError::OwnerMismatch.into());
        }

        if amount_expected_by_taker != pdas_temp_token_account_info.amount {
            return Err(EscrowError::ExpectedFundsMismatch.into());
        }

        let creators_main_account = next_account_info(account_info_iter)?;
        let creators_received_token_account = next_account_info(account_info_iter)?;

        let escrow_account = next_account_info(account_info_iter)?;

        if escrow_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let escrow_info = Escrow::unpack(&escrow_account.data.borrow())?;
        if escrow_info.expected_amount != takers_temp_token_account_info.amount {
            return Err(EscrowError::ExpectedFundsMismatch.into());
        }

        if escrow_info.sending_token_account_pubkey != *pdas_temp_token_account.key {
            return Err(EscrowError::UnknownAccount.into());
        }

        if escrow_info.initializer_pubkey != *creators_main_account.key {
            return Err(EscrowError::UnknownAccount.into());
        }

        if escrow_info.receiving_token_account_pubkey != *creators_received_token_account.key {
            return Err(EscrowError::UnknownAccount.into());
        }

        let token_program = next_account_info(account_info_iter)?;

        let transfer_to_creator_ix = spl_token::instruction::transfer(
            token_program.key,
            takers_temp_token_account.key,
            creators_received_token_account.key,
            taker.key,
            &[&taker.key],
            takers_temp_token_account_info.amount,
        )?;
        info!("Calling the token program to transfer tokens to the creator");
        invoke(
            &transfer_to_creator_ix,
            &[
                takers_temp_token_account.clone(),
                creators_received_token_account.clone(),
                taker.clone(),
                token_program.clone(),
            ],
        )?;

        let close_takers_temp_tacc_ix = spl_token::instruction::close_account(
            token_program.key,
            takers_temp_token_account.key,
            taker.key,
            taker.key,
            &[&taker.key],
        )?;
        info!("Calling the token program to close taker's temp account");
        invoke(
            &close_takers_temp_tacc_ix,
            &[
                takers_temp_token_account.clone(),
                taker.clone(),
                taker.clone(),
                token_program.clone(),
            ],
        )?;

        let pda_account = next_account_info(account_info_iter)?;

        let transfer_to_taker_ix = spl_token::instruction::transfer(
            token_program.key,
            pdas_temp_token_account.key,
            takers_received_token_account.key,
            &pda,
            &[&pda],
            pdas_temp_token_account_info.amount,
        )?;
        info!("Calling the token program to transfer tokens to the taker");
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                pdas_temp_token_account.clone(),
                takers_received_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[nonce]]],
        )?;

        Ok(())
    }
}
