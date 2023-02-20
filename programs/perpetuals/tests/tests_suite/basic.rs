use crate::{
    instructions::{
        test_add_custody, test_add_liquidity, test_add_pool, test_init::test_init,
        test_set_test_oracle_price,
    },
    utils::{
        add_perpetuals_program, create_and_fund_multiple_accounts, find_associated_token_account,
        get_current_unix_timestamp, get_test_oracle_account,
    },
};
use anchor_lang::prelude::Pubkey;
use bonfida_test_utils::{ProgramTestContextExt, ProgramTestExt};
use perpetuals::{
    instructions::{AddCustodyParams, AddLiquidityParams, InitParams, SetTestOraclePriceParams},
    state::{
        custody::{Fees, FeesMode, OracleParams, PricingParams},
        oracle::OracleType,
        perpetuals::Permissions,
    },
};
use solana_program_test::ProgramTest;
use solana_sdk::signer::Signer;

// =============================================
// ===> Keypair details
// =============================================
const ROOT_AUTHORITY: usize = 0;
const PERPETUALS_UPGRADE_AUTHORITY: usize = 1;
const MULTISIG_MEMBER_A: usize = 2;
const MULTISIG_MEMBER_B: usize = 3;
const MULTISIG_MEMBER_C: usize = 4;
const PAYER: usize = 5;
const USER_ALICE: usize = 6;
// ==============================================
const KEYPAIRS_COUNT: usize = 7;
// ==============================================

pub async fn basic_test_suite() {
    // ======================================================================
    // ====> Setup
    // ======================================================================
    let mut program_test = ProgramTest::default();

    // Initialize the accounts that will be used during the test suite
    let keypairs = create_and_fund_multiple_accounts(&mut program_test, KEYPAIRS_COUNT).await;

    // Initialize the mints
    let usdc_mint = program_test
        .add_mint(None, 6, &keypairs[ROOT_AUTHORITY].pubkey())
        .0;

    // Deploy the perpetuals program onchain as upgradeable program
    add_perpetuals_program(&mut program_test, &keypairs[PERPETUALS_UPGRADE_AUTHORITY]).await;

    // Start the client and connect to localnet validator
    let mut program_test_ctx = program_test.start_with_context().await;

    // Initialize Token Accounts
    let keys = keypairs.iter().map(|k| k.pubkey()).collect::<Vec<Pubkey>>();
    let usdc_token_accounts = program_test_ctx
        .initialize_token_accounts(usdc_mint, &keys)
        .await
        .unwrap();
    // let usdc_token_accounts = program_test_ctx.initialize_token_accounts(usdc_mint, &keys); // do the same with other mints

    // ======================================================================
    // ====> Run
    // ======================================================================
    let upgrade_authority = &keypairs[PERPETUALS_UPGRADE_AUTHORITY];

    let multisig_signers = &[
        &keypairs[MULTISIG_MEMBER_A],
        &keypairs[MULTISIG_MEMBER_B],
        &keypairs[MULTISIG_MEMBER_C],
    ];

    {
        let init_params = InitParams {
            min_signatures: 1,
            allow_swap: true,
            allow_add_liquidity: true,
            allow_remove_liquidity: true,
            allow_open_position: true,
            allow_close_position: true,
            allow_pnl_withdrawal: true,
            allow_collateral_withdrawal: true,
            allow_size_change: true,
        };

        test_init(
            &mut program_test_ctx,
            upgrade_authority,
            init_params,
            multisig_signers,
        )
        .await;
    }

    let pool_admin = &keypairs[MULTISIG_MEMBER_A];

    let (pool_pda, _, _lp_token_mint_pda, _) = test_add_pool(
        &mut program_test_ctx,
        pool_admin,
        &keypairs[PAYER],
        "FOO POOL",
        multisig_signers,
    )
    .await;

    // Get USDC test oracle address
    let usdc_test_oracle_pda = get_test_oracle_account(&pool_pda, &usdc_mint).0;

    let usdc_custody_pda = {
        let add_custody_params = AddCustodyParams {
            is_stable: true,
            oracle: OracleParams {
                oracle_account: usdc_test_oracle_pda,
                oracle_type: OracleType::Test,
                max_price_error: 1_000_000,
                max_price_age_sec: 30,
            },
            pricing: PricingParams {
                use_ema: false,
                trade_spread_long: 100,
                trade_spread_short: 100,
                swap_spread: 200,
                min_initial_leverage: 10_000,
                max_leverage: 1_000_000,
            },
            permissions: Permissions {
                allow_swap: true,
                allow_add_liquidity: true,
                allow_remove_liquidity: true,
                allow_open_position: true,
                allow_close_position: true,
                allow_pnl_withdrawal: true,
                allow_collateral_withdrawal: true,
                allow_size_change: true,
            },
            fees: Fees {
                mode: FeesMode::Linear,
                max_increase: 20_000,
                max_decrease: 5_000,
                swap: 100,
                add_liquidity: 100,
                remove_liquidity: 100,
                open_position: 100,
                close_position: 100,
                liquidation: 100,
                protocol_share: 10,
            },
            target_ratio: 50,
            min_ratio: 25,
            max_ratio: 75,
        };

        let usdc_custody_pda = test_add_custody(
            &mut program_test_ctx,
            pool_admin,
            &keypairs[PAYER],
            &pool_pda,
            &usdc_mint,
            6,
            add_custody_params,
            multisig_signers,
        )
        .await
        .0;

        usdc_custody_pda
    };

    let usdc_oracle_test_admin = &keypairs[MULTISIG_MEMBER_A];

    let publish_time = get_current_unix_timestamp(&mut program_test_ctx).await;

    // Price set as 1 +- 0.01
    test_set_test_oracle_price(
        &mut program_test_ctx,
        usdc_oracle_test_admin,
        &keypairs[PAYER],
        &pool_pda,
        &usdc_custody_pda,
        &usdc_test_oracle_pda,
        SetTestOraclePriceParams {
            price: 1_000_000,
            expo: 6,
            conf: 10_000,
            publish_time,
        },
        multisig_signers,
    )
    .await;

    // Mint 10 USDC to Alice
    program_test_ctx
        .mint_tokens(
            &keypairs[ROOT_AUTHORITY],
            &usdc_mint,
            &usdc_token_accounts[USER_ALICE],
            10_000_000,
        )
        .await
        .unwrap();

    // Add 1 USDC liquity
    test_add_liquidity(
        &mut program_test_ctx,
        &keypairs[USER_ALICE],
        &keypairs[PAYER],
        &pool_pda,
        &usdc_mint,
        AddLiquidityParams { amount: 1_000_000 },
    )
    .await;
}
