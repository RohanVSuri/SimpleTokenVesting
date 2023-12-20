use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("DzJ68fvNC6PNfZYQzjSNiyLxgcxHD3nkRGsBAjyGTWyd");

#[program]
pub mod token_vesting {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, token_mint: Pubkey) -> Result<()> {
        let account_data = &mut ctx.accounts.account_data_account;
        account_data.percent_available = 0;
        account_data.initializer = *ctx.accounts.initializer.to_account_info().key;
        account_data.token_mint = token_mint;
        Ok(())
    }

    pub fn create_vesting_account(ctx: Context<CreateVestingAccount>, beneficiary: Pubkey, total_amount: u64) -> Result<()> {
        require!(*ctx.accounts.initializer.to_account_info().key == ctx.accounts.account_data.initializer, VestingError::NotInitializer);
        //initializer of the contract must also be the one who creates these vesting accounts 

        let vesting_account = &mut ctx.accounts.vesting_account;
        vesting_account.beneficiary = beneficiary;
        vesting_account.total_amount = total_amount;
        vesting_account.claimed_amount = 0;
        Ok(())
    }

    pub fn release(ctx: Context<Release>, percent: u8) -> Result<()>{
        require!(*ctx.accounts.sender.to_account_info().key == ctx.accounts.account_data_account.initializer, VestingError::NotInitializer);
        //this contract will 'release' `percent` of token to all beneficiaries
        //should require that sender is one who initialized contract
        ctx.accounts.account_data_account.percent_available = percent;
        Ok(())
    }
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        //check if account exists
        //check if money not claimed yet
        //then send the tokens to the person calling 
        Ok(())
    }
}



#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = initializer, space = 32)] //fix space
    pub account_data_account: Account<'info, AccountData>,

    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,

}

#[derive(Accounts)]
pub struct CreateVestingAccount<'info> {
    #[account(init, payer = initializer, space = 32)] //fix space
    pub vesting_account: Account<'info, VestingAccount>,
    pub account_data: Account<'info, AccountData>,

    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Release<'info> {
    #[account(mut)]
    pub account_data_account: Account<'info, AccountData>,
    pub sender: Signer<'info>
}

#[derive(Accounts)]
pub struct Claim<'info>{
    #[account(mut)]
    pub account_data_account: Account<'info, VestingAccount>,
    pub sender: Signer<'info>
}

#[account]
#[derive(Default)]
pub struct VestingAccount{
    pub beneficiary: Pubkey,
    pub total_amount: u64,
    pub claimed_amount: u64
}

#[account]
#[derive(Default)]
pub struct AccountData{
    pub percent_available: u8,
    pub initializer: Pubkey,
    pub token_mint: Pubkey

}

#[error_code]
pub enum VestingError {
    #[msg("Instruction sender is not initializer of contract")]
    NotInitializer,
    #[msg("Wrong token account")]
    InvalidTokenAccount
}
