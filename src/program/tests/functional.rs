// Mark this test as BPF-only due to current `ProgramTest` limitations when CPIing into the system program
#![cfg(feature = "test-bpf")]

use solana_program::{
    hash::Hash, instruction::*, program_option::COption, program_pack::Pack, pubkey::Pubkey, sysvar,
};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use spl_token_faucet::*;

// PUBKEY VALID TOKEN MINT
const VALID_MINT_PUBKEY: Pubkey = Pubkey::new_from_array([
    158, 191, 152, 115, 118, 236, 6, 196, 91, 157, 75, 167, 234, 145, 45, 94, 89, 179, 19, 193, 48,
    42, 113, 129, 91, 230, 9, 89, 98, 201, 169, 18,
]);

// SECOND MINT
const SECOND_MINT_PUBKEY: Pubkey = Pubkey::new_from_array([
    29, 238, 175, 20, 22, 250, 227, 227, 197, 169, 134, 117, 26, 101, 94, 231, 186, 99, 233, 162,
    186, 46, 252, 212, 47, 23, 25, 152, 192, 22, 147, 248,
]);

// TOKEN FAUCET ADDRESS
// 6S1A9514Msgf4wVc1wNogNpEn2b1HKoYFHNR9VKQzYdR

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
    admin: Option<Pubkey>,
) -> (BanksClient, Keypair, Hash, Pubkey) {
    let mut pc = pc;
    let faucet_pubkey = Pubkey::new_unique();
    pc.add_account(
        faucet_pubkey,
        Account::new(1426800, spl_token_faucet::state::Faucet::LEN, &id()),
    );
    pc.add_account_with_file_data(
        VALID_MINT_PUBKEY,
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

    let mut accounts = vec![
        AccountMeta::new_readonly(VALID_MINT_PUBKEY, false),
        AccountMeta::new(faucet_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    if let Some(admin_pubkey) = admin {
        accounts.push(AccountMeta::new_readonly(admin_pubkey, false));
    }

    let mut init_faucet_tx = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts,
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
    // GIVEN
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
    // WHEN
    let result = banks_client.process_transaction(transaction).await;

    // THEN
    result.unwrap();

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
    // GIVEN
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

    // WHEN
    let result = banks_client.process_transaction(transaction).await;

    // THEN
    result.unwrap();

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
async fn test_mint_authority_not_owned_by_pda() {
    //GIVEN
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

    // WHEN THEN
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
    // GIVEN
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

    // WHEN THEN
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
    // GIVEN
    let token_mint_address = VALID_MINT_PUBKEY;
    let token_account_address = Pubkey::new_unique();
    let mut pc = program_test();
    pc.add_account_with_file_data(
        token_account_address,
        1000000000,
        spl_token::id(),
        "valid-token-account-data.bin",
    );
    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) = create_faucet(pc, None).await;

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

    // THEN
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

#[tokio::test]
async fn test_mint_too_many_tokens() {
    // GIVEN
    let token_mint_address = VALID_MINT_PUBKEY;
    let token_account_address = Pubkey::new_unique();
    let mut pc = program_test();
    pc.add_account_with_file_data(
        token_account_address,
        1000000000,
        spl_token::id(),
        "valid-token-account-data.bin",
    );
    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) = create_faucet(pc, None).await;

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
            data: vec![1, 11, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    mint_tokens_tx.sign(&[&payer], recent_blockhash);

    // THEN
    let error = banks_client
        .process_transaction(mint_tokens_tx)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        TransactionError::InstructionError(0, InstructionError::Custom(0x04)),
        error
    );

    let acc = banks_client
        .get_account(token_account_address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        0,
        spl_token::state::Account::unpack_from_slice(&acc.data)
            .unwrap()
            .amount
    );
}

#[tokio::test]
async fn test_mint_happy_flow_admin_may_mint_too_many_tokens() {
    // GIVEN
    let token_mint_address = VALID_MINT_PUBKEY;
    let token_account_address = Pubkey::new_unique();
    let mut pc = program_test();
    pc.add_account_with_file_data(
        token_account_address,
        1000000000,
        spl_token::id(),
        "valid-token-account-data.bin",
    );

    let admin_keypair = Keypair::new();
    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) =
        create_faucet(pc, Some(admin_keypair.pubkey())).await;

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
                AccountMeta::new_readonly(admin_keypair.pubkey(), true),
            ],
            data: vec![1, 11, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    mint_tokens_tx.sign(&[&payer, &admin_keypair], recent_blockhash);

    // THEN
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
        11,
        spl_token::state::Account::unpack_from_slice(&acc.data)
            .unwrap()
            .amount
    );
}

#[tokio::test]
async fn test_mint_tokens_admin_included_but_didnt_sign() {
    // GIVEN
    let token_mint_address = VALID_MINT_PUBKEY;
    let token_account_address = Pubkey::new_unique();
    let mut pc = program_test();
    pc.add_account_with_file_data(
        token_account_address,
        1000000000,
        spl_token::id(),
        "valid-token-account-data.bin",
    );

    let admin_keypair = Keypair::new();
    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) =
        create_faucet(pc, Some(admin_keypair.pubkey())).await;

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
                AccountMeta::new_readonly(admin_keypair.pubkey(), false),
            ],
            data: vec![1, 11, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    mint_tokens_tx.sign(&[&payer], recent_blockhash);

    // THEN
    let error = banks_client
        .process_transaction(mint_tokens_tx)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        TransactionError::InstructionError(0, InstructionError::Custom(0x04)),
        error
    );

    let acc = banks_client
        .get_account(token_account_address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        0,
        spl_token::state::Account::unpack_from_slice(&acc.data)
            .unwrap()
            .amount
    );
}

