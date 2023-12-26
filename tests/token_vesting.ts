import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorError } from "@coral-xyz/anchor";
import { TokenVesting } from "../target/types/token_vesting";
import * as spl from '@solana/spl-token';
import * as assert from "assert";
import { createMint, createUserAndATA, fundATA, getTokenBalanceWeb3, createPDA } from "./utils";

describe("token_vesting", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TokenVesting as Program<TokenVesting>;

  let mintAddress, sender, senderATA, dataAccount, dataBump, escrowAccount, escrowBump, beneficiary, beneficiaryATA, beneficiaryArray;

  let _dataAccountAfterInit, _dataAccountAfterRelease, _dataAccountAfterClaim; // Used to store State between tests
 
  before(async () => {
    mintAddress = await createMint(provider);
    [sender, senderATA] = await createUserAndATA(provider, mintAddress);
    await fundATA(provider, mintAddress, sender, senderATA);

    // Create PDA's for account_data_account and escrow_wallet
    [dataAccount, dataBump] = await createPDA([Buffer.from("account_data"), mintAddress.toBuffer()], program.programId);
    [escrowAccount, escrowBump] = await createPDA([Buffer.from("escrow_wallet"), mintAddress.toBuffer()], program.programId);

    // Create a test Beneficiary object to send into contract
    [beneficiary, beneficiaryATA] = await createUserAndATA(provider, mintAddress);
    beneficiaryArray = [
      {
        key: beneficiary.publicKey,
        allocatedTokens: new anchor.BN(100),
        claimedTokens: new anchor.BN(0),
      }
    ]
  });

  it("Test Initialize", async () => {
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
    let accountAfterInit = await program.account.accountData.fetch(dataAccount);

    assert.equal(await getTokenBalanceWeb3(escrowAccount, provider), 1000); // Escrow account receives balance of token
    assert.equal(accountAfterInit.beneficiaries[0].allocatedTokens, 100); // Tests allocatedTokens field
    console.log(`init TX: https://explorer.solana.com/tx/${initTx}?cluster=custom`)

    _dataAccountAfterInit = dataAccount;

  });
  it("Test Release With False Sender" , async () => {
    dataAccount = _dataAccountAfterInit;

    const falseSender = anchor.web3.Keypair.generate();
    try {
      const releaseTx = await program.methods.release(dataBump, 43).accounts({
        accountDataAccount: dataAccount,
        sender: falseSender.publicKey,
        tokenMint: mintAddress,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).signers([falseSender]).rpc();
      assert.ok(false, "Error was supposed to be thrown");
    }catch(err) {
      // console.log(_err);
      assert.equal(err instanceof AnchorError, true);
      assert.equal(err.error.errorCode.code, "InvalidSender");
    }
    // let accountAfterRelease = await program.account.accountData.fetch(dataAccount);

    // assert.equal(accountAfterRelease.percentAvailable, 43); // Percent Available updated correctly

  });

  it("Test Release", async () => {
    dataAccount = _dataAccountAfterInit;

    const releaseTx = await program.methods.release(dataBump, 43).accounts({
      accountDataAccount: dataAccount,
      sender: sender.publicKey,
      tokenMint: mintAddress,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([sender]).rpc();
    let accountAfterRelease = await program.account.accountData.fetch(dataAccount);

    assert.equal(accountAfterRelease.percentAvailable, 43); // Percent Available updated correctly
    console.log(`release TX: https://explorer.solana.com/tx/${releaseTx}?cluster=custom`)

    _dataAccountAfterRelease = dataAccount;
  });

  it("Test Claim", async () => {
    // Send initialize transaction  
    dataAccount = _dataAccountAfterRelease;

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

    assert.equal(await getTokenBalanceWeb3(beneficiaryATA, provider), 43); // Claim releases 43% of 100 tokens into beneficiary's account
    assert.equal(await getTokenBalanceWeb3(escrowAccount, provider), 957);

    console.log(`claim TX: https://explorer.solana.com/tx/${claimTx}?cluster=custom`)
    _dataAccountAfterClaim = dataAccount;
  });

  it("Test Double Claim (Should Fail)", async () => {
    dataAccount = _dataAccountAfterClaim;
    try {
      // Should fail
      const doubleClaimTx = await program.methods.claim(dataBump, escrowBump).accounts({
        accountDataAccount: dataAccount,
        escrowWallet: escrowAccount,
        sender: beneficiary.publicKey,
        tokenMint: mintAddress,
        walletToDepositTo: beneficiaryATA,
        associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId
      }).signers([beneficiary]).rpc();
      assert.ok(false, "Error was supposed to be thrown");
    } catch (err) {
      assert.equal(err instanceof AnchorError, true);
      assert.equal(err.error.errorCode.code, "ClaimNotAllowed");
      assert.equal(await getTokenBalanceWeb3(beneficiaryATA, provider), 43);
      // Check that error is thrown, that it's the ClaimNotAllowed error, and that the beneficiary's balance has not changed
    }
  });
  it("Test Beneficiary Not Found (Should Fail)", async () => {
    dataAccount = _dataAccountAfterClaim;
    try {
      // const falseBeneficiary = anchor.web3.Keypair.generate();
      const [falseBeneficiary, falseBeneficiaryATA] = await createUserAndATA(provider, mintAddress);

      const benNotFound = await program.methods.claim(dataBump, escrowBump).accounts({
        accountDataAccount: dataAccount,
        escrowWallet: escrowAccount,
        sender: falseBeneficiary.publicKey,
        tokenMint: mintAddress,
        walletToDepositTo: falseBeneficiaryATA,
        associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId
      }).signers([falseBeneficiary]).rpc();

    }catch(err) {
      // console.log(err);
      assert.equal(err instanceof AnchorError, true);
      assert.equal(err.error.errorCode.code, "BeneficiaryNotFound");
    }
  });
});
