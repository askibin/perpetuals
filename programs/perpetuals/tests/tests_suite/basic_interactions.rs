use std::ops::{Div, Mul};

use crate::{
    instructions::{
        test_add_custody, test_add_liquidity, test_add_pool, test_close_position,
        test_init::test_init, test_open_position, test_remove_liquidity,
        test_set_test_oracle_price, test_swap,
    },
    utils::{
        add_perpetuals_program, create_and_fund_multiple_accounts, get_current_unix_timestamp,
        get_test_oracle_account, initialize_token_account,
    },
};
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

pub async fn basic_interactions_test_suite() {
    let mut program_test = ProgramTest::default();

    // Initialize the accounts that will be used during the test suite
    let keypairs = create_and_fund_multiple_accounts(&mut program_test, KEYPAIRS_COUNT).await;

    // Initialize mints
    let usdc_mint = program_test
        .add_mint(None, 6, &keypairs[ROOT_AUTHORITY].pubkey())
        .0;
    let eth_mint = program_test
        .add_mint(None, 9, &keypairs[ROOT_AUTHORITY].pubkey())
        .0;

    // Deploy the perpetuals program onchain as upgradeable program
    add_perpetuals_program(&mut program_test, &keypairs[PERPETUALS_UPGRADE_AUTHORITY]).await;

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

        test_init(
            &mut program_test_ctx,
            upgrade_authority,
            init_params,
            multisig_signers,
        )
        .await;
    }

    let pool_admin = &keypairs[MULTISIG_MEMBER_A];

    let (pool_pda, _, lp_token_mint_pda, _) = test_add_pool(
        &mut program_test_ctx,
        pool_admin,
        &keypairs[PAYER],
        "FOO POOL",
        multisig_signers,
    )
    .await;

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
            // in BPS, 10_000 = 100%
            target_ratio: 5_000,
            min_ratio: 0,
            max_ratio: 10_000,
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
    let eth_oracle_test_admin = &keypairs[MULTISIG_MEMBER_B];

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
            expo: -6,
            conf: 10_000,
            publish_time,
        },
        multisig_signers,
    )
    .await;

    // Alice: Initialize usdc and lp token associated token accounts
    let alice_usdc_token_account = initialize_token_account(
        &mut program_test_ctx,
        &usdc_mint,
        &keypairs[USER_ALICE].pubkey(),
    )
    .await;

    let alice_lp_token_account = initialize_token_account(
        &mut program_test_ctx,
        &lp_token_mint_pda,
        &keypairs[USER_ALICE].pubkey(),
    )
    .await;

    // Alice: Mint 1k USDC
    program_test_ctx
        .mint_tokens(
            &keypairs[ROOT_AUTHORITY],
            &usdc_mint,
            &alice_usdc_token_account,
            1_000_000_000,
        )
        .await
        .unwrap();

    // Alice: Add 1k USDC liquidity
    test_add_liquidity(
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

    // Martin: Initialize usdc associated token account
    let martin_usdc_token_account = initialize_token_account(
        &mut program_test_ctx,
        &usdc_mint,
        &keypairs[USER_MARTIN].pubkey(),
    )
    .await;

    // Martin: Mint 100 USDC
    program_test_ctx
        .mint_tokens(
            &keypairs[ROOT_AUTHORITY],
            &usdc_mint,
            &martin_usdc_token_account,
            100_000_000,
        )
        .await
        .unwrap();

    // Martin: Open 50 USDC position
    let position_pda = test_open_position(
        &mut program_test_ctx,
        &keypairs[USER_MARTIN],
        &keypairs[PAYER],
        &pool_pda,
        &usdc_mint,
        OpenPositionParams {
            // max price paid (slippage implied)
            price: 1_050_000,
            collateral: 50_000_000,
            size: 50_000_000,
            side: Side::Long,
        },
    )
    .await
    .0;

    // Usdc: Price set as 1.01 +- 0.01 (price increase of 1%)
    test_set_test_oracle_price(
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

    // Martin: Close the 50 USDC position with profit
    test_close_position(
        &mut program_test_ctx,
        &keypairs[USER_MARTIN],
        &keypairs[PAYER],
        &pool_pda,
        &usdc_mint,
        &position_pda,
        ClosePositionParams {
            // lowest exit price paid (slippage implied)
            price: 999_900,
        },
    )
    .await;

    let eth_test_oracle_pda = get_test_oracle_account(&pool_pda, &eth_mint).0;

    let eth_custody_pda = {
        let add_custody_params = AddCustodyParams {
            is_stable: true,
            oracle: OracleParams {
                oracle_account: eth_test_oracle_pda,
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

        let eth_custody_pda = test_add_custody(
            &mut program_test_ctx,
            pool_admin,
            &keypairs[PAYER],
            &pool_pda,
            &eth_mint,
            9,
            add_custody_params,
            multisig_signers,
        )
        .await
        .0;

        eth_custody_pda
    };

    // Eth: Price set as 1,676.04 +- 10
    test_set_test_oracle_price(
        &mut program_test_ctx,
        eth_oracle_test_admin,
        &keypairs[PAYER],
        &pool_pda,
        &eth_custody_pda,
        &eth_test_oracle_pda,
        SetTestOraclePriceParams {
            price: 1_676_040_000_000,
            expo: -9,
            conf: 10_000_000_000,
            publish_time,
        },
        multisig_signers,
    )
    .await;

    // Martin: Initialize ETH and lp associated token accounts
    let martin_eth_token_account = initialize_token_account(
        &mut program_test_ctx,
        &eth_mint,
        &keypairs[USER_MARTIN].pubkey(),
    )
    .await;

    let martin_lp_token_account = initialize_token_account(
        &mut program_test_ctx,
        &lp_token_mint_pda,
        &keypairs[USER_MARTIN].pubkey(),
    )
    .await;

    // Martin: Mint 2 ETH
    program_test_ctx
        .mint_tokens(
            &keypairs[ROOT_AUTHORITY],
            &eth_mint,
            &martin_eth_token_account,
            2_000_000_000,
        )
        .await
        .unwrap();

    // Martin: Add 1 ETH liquidity
    test_add_liquidity(
        &mut program_test_ctx,
        &keypairs[USER_MARTIN],
        &keypairs[PAYER],
        &pool_pda,
        &eth_mint,
        AddLiquidityParams {
            amount: 1_000_000_000,
        },
    )
    .await;

    // Paul: Initialize USDC and ETH accounts
    let paul_usdc_token_account = initialize_token_account(
        &mut program_test_ctx,
        &usdc_mint,
        &keypairs[USER_PAUL].pubkey(),
    )
    .await;

    let paul_eth_token_account = initialize_token_account(
        &mut program_test_ctx,
        &eth_mint,
        &keypairs[USER_PAUL].pubkey(),
    )
    .await;

    // Paul: Mint 150 USDC
    program_test_ctx
        .mint_tokens(
            &keypairs[ROOT_AUTHORITY],
            &usdc_mint,
            &paul_usdc_token_account,
            150_000_000,
        )
        .await
        .unwrap();

    // Paul: Swap 150 USDC for ETH
    test_swap(
        &mut program_test_ctx,
        &keypairs[USER_PAUL],
        &keypairs[PAYER],
        &pool_pda,
        &eth_mint,
        // The program receives USDC
        &usdc_mint,
        SwapParams {
            amount_in: 150_000_000,

            // 1% slippage
            min_amount_out: 150_000_000 / 1_676_040_000 * 99 / 100,
        },
    )
    .await;

    let alice_lp_token_balance = program_test_ctx
        .get_token_account(alice_lp_token_account)
        .await
        .unwrap()
        .amount;

    // Alice: Remove 100% of provided liquidity (1k USDC less fees)
    test_remove_liquidity(
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