#[tokio::test]
async fn test_mint_tokens_impostor_admin_included_and_signed() {
    // GIVEN
    let token_mint_address = VALID_MINT_PUBKEY;
    let token_account_address = Pubkey::new_unique();
    let mut pc = program_test();
    pc.add_account_with_file_data(
        token_account_address,
        1000000000,
        spl_token::id(),
        "valid-token-account-data.bin",
    );

    let admin_keypair = Keypair::new();

    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) =
        create_faucet(pc, Some(admin_keypair.pubkey())).await;

    let (pda, _nonce) = Pubkey::find_program_address(&[b"faucet"], &id());

    let impostor_admin_keypair = Keypair::new();
    let mut mint_tokens_tx = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(pda, false),
                AccountMeta::new(token_mint_address, false),
                AccountMeta::new(token_account_address, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(faucet_pubkey, false),
                AccountMeta::new_readonly(impostor_admin_keypair.pubkey(), true),
            ],
            data: vec![1, 11, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    mint_tokens_tx.sign(&[&payer, &impostor_admin_keypair], recent_blockhash);

    // THEN
    let error = banks_client
        .process_transaction(mint_tokens_tx)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        TransactionError::InstructionError(0, InstructionError::Custom(0x04)),
        error
    );

    let acc = banks_client
        .get_account(token_account_address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        0,
        spl_token::state::Account::unpack_from_slice(&acc.data)
            .unwrap()
            .amount
    );
}

#[tokio::test]
async fn test_mint_tokens_invalid_mint() {
    // GIVEN
    let token_account_address = Pubkey::new_unique();
    let mut pc = program_test();
    pc.add_account_with_file_data(
        token_account_address,
        1000000000,
        spl_token::id(),
        "valid-token-account-data.bin",
    );
    pc.add_account_with_file_data(
        SECOND_MINT_PUBKEY,
        1461600,
        spl_token::id(),
        "another-valid-token-mint-data.bin",
    );

    let admin_keypair = Keypair::new();

    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) =
        create_faucet(pc, Some(admin_keypair.pubkey())).await;

    let (pda, _nonce) = Pubkey::find_program_address(&[b"faucet"], &id());

    let mut mint_tokens_tx = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(pda, false),
                AccountMeta::new(SECOND_MINT_PUBKEY, false),
                AccountMeta::new(token_account_address, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(faucet_pubkey, false),
                AccountMeta::new_readonly(admin_keypair.pubkey(), true),
            ],
            data: vec![1, 11, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    mint_tokens_tx.sign(&[&payer, &admin_keypair], recent_blockhash);

    // THEN
    let error = banks_client
        .process_transaction(mint_tokens_tx)
        .await
        .unwrap_err()
        .unwrap();
    assert_eq!(
        TransactionError::InstructionError(0, InstructionError::Custom(0x08)),
        error
    );

    let acc = banks_client
        .get_account(token_account_address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        0,
        spl_token::state::Account::unpack_from_slice(&acc.data)
            .unwrap()
            .amount
    );
}

