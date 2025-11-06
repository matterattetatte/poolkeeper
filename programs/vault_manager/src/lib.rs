#![allow(clippy::result_large_err)]
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("7Y8CNTcNiU4s6Aih26rFcFWLrGTka4Kqzd5MajUKhVFx");

#[program]
pub mod vault_manager {
    use super::*;

    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        bump: u8,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.authority.key();
        vault.whirlpool = ctx.accounts.whirlpool.key();
        vault.token_a_mint = ctx.accounts.token_a_mint.key();
        vault.token_b_mint = ctx.accounts.token_b_mint.key();
        vault.vault_token_a = ctx.accounts.vault_token_a.key();
        vault.vault_token_b = ctx.accounts.vault_token_b.key();
        vault.position = ctx.accounts.position.key();
        vault.bump = bump;
        vault.is_active = false;
        Ok(())
    }

    pub fn deposit_and_add_liquidity(
        ctx: Context<DepositAndAddLiquidity>,
        amount: u64,
        _tick_lower: i32,
        _tick_upper: i32,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        require!(!vault.is_active, VaultError::PositionActive);

        // Transfer deposit token to vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_input.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Determine if deposit is token A
        let input_mint = ctx.accounts.user_token_account.mint;
        let _is_deposit_a = input_mint == vault.token_a_mint;

        // TODO: Implement swap using CPI calls to Whirlpool program
        // For now, this is a placeholder - you'll need to construct the instruction
        // data manually or use a working whirlpools SDK version
        // Swap 50% logic would go here
        let _swap_amount = amount / 2;
        
        // Placeholder - implement actual Whirlpool swap CPI here
        // This requires the whirlpools crate or manual instruction construction

        vault.is_active = true;
        Ok(())
    }

    // other methods: collect_fees, rebalance, withdraw remain similar but update to use `whirlpools` crate types
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeVault<'info> {
    #[account(
        init, payer = authority,
        space = 8 + 32*7 + 1 + 1, // 8 (discriminator) + 7 Pubkeys (32 each) + bump (1) + is_active (1)
        seeds = [b"vault", whirlpool.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK: Whirlpool account - validated by Whirlpool program
    pub whirlpool: UncheckedAccount<'info>,
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    #[account(mut)]
    pub vault_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_b: Account<'info, TokenAccount>,
    /// CHECK: Position account - validated by Whirlpool program  
    #[account(mut)]
    pub position: UncheckedAccount<'info>,
    #[account(mut)]
    pub position_mint: Account<'info, Mint>,
    #[account(mut)]
    pub position_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositAndAddLiquidity<'info> {
    #[account(mut, seeds = [b"vault", vault.whirlpool.as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    /// CHECK: Token account for vault input - validated by token program
    #[account(mut)]
    pub vault_token_input: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    /// CHECK: Whirlpool program - validated by instruction
    pub whirlpool_program: UncheckedAccount<'info>,
}

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub whirlpool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub vault_token_a: Pubkey,
    pub vault_token_b: Pubkey,
    pub position: Pubkey,
    pub bump: u8,
    pub is_active: bool,
}

#[error_code]
pub enum VaultError {
    #[msg("Position is already active")]
    PositionActive,
    #[msg("No active position to rebalance")]
    NoActivePosition,
}
