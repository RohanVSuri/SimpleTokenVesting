import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorError } from "@coral-xyz/anchor";
import { TokenVesting } from "../target/types/token_vesting";
import * as spl from '@solana/spl-token';
import * as assert from "assert";
import { createMint, createUserAndATA, fundATA, getTokenBalanceWeb3, createPDA } from "./utils";
// Configure the client to use the local cluster.

describe("token_vesting", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TokenVesting as Program<TokenVesting>;
  it("Is initialized!", async () => {

    const mintAddress = await createMint(provider);
    const [sender, senderATA] = await createUserAndATA(provider, mintAddress);
    const _ = await fundATA(provider, mintAddress, sender, senderATA);
    let x = Buffer.from("account_data");
    // Create PDA's for account_data_account and escrow_wallet
    let [dataAccount, dataBump] = await createPDA([Buffer.from("account_data"), mintAddress.toBuffer()], program.programId);
    let [escrowAccount, escrowBump] = await createPDA([Buffer.from("escrow_wallet"), mintAddress.toBuffer()], program.programId);

    console.log("ACCTPDA: ", dataAccount);
    console.log("ESCROWPDA: ", escrowAccount);

    // Create a test Beneficiary object to send into contract
    const [beneficiary, beneficiaryATA] = await createUserAndATA(provider, mintAddress);
    console.log("beneficiary ata: ", beneficiaryATA);

    const beneficiaryArray = [
      {
        key: beneficiary.publicKey,
        allocatedTokens: new anchor.BN(100),
        claimedTokens: new anchor.BN(0),
      }
    ]
    // Send initialize transaction  
    const initTx = await program.methods.initialize(beneficiaryArray, new anchor.BN(1000), dataBump, escrowBump).accounts({

      accountDataAccount: dataAccount,
      escrowWallet: escrowAccount,
      walletToWithdrawFrom: senderATA,
      tokenMint: mintAddress,
      sender: sender.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
    }).signers([sender]).rpc();

    console.log("Initialize transaction signature", initTx);

    let account = await program.account.accountData.fetch(dataAccount);
    console.log(account);

    const releaseTx = await program.methods.release(dataBump, 43).accounts({
      accountDataAccount: dataAccount,
      sender: sender.publicKey,
      tokenMint: mintAddress,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([sender]).rpc();
    console.log("Release TX Sig: ", releaseTx)

    let account2 = await program.account.accountData.fetch(dataAccount);
    console.log(account2);

    const claimTx = await program.methods.claim(dataBump, escrowBump).accounts({
      accountDataAccount: dataAccount,
      escrowWallet: escrowAccount,
      sender: beneficiary.publicKey,
      tokenMint: mintAddress,
      walletToDepositTo: beneficiaryATA,
      associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId
    }).signers([beneficiary]).rpc();

    let account3 = await program.account.accountData.fetch(dataAccount);
    console.log(account3);
    console.log(`init TX: https://explorer.solana.com/tx/${initTx}?cluster=custom`)
    console.log(`release TX: https://explorer.solana.com/tx/${releaseTx}?cluster=custom`)
    console.log(`claim TX: https://explorer.solana.com/tx/${claimTx}?cluster=custom`)
    // console.log("Initialize transaction signature", initTx);
    // console.log("Release TX Sig: ", releaseTx)
    // console.log("Claim TX Sig: ", claimTx);
    assert.equal(await getTokenBalanceWeb3(beneficiaryATA, provider), 43); // Claim releases 43% of 100 tokens into beneficiary's account
    try {
      const claimTx2 = await program.methods.claim(dataBump, escrowBump).accounts({
        accountDataAccount: dataAccount,
        escrowWallet: escrowAccount,
        sender: beneficiary.publicKey,
        tokenMint: mintAddress,
        walletToDepositTo: beneficiaryATA,
        associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId
      }).signers([beneficiary]).rpc();
      let account4 = await program.account.accountData.fetch(dataAccount);
      console.log(account4);
      console.log(await getTokenBalanceWeb3(beneficiaryATA, provider));
      assert.ok(false, "Error was supposed to be thrown");
    }catch(_err){
      assert.equal(_err instanceof AnchorError, true);
      const err: AnchorError = _err;
      console.log(err);
      // const errMsg =
      //   "This is an error message clients will automatically display";
      // assert.strictEqual(err.error.errorMessage, errMsg);
      // assert.strictEqual(err.error.errorCode.number, 6000);

    }
  });
});
