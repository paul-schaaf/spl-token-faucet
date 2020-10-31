//! Program entrypoint

#![cfg(feature = "program")]
#![cfg(not(feature = "no-entrypoint"))]

use solana_sdk::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey, info
};
use crate::{error::EscrowError, processor::Processor};

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    info!("Solana escrow entrypoint");

    if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<EscrowError>();
        return Err(error);
    }

    Ok(())
}
