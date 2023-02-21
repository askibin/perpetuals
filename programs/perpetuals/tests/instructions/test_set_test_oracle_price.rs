use crate::utils::{self, pda};
use anchor_lang::{
    prelude::{AccountMeta, Pubkey},
    InstructionData, ToAccountMetas,
};
use perpetuals::{
    instructions::SetTestOraclePriceParams,
    state::{multisig::Multisig, oracle::TestOracle},
};
use solana_program_test::ProgramTestContext;
use solana_sdk::signer::{keypair::Keypair, Signer};

pub async fn test_set_test_oracle_price(
    program_test_ctx: &mut ProgramTestContext,
    admin: &Keypair,
    payer: &Keypair,
    pool_pda: &Pubkey,
    custody_pda: &Pubkey,
    oracle_pda: &Pubkey,
    params: SetTestOraclePriceParams,
    multisig_signers: &[&Keypair],
) {
    // ==== WHEN ==============================================================
    let multisig_pda = pda::get_multisig_pda().0;
    let perpetuals_pda = pda::get_perpetuals_pda().0;

    let multisig_account = utils::get_account::<Multisig>(program_test_ctx, multisig_pda).await;

    // One Tx per multisig signer
    for i in 0..multisig_account.min_signatures {
        let signer: &Keypair = multisig_signers[i as usize];

        let accounts_meta = {
            let accounts = perpetuals::accounts::SetTestOraclePrice {
                admin: admin.pubkey(),
                multisig: multisig_pda,
                perpetuals: perpetuals_pda,
                pool: *pool_pda,
                custody: *custody_pda,
                oracle_account: *oracle_pda,
                system_program: anchor_lang::system_program::ID,
            };

            let mut accounts_meta = accounts.to_account_metas(None);

            accounts_meta.push(AccountMeta {
                pubkey: signer.pubkey(),
                is_signer: true,
                is_writable: false,
            });

            accounts_meta
        };

        let arguments = perpetuals::instruction::SetTestOraclePrice { params };

        let ix = solana_sdk::instruction::Instruction {
            program_id: perpetuals::id(),
            accounts: accounts_meta,
            data: arguments.data(),
        };

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[admin, payer, signer],
            program_test_ctx.last_blockhash,
        );

        program_test_ctx
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap();
    }

    // ==== THEN ==============================================================
    let test_oracle_account = utils::get_account::<TestOracle>(program_test_ctx, *oracle_pda).await;

    assert_eq!(test_oracle_account.price, params.price);
    assert_eq!(test_oracle_account.expo, params.expo);
    assert_eq!(test_oracle_account.conf, params.conf);
    assert_eq!(test_oracle_account.publish_time, params.publish_time);
}
