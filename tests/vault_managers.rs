#![cfg(feature = "test-bpf")]

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
    transport::TransportError,
};
use vault_manager::{self, instruction::*, *};
use whirlpool::{
    self,
    state::{TickArray, Whirlpool},
    manager::WhirlpoolContext,
    test::WhirlpoolTestFixture,
};

#[tokio::test]
async fn test_full_vault_lifecycle() -> Result<(), TransportError> {
    let mut program_test = ProgramTest::new(
        "vault_manager",
        vault_manager::ID,
        processor!(vault_manager::entry),
    );

    // Add Whirlpool program
    program_test.add_program("whirlpool", whirlpool::ID, None);

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // === 1. Setup mints and token accounts ===
    let token_a_mint = Mint::new(&mut banks_client, &payer).await?;
    let token_b_mint = Mint::new(&mut banks_client, &payer).await?;

    let user_token_a = TokenAccount::new(
        &mut banks_client,
        &payer,
        &token_a_mint.pubkey,
        &payer.pubkey(),
    )
    .await?;
    let user_token_b = TokenAccount::new(
        &mut banks_client,
        &payer,
        &token_b_mint.pubkey,
        &payer.pubkey(),
    )
    .await?;

    // Mint initial tokens to user
    token_a_mint
        .mint_to(&mut banks_client, &payer, &user_token_a.pubkey, 1_000_000_000)
        .await?;
    token_b_mint
        .mint_to(&mut banks_client, &payer, &user_token_b.pubkey, 1_000_000_000)
        .await?;

    // === 2. Setup Whirlpool (mock) ===
    let mut fixture = WhirlpoolTestFixture::new(&mut banks_client, &payer).await?;
    let whirlpool = fixture.create_whirlpool(
        &token_a_mint.pubkey,
        &token_b_mint.pubkey,
        3000, // fee tier 0.3%
        0,    // tick spacing
    ).await?;

    // Get vault PDAs
    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[b"vault", whirlpool.pubkey.as_ref()],
        &vault_manager::ID,
    );

    let (position_pda, _pos_bump) = Pubkey::find_program_address(
        &[b"position", vault_pda.as_ref()],
        &whirlpool::ID,
    );

    let position_mint = Keypair::new();
    let position_token_account = TokenAccount::new(
        &mut banks_client,
        &payer,
        &position_mint.pubkey(),
        &vault_pda,
    )
    .await?;

    // Create vault token accounts (owned by vault PDA)
    let vault_token_a = TokenAccount::new(
        &mut banks_client,
        &payer,
        &token_a_mint.pubkey,
        &vault_pda,
    )
    .await?;
    let vault_token_b = TokenAccount::new(
        &mut banks_client,
        &payer,
        &token_b_mint.pubkey,
        &vault_pda,
    )
    .await?;

    // === 3. Initialize Vault ===
    let ix_init = Instruction {
        program_id: vault_manager::ID,
        accounts: vault_manager::accounts::InitializeVault {
            vault: vault_pda,
            whirlpool: whirlpool.pubkey,
            token_a_mint: token_a_mint.pubkey,
            token_b_mint: token_b_mint.pubkey,
            vault_token_a: vault_token_a.pubkey,
            vault_token_b: vault_token_b.pubkey,
            position: position_pda,
            position_mint: position_mint.pubkey(),
            position_token_account: position_token_account.pubkey,
            authority: payer.pubkey(),
            system_program: System::id(),
            token_program: Token::id(),
            rent: Rent::id(),
        }
        .to_account_metas(None),
        data: vault_manager::instruction::InitializeVault { bump: vault_bump }.data(),
    };

    let mut tx = Transaction::new_with_payer(&[ix_init], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(tx).await?;

    // === 4. Deposit A, Swap 50%, Add Liquidity ===
    let tick_lower = -1000;
    let tick_upper = 1000;

    let deposit_amount = 100_000_000; // 100 tokens

    let ix_deposit = Instruction {
        program_id: vault_manager::ID,
        accounts: vault_manager::accounts::DepositAndAddLiquidity {
            vault: vault_pda,
            user: payer.pubkey(),
            user_token_account: user_token_a.pubkey,
            vault_token_a: vault_token_a.pubkey,
            vault_token_b: vault_token_b.pubkey,
            vault_token_input: vault_token_a.pubkey, // same as vault_token_a when depositing A
            whirlpool: whirlpool.pubkey,
            position: position_pda,
            position_token_account: position_token_account.pubkey,
            position_mint: position_mint.pubkey(),
            whirlpool_token_vault_a: whirlpool.token_vault_a,
            whirlpool_token_vault_b: whirlpool.token_vault_b,
            tick_array_0: whirlpool.get_tick_array_pubkey(0),
            tick_array_1: whirlpool.get_tick_array_pubkey(1),
            tick_array_2: whirlpool.get_tick_array_pubkey(2),
            tick_array_lower: whirlpool.get_tick_array_pubkey_for_tick(tick_lower),
            tick_array_upper: whirlpool.get_tick_array_pubkey_for_tick(tick_upper),
            oracle: whirlpool.oracle,
            token_program: Token::id(),
            whirlpool_program: whirlpool::ID,
            system_program: System::id(),
        }
        .to_account_metas(None),
        data: vault_manager::instruction::DepositAndAddLiquidity {
            amount: deposit_amount,
            tick_lower,
            tick_upper,
        }
        .data(),
    };

    let mut tx = Transaction::new_with_payer(&[ix_deposit], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(tx).await?;

    // Verify vault has ~50% in A and ~50% in B
    let vault_a = banks_client.get_account(vault_token_a.pubkey).await?.unwrap();
    let vault_b = banks_client.get_account(vault_token_b.pubkey).await?.unwrap();

    let vault_a_amount = TokenAccount::unpack(&vault_a.data).unwrap().amount;
    let vault_b_amount = TokenAccount::unpack(&vault_b.data).unwrap().amount;

    assert!(vault_a_amount > 0);
    assert!(vault_b_amount > 0);
    assert!(vault_a_amount.abs_diff(vault_b_amount) < deposit_amount / 10); // rough 50/50

    // === 5. Collect Fees (simulate some fee accrual) ===
    // Simulate fee growth by advancing time and swapping externally
    fixture
        .swap(&payer, true, 10_000, 0)
        .await?; // A -> B
    fixture
        .swap(&payer, false, 10_000, 0)
        .await?; // B -> A

    let ix_collect = Instruction {
        program_id: vault_manager::ID,
        accounts: vault_manager::accounts::CollectFees {
            vault: vault_pda,
            whirlpool: whirlpool.pubkey,
            position: position_pda,
            position_token_account: position_token_account.pubkey,
            vault_token_a: vault_token_a.pubkey,
            vault_token_b: vault_token_b.pubkey,
            whirlpool_token_vault_a: whirlpool.token_vault_a,
            whirlpool_token_vault_b: whirlpool.token_vault_b,
            token_program: Token::id(),
            whirlpool_program: whirlpool::ID,
        }
        .to_account_metas(None),
        data: vault_manager::instruction::CollectFees {}.data(),
    };

    let mut tx = Transaction::new_with_payer(&[ix_collect], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(tx).await?;

    // === 6. Rebalance to new range ===
    let new_lower = 500;
    let new_upper = 1500;

    let ix_rebalance = Instruction {
        program_id: vault_manager::ID,
        accounts: vault_manager::accounts::Rebalance {
            vault: vault_pda,
            whirlpool: whirlpool.pubkey,
            position: position_pda,
            position_token_account: position_token_account.pubkey,
            vault_token_a: vault_token_a.pubkey,
            vault_token_b: vault_token_b.pubkey,
            whirlpool_token_vault_a: whirlpool.token_vault_a,
            whirlpool_token_vault_b: whirlpool.token_vault_b,
            tick_array_lower: whirlpool.get_tick_array_pubkey_for_tick(new_lower),
            tick_array_upper: whirlpool.get_tick_array_pubkey_for_tick(new_upper),
            tick_array_0: whirlpool.get_tick_array_pubkey(0),
            tick_array_1: whirlpool.get_tick_array_pubkey(1),
            tick_array_2: whirlpool.get_tick_array_pubkey(2),
            oracle: whirlpool.oracle,
            token_program: Token::id(),
            whirlpool_program: whirlpool::ID,
        }
        .to_account_metas(None),
        data: vault_manager::instruction::Rebalance {
            tick_lower: new_lower,
            tick_upper: new_upper,
        }
        .data(),
    };

    let mut tx = Transaction::new_with_payer(&[ix_rebalance], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(tx).await?;

    // === 7. Withdraw everything ===
    let ix_withdraw = Instruction {
        program_id: vault_manager::ID,
        accounts: vault_manager::accounts::Withdraw {
            vault: vault_pda,
            user: payer.pubkey(),
            whirlpool: whirlpool.pubkey,
            position: position_pda,
            position_token_account: position_token_account.pubkey,
            vault_token_a: vault_token_a.pubkey,
            vault_token_b: vault_token_b.pubkey,
            whirlpool_token_vault_a: whirlpool.token_vault_a,
            whirlpool_token_vault_b: whirlpool.token_vault_b,
            user_token_a: user_token_a.pubkey,
            user_token_b: user_token_b.pubkey,
            tick_array_lower: whirlpool.get_tick_array_pubkey_for_tick(new_lower),
            tick_array_upper: whirlpool.get_tick_array_pubkey_for_tick(new_upper),
            token_program: Token::id(),
            whirlpool_program: whirlpool::ID,
        }
        .to_account_metas(None),
        data: vault_manager::instruction::Withdraw {}.data(),
    };

    let mut tx = Transaction::new_with_payer(&[ix_withdraw], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(tx).await?;

    // Verify user got tokens back
    let final_a = TokenAccount::get(&mut banks_client, user_token_a.pubkey).await?.amount;
    let final_b = TokenAccount::get(&mut banks_client, user_token_b.pubkey).await?.amount;

    assert!(final_a >= 900_000_000); // some loss due to fees/slippage
    assert!(final_b >= 900_000_000);

    Ok(())
}