use crate::utils::{find_associated_token_account, get_account, pda};
use anchor_lang::{prelude::Pubkey, InstructionData, ToAccountMetas};
use bonfida_test_utils::ProgramTestContextExt;
use perpetuals::{
    instructions::OpenPositionParams,
    state::{custody::Custody, position::Position, perpetuals::Perpetuals},
};
use solana_program_test::ProgramTestContext;
use solana_sdk::signer::{keypair::Keypair, Signer};

pub async fn test_open_position(
    program_test_ctx: &mut ProgramTestContext,
    owner: &Keypair,
    payer: &Keypair,
    pool_pda: &Pubkey,
    custody_token_mint: &Pubkey,
    params: OpenPositionParams,
) {
    // ==== WHEN ==============================================================

    // Prepare PDA and addresses
    let transfer_authority_pda = pda::get_transfer_authority_pda().0;
    let perpetuals_pda = pda::get_perpetuals_pda().0;
    let custody_pda = pda::get_custody_pda(pool_pda, custody_token_mint).0;
    let custody_token_account_pda =
        pda::get_custody_token_account_pda(pool_pda, custody_token_mint).0;

    let (position_pda, position_bump) =
        pda::get_position_pda(&owner.pubkey(), pool_pda, &custody_pda, params.side);

    let funding_account_address =
        find_associated_token_account(&owner.pubkey(), custody_token_mint).0;

    let custody_account = get_account::<Custody>(program_test_ctx, custody_pda).await;
    let custody_oracle_account_address = custody_account.oracle.oracle_account;

    // Save account state before tx execution
    let owner_funding_account_before = program_test_ctx
        .get_token_account(funding_account_address)
        .await
        .unwrap();
    let custody_token_account_before = program_test_ctx
        .get_token_account(custody_token_account_pda)
        .await
        .unwrap();

    let accounts_meta = {
        let accounts = perpetuals::accounts::OpenPosition {
            owner: owner.pubkey(),
            funding_account: funding_account_address,
            transfer_authority: transfer_authority_pda,
            perpetuals: perpetuals_pda,
            pool: *pool_pda,
            position: position_pda,
            custody: custody_pda,
            custody_oracle_account: custody_oracle_account_address,
            custody_token_account: custody_token_account_pda,
            system_program: anchor_lang::system_program::ID,
            token_program: anchor_spl::token::ID,
        };

        accounts.to_account_metas(None)
    };

    let arguments = perpetuals::instruction::OpenPosition { params };

    let ix = solana_sdk::instruction::Instruction {
        program_id: perpetuals::id(),
        accounts: accounts_meta,
        data: arguments.data(),
    };

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[owner, payer],
        program_test_ctx.last_blockhash,
    );

    program_test_ctx
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // ==== THEN ==============================================================
    // Check the balance change
    {
        let owner_funding_account_after = program_test_ctx
            .get_token_account(funding_account_address)
            .await
            .unwrap();
        let custody_token_account_after = program_test_ctx
            .get_token_account(custody_token_account_pda)
            .await
            .unwrap();

        assert!(owner_funding_account_after.amount < owner_funding_account_before.amount);
        assert!(custody_token_account_after.amount > custody_token_account_before.amount);
    }

    // Check the position
    {
        let custody_account = get_account::<Custody>(program_test_ctx, custody_pda).await;
        let position_account = get_account::<Position>(program_test_ctx, position_pda).await;
        let perpetuals_account = get_account::<Perpetuals>(program_test_ctx, perpetuals_pda).await;

        assert_eq!(position_account.owner, owner.pubkey());
        assert_eq!(position_account.pool, *pool_pda);
        assert_eq!(position_account.custody, custody_pda);
        assert_eq!(position_account.open_time, perpetuals_account.inception_time);
        assert_eq!(position_account.update_time, 0);
        assert_eq!(position_account.side, params.side);
        assert_eq!(position_account.unrealized_profit_usd, 0);
        assert_eq!(position_account.unrealized_loss_usd, 0);
        assert_eq!(position_account.borrow_rate_sum, custody_account.borrow_rate_sum);
        assert_eq!(position_account.collateral_amount, params.collateral);
        assert_eq!(position_account.bump, position_bump);
    }
}
