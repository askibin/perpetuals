use std::path::Path;

use anchor_lang::prelude::{Rent, UpgradeableLoaderState};
use bonfida_test_utils::ProgramTestExt;
use instructions::*;
use perpetuals::instructions::InitParams;
use solana_program::{bpf_loader_upgradeable, stake_history::Epoch};
use solana_program_test::{read_file, tokio, ProgramTest};
use solana_sdk::{
    account::Account,
    signer::{keypair::Keypair, Signer},
};

pub mod instructions;
pub mod pda;
pub mod utils;

const ROOT_AUTHORITY: usize = 0;
const PERPETUALS_UPGRADE_AUTHORITY: usize = 1;
const MULTISIG_MEMBER_A: usize = 2;

#[tokio::test]
async fn test_integration() {
    // ======================================================================
    // ====> Test Setup
    // ======================================================================
    let mut program_test = ProgramTest::default();

    let keypairs = {
        let keypairs = [Keypair::new(), Keypair::new(), Keypair::new()];

        keypairs
            .iter()
            .for_each(|k| utils::create_and_fund_account(&k.pubkey(), &mut program_test));

        keypairs
    };

    let (_mints, _mints_key) = {
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

    // Init
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
            &[&keypairs[MULTISIG_MEMBER_A]],
        )
        .await;
    }
}
