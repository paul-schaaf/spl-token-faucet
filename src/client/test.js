import {
  Account,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  Transaction,
  SystemProgram,
  TransactionInstruction
} from "@solana/web3.js";
import * as BufferLayout from "buffer-layout";

import { url } from "./url";
import { Store } from "./util/store";
import { newAccountWithLamports } from "./util/new-account-with-lamports";
import { sendAndConfirmTransaction } from "./util/send-and-confirm-transaction";

const escrowAccountDataLayout = BufferLayout.struct([
  BufferLayout.seq(BufferLayout.u8(), 105, "escrow"),
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

  console.log(await connection.getBalance(payerAccount.publicKey, 'singleGossip') / LAMPORTS_PER_SOL);

  const escrowAccount = await initEscrow(payerAccount, connection, programId);

  console.log("Saving escrow state to store...");
  await store.save("escrow.json", {
    escrowAccountPubkey: escrowAccount.publicKey.toBase58(),
    creatorSecret: payerAccount.secretKey,
  });
};

const initEscrow = async (payerAccount, connection, programId) => {
  const escrowAccount = new Account();
  let escrowAccountPubkey = escrowAccount.publicKey;
  const space = escrowAccountDataLayout.span;
  const lamports = await connection.getMinimumBalanceForRentExemption(
    escrowAccountDataLayout.span
  );
  const initEscrowInstruction = new TransactionInstruction({
    keys: [
      { pubkey: payerAccount.publicKey, isSigner: true, isWritable: false },
      { pubkey: new PublicKey("7G4eQSF3hM6jPv9bzLh2byx7dgqMDNiFqJY9pT6Kqdqm"), isSigner: false, isWritable: true},
      { pubkey: new PublicKey("6LrR7bzTCk8LHaUuCKBmc3TB4wAKeQqvpSqF6EvThM9p"), isSigner: false, isWritable: true },
      { pubkey: escrowAccountPubkey, isSigner: false, isWritable: true}
    ],
    programId
  });

  const transaction = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payerAccount.publicKey,
      newAccountPubkey: escrowAccountPubkey,
      lamports,
      space,
      programId,
    })
  ).add(initEscrowInstruction);
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
