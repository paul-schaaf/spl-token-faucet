//! Program entrypoint

use crate::{error::EscrowError, processor::Processor};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, info,
    program_error::PrintProgramError, pubkey::Pubkey,
};

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    info!("Solana escrow entrypoint");

    if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
        error.print::<EscrowError>();
        return Err(error);
    }

    Ok(())
}
