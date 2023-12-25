use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
declare_id!("DzJ68fvNC6PNfZYQzjSNiyLxgcxHD3nkRGsBAjyGTWyd");


// TODO:
// - Implement Custom Errors
// - Write Comprehensive Documentation & Test Cases
// - 
#[program]
pub mod token_vesting {

    use super::*;

    pub fn initialize(ctx: Context<InitializeNewVest>, beneficiaries: Vec<Beneficiary>, amount: u64, data_bump: u8, _escrow_bump: u8) -> Result<()> {
        msg!("ALLOCATED TOKENS: {}", beneficiaries[0].allocated_tokens);
        let account_data = &mut ctx.accounts.account_data_account;
        account_data.beneficiaries = beneficiaries;
        account_data.percent_available = 0;
        account_data.token_amount = amount;
        account_data.initializer = ctx.accounts.sender.to_account_info().key();
        account_data.escrow_wallet = ctx.accounts.escrow_wallet.to_account_info().key();
        account_data.token_mint = ctx.accounts.token_mint.to_account_info().key();

        msg!("account_data_account: {:?}", account_data.to_account_info().key);
        msg!("escrow_wallet: {:?}", ctx.accounts.escrow_wallet.to_account_info().key);
        msg!("wallet_to_withdraw_from: {:?}", ctx.accounts.wallet_to_withdraw_from.to_account_info().key);
        msg!("sender: {:?}", account_data.initializer);
        msg!("token_mint: {:?}", account_data.token_mint);
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
        
        let seeds = &["account_data".as_bytes(), account_data.token_mint.as_ref(), &[data_bump]];
        let signer_seeds = &[&seeds[..]];
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer_seeds
        );
        // The `?` at the end will cause the function to return early in case of an error.
        // This pattern is common in Rust.
        msg!("token amount: {}", account_data.token_amount);
        token::transfer(cpi_ctx, account_data.token_amount * 1000000)?;
        // token::transfer(cpi_ctx, account_data.token_amount )?;

        Ok(())
    }

    pub fn release(ctx: Context<Release>, _data_bump: u8, percent: u8 ) -> Result<()> {
        let data_account = &mut ctx.accounts.account_data_account;

        //for testing:
        // data_account.beneficiaries[0].claimed_tokens = 83;

        require!(data_account.initializer == *ctx.accounts.sender.to_account_info().key, ErrorCode::RequireEqViolated);
        require!(percent > 0, ErrorCode::RequireEqViolated);
        data_account.percent_available = percent;
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, data_bump: u8, _escrow_bump: u8) -> Result<()> {
        let sender = &mut ctx.accounts.sender;
        let escrow_wallet = &mut ctx.accounts.escrow_wallet;
        let data_account = &mut ctx.accounts.account_data_account;
        let beneficiaries = &data_account.beneficiaries;
        let token_program = &mut ctx.accounts.token_program;
        let token_mint_key = &mut ctx.accounts.token_mint.key();

        let beneficiary_ata = &mut ctx.accounts.wallet_to_deposit_to;

        msg!("CLAIM IN RUST, DATA BUMP: {}", data_bump);
        let (index, beneficiary) = beneficiaries.iter().enumerate().find(|(_, beneficiary)| beneficiary.key == *sender.to_account_info().key)
        .ok_or(ErrorCode::RequireEqViolated)?;

        let amount_to_transfer = ((data_account.percent_available as f32 / 100.0) * beneficiary.allocated_tokens as f32) as u64;
        require!(amount_to_transfer > beneficiary.claimed_tokens, ErrorCode::RequireEqViolated); //allowed to claim new tokens
        msg!("amount to transfer: {}", amount_to_transfer);
        msg!("allocated_tokens: {}", beneficiary.allocated_tokens);
        msg!("tokens: {}", data_account.token_amount);


        // Transfer Logic:
        
        // Creating seeds for the CPI 
        let seeds = &["account_data".as_bytes(), token_mint_key.as_ref(), &[data_bump]];
        let signer_seeds = &[&seeds[..]];

        let transfer_instruction = Transfer{
            from: escrow_wallet.to_account_info(),
            to: beneficiary_ata.to_account_info(),
            authority: data_account.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new_with_signer(
            token_program.to_account_info(),
            transfer_instruction,
            signer_seeds
        );

        token::transfer(cpi_ctx, amount_to_transfer * 1000000)?;
        data_account.beneficiaries[index].claimed_tokens = amount_to_transfer;
        
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(data_bump: u8, wallet_bump: u8)]
pub struct Claim<'info> {
    #[account(
        mut,
        seeds = [b"account_data", token_mint.key().as_ref()], 
        bump = data_bump,
    )]
    pub account_data_account: Account<'info, AccountData>,
    
    #[account(
        mut,
        seeds=[b"escrow_wallet".as_ref(), token_mint.key().as_ref()], // MIGHT have to remove .as_ref() for b"escrow_wallet", if bugs try that
        bump = wallet_bump,
    )]
    escrow_wallet: Account<'info, TokenAccount>,

    #[account(mut)]
    pub sender: Signer<'info>,
    
    pub token_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = sender,
        associated_token::mint = token_mint,
        associated_token::authority = sender,
    )]
    pub wallet_to_deposit_to: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>, // Don't actually use it in the instruction, but used for the wallet_to_deposit_to account
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(data_bump: u8)]
pub struct Release<'info> {
    #[account(
        mut,
        seeds = [b"account_data", token_mint.key().as_ref()], 
        bump = data_bump,
    )]
    pub account_data_account: Account<'info, AccountData>,
    pub token_mint: Account<'info, Mint>,

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
        seeds = [b"account_data", token_mint.key().as_ref()], 
        bump
    )]
    pub account_data_account: Account<'info, AccountData>,

    #[account(
        init,
        payer = sender,
        seeds=[b"escrow_wallet".as_ref(), token_mint.key().as_ref()],
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
    pub beneficiaries: Vec<Beneficiary>, // (4 + (n * (32 + 8 + 8)))
}
