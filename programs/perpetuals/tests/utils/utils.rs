use std::path::Path;

use anchor_lang::{prelude::*, InstructionData};
use anchor_spl::token::spl_token;
use bonfida_test_utils::ProgramTestContextExt;
use solana_program::{bpf_loader_upgradeable, program_pack::Pack, stake_history::Epoch};
use solana_program_test::{read_file, ProgramTest, ProgramTestContext};
use solana_sdk::{account, signature::Keypair, signer::Signer, signers::Signers};

use super::get_program_data_pda;

pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

pub fn create_and_fund_account(address: &Pubkey, program_test: &mut ProgramTest) {
    program_test.add_account(
        *address,
        account::Account {
            lamports: 1_000_000_000,
            ..account::Account::default()
        },
    );
}

pub fn find_associated_token_account(owner: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            owner.as_ref(),
            anchor_spl::token::ID.as_ref(),
            mint.as_ref(),
        ],
        &anchor_spl::associated_token::ID,
    )
}

pub fn copy_keypair(keypair: &Keypair) -> Keypair {
    Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}

pub async fn get_token_account(
    program_test_ctx: &mut ProgramTestContext,
    key: Pubkey,
) -> spl_token::state::Account {
    let raw_account = program_test_ctx
        .banks_client
        .get_account(key)
        .await
        .unwrap()
        .unwrap();

    spl_token::state::Account::unpack(&raw_account.data).unwrap()
}

pub async fn get_account<T: anchor_lang::AccountDeserialize>(
    program_test_ctx: &mut ProgramTestContext,
    key: Pubkey,
) -> T {
    let account = program_test_ctx
        .banks_client
        .get_account(key)
        .await
        .unwrap()
        .unwrap();

    T::try_deserialize(&mut account.data.as_slice()).unwrap()
}

pub async fn get_current_unix_timestamp(program_test_ctx: &mut ProgramTestContext) -> i64 {
    program_test_ctx
        .banks_client
        .get_sysvar::<solana_program::sysvar::clock::Clock>()
        .await
        .unwrap()
        .unix_timestamp
}

pub async fn initialize_token_account(
    program_test_ctx: &mut ProgramTestContext,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Pubkey {
    program_test_ctx
        .initialize_token_accounts(*mint, &[*owner])
        .await
        .unwrap()[0]
}

pub async fn mint_tokens(
    program_test_ctx: &mut ProgramTestContext,
    mint_authority: &Keypair,
    mint: &Pubkey,
    token_account: &Pubkey,
    amount: u64,
) {
    program_test_ctx
        .mint_tokens(mint_authority, mint, token_account, amount)
        .await
        .unwrap();
}

// Deploy the perpetuals program onchain as upgradeable program
pub async fn add_perpetuals_program(program_test: &mut ProgramTest, upgrade_authority: &Keypair) {
    // Deploy two accounts, one describing the program
    // and a second one holding the program's binary bytes
    let mut program_bytes = read_file(
        std::env::current_dir()
            .unwrap()
            .join(Path::new("../../target/deploy/perpetuals.so")),
    );

    let program_data_pda = get_program_data_pda().0;

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

    let program_account = account::Account {
        lamports: Rent::default().minimum_balance(serialized_program.len()),
        data: serialized_program,
        owner: bpf_loader_upgradeable::ID,
        executable: true,
        rent_epoch: Epoch::default(),
    };
    let program_data_account = account::Account {
        lamports: Rent::default().minimum_balance(serialzed_program_data.len()),
        data: serialzed_program_data,
        owner: bpf_loader_upgradeable::ID,
        executable: false,
        rent_epoch: Epoch::default(),
    };

    program_test.add_account(perpetuals::id(), program_account);
    program_test.add_account(program_data_pda, program_data_account);
}

pub async fn create_and_fund_multiple_accounts(
    program_test: &mut ProgramTest,
    number: usize,
) -> Vec<Keypair> {
    let mut keypairs = Vec::new();

    for _ in 0..number {
        keypairs.push(Keypair::new());
    }

    keypairs
        .iter()
        .for_each(|k| create_and_fund_account(&k.pubkey(), program_test));

    keypairs
}

pub async fn create_and_execute_perpetuals_ix<T: InstructionData, U: Signers>(
    program_test_ctx: &mut ProgramTestContext,
    accounts_meta: Vec<AccountMeta>,
    args: T,
    payer: Option<&Pubkey>,
    signing_keypairs: &U,
) {
    let ix = solana_sdk::instruction::Instruction {
        program_id: perpetuals::id(),
        accounts: accounts_meta,
        data: args.data(),
    };

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        payer,
        signing_keypairs,
        program_test_ctx.last_blockhash,
    );

    program_test_ctx
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap();
}
