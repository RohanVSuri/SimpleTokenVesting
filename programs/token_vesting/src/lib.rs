use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("DzJ68fvNC6PNfZYQzjSNiyLxgcxHD3nkRGsBAjyGTWyd");


// TODO:
// - Complete Claim Function
// - Implement Custom Errors
// - Write Comprehensive Documentation & Test Cases
// - 
#[program]
pub mod token_vesting {

    use super::*;

    pub fn initialize(
        ctx: Context<InitializeNewVest>,
        beneficiaries: Vec<Beneficiary>,
        amount: u64,
        data_bump: u8,
        _escrow_bump: u8,
    ) -> Result<()> {
        msg!("ALLOCATED TOKENS: {}", beneficiaries[0].allocated_tokens);
        let account_data = &mut ctx.accounts.account_data_account;
        account_data.beneficiaries = beneficiaries;
        account_data.percent_available = 0;
        account_data.token_amount = amount;
        account_data.initializer = *ctx.accounts.sender.to_account_info().key;
        account_data.escrow_wallet = *ctx.accounts.escrow_wallet.to_account_info().key;
        account_data.token_mint = *ctx.accounts.token_mint.to_account_info().key;

        // msg!("account_data_account: {:?}", account_data.to_account_info().key);
        // msg!("escrow_wallet: {:?}", ctx.accounts.escrow_wallet.to_account_info().key);
        // msg!("wallet_to_withdraw_from: {:?}", ctx.accounts.wallet_to_withdraw_from.to_account_info().key);
        // msg!("sender: {:?}", account_data.initializer);
        // msg!("token_mint: {:?}", account_data.token_mint);
        // account_data.token_amount = beneficiaries[0].allocated_tokens;
        // account_data.beneficiaries = beneficiaries;
        // account_data.reload()?;
        // msg!("{}", account_data.beneficiaries[0].allocated_tokens);

        // Below is the actual instruction that we are going to send to the Token program.
        let transfer_instruction = Transfer {
            from: ctx.accounts.wallet_to_withdraw_from.to_account_info(),
            to: ctx.accounts.escrow_wallet.to_account_info(),
            authority: ctx.accounts.sender.to_account_info(),
        };
        // let (pda, bump) = Pubkey::find_program_address(&[b"account_data", ctx.accounts.sender.key.as_ref()], ctx.program_id);
        // require!(pda == *account_data.to_account_info().key, ErrorCode::RequireEqViolated);
        // msg!("PDA: {:?}", pda);
        
        let seeds = &["account_data".as_bytes(), ctx.accounts.sender.key.as_ref(), &[data_bump]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer_seeds
        );
        // The `?` at the end will cause the function to return early in case of an error.
        // This pattern is common in Rust.
        token::transfer(cpi_ctx, account_data.token_amount)?;

        Ok(())
    }

    pub fn release(ctx: Context<Release>, _data_bump: u8, percent: u8 ) -> Result<()> {
        let data_account = &mut ctx.accounts.account_data_account;

        //for testing:
        data_account.beneficiaries[0].claimed_tokens = 83;

        require!(data_account.initializer == *ctx.accounts.sender.to_account_info().key, ErrorCode::RequireEqViolated);
        require!(percent > 0, ErrorCode::RequireEqViolated);
        data_account.percent_available = percent;
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, _data_bump: u8) -> Result<()> {
        let sender = &mut ctx.accounts.sender;
        let data_account = &mut ctx.accounts.account_data_account;
        let beneficiaries = &data_account.beneficiaries;

        let beneficiary = beneficiaries.iter().find(|&beneficiary| beneficiary.key == *sender.to_account_info().key)
        .ok_or(ErrorCode::RequireEqViolated)?;

    // Use beneficiary here
    println!("Found beneficiary: {}", beneficiary.key);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(data_bump: u8)]
pub struct Claim<'info> {
    #[account(
        mut,
        seeds = [b"account_data", sender.key().as_ref()], 
        bump = data_bump,
    )]
    pub account_data_account: Account<'info, AccountData>,
    
    #[account(mut)]
    pub sender: Signer<'info>,
    // #[account(mut)]
    // pub beneficiary: Account<'info, Beneficiary>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(data_bump: u8)]
pub struct Release<'info> {
    #[account(
        mut,
        seeds = [b"account_data", sender.key().as_ref()], 
        bump = data_bump,
    )]
    pub account_data_account: Account<'info, AccountData>,

    #[account(mut)]
    pub sender: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
// #[instruction(application_idx: u64, state_bump: u8, wallet_bump: u8)]
pub struct InitializeNewVest<'info> {
    #[account(
        init,
        payer = sender,
        space = 8 + 1 + 8 + 32 + 32 + 32 + 1 + (4 + 50 * (32 + 8 + 8)), //can take 50 accounts to vest to
        seeds = [b"account_data", sender.key().as_ref()], 
        bump
    )]
    pub account_data_account: Account<'info, AccountData>,

    #[account(
        init,
        payer = sender,
        seeds=[b"escrow_wallet".as_ref(), sender.key().as_ref()],
        bump,
        token::mint=token_mint,
        token::authority=account_data_account,
    )]
    pub escrow_wallet: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint=wallet_to_withdraw_from.owner == sender.key(),
        constraint=wallet_to_withdraw_from.mint == token_mint.key()
    )]
    wallet_to_withdraw_from: Account<'info, TokenAccount>,

    token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub sender: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Default, Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct Beneficiary {
    pub key: Pubkey,           // 32
    pub allocated_tokens: u64, // 8
    pub claimed_tokens: u64,   // 8
}

#[account]
#[derive(Default)]
pub struct AccountData {
    // 8 + 1 + 8 + 32 + 32 + 32 + 1 + (4 + (100 * (32 + 8 + 8)))
    pub percent_available: u8, // 1
    pub token_amount: u64,     // 8
    pub initializer: Pubkey,   // 32
    pub escrow_wallet: Pubkey, // 32
    pub token_mint: Pubkey,    // 32
    // pub stage: u8, // 1
    pub beneficiaries: Vec<Beneficiary>, // (4 + (n * (32 + 8 + 8)))
}
