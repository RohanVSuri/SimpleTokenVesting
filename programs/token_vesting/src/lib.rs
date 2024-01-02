use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
declare_id!("DzJ68fvNC6PNfZYQzjSNiyLxgcxHD3nkRGsBAjyGTWyd");

#[program]
pub mod token_vesting {

    use super::*;

    pub fn initialize(ctx: Context<InitializeNewVest>, beneficiaries: Vec<Beneficiary>, amount: u64, decimals: u8, data_bump: u8, _escrow_bump: u8) -> Result<()> {
        let data_account = &mut ctx.accounts.data_account;
        data_account.beneficiaries = beneficiaries;
        data_account.percent_available = 0;
        data_account.token_amount = amount;
        data_account.decimals = decimals;
        data_account.initializer = ctx.accounts.sender.to_account_info().key();
        data_account.escrow_wallet = ctx.accounts.escrow_wallet.to_account_info().key();
        data_account.token_mint = ctx.accounts.token_mint.to_account_info().key();

        let transfer_instruction = Transfer {
            from: ctx.accounts.wallet_to_withdraw_from.to_account_info(),
            to: ctx.accounts.escrow_wallet.to_account_info(),
            authority: ctx.accounts.sender.to_account_info(),
        };
        
        let seeds = &["data_account".as_bytes(), data_account.token_mint.as_ref(), &[data_bump]];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer_seeds
        );

        token::transfer(cpi_ctx, data_account.token_amount * u64::pow(10, decimals as u32))?;

        Ok(())
    }

    pub fn release(ctx: Context<Release>, _data_bump: u8, percent: u8 ) -> Result<()> {
        let data_account = &mut ctx.accounts.data_account;

        require!(data_account.initializer == *ctx.accounts.sender.to_account_info().key, VestingError::InvalidSender); 
        // Here we are testing the Public Key because Solana won't let you sign a transaction with conflicting signers & sender fields

        data_account.percent_available = percent;
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, data_bump: u8, _escrow_bump: u8) -> Result<()> {
        let sender = &mut ctx.accounts.sender;
        let escrow_wallet = &mut ctx.accounts.escrow_wallet;
        let data_account = &mut ctx.accounts.data_account;
        let beneficiaries = &data_account.beneficiaries;
        let token_program = &mut ctx.accounts.token_program;
        let token_mint_key = &mut ctx.accounts.token_mint.key();
        let beneficiary_ata = &mut ctx.accounts.wallet_to_deposit_to;
        let decimals = data_account.decimals;

        let (index, beneficiary) = beneficiaries.iter().enumerate().find(|(_, beneficiary)| beneficiary.key == *sender.to_account_info().key)
        .ok_or(VestingError::BeneficiaryNotFound)?;

        let amount_to_transfer = ((data_account.percent_available as f32 / 100.0) * beneficiary.allocated_tokens as f32) as u64;
        require!(amount_to_transfer > beneficiary.claimed_tokens, VestingError::ClaimNotAllowed); // Allowed to claim new tokens

        // Transfer Logic:
        let seeds = &["data_account".as_bytes(), token_mint_key.as_ref(), &[data_bump]];
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

        token::transfer(cpi_ctx, amount_to_transfer * u64::pow(10, decimals as u32))?;
        data_account.beneficiaries[index].claimed_tokens = amount_to_transfer;
        
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(data_bump: u8, wallet_bump: u8)]
pub struct Claim<'info> {
    #[account(
        mut,
        seeds = [b"data_account", token_mint.key().as_ref()], 
        bump = data_bump,
    )]
    pub data_account: Account<'info, DataAccount>,
    
    #[account(
        mut,
        seeds=[b"escrow_wallet".as_ref(), token_mint.key().as_ref()],
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
        seeds = [b"data_account", token_mint.key().as_ref()], 
        bump = data_bump,
    )]
    pub data_account: Account<'info, DataAccount>,
    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub sender: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeNewVest<'info> {
    #[account(
        init,
        payer = sender,
        space = 8 + 1 + 8 + 32 + 32 + 32 + 1 + (4 + 50 * (32 + 8 + 8) + 1), // Can take 50 accounts to vest to
        seeds = [b"data_account", token_mint.key().as_ref()], 
        bump
    )]
    pub data_account: Account<'info, DataAccount>,

    #[account(
        init,
        payer = sender,
        seeds=[b"escrow_wallet".as_ref(), token_mint.key().as_ref()],
        bump,
        token::mint=token_mint,
        token::authority=data_account,
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
pub struct DataAccount {
    // Space in bytes: 8 + 1 + 8 + 32 + 32 + 32 + 1 + (4 + (100 * (32 + 8 + 8)))
    pub percent_available: u8, // 1
    pub token_amount: u64,     // 8
    pub initializer: Pubkey,   // 32
    pub escrow_wallet: Pubkey, // 32
    pub token_mint: Pubkey,    // 32
    pub beneficiaries: Vec<Beneficiary>, // (4 + (n * (32 + 8 + 8)))
    pub decimals: u8           // 1
}

#[error_code]
pub enum VestingError {
    #[msg("Sender is not owner of Data Account")]
    InvalidSender,
    #[msg("Not allowed to claim new tokens currently")]
    ClaimNotAllowed,
    #[msg("Beneficiary does not exist in account")]
    BeneficiaryNotFound
}