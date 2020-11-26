// Mark this test as BPF-only due to current `ProgramTest` limitations when CPIing into the system program
#![cfg(feature = "test-bpf")]

use solana_program::{
    instruction::*, program_option::COption, program_pack::Pack, pubkey::Pubkey, sysvar,
};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::Signer,
    transaction::{Transaction, TransactionError},
};
use spl_token_faucet::*;

//PUBKEY VALID TOKEN MINT
const VALID_MINT_PUBKEY: Pubkey = Pubkey::new_from_array([
    158, 191, 152, 115, 118, 236, 6, 196, 91, 157, 75, 167, 234, 145, 45, 94, 89, 179, 19, 193, 48,
    42, 113, 129, 91, 230, 9, 89, 98, 201, 169, 18,
]);

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

async fn create_faucet(
    pc: ProgramTest,
) -> (
    BanksClient,
    solana_sdk::signature::Keypair,
    solana_program::hash::Hash,
    Pubkey,
) {
    let mut pc = pc;
    let token_mint_address = VALID_MINT_PUBKEY;
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

    let mut init_faucet_tx = Transaction::new_with_payer(
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

    init_faucet_tx.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(init_faucet_tx)
        .await
        .unwrap();
    (banks_client, payer, recent_blockhash, faucet_pubkey)
}

#[tokio::test]
async fn test_happy_flow_init_faucet_no_admin() {
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
    let faucet_acc = banks_client
        .get_account(faucet_pubkey)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        COption::None,
        state::Faucet::unpack_from_slice(&faucet_acc.data)
            .unwrap()
            .admin
    );
}

#[tokio::test]
async fn test_happy_flow_init_faucet_with_admin() {
    let token_mint_address = Pubkey::new_unique();
    let mut pc = program_test();
    let faucet_pubkey = Pubkey::new_unique();
    let admin_pubkey = Pubkey::new_unique();
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
                AccountMeta::new_readonly(admin_pubkey, false),
            ],
            data: vec![0, 1, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    let faucet_acc = banks_client
        .get_account(faucet_pubkey)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        admin_pubkey,
        state::Faucet::unpack_from_slice(&faucet_acc.data)
            .unwrap()
            .admin
            .unwrap()
    );
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
    let token_mint_address = VALID_MINT_PUBKEY;
    let token_account_address = Pubkey::new_unique();
    let mut pc = program_test();
    pc.add_account_with_file_data(
        token_account_address,
        1000000000,
        spl_token::id(),
        "valid-token-account-data.bin",
    );
    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) = create_faucet(pc).await;

    let (pda, _nonce) = Pubkey::find_program_address(&[b"faucet"], &id());

    let mut mint_tokens_tx = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(pda, false),
                AccountMeta::new(token_mint_address, false),
                AccountMeta::new(token_account_address, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(faucet_pubkey, false),
            ],
            data: vec![1, 5, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    mint_tokens_tx.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(mint_tokens_tx)
        .await
        .unwrap();

    let acc = banks_client
        .get_account(token_account_address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        5,
        spl_token::state::Account::unpack_from_slice(&acc.data)
            .unwrap()
            .amount
    );
}
