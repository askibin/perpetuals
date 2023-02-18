use bonfida_test_utils::ProgramTestExt;
use instructions::*;
use perpetuals::instructions::InitParams;
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::signer::{keypair::Keypair, Signer};

pub mod instructions;
pub mod pda;
pub mod utils;

const _ALICE: usize = 0;
const _BOB: usize = 1;
const ROOT_AUTHORITY: usize = 2;
const PERPETUALS_UPGRADE_AUTHORITY: usize = 3;

const USDC: usize = 0;
const BTC: usize = 1;

#[tokio::test]
async fn test_integration() {
    // ==== GIVEN ==============================================================
    let mut program_test =
        ProgramTest::new("perpetuals", perpetuals::ID, processor!(perpetuals::entry));

    let keypairs = [
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
        Keypair::new(),
    ];

    let multisig_pda = pda::get_multisig_pda().0;
    let transfer_authority_pda = pda::get_transfer_authority_pda().0;
    let perpetuals_pda = pda::get_perpetuals_pda().0;

    keypairs
        .iter()
        .for_each(|k| utils::create_and_fund_account(&k.pubkey(), &mut program_test));

    let (usdc_mint_key, usdc_mint) =
        program_test.add_mint(None, 6, &keypairs[ROOT_AUTHORITY].pubkey());
    let (btc_mint_key, btc_mint) =
        program_test.add_mint(None, 9, &keypairs[ROOT_AUTHORITY].pubkey());

    let _mints = [usdc_mint, btc_mint];
    let _mints_key = [usdc_mint_key, btc_mint_key];

    // Start and process transactions on the test network
    let mut program_test_ctx = program_test.start_with_context().await;

    // ==== Init ==============================================================
    let upgrade_authority = &keypairs[PERPETUALS_UPGRADE_AUTHORITY];
    let params = InitParams {
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
        &multisig_pda,
        &transfer_authority_pda,
        &perpetuals_pda,
        &perpetuals::ID,
        params,
    )
    .await;
}
