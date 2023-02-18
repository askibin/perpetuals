use anchor_lang::prelude::*;

use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{account, signature::Keypair};

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
            anchor_spl::associated_token::ID.as_ref(),
            mint.as_ref(),
        ],
        &perpetuals::id(),
    )
}

pub fn copy_keypair(keypair: &Keypair) -> Keypair {
    Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}

pub async fn get_account<T: anchor_lang::AccountDeserialize>(
    program_test_context: &mut ProgramTestContext,
    key: Pubkey,
) -> T {
    let account = program_test_context
        .banks_client
        .get_account(key)
        .await
        .unwrap()
        .unwrap();

    T::try_deserialize(&mut account.data.as_slice()).unwrap()
}
