import {
  Account,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  Transaction,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import * as BufferLayout from "buffer-layout";

import { url } from "./url";
import { Store } from "./util/store";
import { newAccountWithLamports } from "./util/new-account-with-lamports";
import { sendAndConfirmTransaction } from "./util/send-and-confirm-transaction";
import * as Layout from "./util/layout";

const ESCROW_ACCOUNT_DATA_LAYOUT = BufferLayout.struct([
  BufferLayout.u8("isInitialized"),
  Layout.publicKey("initializerPubkey"),
  Layout.publicKey("sendingTokenAccountPubkey"),
  Layout.publicKey("receivingTokenAccountPubkey"),
  Layout.uint64("expectedAmount"),
]);

const CREATOR_EXPECTED_AMOUNT = 76;


const test = async () => {
  const store = new Store();

  let deployConfig = await store.load("deploy.json");
  if (!deployConfig.programId) {
    throw new Error(
      "Deployment config file contains JSON data but not the programId"
    );
  }

  const connection = new Connection(url, "singleGossip");
  const version = await connection.getVersion();
  console.log("Connection to cluster established:", url, version);

  const programId = new PublicKey(deployConfig.programId);

  console.log("Creating master account...");
  let masterAccount = await newAccountWithLamports(
    connection,
    LAMPORTS_PER_SOL * 10
  );

  console.log("Creating creator account...");
  let creatorAccount = await newAccountWithLamports(
    connection,
    LAMPORTS_PER_SOL * 10
  );

  console.log("Creating sending token mint account...");
  const mintSending = await Token.createMint(
    connection,
    masterAccount,
    masterAccount.publicKey,
    masterAccount.publicKey,
    0,
    TOKEN_PROGRAM_ID
  );

  console.log("Creating sending token account...");
  const taccSending = await mintSending.createAccount(creatorAccount.publicKey);

  console.log("Minting tokens to taccSending account...");
  await mintSending.mintTo(taccSending, masterAccount, [], 100);

  console.log("Creating receiving token mint account...");
  const mintReceiving = await Token.createMint(
    connection,
    masterAccount,
    masterAccount.publicKey,
    masterAccount.publicKey,
    0,
    TOKEN_PROGRAM_ID
  );

  console.log("Creating receiving token account...");
  const taccReceiving = await mintReceiving.createAccount(
    creatorAccount.publicKey
  );

  const escrowAccount = await initEscrow(
    creatorAccount,
    connection,
    programId,
    taccSending,
    taccReceiving
  );

  const tempAccountData = await connection.getParsedAccountInfo(
    taccSending,
    "singleGossip"
  );
  const pda = (
    await PublicKey.findProgramAddress([Buffer.from("escrow")], programId)
  )[0].toBase58();

  if (pda !== tempAccountData.value.data.parsed.info.owner) {
    throw new Error("Failed to transfer token account ownership to PDA");
  }

  console.log("Saving escrow state to store...");
  await store.save("escrow.json", {
    escrowAccountPubkey: escrowAccount.publicKey.toBase58(),
    initializerPubkey: creatorAccount.publicKey.toBase58(),
    taccSending: taccSending.toBase58(),
    taccReceiving: taccSending.toBase58(),
    creatorSecret: creatorAccount.secretKey,
  });

  const decodedData = await getEscrowAccountData(connection, escrowAccount);

  console.log("Escrow account address: " + escrowAccount.publicKey.toBase58());
  console.log("Escrow account data: ");
  console.log(decodedData);

  console.log("Creating taker account...");
  let takerAccount = await newAccountWithLamports(
    connection,
    LAMPORTS_PER_SOL * 10
  );

  console.log("Creating taker sending token account...");
  const takerTaccSending = await mintReceiving.createAccount(
    takerAccount.publicKey
  );

  console.log("Minting tokens to takerTaccSending account...");
  await mintReceiving.mintTo(takerTaccSending, masterAccount, [], CREATOR_EXPECTED_AMOUNT);

  console.log("Creating taker receiving token account...");
  const takerTaccReceiving = await mintSending.createAccount(
    takerAccount.publicKey
  );

  await exchange(
    takerAccount,
    connection,
    programId,
    takerTaccSending,
    takerTaccReceiving,
    taccSending,
    taccReceiving,
    creatorAccount.publicKey,
    escrowAccount.publicKey
  );

  const creatorReceivedTokenAccount = await connection.getParsedAccountInfo(taccReceiving, "singleGossip");
  if (creatorReceivedTokenAccount.value.data.parsed.info.tokenAmount.amount !== '' + CREATOR_EXPECTED_AMOUNT) {
    console.log("Creator did not get his tokens");
  }
};

const initEscrow = async (
  payerAccount,
  connection,
  programId,
  taccSending,
  taccReceiving
) => {
  const escrowAccount = new Account();
  let escrowAccountPubkey = escrowAccount.publicKey;
  const space = ESCROW_ACCOUNT_DATA_LAYOUT.span;
  const lamports = await connection.getMinimumBalanceForRentExemption(
    ESCROW_ACCOUNT_DATA_LAYOUT.span
  );
  const initEscrowInstruction = new TransactionInstruction({
    keys: [
      { pubkey: payerAccount.publicKey, isSigner: true, isWritable: false },
      { pubkey: taccSending, isSigner: false, isWritable: true },
      { pubkey: taccReceiving, isSigner: false, isWritable: true },
      { pubkey: escrowAccountPubkey, isSigner: true, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    programId,
    data: Uint8Array.of(0, CREATOR_EXPECTED_AMOUNT, 0, 0, 0, 0, 0, 0, 0),
  });

  const transaction = new Transaction()
    .add(
      SystemProgram.createAccount({
        fromPubkey: payerAccount.publicKey,
        newAccountPubkey: escrowAccountPubkey,
        lamports,
        space,
        programId,
      })
    )
    .add(initEscrowInstruction);
  console.log("Initializing escrow account...");
  await sendAndConfirmTransaction(
    connection,
    transaction,
    payerAccount,
    escrowAccount
  );

  return escrowAccount;
};

const exchange = async (
  takerMain,
  connection,
  programId,
  takerTaccSending,
  takerTaccReceiving,
  taccSending,
  taccReceiving,
  creatorMain,
  escrow
) => {
  const exchangeInstruction = new TransactionInstruction({
    keys: [
      { pubkey: takerMain.publicKey, isSigner: true, isWritable: false },
      { pubkey: takerTaccSending, isSigner: false, isWritable: true },
      { pubkey: takerTaccReceiving, isSigner: false, isWritable: true },
      { pubkey: taccSending, isSigner: false, isWritable: true },
      { pubkey: creatorMain, isSigner: false, isWritable: true },
      { pubkey: taccReceiving, isSigner: false, isWritable: true },
      { pubkey: escrow, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    programId,
    data: Uint8Array.of(1, 100, 0, 0, 0, 0, 0, 0, 0),
  });

  console.log("Taking the trade...");
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(exchangeInstruction),
    takerMain
  );
};

const getEscrowAccountData = async (connection, escrowAccount) => {
  const accountDataBuffer = Buffer.from(
    JSON.parse(
      JSON.stringify(
        await connection.getParsedAccountInfo(
          escrowAccount.publicKey,
          "singleGossip"
        )
      )
    ).value.data.data
  );

  const decodedData = ESCROW_ACCOUNT_DATA_LAYOUT.decode(accountDataBuffer);

  decodedData.initializerPubkey = new PublicKey(
    decodedData.initializerPubkey
  ).toBase58();
  decodedData.sendingTokenAccountPubkey = new PublicKey(
    decodedData.sendingTokenAccountPubkey
  ).toBase58();
  decodedData.receivingTokenAccountPubkey = new PublicKey(
    decodedData.receivingTokenAccountPubkey
  ).toBase58();
  decodedData.expectedAmount = parseInt(
    decodedData.expectedAmount.toString("hex").match(/../g).reverse().join(""),
    16
  );

  return decodedData;
};

test()
  .catch((err) => {
    console.error(err);
    process.exit(1);
  })
  .then(() => process.exit());
