use crate::{
    instructions,
    utils::{self, fixtures},
};
use bonfida_test_utils::ProgramTestExt;
use perpetuals::{
    instructions::{
        AddLiquidityParams, ClosePositionParams, OpenPositionParams, RemoveLiquidityParams,
        SwapParams,
    },
    state::position::Side,
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
    let keypairs =
        utils::create_and_fund_multiple_accounts(&mut program_test, KEYPAIRS_COUNT).await;

    // Initialize mints
    let usdc_mint = program_test
        .add_mint(None, 6, &keypairs[ROOT_AUTHORITY].pubkey())
        .0;
    let eth_mint = program_test
        .add_mint(None, 9, &keypairs[ROOT_AUTHORITY].pubkey())
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

    instructions::test_init(
        &mut program_test_ctx,
        upgrade_authority,
        fixtures::init_params_permissions_full(1),
        multisig_signers,
    )
    .await;

    let (pool_pda, _, lp_token_mint_pda, _, _) = utils::setup_pool_with_custodies(
        &mut program_test_ctx,
        &keypairs[MULTISIG_MEMBER_A],
        "FOO",
        &keypairs[PAYER],
        multisig_signers,
        vec![
            utils::SetupCustodyParams {
                mint: usdc_mint,
                decimals: 6,
                is_stable: true,
                target_ratio: 5_000,
                min_ratio: 0,
                max_ratio: 10_000,
                initial_price: 1_000_000,
                initial_conf: 10_000,
                oracle_admin: utils::copy_keypair(&keypairs[MULTISIG_MEMBER_A]),
                pricing_params: None,
                permissions: None,
                fees: None,
            },
            utils::SetupCustodyParams {
                mint: eth_mint,
                decimals: 9,
                is_stable: false,
                target_ratio: 5_000,
                min_ratio: 0,
                max_ratio: 10_000,
                initial_price: 1_676_040_000_000,
                initial_conf: 10_000_000_000,
                oracle_admin: utils::copy_keypair(&keypairs[MULTISIG_MEMBER_A]),
                pricing_params: None,
                permissions: None,
                fees: None,
            },
        ],
    )
    .await;

    // Alice: Initialize usdc and lp token associated token accounts
    utils::initialize_and_fund_token_account(
        &mut program_test_ctx,
        &usdc_mint,
        &keypairs[USER_ALICE].pubkey(),
        &keypairs[ROOT_AUTHORITY],
        1_000_000_000,
    )
    .await;

    let alice_lp_token_account = utils::initialize_token_account(
        &mut program_test_ctx,
        &lp_token_mint_pda,
        &keypairs[USER_ALICE].pubkey(),
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

    // Martin: Initialize usdc associated token account
    utils::initialize_and_fund_token_account(
        &mut program_test_ctx,
        &usdc_mint,
        &keypairs[USER_MARTIN].pubkey(),
        &keypairs[ROOT_AUTHORITY],
        100_000_000,
    )
    .await;

    // Martin: Open 50 USDC position
    let position_pda = instructions::test_open_position(
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

    // Martin: Close the 50 USDC position
    instructions::test_close_position(
        &mut program_test_ctx,
        &keypairs[USER_MARTIN],
        &keypairs[PAYER],
        &pool_pda,
        &usdc_mint,
        &position_pda,
        ClosePositionParams {
            // lowest exit price paid (slippage implied)
            price: 990_000,
        },
    )
    .await;

    // Martin: Initialize ETH and lp associated token accounts
    utils::initialize_and_fund_token_account(
        &mut program_test_ctx,
        &eth_mint,
        &keypairs[USER_MARTIN].pubkey(),
        &keypairs[ROOT_AUTHORITY],
        2_000_000_000,
    )
    .await;

    utils::initialize_token_account(
        &mut program_test_ctx,
        &lp_token_mint_pda,
        &keypairs[USER_MARTIN].pubkey(),
    )
    .await;

    // Martin: Add 1 ETH liquidity
    instructions::test_add_liquidity(
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
    utils::initialize_and_fund_token_account(
        &mut program_test_ctx,
        &usdc_mint,
        &keypairs[USER_PAUL].pubkey(),
        &keypairs[ROOT_AUTHORITY],
        150_000_000,
    )
    .await;

    utils::initialize_token_account(
        &mut program_test_ctx,
        &eth_mint,
        &keypairs[USER_PAUL].pubkey(),
    )
    .await;

    // Paul: Swap 150 USDC for ETH
    instructions::test_swap(
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

    let alice_lp_token_balance =
        utils::get_token_account_balance(&mut program_test_ctx, alice_lp_token_account).await;

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
