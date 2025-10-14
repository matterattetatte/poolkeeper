use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("YourProgramIdHere");  // Replace with your deployed program ID

#[program]
pub mod solana_sickle_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, bump: u8) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.authority = ctx.accounts.user.key();
        vault.token_mint = ctx.accounts.token_mint.key();
        vault.bump = bump;
        vault.lp_position = Pubkey::default();  // Placeholder for LP token account or position
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Basic transfer to vault's token account
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;
        // TODO: Call swap and add to LP here (see extensions below)
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        // TODO: Remove from LP, swap back if needed
        // Sign with PDA seeds for program-controlled transfer
        let seeds = &[b"vault".as_ref(), ctx.accounts.user.key().as_ref(), &[ctx.accounts.vault.bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.vault.to_account_info(),
                },
                &[seeds],
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn rebalance(ctx: Context<Rebalance>) -> Result<()> {
        // TODO: Implement rebalancing logic (see extensions)
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 32 + 32 + 1 + 32, seeds = [b"vault", user.key().as_ref()], bump)]
    pub vault: Account<'info, Vault>,
    pub token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds = [b"vault", user.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut, token::mint = vault.token_mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = vault.token_mint, token::authority = vault)]  // Program-owned
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"vault", user.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut, token::mint = vault.token_mint, token::authority = user)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut, token::mint = vault.token_mint, token::authority = vault)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Rebalance<'info> {
    #[account(mut, seeds = [b"vault", authority.key().as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub authority: Signer<'info>,
    // Add accounts for swap/LP programs here
}

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub bump: u8,
    pub lp_position: Pubkey,  // Track LP position or farm stake
}