use {
    crate::{
        instructions,
        utils::{self, fixtures},
    },
    bonfida_test_utils::ProgramTestExt,
    perpetuals::instructions::AddLiquidityParams,
    perpetuals::instructions::RemoveLiquidityParams,
    perpetuals::state::custody::Custody,
    perpetuals::state::custody::{Fees, FeesMode},
    perpetuals::state::perpetuals::Perpetuals,
    perpetuals::state::pool::Pool,
    solana_program_test::ProgramTest,
    solana_sdk::signer::Signer,
};

const ROOT_AUTHORITY: usize = 0;
const PERPETUALS_UPGRADE_AUTHORITY: usize = 1;
const MULTISIG_MEMBER_A: usize = 2;
const MULTISIG_MEMBER_B: usize = 3;
const MULTISIG_MEMBER_C: usize = 4;
const PAYER: usize = 5;
const USER_ALICE: usize = 6;

const KEYPAIRS_COUNT: usize = 7;

const USDC_DECIMALS: u8 = 6;

#[tokio::test]
pub async fn fixed_fees() {
    let mut program_test = ProgramTest::default();

    // Initialize the accounts that will be used during the test suite
    let keypairs =
        utils::create_and_fund_multiple_accounts(&mut program_test, KEYPAIRS_COUNT).await;

    // Initialize mints
    let usdc_mint = program_test
        .add_mint(None, USDC_DECIMALS, &keypairs[ROOT_AUTHORITY].pubkey())
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
    .await
    .unwrap();

    // Initialize and fund associated token accounts
    {
        // Alice: mint 100k USDC
        {
            utils::initialize_and_fund_token_account(
                &mut program_test_ctx,
                &usdc_mint,
                &keypairs[USER_ALICE].pubkey(),
                &keypairs[ROOT_AUTHORITY],
                utils::scale(100_000, USDC_DECIMALS),
            )
            .await;
        }
    }

    let (pool_pda, _, _, _, custodies_info) =
        utils::setup_pool_with_custodies_and_liquidity(
            &mut program_test_ctx,
            &keypairs[MULTISIG_MEMBER_A],
            "FOO",
            &keypairs[PAYER],
            multisig_signers,
            vec![utils::SetupCustodyWithLiquidityParams {
                setup_custody_params: utils::SetupCustodyParams {
                    mint: usdc_mint,
                    decimals: USDC_DECIMALS,
                    is_stable: true,
                    target_ratio: utils::ratio_from_percentage(50.0),
                    min_ratio: utils::ratio_from_percentage(0.0),
                    max_ratio: utils::ratio_from_percentage(100.0),
                    initial_price: utils::scale(1, USDC_DECIMALS),
                    initial_conf: utils::scale_f64(0.01, USDC_DECIMALS),
                    pricing_params: None,
                    permissions: None,
                    borrow_rate: None,
                    fees: Some(Fees {
                        mode: FeesMode::Fixed,
                        add_liquidity: 200,
                        remove_liquidity: 300,
                        protocol_share: 25,
                        ..fixtures::fees_linear_regular()
                    }),
                },
                liquidity_amount: utils::scale(0, USDC_DECIMALS),
                payer: utils::copy_keypair(&keypairs[USER_ALICE]),
            }],
        )
        .await;

    // Check add liquidity fee
    {
        instructions::test_add_liquidity(
            &mut program_test_ctx,
            &keypairs[USER_ALICE],
            &keypairs[PAYER],
            &pool_pda,
            &usdc_mint,
            AddLiquidityParams {
                amount: utils::scale(1_000, USDC_DECIMALS),
            },
        )
        .await
        .unwrap();

        {
            let pool_account = utils::get_account::<Pool>(&mut program_test_ctx, pool_pda).await;
            let custody_account =
                utils::get_account::<Custody>(&mut program_test_ctx, custodies_info[0].custody_pda)
                    .await;

            assert_eq!(
                pool_account.aum_usd,
                utils::scale_f64(999.95, USDC_DECIMALS).into(),
            );

            assert_eq!(
                custody_account.collected_fees.add_liquidity_usd,
                utils::scale(20, USDC_DECIMALS),
            );

            assert_eq!(
                custody_account.assets.protocol_fees,
                utils::scale_f64(0.05, USDC_DECIMALS),
            );
        }
    }

    // Check remove liquidity fee
    {
        instructions::test_remove_liquidity(
            &mut program_test_ctx,
            &keypairs[USER_ALICE],
            &keypairs[PAYER],
            &pool_pda,
            &usdc_mint,
            RemoveLiquidityParams {
                lp_amount: utils::scale(100, Perpetuals::LP_DECIMALS),
            },
        )
        .await
        .unwrap();

        {
            let pool_account = utils::get_account::<Pool>(&mut program_test_ctx, pool_pda).await;
            let custody_account =
                utils::get_account::<Custody>(&mut program_test_ctx, custodies_info[0].custody_pda)
                    .await;

            assert_eq!(
                pool_account.aum_usd,
                utils::scale_f64(900.967705, USDC_DECIMALS).into(),
            );

            assert_eq!(
                custody_account.collected_fees.remove_liquidity_usd,
                utils::scale_f64(3.061072, USDC_DECIMALS),
            );

            assert_eq!(
                custody_account.assets.protocol_fees,
                utils::scale_f64(0.057653, USDC_DECIMALS),
            );
        }
    }
}
