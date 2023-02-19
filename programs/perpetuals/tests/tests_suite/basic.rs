use std::path::Path;

use anchor_lang::prelude::{Rent, SolanaSysvar, UpgradeableLoaderState};
use bonfida_test_utils::ProgramTestExt;
use perpetuals::{
    instructions::{AddCustodyParams, InitParams, SetTestOraclePriceParams},
    state::{
        custody::{Fees, FeesMode, OracleParams, PricingParams},
        oracle::OracleType,
        perpetuals::Permissions,
    },
};
use solana_program::{bpf_loader_upgradeable, stake_history::Epoch};
use solana_program_test::{read_file, ProgramTest};
use solana_sdk::{
    account::Account,
    signer::{keypair::Keypair, Signer},
};

use crate::{
    instructions::{
        test_add_custody, test_add_pool, test_init::test_init, test_set_test_oracle_price,
    },
    pda::{self, get_test_oracle_account},
    utils::{self, get_current_unix_timestamp},
};

const ROOT_AUTHORITY: usize = 0;
const PERPETUALS_UPGRADE_AUTHORITY: usize = 1;
const MULTISIG_MEMBER_A: usize = 2;
const MULTISIG_MEMBER_B: usize = 3;
const MULTISIG_MEMBER_C: usize = 4;
const PAYER: usize = 5;

const USDC: usize = 0;
const _BTC: usize = 1;

pub async fn basic_test_suite() {
    // ======================================================================
    // ====> Test Setup
    // ======================================================================
    let mut program_test = ProgramTest::default();

    let keypairs = {
        let keypairs = [
            Keypair::new(),
            Keypair::new(),
            Keypair::new(),
            Keypair::new(),
            Keypair::new(),
            Keypair::new(),
        ];

        keypairs
            .iter()
            .for_each(|k| utils::create_and_fund_account(&k.pubkey(), &mut program_test));

        keypairs
    };

    let (mints, mints_key) = {
        let (usdc_mint_key, usdc_mint) =
            program_test.add_mint(None, 6, &keypairs[ROOT_AUTHORITY].pubkey());
        let (btc_mint_key, btc_mint) =
            program_test.add_mint(None, 9, &keypairs[ROOT_AUTHORITY].pubkey());

        ([usdc_mint, btc_mint], [usdc_mint_key, btc_mint_key])
    };

    // Deploy the perpetuals program onchain as upgradeable program
    {
        // Deploy two accounts, one describing the program
        // and a second one holding the program's binary bytes
        let upgrade_authority = &keypairs[PERPETUALS_UPGRADE_AUTHORITY];

        let mut program_bytes = read_file(
            std::env::current_dir()
                .unwrap()
                .join(Path::new("../../target/deploy/perpetuals.so")),
        );

        let program_data_pda = pda::get_program_data_pda().0;

        let program = UpgradeableLoaderState::Program {
            programdata_address: program_data_pda,
        };
        let program_data = UpgradeableLoaderState::ProgramData {
            slot: 1,
            upgrade_authority_address: Some(upgrade_authority.pubkey()),
        };

        let serialized_program = bincode::serialize(&program).unwrap();

        let mut serialzed_program_data = bincode::serialize(&program_data).unwrap();
        serialzed_program_data.append(&mut program_bytes);

        let program_account = Account {
            lamports: Rent::default().minimum_balance(serialized_program.len()),
            data: serialized_program,
            owner: bpf_loader_upgradeable::ID,
            executable: true,
            rent_epoch: Epoch::default(),
        };
        let program_data_account = Account {
            lamports: Rent::default().minimum_balance(serialzed_program_data.len()),
            data: serialzed_program_data,
            owner: bpf_loader_upgradeable::ID,
            executable: false,
            rent_epoch: Epoch::default(),
        };

        program_test.add_account(perpetuals::id(), program_account);
        program_test.add_account(program_data_pda, program_data_account);
    }

    let mut program_test_ctx = program_test.start_with_context().await;

    // ======================================================================
    // ====> Test Run
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

    let (pool_pda, _, lp_token_mint_pda, _) = test_add_pool(
        &mut program_test_ctx,
        pool_admin,
        &keypairs[PAYER],
        "POOL A",
        multisig_signers,
    )
    .await;

    // Get USDC test oracle address
    let usdc_test_oracle_pda = get_test_oracle_account(&pool_pda, &mints_key[USDC]).0;

    let usdc_custody_pda = {
        let add_custody_params = AddCustodyParams {
            is_stable: true,
            oracle: OracleParams {
                oracle_account: usdc_test_oracle_pda,
                oracle_type: OracleType::Test,
                max_price_error: 1_000_000, // TO THINK ABOUT
                max_price_age_sec: 30,      // TO THINK ABOUT
            },
            pricing: PricingParams {
                use_ema: false,               // TO THINK ABOUT
                trade_spread_long: 100,       // TO THINK ABOUT
                trade_spread_short: 100,      // TO THINK ABOUT
                swap_spread: 200,             // TO THINK ABOUT
                min_initial_leverage: 10_000, // TO THINK ABOUT
                max_leverage: 1_000_000,      // TO THINK ABOUT
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
                max_increase: 20_000,  // TO THINK ABOUT
                max_decrease: 5_000,   // TO THINK ABOUT
                swap: 100,             // TO THINK ABOUT
                add_liquidity: 100,    // TO THINK ABOUT
                remove_liquidity: 100, // TO THINK ABOUT
                open_position: 100,    // TO THINK ABOUT
                close_position: 100,   // TO THINK ABOUT
                liquidation: 100,      // TO THINK ABOUT
                protocol_share: 10,    // TO THINK ABOUT
            },
            target_ratio: 50,
            min_ratio: 25, // TO THINK ABOUT
            max_ratio: 75, // TO THINK ABOUT
        };

        let usdc_custody_pda = test_add_custody(
            &mut program_test_ctx,
            pool_admin,
            &keypairs[PAYER],
            &pool_pda,
            &mints_key[USDC],
            mints[USDC].decimals,
            add_custody_params,
            multisig_signers,
        )
        .await
        .0;

        usdc_custody_pda
    };

    let usdc_oracle_test_admin = &keypairs[MULTISIG_MEMBER_A];

    let publish_time = get_current_unix_timestamp(&mut program_test_ctx).await;

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
}
