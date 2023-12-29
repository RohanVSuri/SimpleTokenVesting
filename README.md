# Simple Token Vesting

A Rust-based Anchor smart contract for a token vesting system on the Solana blockchain. The contract enables the creation of vesting accounts and the gradual release of tokens according to predefined vesting schedules.

## Contract Overview
This contract provides the ability to lock tokens under a PDA controlled by the contract, allocate a set amount of tokens to beneficiaries, release tokens to these accounts, and allows beneficiaries to claim their vested tokens. 

## Functionalities
The contract supports the following primary functionalities:

- Initialization (`initialize`): Set up the vesting contract with the necessary parameters, including the token mint address.
- Releasing Tokens (`release`): Allows the contract initializer to release a specified percentage of tokens to all beneficiaries.
- Claiming Tokens (`claim`): Beneficiaries can claim their vested tokens as per the released percentage.

## Getting Started
### Prerequisites
- Rust and Anchor installed on your system.
- Solana CLI tool.
- Basic understanding of Solana smart contracts and the Anchor framework.
### Installation
1. Clone the repository:
```bash
git clone https://github.com/RohanVSuri/SimpleTokenVesting.git
```

2. Build the contract:
```bash
# Install dependencies
npm install

# Build Anchor project
anchor build
```

3. Fetch the Program ID for the build:
```bash
solana address -k target/deploy/token_vesting-keypair.json
```

4. Replace Program ID in in `lib.rs` and `Anchor.toml` with the output from above:
```rust
// lib.rs: 
declare_id!("<REPLACE_PROGRAM_ID_HERE>");

// Anchor.toml: 
[programs.localnet]
token_vesting = "<REPLACE_PROGRAM_ID_HERE>"
```

5. Test the contract:
```bash
anchor test
```

## Usage
### Step 1: Token Creation 
Before using the contract, you must create a token on Solana & fund it. This is the token that is going to be vested to all beneficiaries. 
```bash
# Create the token
spl-token create-token

# Previous command will output a token mint address, use that to create an ATA, save this output as we will use it soon
spl-token create-account <TOKEN_MINT>

# Mint tokens to original token mint 
spl-token mint <TOKEN_MINT> <TOKEN_AMOUNT>
```
### Step 2: Generate PDA's
This contract uses 2 PDA's to govern both the Data & Escrow Account, and we must generate those PDA's & their bumps off-chain.
```js
// Use TOKEN_MINT from above as mintAddress and program ID as programID
// Use utils.js inside of tests directory as utils

[dataAccount, dataBump] = await utils.createPDA([Buffer.from("data_account"), mintAddress.toBuffer()], program.programId);

[escrowWallet, escrowBump] = await utils.createPDA([Buffer.from("escrow_wallet"), mintAddress.toBuffer()], program.programId);
```

### Step 3: Create Arrays of Beneficiaries
Create one array that contains all beneficiaries that contains all necessary information for the Beneficiary account
```js
beneficiaryArray = [
    {
        key: beneficiary.publicKey,
        allocatedTokens: new anchor.BN(<ALLOCATED_TOKENS>),
        claimedTokens: new anchor.BN(0),
    },
    {
        key: beneficiary2.publicKey,
        allocatedTokens: new anchor.BN(<ALLOCATED_TOKENS>),
        claimedTokens: new anchor.BN(0),
    }
]
```
### Step 4: Initialize the Contract
- Call the `initialize` function with the public key of the created token mint, an array of all beneficiaries to be vested to, and bumps for PDA's of the Data Account & Escrow Account.
```js
// TOKEN_MINT from above as mintAddress
// senderATA as the output from spl-token create-account <TOKEN_MINT>
// sender as an Anchor Keypair

const initTx = await program.methods.initialize(beneficiaryArray, new anchor.BN(<AMOUNT_TO_ESCROW>), decimals, dataBump, escrowBump)
    .accounts({
        dataAccount: dataAccount,
        escrowWallet: escrowWallet,
        walletToWithdrawFrom: senderATA,
        tokenMint: mintAddress,
        sender: sender.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
    })
    .signers([sender])
    .rpc();
```

### Step 5: Releasing Tokens
- When you are ready to release tokens to beneficiaries, call the `release` function specifying the percentage of tokens to be released.
```js
const releaseTx = await program.methods.release(dataBump, <PERCENT_TO_RELEASE>)
    .accounts({
        dataAccount: dataAccount,
        sender: sender.publicKey,
        tokenMint: mintAddress,
        systemProgram: anchor.web3.SystemProgram.programId,
    })
    .signers([sender])
    .rpc();
```
### Step 6: Claiming Tokens
- Beneficiaries can then call the `claim` function to claim their vested tokens according to the released percentage.
```js
const claimTx = await program.methods.claim(dataBump, escrowBump)
    .accounts({
        dataAccount: dataAccount,
        escrowWallet: escrowWallet,
        sender: beneficiary.publicKey,
        tokenMint: mintAddress,
        walletToDepositTo: beneficiaryATA,
        associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId
    })
    .signers([beneficiary])
    .rpc();
```
*Refer to `tests/token_vesting.ts` for example usage*

## Author
Bulit by Rohan Suri
