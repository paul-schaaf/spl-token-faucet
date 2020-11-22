import { Account } from "@solana/web3.js";
import * as bip39 from "bip39";
import * as bip32 from "bip32";
import nacl from "tweetnacl";

async function mnemonicToSeed(mnemonic) {
  if (!bip39.validateMnemonic(mnemonic)) {
    throw new Error("Invalid seed phrase");
  }
  return await bip39.mnemonicToSeed(mnemonic);
}

function getAccountFromSeed(seed, walletIndex, accountIndex = 0) {
  const derivedSeed = bip32
    .fromSeed(seed)
    .derivePath(`m/501'/${walletIndex}'/0/${accountIndex}`).privateKey;
  return new Account(nacl.sign.keyPair.fromSeed(derivedSeed).secretKey);
}

export const createAccount = async (secret) => {
  if (secret.includes(",")) {
    return new Account(
      secret
        .replace(/ /g, "")
        .split(",")
        .map((n) => parseInt(n))
    );
  }
  const seed = await mnemonicToSeed(secret);
  return getAccountFromSeed(seed, 0, 0);
};
