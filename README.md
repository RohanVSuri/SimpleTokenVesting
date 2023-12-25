# Simple Token Vesting

A Rust-based Anchor smart contract for a token vesting system on the Solana blockchain. The contract enables the creation of vesting accounts and the gradual release of tokens according to predefined vesting schedules.
## Contract Overview
The contract provides functionalities for initializing the contract with a token mint, creating vesting accounts for beneficiaries, releasing tokens to these accounts, and allowing beneficiaries to claim their vested tokens.

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
Ensure that Anchor & Rust are installed on your machine & repository is cloned.  

Build the contract:
```console
anchor build
```
Test the contract:
```console
anchor test
```

## Usage
### Step 1: Token Creation 
- Before using the contract, you must create a token on Solana. This is the token that is going to be vested to all beneficiaries. 

### Step 2: Initialize the Contract
- Call the `initialize` function with the public key of the created token mint, an array of all beneficiaries to be vested to, and bumps for PDA's of the Data Account & Escrow Account.

### Step 3: Releasing Tokens
- When you are ready to release tokens to beneficiaries, call the `release` function specifying the percentage of tokens to be released.

### Step 4: Claiming Tokens
- Beneficiaries can then call the `claim` function to claim their vested tokens according to the released percentage.

## Author
Bulit by Rohan Suri
