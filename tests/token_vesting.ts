import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenVesting } from "../target/types/token_vesting";

describe("token_vesting", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenVesting as Program<TokenVesting>;

  const account = anchor.web3.Keypair.generate();
  console.log(account);
  it("Is initialized!", async () => {
    // Add your test here.
    // const vestingAccounts = [
    //   {
    //     beneficiary: new PublicKey('BeneficiaryPublicKey1'),
    //     total_amount: 1000,
    //   },
    //   {
    //     beneficiary: new PublicKey('BeneficiaryPublicKey2'),
    //     total_amount: 2000,
    //   },
    //   // Add more vesting accounts as needed
    // ];

    // const tx = await program.methods.initialize().accounts({}).signers([account]).rpc();
    // console.log("Your transaction signature", tx);
  });
});
