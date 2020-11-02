import { sendAndConfirmTransaction as realSendAndConfirmTransaction } from "@solana/web3.js";

export async function sendAndConfirmTransaction(
  connection,
  transaction,
  ...signers
) {
  await realSendAndConfirmTransaction(connection, transaction, signers, {
    skipPreflight: true,
    commitment: "singleGossip",
    preflightCommitment: null,
  });
}
