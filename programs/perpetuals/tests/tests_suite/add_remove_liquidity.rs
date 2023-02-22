use crate::{instructions, utils};
use bonfida_test_utils::{ProgramTestContextExt, ProgramTestExt};
use perpetuals::{
    instructions::{
        AddCustodyParams, AddLiquidityParams, ClosePositionParams, InitParams, OpenPositionParams,
        RemoveLiquidityParams, SetTestOraclePriceParams, SwapParams,
    },
    state::{
        custody::{Fees, FeesMode, OracleParams, PricingParams},
        oracle::OracleType,
        perpetuals::Permissions,
        position::Side,
    },
};
use solana_program_test::ProgramTest;
use solana_sdk::signer::Signer;

const ROOT_AUTHORITY: usize = 0;
const PERPETUALS_UPGRADE_AUTHORITY: usize = 1;
const MULTISIG_MEMBER_A: usize = 2;
const MULTISIG_MEMBER_B: usize = 3;
const MULTISIG_MEMBER_C: usize = 4;
const PAYER: usize = 5;
const USER_ALICE: usize = 6;
const USER_MARTIN: usize = 7;
const USER_PAUL: usize = 8;

const KEYPAIRS_COUNT: usize = 9;

pub async fn add_remove_liquidity_test_suite() {
    let mut program_test = ProgramTest::default();

    // Initialize the accounts that will be used during the test suite
    let keypairs =
        utils::create_and_fund_multiple_accounts(&mut program_test, KEYPAIRS_COUNT).await;

    // Initialize mints
    let usdc_mint = program_test
        .add_mint(None, 6, &keypairs[ROOT_AUTHORITY].pubkey())
        .0;

    // Deploy the perpetuals program onchain as upgradeable program
    utils::add_perpetuals_program(&mut program_test, &keypairs[PERPETUALS_UPGRADE_AUTHORITY]).await;

    // Start the client and connect to localnet validator
    let mut program_test_ctx = program_test.start_with_context().await;

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

        instructions::test_init(
            &mut program_test_ctx,
            upgrade_authority,
            init_params,
            multisig_signers,
        )
        .await;
    }

    let pool_admin = &keypairs[MULTISIG_MEMBER_A];

    let (pool_pda, _, lp_token_mint_pda, _) = instructions::test_add_pool(
        &mut program_test_ctx,
        pool_admin,
        &keypairs[PAYER],
        "FOO POOL",
        multisig_signers,
    )
    .await;

    let usdc_test_oracle_pda = utils::get_test_oracle_account(&pool_pda, &usdc_mint).0;

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
                max_payoff_mult: 10,
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
            // TODO: explain
            target_ratio: 10_000,
            min_ratio: 0,
            max_ratio: 1_000_000,
        };

        let usdc_custody_pda = instructions::test_add_custody(
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

    let publish_time = utils::get_current_unix_timestamp(&mut program_test_ctx).await;

    // Price set as 1 +- 0.01
    instructions::test_set_test_oracle_price(
        &mut program_test_ctx,
        usdc_oracle_test_admin,
        &keypairs[PAYER],
        &pool_pda,
        &usdc_custody_pda,
        &usdc_test_oracle_pda,
        SetTestOraclePriceParams {
            price: 1_000_000,
            expo: -6,
            conf: 10_000,
            publish_time,
        },
        multisig_signers,
    )
    .await;

    // Alice: Initialize usdc and lp token associated token accounts
    let alice_usdc_token_account = utils::initialize_token_account(
        &mut program_test_ctx,
        &usdc_mint,
        &keypairs[USER_ALICE].pubkey(),
    )
    .await;

    let alice_lp_token_account = utils::initialize_token_account(
        &mut program_test_ctx,
        &lp_token_mint_pda,
        &keypairs[USER_ALICE].pubkey(),
    )
    .await;

    // Alice: Mint 5k USDC
    utils::mint_tokens(
        &mut program_test_ctx,
        &keypairs[ROOT_AUTHORITY],
        &usdc_mint,
        &alice_usdc_token_account,
        5_000_000_000,
    )
    .await;

    // Alice: Add 1k USDC liquidity
    instructions::test_add_liquidity(
        &mut program_test_ctx,
        &keypairs[USER_ALICE],
        &keypairs[PAYER],
        &pool_pda,
        &usdc_mint,
        AddLiquidityParams {
            amount: 1_000_000_000,
        },
    )
    .await;

    // Alice: Add 2k USDC liquidity
    instructions::test_add_liquidity(
        &mut program_test_ctx,
        &keypairs[USER_ALICE],
        &keypairs[PAYER],
        &pool_pda,
        &usdc_mint,
        AddLiquidityParams {
            amount: 2_000_000_000,
        },
    )
    .await;

    // Alice: Add 500 USDC liquidity
    instructions::test_add_liquidity(
        &mut program_test_ctx,
        &keypairs[USER_ALICE],
        &keypairs[PAYER],
        &pool_pda,
        &usdc_mint,
        AddLiquidityParams {
            amount: 500_000_000,
        },
    )
    .await;

    // Usdc: Price set as 1.01 +- 0.01 (price increase of 1%)
    instructions::test_set_test_oracle_price(
        &mut program_test_ctx,
        usdc_oracle_test_admin,
        &keypairs[PAYER],
        &pool_pda,
        &usdc_custody_pda,
        &usdc_test_oracle_pda,
        SetTestOraclePriceParams {
            price: 1_010_000,
            expo: -6,
            conf: 10_000,
            publish_time,
        },
        multisig_signers,
    )
    .await;

    let alice_lp_token_balance = program_test_ctx
        .get_token_account(alice_lp_token_account)
        .await
        .unwrap()
        .amount;

    // Alice: Remove 100% of provided liquidity (1k USDC less fees)
    instructions::test_remove_liquidity(
        &mut program_test_ctx,
        &keypairs[USER_ALICE],
        &keypairs[PAYER],
        &pool_pda,
        &usdc_mint,
        RemoveLiquidityParams {
            lp_amount: alice_lp_token_balance,
        },
    )
    .await;
}
