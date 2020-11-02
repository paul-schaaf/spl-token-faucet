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

const util = require('util');

const escrowAccountDataLayout = BufferLayout.struct([
  BufferLayout.u8('isInitialized'),
  Layout.publicKey('initializerPubkey'),
  Layout.publicKey('sendingTokenAccountPubkey'),
  Layout.publicKey('receivingTokenAccountPubkey'),
  Layout.uint64('expectedAmount'),
]);

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

  let payerAccount = await newAccountWithLamports(
    connection,
    LAMPORTS_PER_SOL * 10
  );

  const mintSending = await Token.createMint(
    connection,
    payerAccount,
    payerAccount.publicKey,
    payerAccount.publicKey,
    0,
    TOKEN_PROGRAM_ID
  );

  const taccSending = await mintSending.createAccount(payerAccount.publicKey);

  const mintReceiving = await Token.createMint(
    connection,
    payerAccount,
    payerAccount.publicKey,
    payerAccount.publicKey,
    0,
    TOKEN_PROGRAM_ID
  );

  const taccReceiving = await mintReceiving.createAccount(
    payerAccount.publicKey
  );

  const escrowAccount = await initEscrow(
    payerAccount,
    connection,
    programId,
    taccSending,
    taccReceiving
  );

  console.log("Saving escrow state to store...");
  await store.save("escrow.json", {
    escrowAccountPubkey: escrowAccount.publicKey.toBase58(),
    initializerPubkey: payerAccount.publicKey.toBase58(),
    taccSending: taccSending.toBase58(),
    taccReceiving: taccSending.toBase58(),
    creatorSecret: payerAccount.secretKey,
  });

  const accountDataBuffer = Buffer.from(JSON.parse(JSON.stringify(await connection.getParsedAccountInfo(escrowAccount.publicKey, 'singleGossip'))).value.data.data);

  const decodedData = escrowAccountDataLayout.decode(accountDataBuffer);

  decodedData.initializerPubkey = (new PublicKey(decodedData.initializerPubkey)).toBase58();
  decodedData.sendingTokenAccountPubkey = (new PublicKey(decodedData.sendingTokenAccountPubkey)).toBase58();
  decodedData.receivingTokenAccountPubkey = (new PublicKey(decodedData.receivingTokenAccountPubkey)).toBase58();
  decodedData.expectedAmount = parseInt(decodedData.expectedAmount.toString("hex"), 16)

  console.log("Escrow account data: ");
  console.log(decodedData);
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
  const space = escrowAccountDataLayout.span;
  const lamports = await connection.getMinimumBalanceForRentExemption(
    escrowAccountDataLayout.span
  );
  const initEscrowInstruction = new TransactionInstruction({
    keys: [
      { pubkey: payerAccount.publicKey, isSigner: true, isWritable: false },
      { pubkey: taccSending, isSigner: false, isWritable: true },
      { pubkey: taccReceiving, isSigner: false, isWritable: true },
      { pubkey: escrowAccountPubkey, isSigner: true, isWritable: true },
    ],
    programId,
    data: Uint8Array.of(0, 0, 0, 0, 0, 0, 0, 0, 76),
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
  console.log("Escrow account initialized!");

  return escrowAccount;
};

test()
  .catch((err) => {
    console.error(err);
    process.exit(1);
  })
  .then(() => process.exit());
