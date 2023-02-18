use std::mem::size_of;

use anchor_lang::{
    __private::bytemuck::{try_from_bytes, Pod},
    prelude::*,
};

use perpetuals::error::PerpetualsError;
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{account, account::Account, signature::Keypair};

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

pub async fn get_account(
    program_test_context: &mut ProgramTestContext,
    key: Pubkey,
) -> Option<Account> {
    program_test_context
        .banks_client
        .get_account(key)
        .await
        .unwrap()
}

pub async fn get_anchor_account_data_as<T: Pod>(
    program_test_context: &mut ProgramTestContext,
    key: Pubkey,
) -> Option<T> {
    get_account(program_test_context, key).await.map(|x| {
        let mut data = x.data.clone();

        // Remove the Anchor discriminator
        for _ in 0..ANCHOR_DISCRIMINATOR_SIZE {
            data.remove(0);
        }

        let data_as_u8_array: &[u8] = &data;
        *load::<T>(data_as_u8_array).unwrap()
    })
}

pub async fn get_account_data_as<T: Pod>(
    program_test_context: &mut ProgramTestContext,
    key: Pubkey,
) -> Option<T> {
    get_account(program_test_context, key)
        .await
        .map(|x| *load::<T>(&x.data).unwrap())
}

/// Interpret the bytes in `data` as a value of type `T`
/// This will fail if :
/// - `data` is too short
/// - `data` is not aligned for T
fn load<T: Pod>(data: &[u8]) -> std::result::Result<&T, PerpetualsError> {
    try_from_bytes(
        data.get(0..size_of::<T>())
            .ok_or(PerpetualsError::InstructionDataTooShort)?,
    )
    .map_err(|_| PerpetualsError::InstructionDataSliceMisaligned)
}
