// vault_manager.rs
// Single-file Solana program for a Vault Manager that:
// - Accepts a single token deposit
// - Auto-swaps 50% to the paired token
// - Adds liquidity to a CLMM pool (Orca Whirlpool)
// - Allows rebalancing (remove, swap, re-add) when out of range
// - Collects earned fees
//
// Uses Anchor framework. Deploy with `anchor build` and `anchor deploy`.
//
// WARNING: This is a simplified but functional example. Audit thoroughly before production use.
// Supports only one pool per vault for simplicity.

#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("7Y8CNTcNiU4s6Aih26rFcFWLrGTka4Kqzd5MajUKhVFx");

#[program]
pub mod vault_manager {
    use super::*;

    /// Initialize a new vault for a specific Whirlpool
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

    /// Deposit a single token (A or B), auto-swap 50%, and add liquidity
    pub fn deposit_and_add_liquidity(
        ctx: Context<DepositAndAddLiquidity>,
        amount: u64,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        require!(!vault.is_active, VaultError::PositionActive);

        // Transfer deposit token to vault
        let token_program = &ctx.accounts.token_program;
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_input.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let input_mint = ctx.accounts.user_token_account.mint;
        let is_deposit_a = input_mint == vault.token_a_mint;

        // Swap 50% of input to the other token
        let swap_amount = amount / 2;
        let swap_cpi = if is_deposit_a {
            // Swap A -> B
            whirlpool::cpi::swap(
                CpiContext::new_with_signer(
                    ctx.accounts.whirlpool_program.to_account_info(),
                    whirlpool::cpi::accounts::Swap {
                        token_program: token_program.to_account_info(),
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
                    &[&[
                        b"vault",
                        vault.whirlpool.as_ref(),
                        &[vault.bump],
                    ]],
                ),
                whirlpool::Swap {
                    amount: swap_amount,
                    other_amount_threshold: 0,
                    sqrt_price_limit: whirlpool::state::SqrtPrice::default(),
                    amount_specified_is_input: true,
                    a_to_b: true,
                    tick_array_0: ctx.accounts.tick_array_0.key(),
                    tick_array_1: ctx.accounts.tick_array_1.key(),
                    tick_array_2: ctx.accounts.tick_array_2.key(),
                },
            )?
        } else {
            // Swap B -> A
            whirlpool::cpi::swap(
                CpiContext::new_with_signer(
                    ctx.accounts.whirlpool_program.to_account_info(),
                    whirlpool::cpi::accounts::Swap {
                        token_program: token_program.to_account_info(),
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
                    &[&[
                        b"vault",
                        vault.whirlpool.as_ref(),
                        &[vault.bump],
                    ]],
                ),
                whirlpool::Swap {
                    amount: swap_amount,
                    other_amount_threshold: 0,
                    sqrt_price_limit: whirlpool::state::SqrtPrice::default(),
                    amount_specified_is_input: true,
                    a_to_b: false,
                    tick_array_0: ctx.accounts.tick_array_0.key(),
                    tick_array_1: ctx.accounts.tick_array_1.key(),
                    tick_array_2: ctx.accounts.tick_array_2.key(),
                },
            )?
        };

        // Open position if not exists
        if ctx.accounts.position.amount == 0 {
            whirlpool::cpi::open_position(
                CpiContext::new_with_signer(
                    ctx.accounts.whirlpool_program.to_account_info(),
                    whirlpool::cpi::accounts::OpenPosition {
                        funder: ctx.accounts.vault.to_account_info(),
                        owner: ctx.accounts.vault.to_account_info(),
                        position: ctx.accounts.position.to_account_info(),
                        position_mint: ctx.accounts.position_mint.to_account_info(),
                        position_token_account: ctx.accounts.position_token_account.to_account_info(),
                        whirlpool: ctx.accounts.whirlpool.to_account_info(),
                        token_program: token_program.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info(),
                        rent: ctx.accounts.rent.to_account_info(),
                    },
                    &[&[
                        b"vault",
                        vault.whirlpool.as_ref(),
                        &[vault.bump],
                    ]],
                ),
                tick_lower,
                tick_upper,
            )?;
        }

        // Add liquidity
        let amount_a = ctx.accounts.vault_token_a.amount;
        let amount_b = ctx.accounts.vault_token_b.amount;

        whirlpool::cpi::increase_liquidity(
            CpiContext::new_with_signer(
                ctx.accounts.whirlpool_program.to_account_info(),
                whirlpool::cpi::accounts::IncreaseLiquidity {
                    whirlpool: ctx.accounts.whirlpool.to_account_info(),
                    position: ctx.accounts.position.to_account_info(),
                    position_token_account: ctx.accounts.position_token_account.to_account_info(),
                    token_owner_account_a: ctx.accounts.vault_token_a.to_account_info(),
                    token_owner_account_b: ctx.accounts.vault_token_b.to_account_info(),
                    token_vault_a: ctx.accounts.whirlpool_token_vault_a.to_account_info(),
                    token_vault_b: ctx.accounts.whirlpool_token_vault_b.to_account_info(),
                    tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
                    tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
                    token_program: token_program.to_account_info(),
                },
                &[&[
                    b"vault",
                    vault.whirlpool.as_ref(),
                    &[vault.bump],
                ]],
            ),
            amount_a.min(amount_b),
            amount_a.min(amount_b),
            0,
            0,
        )?;

        vault.is_active = true;

        Ok(())
    }

    /// Collect earned fees
    pub fn collect_fees(ctx: Context<CollectFees>) -> Result<()> {
        let vault = &ctx.accounts.vault;

        whirlpool::cpi::collect_fees(
            CpiContext::new_with_signer(
                ctx.accounts.whirlpool_program.to_account_info(),
                whirlpool::cpi::accounts::CollectFees {
                    whirlpool: ctx.accounts.whirlpool.to_account_info(),
                    position: ctx.accounts.position.to_account_info(),
                    position_token_account: ctx.accounts.position_token_account.to_account_info(),
                    token_owner_account_a: ctx.accounts.vault_token_a.to_account_info(),
                    token_owner_account_b: ctx.accounts.vault_token_b.to_account_info(),
                    token_vault_a: ctx.accounts.whirlpool_token_vault_a.to_account_info(),
                    token_vault_b: ctx.accounts.whirlpool_token_vault_b.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                &[&[
                    b"vault",
                    vault.whirlpool.as_ref(),
                    &[vault.bump],
                ]],
            ),
        )?;

        Ok(())
    }

    /// Rebalance: remove liquidity, swap to rebalance, re-add in new range
    pub fn rebalance(
        ctx: Context<Rebalance>,
        tick_lower: i32,
        tick_upper: i32,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        require!(vault.is_active, VaultError::NoActivePosition);

        // Collect fees first
        collect_fees(Context::new(
            ctx.program_id,
            &mut CollectFees {
                vault: vault.to_account_info(),
                whirlpool: ctx.accounts.whirlpool.to_account_info(),
                position: ctx.accounts.position.to_account_info(),
                position_token_account: ctx.accounts.position_token_account.to_account_info(),
                vault_token_a: ctx.accounts.vault_token_a.to_account_info(),
                vault_token_b: ctx.accounts.vault_token_b.to_account_info(),
                whirlpool_token_vault_a: ctx.accounts.whirlpool_token_vault_a.to_account_info(),
                whirlpool_token_vault_b: ctx.accounts.whirlpool_token_vault_b.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                whirlpool_program: ctx.accounts.whirlpool_program.to_account_info(),
            },
            &[],
            ctx.bumps.clone(),
        ))?;

        // Remove all liquidity
        let liquidity = ctx.accounts.position.liquidity;
        whirlpool::cpi::decrease_liquidity(
            CpiContext::new_with_signer(
                ctx.accounts.whirlpool_program.to_account_info(),
                whirlpool::cpi::accounts::DecreaseLiquidity {
                    whirlpool: ctx.accounts.whirlpool.to_account_info(),
                    position: ctx.accounts.position.to_account_info(),
                    position_token_account: ctx.accounts.position_token_account.to_account_info(),
                    token_owner_account_a: ctx.accounts.vault_token_a.to_account_info(),
                    token_owner_account_b: ctx.accounts.vault_token_b.to_account_info(),
                    token_vault_a: ctx.accounts.whirlpool_token_vault_a.to_account_info(),
                    token_vault_b: ctx.accounts.whirlpool_token_vault_b.to_account_info(),
                    tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
                    tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                &[&[
                    b"vault",
                    vault.whirlpool.as_ref(),
                    &[vault.bump],
                ]],
            ),
            liquidity,
            0,
            0,
        )?;

        // Swap to 50/50
        let amount_a = ctx.accounts.vault_token_a.amount;
        let amount_b = ctx.accounts.vault_token_b.amount;
        let target = amount_a.max(amount_b);

        if amount_a > amount_b {
            let excess = amount_a - target;
            whirlpool::cpi::swap(
                CpiContext::new_with_signer(
                    ctx.accounts.whirlpool_program.to_account_info(),
                    whirlpool::cpi::accounts::Swap {
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
                    &[&[
                        b"vault",
                        vault.whirlpool.as_ref(),
                        &[vault.bump],
                    ]],
                ),
                whirlpool::Swap {
                    amount: excess,
                    other_amount_threshold: 0,
                    sqrt_price_limit: whirlpool::state::SqrtPrice::default(),
                    amount_specified_is_input: true,
                    a_to_b: true,
                    tick_array_0: ctx.accounts.tick_array_0.key(),
                    tick_array_1: ctx.accounts.tick_array_1.key(),
                    tick_array_2: ctx.accounts.tick_array_2.key(),
                },
            )?;
        } else if amount_b > amount_a {
            let excess = amount_b - target;
            whirlpool::cpi::swap(
                CpiContext::new_with_signer(
                    ctx.accounts.whirlpool_program.to_account_info(),
                    whirlpool::cpi::accounts::Swap {
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
                    &[&[
                        b"vault",
                        vault.whirlpool.as_ref(),
                        &[vault.bump],
                    ]],
                ),
                whirlpool::Swap {
                    amount: excess,
                    other_amount_threshold: 0,
                    sqrt_price_limit: whirlpool::state::SqrtPrice::default(),
                    amount_specified_is_input: true,
                    a_to_b: false,
                    tick_array_0: ctx.accounts.tick_array_0.key(),
                    tick_array_1: ctx.accounts.tick_array_1.key(),
                    tick_array_2: ctx.accounts.tick_array_2.key(),
                },
            )?;
        }

        // Re-add liquidity in new range
        let amount_a = ctx.accounts.vault_token_a.amount;
        let amount_b = ctx.accounts.vault_token_b.amount;

        whirlpool::cpi::increase_liquidity(
            CpiContext::new_with_signer(
                ctx.accounts.whirlpool_program.to_account_info(),
                whirlpool::cpi::accounts::IncreaseLiquidity {
                    whirlpool: ctx.accounts.whirlpool.to_account_info(),
                    position: ctx.accounts.position.to_account_info(),
                    position_token_account: ctx.accounts.position_token_account.to_account_info(),
                    token_owner_account_a: ctx.accounts.vault_token_a.to_account_info(),
                    token_owner_account_b: ctx.accounts.vault_token_b.to_account_info(),
                    token_vault_a: ctx.accounts.whirlpool_token_vault_a.to_account_info(),
                    token_vault_b: ctx.accounts.whirlpool_token_vault_b.to_account_info(),
                    tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
                    tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                &[&[
                    b"vault",
                    vault.whirlpool.as_ref(),
                    &[vault.bump],
                ]],
            ),
            amount_a.min(amount_b),
            amount_a.min(amount_b),
            0,
            0,
        )?;

        Ok(())
    }

    /// Withdraw all liquidity and tokens
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let vault = &ctx.accounts.vault;

        // Collect fees
        collect_fees(Context::new(
            ctx.program_id,
            &mut CollectFees {
                vault: vault.to_account_info(),
                whirlpool: ctx.accounts.whirlpool.to_account_info(),
                position: ctx.accounts.position.to_account_info(),
                position_token_account: ctx.accounts.position_token_account.to_account_info(),
                vault_token_a: ctx.accounts.vault_token_a.to_account_info(),
                vault_token_b: ctx.accounts.vault_token_b.to_account_info(),
                whirlpool_token_vault_a: ctx.accounts.whirlpool_token_vault_a.to_account_info(),
                whirlpool_token_vault_b: ctx.accounts.whirlpool_token_vault_b.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                whirlpool_program: ctx.accounts.whirlpool_program.to_account_info(),
            },
            &[],
            ctx.bumps.clone(),
        ))?;

        // Remove liquidity
        if ctx.accounts.position.liquidity > 0 {
            whirlpool::cpi::decrease_liquidity(
                CpiContext::new_with_signer(
                    ctx.accounts.whirlpool_program.to_account_info(),
                    whirlpool::cpi::accounts::DecreaseLiquidity {
                        whirlpool: ctx.accounts.whirlpool.to_account_info(),
                        position: ctx.accounts.position.to_account_info(),
                        position_token_account: ctx.accounts.position_token_account.to_account_info(),
                        token_owner_account_a: ctx.accounts.vault_token_a.to_account_info(),
                        token_owner_account_b: ctx.accounts.vault_token_b.to_account_info(),
                        token_vault_a: ctx.accounts.whirlpool_token_vault_a.to_account_info(),
                        token_vault_b: ctx.accounts.whirlpool_token_vault_b.to_account_info(),
                        tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
                        tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                    },
                    &[&[
                        b"vault",
                        vault.whirlpool.as_ref(),
                        &[vault.bump],
                    ]],
                ),
                ctx.accounts.position.liquidity,
                0,
                0,
            )?;
        }

        // Transfer all tokens back
        let seeds = &[b"vault", vault.whirlpool.as_ref(), &[vault.bump]];
        let signer = &[&seeds[..]];

        if ctx.accounts.vault_token_a.amount > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.vault_token_a.to_account_info(),
                        to: ctx.accounts.user_token_a.to_account_info(),
                        authority: ctx.accounts.vault.to_account_info(),
                    },
                    signer,
                ),
                ctx.accounts.vault_token_a.amount,
            )?;
        }

        if ctx.accounts.vault_token_b.amount > 0 {
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.vault_token_b.to_account_info(),
                        to: ctx.accounts.user_token_b.to_account_info(),
                        authority: ctx.accounts.vault.to_account_info(),
                    },
                    signer,
                ),
                ctx.accounts.vault_token_b.amount,
            )?;
        }

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 32 + 32 + 32 + 32 + 1 + 1,
        seeds = [b"vault", whirlpool.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub whirlpool: Account<'info, whirlpool::state::Whirlpool>,
    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,
    #[account(mut)]
    pub vault_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub position: Account<'info, whirlpool::state::Position>,
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
    #[account(mut, has_one = vault_token_a, has_one = vault_token_b, has_one = position)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_input: Account<'info, TokenAccount>,
    #[account(mut)]
    pub whirlpool: Account<'info, whirlpool::state::Whirlpool>,
    #[account(mut)]
    pub position: Account<'info, whirlpool::state::Position>,
    #[account(mut)]
    pub position_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub position_mint: Account<'info, Mint>,
    #[account(mut)]
    pub whirlpool_token_vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub whirlpool_token_vault_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub tick_array_0: AccountLoader<'info, whirlpool::state::TickArray>,
    #[account(mut)]
    pub tick_array_1: AccountLoader<'info, whirlpool::state::TickArray>,
    #[account(mut)]
    pub tick_array_2: AccountLoader<'info, whirlpool::state::TickArray>,
    #[account(mut)]
    pub tick_array_lower: AccountLoader<'info, whirlpool::state::TickArray>,
    #[account(mut)]
    pub tick_array_upper: AccountLoader<'info, whirlpool::state::TickArray>,
    pub oracle: Account<'info, whirlpool::state::Oracle>,
    pub token_program: Program<'info, Token>,
    pub whirlpool_program: Program<'info, whirlpool::Whirlpool>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(mut, seeds = [b"vault", whirlpool.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub whirlpool: Account<'info, whirlpool::state::Whirlpool>,
    #[account(mut)]
    pub position: Account<'info, whirlpool::state::Position>,
    #[account(mut)]
    pub position_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub whirlpool_token_vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub whirlpool_token_vault_b: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub whirlpool_program: Program<'info, whirlpool::Whirlpool>,
}

#[derive(Accounts)]
pub struct Rebalance<'info> {
    #[account(mut, seeds = [b"vault", whirlpool.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub whirlpool: Account<'info, whirlpool::state::Whirlpool>,
    #[account(mut)]
    pub position: Account<'info, whirlpool::state::Position>,
    #[account(mut)]
    pub position_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub whirlpool_token_vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub whirlpool_token_vault_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub tick_array_lower: AccountLoader<'info, whirlpool::state::TickArray>,
    #[account(mut)]
    pub tick_array_upper: AccountLoader<'info, whirlpool::state::TickArray>,
    #[account(mut)]
    pub tick_array_0: AccountLoader<'info, whirlpool::state::TickArray>,
    #[account(mut)]
    pub tick_array_1: AccountLoader<'info, whirlpool::state::TickArray>,
    #[account(mut)]
    pub tick_array_2: AccountLoader<'info, whirlpool::state::TickArray>,
    pub oracle: Account<'info, whirlpool::state::Oracle>,
    pub token_program: Program<'info, Token>,
    pub whirlpool_program: Program<'info, whirlpool::Whirlpool>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"vault", whirlpool.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub whirlpool: Account<'info, whirlpool::state::Whirlpool>,
    #[account(mut)]
    pub position: Account<'info, whirlpool::state::Position>,
    #[account(mut)]
    pub position_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub whirlpool_token_vault_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub whirlpool_token_vault_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub tick_array_lower: AccountLoader<'info, whirlpool::state::TickArray>,
    #[account(mut)]
    pub tick_array_upper: AccountLoader<'info, whirlpool::state::TickArray>,
    pub token_program: Program<'info, Token>,
    pub whirlpool_program: Program<'info, whirlpool::Whirlpool>,
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

// Add Whirlpool dependency in Cargo.toml:
// whirlpool = { version = "0.3", features = ["cpi"] }
// anchor-lang = "0.29"
// anchor-spl = "0.29"