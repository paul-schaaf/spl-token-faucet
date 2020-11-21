use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    info,
    program_error::ProgramError,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::state::Mint;

use crate::error::FaucetError;
use crate::instruction::FaucetInstruction;
use crate::state::Faucet;

pub struct Processor;

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = FaucetInstruction::unpack(input)?;
        match instruction {
            FaucetInstruction::InitFaucet { amount } => {
                info!("Instruction: InitFaucet");
                Self::process_init_faucet(accounts, amount, program_id)?
            }
            FaucetInstruction::MintTokens { amount } => {
                info!("Instruction: MintTokens");
            }
            FaucetInstruction::CloseFaucet => {
                info!("Instruction: CloseFaucet");
            }
        }
        Ok(())
    }

    pub fn process_init_faucet(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let (pda, _nonce) = Pubkey::find_program_address(&[b"faucet"], program_id);

        let mint_account = next_account_info(account_info_iter)?;
        let mint_state = Mint::unpack(&mint_account.data.borrow())?;

        if pda
            != mint_state
                .mint_authority
                .ok_or(ProgramError::InvalidAccountData)?
        {
            return Err(ProgramError::InvalidAccountData);
        }

        let faucet_account = next_account_info(account_info_iter)?;

        let mut faucet = Faucet::unpack_unchecked(&faucet_account.data.borrow())?;
        if faucet.is_initialized {
            return Err(FaucetError::AccountAlreadyInUse.into());
        }
        if faucet_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(faucet_account.lamports(), faucet_account.data_len()) {
            return Err(FaucetError::AccountNotRentExempt.into());
        }

        let admin_acc = next_account_info(account_info_iter);

        let admin_pubkey = match admin_acc {
            Ok(acc) => COption::Some(*acc.key),
            Err(_) => COption::None,
        };

        faucet.is_initialized = true;
        faucet.admin = admin_pubkey;
        faucet.amount = amount;

        Faucet::pack(faucet, &mut faucet_account.data.borrow_mut())?;

        Ok(())
    }
}
