use crate::utils::{self, pda};
use anchor_lang::{prelude::Pubkey, InstructionData, ToAccountMetas};
use bonfida_test_utils::ProgramTestContextExt;
use perpetuals::{instructions::SwapParams, state::custody::Custody};
use solana_program_test::ProgramTestContext;
use solana_sdk::signer::{keypair::Keypair, Signer};

pub async fn test_swap(
    program_test_ctx: &mut ProgramTestContext,
    owner: &Keypair,
    payer: &Keypair,
    pool_pda: &Pubkey,
    // Mint received by the User
    dispensing_custody_token_mint: &Pubkey,
    // Mint sent by the User
    receiving_custody_token_mint: &Pubkey,
    params: SwapParams,
) {
    // ==== WHEN ==============================================================
    // Prepare PDA and addresses
    let transfer_authority_pda = pda::get_transfer_authority_pda().0;
    let perpetuals_pda = pda::get_perpetuals_pda().0;
    let dispensing_custody_pda = pda::get_custody_pda(pool_pda, dispensing_custody_token_mint).0;
    let dispensing_custody_token_account_pda =
        pda::get_custody_token_account_pda(pool_pda, dispensing_custody_token_mint).0;
    let receiving_custody_pda = pda::get_custody_pda(pool_pda, receiving_custody_token_mint).0;
    let receiving_custody_token_account_pda =
        pda::get_custody_token_account_pda(pool_pda, receiving_custody_token_mint).0;

    let funding_account_address =
        utils::find_associated_token_account(&owner.pubkey(), receiving_custody_token_mint).0;
    let receiving_account_address =
        utils::find_associated_token_account(&owner.pubkey(), dispensing_custody_token_mint).0;

    let dispensing_custody_account =
        utils::get_account::<Custody>(program_test_ctx, dispensing_custody_pda).await;
    let dispensing_custody_oracle_account_address =
        dispensing_custody_account.oracle.oracle_account;

    let receiving_custody_account =
        utils::get_account::<Custody>(program_test_ctx, receiving_custody_pda).await;
    let receiving_custody_oracle_account_address = receiving_custody_account.oracle.oracle_account;

    // Save account state before tx execution
    let owner_funding_account_before = program_test_ctx
        .get_token_account(funding_account_address)
        .await
        .unwrap();
    let custody_receiving_account_before = program_test_ctx
        .get_token_account(receiving_account_address)
        .await
        .unwrap();

    let accounts_meta = {
        let accounts = perpetuals::accounts::Swap {
            owner: owner.pubkey(),
            funding_account: funding_account_address,
            receiving_account: receiving_account_address,
            transfer_authority: transfer_authority_pda,
            perpetuals: perpetuals_pda,
            pool: *pool_pda,
            receiving_custody: receiving_custody_pda,
            receiving_custody_oracle_account: receiving_custody_oracle_account_address,
            receiving_custody_token_account: receiving_custody_token_account_pda,
            dispensing_custody: dispensing_custody_pda,
            dispensing_custody_oracle_account: dispensing_custody_oracle_account_address,
            dispensing_custody_token_account: dispensing_custody_token_account_pda,
            token_program: anchor_spl::token::ID,
        };

        accounts.to_account_metas(None)
    };

    let arguments = perpetuals::instruction::Swap { params };

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
    let owner_funding_account_after = program_test_ctx
        .get_token_account(funding_account_address)
        .await
        .unwrap();
    let custody_receiving_account_after = program_test_ctx
        .get_token_account(receiving_account_address)
        .await
        .unwrap();

    assert!(owner_funding_account_after.amount < owner_funding_account_before.amount);
    assert!(custody_receiving_account_after.amount > custody_receiving_account_before.amount);
}
