import {
  Account,
  Connection,
  BpfLoader,
  BPF_LOADER_PROGRAM_ID,
  LAMPORTS_PER_SOL,
  SystemProgram,
  TransactionInstruction,
  Transaction,
} from "@solana/web3.js";
import fs from "mz/fs";
import * as BufferLayout from "buffer-layout";

import { url } from "./url";
import { newAccountWithLamports } from "./util/new-account-with-lamports";
import { sendAndConfirmTransaction } from "./util/send-and-confirm-transaction";

/**
 * Connection to the network
 */
let connection;

/**
 * Connection to the network
 */
let payerAccount;

/**
 * Hello world's program id
 */
let programId;

/**
 * The public key of the account we are saying hello to
 */
let greetedPubkey;

const pathToProgram = "dist/program/solana_escrow.so";

/**
 * Layout of the greeted account data
 */
const greetedAccountDataLayout = BufferLayout.struct([
  BufferLayout.u32("numGreets"),
]);

/**
 * Establish a connection to the cluster
 */
export async function establishConnection() {
  connection = new Connection(url, "singleGossip");
  const version = await connection.getVersion();
  console.log("Connection to cluster established:", url, version);
}

/**
 * Establish an account to pay for everything
 */
export async function establishPayer() {
  if (!payerAccount) {
    let fees = 0;
    const { feeCalculator } = await connection.getRecentBlockhash();

    // Calculate the cost to load the program
    const data = await fs.readFile(pathToProgram);
    const NUM_RETRIES = 500; // allow some number of retries
    fees +=
      feeCalculator.lamportsPerSignature *
        (BpfLoader.getMinNumSignatures(data.length) + NUM_RETRIES) +
      (await connection.getMinimumBalanceForRentExemption(data.length));

    // Calculate the cost to fund the greeter account
    fees += await await connection.getMinimumBalanceForRentExemption(
      greetedAccountDataLayout.span
    );

    // Calculate the cost of sending the transactions
    fees += feeCalculator.lamportsPerSignature * 100; // wag

    // Fund a new payer via airdrop
    payerAccount = await newAccountWithLamports(connection, fees);
  }

  const lamports = await connection.getBalance(payerAccount.publicKey);
  console.log(
    "Using account",
    payerAccount.publicKey.toBase58(),
    "containing",
    lamports / LAMPORTS_PER_SOL,
    "Sol to pay for fees"
  );
}

/**
 * Load the hello world BPF program if not already loaded
 */
export async function loadProgram() {
  // Load the program
  console.log("Loading hello world program...");
  const data = await fs.readFile(pathToProgram);
  const programAccount = new Account();
  await BpfLoader.load(
    connection,
    payerAccount,
    programAccount,
    data,
    BPF_LOADER_PROGRAM_ID
  );
  programId = programAccount.publicKey;
  console.log("Program loaded to account", programId.toBase58());

  // Create the greeted account
  const greetedAccount = new Account();
  greetedPubkey = greetedAccount.publicKey;
  console.log("Creating account", greetedPubkey.toBase58(), "to say hello to");
  const space = greetedAccountDataLayout.span;
  const lamports = await connection.getMinimumBalanceForRentExemption(
    greetedAccountDataLayout.span
  );
  const transaction = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payerAccount.publicKey,
      newAccountPubkey: greetedPubkey,
      lamports,
      space,
      programId,
    })
  );
  await sendAndConfirmTransaction(
    "createAccount",
    connection,
    transaction,
    payerAccount,
    greetedAccount
  );
}

/**
 * Say hello
 */
export async function sayHello() {
  console.log("Saying hello to", greetedPubkey.toBase58());
  const instruction = new TransactionInstruction({
    keys: [{ pubkey: greetedPubkey, isSigner: false, isWritable: true }],
    programId,
    data: Buffer.alloc(0), // All instructions are hellos
  });
  await sendAndConfirmTransaction(
    "sayHello",
    connection,
    new Transaction().add(instruction),
    payerAccount
  );
}

/**
 * Report the number of times the greeted account has been said hello to
 */
export async function reportHellos() {
  const accountInfo = await connection.getAccountInfo(greetedPubkey);
  if (accountInfo === null) {
    throw "Error: cannot find the greeted account";
  }
  const info = greetedAccountDataLayout.decode(Buffer.from(accountInfo.data));
  console.log(
    greetedPubkey.toBase58(),
    "has been greeted",
    info.numGreets.toString(),
    "times"
  );
}