#[tokio::test]
async fn test_happy_flow_close_faucet() {
    // GIVEN
    let admin_keypair = Keypair::new();
    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) =
        create_faucet(program_test(), Some(admin_keypair.pubkey())).await;
    let (pda, _nonce) = Pubkey::find_program_address(&[b"faucet"], &id());
    let mut close_faucet_tx = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(admin_keypair.pubkey(), true),
                AccountMeta::new(faucet_pubkey, false),
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(VALID_MINT_PUBKEY, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(pda, false),
            ],
            data: vec![2, 11, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    close_faucet_tx.sign(&[&payer, &admin_keypair], recent_blockhash);

    banks_client
        .process_transaction(close_faucet_tx)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_close_faucet_admin_didnt_sign() {
    // GIVEN
    let admin_keypair = Keypair::new();
    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) =
        create_faucet(program_test(), Some(admin_keypair.pubkey())).await;
    let (pda, _nonce) = Pubkey::find_program_address(&[b"faucet"], &id());
    let mut close_faucet_tx = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(admin_keypair.pubkey(), false),
                AccountMeta::new(faucet_pubkey, false),
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(VALID_MINT_PUBKEY, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(pda, false),
            ],
            data: vec![2, 11, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    close_faucet_tx.sign(&[&payer], recent_blockhash);

    let error = banks_client
        .process_transaction(close_faucet_tx)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        TransactionError::InstructionError(
            0,
            solana_program::instruction::InstructionError::MissingRequiredSignature
        ),
        error
    );
}

#[tokio::test]
async fn test_close_faucet_admin_sign_not_closable() {
    // GIVEN
    let admin_keypair = Keypair::new();
    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) =
        create_faucet(program_test(), None).await;
    let (pda, _nonce) = Pubkey::find_program_address(&[b"faucet"], &id());
    let mut close_faucet_tx = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(admin_keypair.pubkey(), true),
                AccountMeta::new(faucet_pubkey, false),
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(VALID_MINT_PUBKEY, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(pda, false),
            ],
            data: vec![2, 11, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    close_faucet_tx.sign(&[&payer, &admin_keypair], recent_blockhash);

    let error = banks_client
        .process_transaction(close_faucet_tx)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        TransactionError::InstructionError(0, InstructionError::Custom(0x06)),
        error
    );
}

#[tokio::test]
async fn test_close_faucet_impostor_admin() {
    // GIVEN
    let admin_keypair = Keypair::new();
    let (mut banks_client, payer, recent_blockhash, faucet_pubkey) =
        create_faucet(program_test(), Some(admin_keypair.pubkey())).await;
    let (pda, _nonce) = Pubkey::find_program_address(&[b"faucet"], &id());

    let impostor_admin_keypair = Keypair::new();
    let mut close_faucet_tx = Transaction::new_with_payer(
        &[Instruction {
            program_id: id(),
            accounts: vec![
                AccountMeta::new_readonly(impostor_admin_keypair.pubkey(), true),
                AccountMeta::new(faucet_pubkey, false),
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(VALID_MINT_PUBKEY, false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(pda, false),
            ],
            data: vec![2, 11, 0, 0, 0, 0, 0, 0, 0],
        }],
        Some(&payer.pubkey()),
    );

    close_faucet_tx.sign(&[&payer, &impostor_admin_keypair], recent_blockhash);

    let error = banks_client
        .process_transaction(close_faucet_tx)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        TransactionError::InstructionError(0, InstructionError::Custom(0x05)),
        error
    );
}
