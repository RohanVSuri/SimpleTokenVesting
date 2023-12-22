import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenVesting } from "../target/types/token_vesting";
import * as spl from '@solana/spl-token';

describe("token_vesting", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenVesting as Program<TokenVesting>;

  const account = anchor.web3.Keypair.generate();
  it("Is initialized!", async () => {

    const toKp = anchor.web3.Keypair.generate();

    // Create a new mint and initialize it
    // const mintKp = anchor.web3.Keypair.generate();
    // console.log("before");
    // const mint = await spl.createMint(
    //   program.provider.connection,
    //   account,
    //   account.publicKey,
    //   null,
    //   0
    // );
    // console.log("after");



    // const tx = await program.methods.initialize().accounts({

    // }).signers([account]).rpc();

    // console.log("Your transaction signature", tx);
  });
});
