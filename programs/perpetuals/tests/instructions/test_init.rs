use anchor_lang::{prelude::Pubkey, InstructionData, ToAccountMetas};
use perpetuals::instructions::InitParams;
use solana_program_test::ProgramTestContext;
use solana_sdk::signer::{keypair::Keypair, Signer};

use crate::pda;

#[allow(unaligned_references)]
pub async fn test_init(
    program_test_ctx: &mut ProgramTestContext,
    upgrade_authority: &Keypair,
    multisig: &Pubkey,
    transfer_authority: &Pubkey,
    perpetuals: &Pubkey,
    perpetuals_program: &Pubkey,
    params: InitParams,
) {
    // ==== WHEN ==============================================================
    let (perpetuals_program_data, _) = pda::get_program_buffer_pda();
    let accounts = perpetuals::accounts::Init {
        upgrade_authority: upgrade_authority.pubkey(),
        multisig: *multisig,
        transfer_authority: *transfer_authority,
        perpetuals: *perpetuals,
        perpetuals_program: *perpetuals_program,
        perpetuals_program_data: perpetuals_program_data,
        system_program: anchor_lang::system_program::ID,
        token_program: anchor_spl::token::ID,
    };

    let arguments = perpetuals::instruction::Init { params };

    let ix = solana_sdk::instruction::Instruction {
        program_id: perpetuals::id(),
        accounts: accounts.to_account_metas(None),
        data: arguments.data(),
    };

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        Some(&upgrade_authority.pubkey()),
        &[upgrade_authority],
        program_test_ctx.last_blockhash,
    );
    program_test_ctx
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // ==== THEN ==============================================================
    // let cortex_account = program_test_ctx
    //     .banks_client
    //     .get_account(cortex_pda)
    //     .await
    //     .unwrap()
    //     .unwrap();
    // let cortex_account_data =
    //     adrena::state::Cortex::try_deserialize(&mut cortex_account.data.as_slice()).unwrap();
    // assert_eq!(cortex_account_data.bump, cortex_bump);
    // assert_eq!(
    //     cortex_account_data.fee_token_account_bump,
    //     fee_token_account_bump
    // );
    // assert_eq!(cortex_account_data.lp_token_mint_bump, lp_token_mint_bump);
    // assert_eq!(cortex_account_data.authority, authority.pubkey());
    // assert_eq!(cortex_account_data.fee_token_account, fee_token_account_pda);
    // assert_eq!(cortex_account_data.lp_token_mint, lp_token_mint_pda);
    // assert_eq!(cortex_account_data.lp_token_mint_decimals, lp_mint_decimals);
    // // vaults
    // let vault_count = cortex_account_data.vaults_count;
    // assert_eq!(vault_count, DEFAULT_VAULT_COUNT);
    // // Vaults uninitialized content unchecked
    // // cache
    // let default_cache = CortexCache::default();
    // let cache = cortex_account_data.cache;
    // let fee_token_usd_price = cache.fee_token_usd_price;
    // let default_fee_token_usd_price = default_cache.fee_token_usd_price;
    // assert_eq!(fee_token_usd_price, default_fee_token_usd_price);
    // let last_update = cache.last_update;
    // let default_last_update = default_cache.last_update;
    // assert_eq!(last_update, default_last_update);
    // // metrics
    // let default_metrics = CortexMetrics::default();
    // let metrics = cortex_account_data.metrics;
    // let fee_generated = metrics.fee_generated;
    // let default_fee_generated = default_metrics.fee_generated;
    // assert_eq!(fee_generated, default_fee_generated);
    // let lp_token_supply = metrics.lp_token_supply;
    // let default_lp_token_supply = default_metrics.lp_token_supply;
    // assert_eq!(lp_token_supply, default_lp_token_supply);
    // let usd_cumulative_volume = metrics.usd_cumulative_volume;
    // let default_usd_cumulative_volume = default_metrics.usd_cumulative_volume;
    // assert_eq!(usd_cumulative_volume, default_usd_cumulative_volume);
}
