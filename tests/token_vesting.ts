import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenVesting } from "../target/types/token_vesting";
const { SystemProgram } = anchor.web3;
import { LAMPORTS_PER_SOL } from '@solana/web3.js';
import * as spl from '@solana/spl-token';
import { token } from "@coral-xyz/anchor/dist/cjs/utils";



describe("token_vesting", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenVesting as Program<TokenVesting>;

  const createMint = async (connection: anchor.web3.Connection): Promise<anchor.web3.PublicKey> => {
    const tokenMint = new anchor.web3.Keypair();
    const lamportsForMint = await provider.connection.getMinimumBalanceForRentExemption(spl.MintLayout.span);
    let tx = new anchor.web3.Transaction();

    // Allocate mint
    tx.add(
      anchor.web3.SystemProgram.createAccount({
        programId: spl.TOKEN_PROGRAM_ID,
        space: spl.MintLayout.span,
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: tokenMint.publicKey,
        lamports: lamportsForMint,
      })
    )
    // Allocate wallet account
    tx.add(
      spl.createInitializeMintInstruction(
        tokenMint.publicKey,
        6,
        provider.wallet.publicKey,
        provider.wallet.publicKey,
        spl.TOKEN_PROGRAM_ID,
      )
    );
    const signature = await provider.sendAndConfirm(tx, [tokenMint]);

    console.log(`[${tokenMint.publicKey}] Created new mint account at ${signature}`);
    return tokenMint.publicKey;
  }

  const createUserAndATA = async (connection: anchor.web3.Connection, mint: anchor.web3.PublicKey): Promise<[anchor.web3.Keypair, anchor.web3.PublicKey | undefined]> => {
    // Create the User, fund with 10 SOL to be able to execute tx's
    const user = anchor.web3.Keypair.generate();
    let token_airdrop = await provider.connection.requestAirdrop(user.publicKey,
      10 * LAMPORTS_PER_SOL);

    const latestBlockHash = await provider.connection.getLatestBlockhash();

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: token_airdrop,
    });

    console.log("airdrop successful??");

    // Find ATA for the User
    let userATA = await spl.getAssociatedTokenAddress(
      mint,
      user.publicKey,
      false,
      spl.TOKEN_PROGRAM_ID,
      spl.ASSOCIATED_TOKEN_PROGRAM_ID
    );
    console.log("userata finding successful");
    // Create TX to mint tokens to the User
    const txFundATA = new anchor.web3.Transaction();
    
    txFundATA.add(
      spl.createAssociatedTokenAccountInstruction(
        user.publicKey,
        userATA,
        user.publicKey,
        mint,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID
      )
    );

    txFundATA.add(
      spl.createMintToInstruction(
        mint,
        userATA,
        provider.wallet.publicKey,
        2000000000,
        [],
        spl.TOKEN_PROGRAM_ID
      )
    );
    const txFundToken = await provider.sendAndConfirm(txFundATA, [user]);
    // console.log("first fund successful");

    // const txFundATA2 = new anchor.web3.Transaction();

    // txFundATA2.add(
    //   spl.createMintToInstruction(
    //     mint,
    //     userATA,
    //     provider.wallet.publicKey,
    //     2000000000,
    //     [],
    //     spl.TOKEN_PROGRAM_ID
    //   )
    // );

    // const txFundToken2 = await provider.sendAndConfirm(txFundATA2, [user]);
    console.log("second fund successful");
    console.log(userATA.toBase58(), txFundToken);
    return [user, userATA]

  }
//   const readAccount = async (accountPublicKey: anchor.web3.PublicKey, provider: anchor.Provider): Promise<[spl.AccountInfo, string]> => {
//     const tokenInfoLol = await provider.connection.getAccountInfo(accountPublicKey);
//     const data = Buffer.from(tokenInfoLol.data);
//     const accountInfo = spl.AccountLayout.decode(data);
//     console.log(accountInfo);

//     const amount = (accountInfo.amount as any as Buffer).readBigUInt64LE();
//     return [accountInfo, amount.toString()];
// }

  it("Is initialized!", async () => {
    
    const mintAddress = await createMint(provider.connection);
    const [sender, senderATA] = await createUserAndATA(provider.connection, mintAddress);

    // Airdrop sender wallet with 10 SOL so that they can send the tx
    // const sender = anchor.web3.Keypair.generate();
    // let token_airdrop = await provider.connection.requestAirdrop(sender.publicKey,
    //   10 * LAMPORTS_PER_SOL);

    // const latestBlockHash = await provider.connection.getLatestBlockhash();
    // await provider.connection.confirmTransaction({
    //   blockhash: latestBlockHash.blockhash,
    //   lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
    //   signature: token_airdrop,
    // });


    //  let txFund = new anchor.web3.Transaction();
    //       txFund.add(anchor.web3.SystemProgram.transfer({
    //           fromPubkey: provider.wallet.publicKey,
    //           toPubkey: sender.publicKey,
    //           lamports: 5 * anchor.web3.LAMPORTS_PER_SOL,
    //       }));
    //       const sigTxFund = await provider.sendAndConfirm(txFund);
    //       console.log(`[${sender.publicKey.toBase58()}] Funded new account with 5 SOL: ${sigTxFund}`);


    console.log("all success");
    // Create PDA's for account_data_account and escrow_wallet
    let [accountData, accountBump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("account_data"), sender.publicKey.toBuffer()],
      program.programId
    );

    let [escrowWallet, escrowBump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow_wallet"), sender.publicKey.toBuffer()],
      program.programId
    );
    console.log("ACCTPDA: ", accountData);
    console.log("ESCROWPDA: ", escrowWallet);
    // Create a test Beneficiary object to send into contract
    const test_beneficiary = anchor.web3.Keypair.generate();
    const beneficiary = [
      {
        key: test_beneficiary.publicKey,
        allocatedTokens: new anchor.BN(69),
        claimedTokens: new anchor.BN(0),
      }
    ]

    // Send initialize transaction
    const tx = await program.methods.initialize(beneficiary, new anchor.BN(200000000), accountBump, escrowBump).accounts({
      accountDataAccount: accountData,
      escrowWallet: escrowWallet,
      walletToWithdrawFrom: senderATA,
      tokenMint: mintAddress,
      sender: sender.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: spl.TOKEN_PROGRAM_ID,
    }).signers([sender]).rpc();

    console.log("Your transaction signature", tx);

    let account = await program.account.accountData.fetch(accountData);
    console.log(account.beneficiaries);

    const tx2 = await program.methods.release(accountBump, 43).accounts({
      accountDataAccount: accountData,
      sender: sender.publicKey,
      systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([sender]).rpc();

    let account2 = await program.account.accountData.fetch(accountData);
    console.log(account2.beneficiaries);
    // const [x, y] = await readAccount(escrowWallet, provider);
    // console.log(x, y);
  });
});
