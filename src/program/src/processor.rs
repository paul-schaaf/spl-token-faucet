use solana_sdk::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::EscrowInstruction;

pub struct Processor;

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = EscrowInstruction::unpack(input)?;
        Ok(())
    }
}
