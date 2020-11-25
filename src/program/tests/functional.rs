// Mark this test as BPF-only due to current `ProgramTest` limitations when CPIing into the system program
#![cfg(feature = "test-bpf")]

use solana_program::{
    instruction::*, program_pack::Pack, pubkey::Pubkey, rent::Rent,
    system_instruction, sysvar,
};
use solana_program_test::*;
use solana_sdk::{
    account::{create_account, Account},
    signature::Signer,
    transaction::{Transaction, TransactionError},
};
use spl_token_faucet::*;

fn program_test() -> ProgramTest {
    let mut pc = ProgramTest::new(
        "spl_token_faucet",
        id(),
        processor!(processor::Processor::process),
    );

    // Add SPL Token program
    pc.add_program(
        "spl_token",
        spl_token::id(),
        processor!(spl_token::processor::Processor::process),
    );

    // Dial down the BPF compute budget to detect if the program gets bloated in the future
    pc.set_bpf_compute_max_units(50_000);

    pc
}

#[tokio::test]
async fn test_happy_flow_init_faucet() {
    let token_mint_address = Pubkey::new_unique();
    let mut pc = program_test();
    let faucet_pubkey = Pubkey::new_unique();
    pc.add_account(
        faucet_pubkey,
        Account::new(1426800, spl_token_faucet::state::Faucet::LEN, &id()),
    );
    pc.add_account_with_file_data(
        token_mint_address,
        1461600,
        spl_token::id(),
        "valid-token-mint-data.bin",
    );
    let (mut banks_client, payer, recent_blockhash) = pc.start().await;
    let rent = banks_client.get_rent().await.unwrap();
    let expected_token_account_balance = rent.minimum_balance(spl_token_faucet::state::Faucet::LEN);

    assert_eq!(
        banks_client.get_balance(faucet_pubkey).await.unwrap(),
        expected_token_account_balance,
    );

    let mut transaction = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(token_mint_address, false),
                AccountMeta::new(faucet_pubkey, false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
            ],
            data: vec![0, 1, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}

#[tokio::test]
async fn test_incorrect_mint_authority() {
    let token_mint_address = Pubkey::new_unique();
    let mut pc = program_test();
    let faucet_pubkey = Pubkey::new_unique();
    pc.add_account(
        faucet_pubkey,
        Account::new(1426800, spl_token_faucet::state::Faucet::LEN, &id()),
    );
    pc.add_account_with_file_data(
        token_mint_address,
        1461600,
        spl_token::id(),
        "incorrect-token-mint-data.bin",
    );
    let (mut banks_client, payer, recent_blockhash) = pc.start().await;
    let rent = banks_client.get_rent().await.unwrap();
    let expected_token_account_balance = rent.minimum_balance(spl_token_faucet::state::Faucet::LEN);

    assert_eq!(
        banks_client.get_balance(faucet_pubkey).await.unwrap(),
        expected_token_account_balance,
    );

    let mut transaction = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(token_mint_address, false),
                AccountMeta::new(faucet_pubkey, false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
            ],
            data: vec![0, 1, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        TransactionError::InstructionError(0, InstructionError::Custom(0x09)),
        result
    );
}

#[tokio::test]
async fn test_faucet_already_initialized() {
    let token_mint_address = Pubkey::new_unique();
    let mut pc = program_test();
    let faucet_pubkey = Pubkey::new_unique();
    pc.add_account(
        faucet_pubkey,
        Account::new(1426800, spl_token_faucet::state::Faucet::LEN, &id()),
    );
    pc.add_account_with_file_data(
        token_mint_address,
        1461600,
        spl_token::id(),
        "valid-token-mint-data.bin",
    );
    let (mut banks_client, payer, recent_blockhash) = pc.start().await;
    let rent = banks_client.get_rent().await.unwrap();
    let expected_token_account_balance = rent.minimum_balance(spl_token_faucet::state::Faucet::LEN);

    assert_eq!(
        banks_client.get_balance(faucet_pubkey).await.unwrap(),
        expected_token_account_balance,
    );

    let mut transaction = Transaction::new_with_payer(
        &[
            Instruction {
                program_id: id(),
                accounts: vec![
                    AccountMeta::new_readonly(token_mint_address, false),
                    AccountMeta::new(faucet_pubkey, false),
                    AccountMeta::new_readonly(sysvar::rent::id(), false),
                ],
                data: vec![0, 1, 0, 0, 0, 0, 0, 0, 0],
            },
            Instruction {
                program_id: id(),
                accounts: vec![
                    AccountMeta::new_readonly(token_mint_address, false),
                    AccountMeta::new(faucet_pubkey, false),
                    AccountMeta::new_readonly(sysvar::rent::id(), false),
                ],
                data: vec![0, 1, 0, 0, 0, 0, 0, 0, 0],
            },
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer], recent_blockhash);
    let result = banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        TransactionError::InstructionError(1, InstructionError::Custom(0x03)),
        result
    );
}

#[tokio::test]
async fn test_happy_flow_mint_tokens() {
    let token_mint_address = Pubkey::new_unique();
    let token_account_address = Pubkey::new_unique();
    let mut pc = program_test();
    let faucet_pubkey = Pubkey::new_unique();
    pc.add_account(
        faucet_pubkey,
        Account::new(1426800, spl_token_faucet::state::Faucet::LEN, &id()),
    );
    pc.add_account_with_file_data(
        token_mint_address,
        1461600,
        spl_token::id(),
        "valid-token-mint-data.bin",
    );
    pc.add_account_with_file_data(
        token_account_address,
        1000000000,
        spl_token::id(),
        "valid-token-account-data.bin",
    );
    let (mut banks_client, payer, recent_blockhash) = pc.start().await;
    let rent = banks_client.get_rent().await.unwrap();
    let expected_token_account_balance = rent.minimum_balance(spl_token_faucet::state::Faucet::LEN);

    assert_eq!(
        banks_client.get_balance(faucet_pubkey).await.unwrap(),
        expected_token_account_balance,
    );

    let mut initFaucetTx = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(token_mint_address, false),
                AccountMeta::new(faucet_pubkey, false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
            ],
            data: vec![0, 10, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    initFaucetTx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(initFaucetTx).await.unwrap();

    let (pda, _nonce) = Pubkey::find_program_address(&[b"faucet"], &id());

    let mut mintTokensTx = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(pda, false),
                AccountMeta::new(token_mint_address, false),
                AccountMeta::new(token_account_address, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(faucet_pubkey, false)
            ],
            data: vec![1, 5, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    mintTokensTx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(mintTokensTx).await.unwrap();
}
