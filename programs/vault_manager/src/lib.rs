#![allow(clippy::result_large_err)]
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

// Replace with your actual published program ID
declare_id!("Vault1111111111111111111111111111111111111111");

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
        tick_lower: i32,
        tick_upper: i32,
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
        let is_deposit_a = input_mint == vault.token_a_mint;

        // Swap 50% (assuming you import whirlpools crate correctly)
        let swap_amount = amount / 2;
        if is_deposit_a {
            whirlpools::cpi::swap(
                CpiContext::new_with_signer(
                    ctx.accounts.whirlpool_program.to_account_info(),
                    whirlpools::cpi::accounts::Swap {
                        token_program: ctx.accounts.token_program.to_account_info(),
                        token_authority: ctx.accounts.vault.to_account_info(),
                        whirlpool: ctx.accounts.whirlpool.to_account_info(),
                        token_owner_account_a: ctx.accounts.vault_token_a.to_account_info(),
                        token_vault_a: ctx.accounts.whirlpool_token_vault_a.to_account_info(),
                        token_owner_account_b: ctx.accounts.vault_token_b.to_account_info(),
                        token_vault_b: ctx.accounts.whirlpool_token_vault_b.to_account_info(),
                        tick_array_0: ctx.accounts.tick_array_0.to_account_info(),
                        tick_array_1: ctx.accounts.tick_array_1.to_account_info(),
                        tick_array_2: ctx.accounts.tick_array_2.to_account_info(),
                        oracle: ctx.accounts.oracle.to_account_info(),
                    },
                    &[&[ b"vault", vault.whirlpool.as_ref(), &[vault.bump] ]],
                ),
                whirlpools::instruction::Swap {
                    amount: swap_amount,
                    other_amount_threshold: 0,
                    sqrt_price_limit: whirlpools::state::SqrtPrice::default(),
                    amount_specified_is_input: true,
                    a_to_b: true,
                    tick_array_0: ctx.accounts.tick_array_0.key(),
                    tick_array_1: ctx.accounts.tick_array_1.key(),
                    tick_array_2: ctx.accounts.tick_array_2.key(),
                },
            )?;
        } else {
            whirlpools::cpi::swap(
                CpiContext::new_with_signer(
                    ctx.accounts.whirlpool_program.to_account_info(),
                    whirlpools::cpi::accounts::Swap {
                        token_program: ctx.accounts.token_program.to_account_info(),
                        token_authority: ctx.accounts.vault.to_account_info(),
                        whirlpool: ctx.accounts.whirlpool.to_account_info(),
                        token_owner_account_a: ctx.accounts.vault_token_a.to_account_info(),
                        token_vault_a: ctx.accounts.whirlpool_token_vault_a.to_account_info(),
                        token_owner_account_b: ctx.accounts.vault_token_b.to_account_info(),
                        token_vault_b: ctx.accounts.whirlpool_token_vault_b.to_account_info(),
                        tick_array_0: ctx.accounts.tick_array_0.to_account_info(),
                        tick_array_1: ctx.accounts.tick_array_1.to_account_info(),
                        tick_array_2: ctx.accounts.tick_array_2.to_account_info(),
                        oracle: ctx.accounts.oracle.to_account_info(),
                    },
                    &[&[ b"vault", vault.whirlpool.as_ref(), &[vault.bump] ]],
                ),
                whirlpools::instruction::Swap {
                    amount: swap_amount,
                    other_amount_threshold: 0,
                    sqrt_price_limit: whirlpools::state::SqrtPrice::default(),
                    amount_specified_is_input: true,
                    a_to_b: false,
                    tick_array_0: ctx.accounts.tick_array_0.key(),
                    tick_array_1: ctx.accounts.tick_array_1.key(),
                    tick_array_2: ctx.accounts.tick_array_2.key(),
                },
            )?;
        }

        // … the rest of your add-liquidity logic …
        // (increase_liquidity, open_position, etc similarly updated to use whirlpools crate)

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
        space = 8 + 32*6 + 1 + 1, // simpler expression
        seeds = [b"vault", whirlpool.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub whirlpool: Account<'info, whirlpools::state::Whirlpool>,
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    #[account(mut)]
    pub vault_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub position: Account<'info, whirlpools::state::Position>,
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
