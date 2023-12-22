use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("DzJ68fvNC6PNfZYQzjSNiyLxgcxHD3nkRGsBAjyGTWyd");

#[program]
pub mod token_vesting {

    use super::*;
    // When creating the token account, the owner should setAuthority for the amount of tokens to be vested to this contract's pda
    pub fn initialize(ctx: Context<Initialize>, token_mint: Pubkey) -> Result<()> {
        let account_data = &mut ctx.accounts.account_data_account;
        account_data.percent_available = 0;
        account_data.initializer = *ctx.accounts.initializer.to_account_info().key;
        account_data.token_mint = token_mint;
        Ok(())
    }

    // Owner must create Vesting Account for each Beneficiary
    pub fn create_vesting_account(ctx: Context<CreateVestingAccount>, beneficiary: Pubkey, total_amount: u64) -> Result<()> {
        require!(*ctx.accounts.initializer.to_account_info().key == ctx.accounts.account_data.initializer, VestingError::NotInitializer);

        let vesting_account = &mut ctx.accounts.vesting_account;
        vesting_account.beneficiary = beneficiary;
        vesting_account.total_amount = total_amount;
        vesting_account.claimed_amount = 0;
        Ok(())
    }

    // Owner Releases `percent` % of tokens to all beneficiaries
    pub fn release(ctx: Context<Release>, percent: u8) -> Result<()>{
        require!(*ctx.accounts.sender.to_account_info().key == ctx.accounts.account_data_account.initializer, VestingError::NotInitializer);
        
        ctx.accounts.account_data_account.percent_available = percent;
        Ok(())
    }

    // Beneficiary calls `claim` to claim tokens
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let vesting_account = &mut ctx.accounts.vesting_account;
        let account_data = &mut ctx.accounts.account_data_account;
        require!(*ctx.accounts.sender.to_account_info().key == account_data.initializer, VestingError::NotInitializer);

        let claimable_amount = vesting_account.total_amount * account_data.percent_available as u64/ 100;
        require!(vesting_account.claimed_amount < claimable_amount, VestingError::NoUnclaimedTokens);
        
        let amount_to_claim = claimable_amount - vesting_account.claimed_amount;

        let (pda, bump) = Pubkey::find_program_address(&[b"vesting_pool"], ctx.program_id);
        require!(ctx.accounts.vesting_pool.owner == pda, VestingError::InvalidPoolAuthority);

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.vesting_pool.to_account_info(),
            to: ctx.accounts.to_ata.to_account_info(),
            authority: ctx.accounts.vesting_pool.to_account_info(),
        };

        let signer_seeds: &[&[u8]] = &[b"vesting_pool", &[bump]];
        let signer_slice = &[signer_seeds]; // have to do this b/c the value might be dropped before being used
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_slice);
        
        token::transfer(cpi_context, amount_to_claim)?;

        vesting_account.claimed_amount = amount_to_claim;
        Ok(())
    }
}



#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = initializer, space = 32)] 
    pub account_data_account: Account<'info, AccountData>,

    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,

}

#[derive(Accounts)]
pub struct CreateVestingAccount<'info> {
    #[account(init, payer = initializer, space = 32)] 
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
    pub sender: Signer<'info>,
    #[account(mut)]
    pub vesting_account: Account<'info, VestingAccount>,
    pub account_data_account: Account<'info, AccountData>,

    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub to_ata: Account<'info, TokenAccount>,
    pub vesting_pool: Account<'info, TokenAccount>
}

// #[account]
// #[derive(Default)]
pub struct VestingAccount{
    pub beneficiary: Pubkey,
    pub total_amount: u64,
    pub claimed_amount: u64
}

#[account]
#[derive(Default)]
pub struct AccountData{
    pub percent_available: u8,
    pub token_amount: u64,
    pub initializer: Pubkey,
    pub token_mint: Pubkey,
    pub escrow_wallet: Pubkey,
    pub stage: u8,
    // pub beneficiaries: Vec<Pubkey>,

}

#[error_code]
pub enum VestingError {
    #[msg("Instruction sender is not initializer of contract")]
    NotInitializer,
    #[msg("Wrong token account")]
    InvalidTokenAccount,
    #[msg("No unclaimed tokens left")]
    NoUnclaimedTokens,
    #[msg("PDA does not have authority over pool")]
    InvalidPoolAuthority
}
