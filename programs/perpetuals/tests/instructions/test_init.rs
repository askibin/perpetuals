use anchor_lang::{prelude::AccountMeta, InstructionData, ToAccountMetas};
use perpetuals::{
    instructions::InitParams,
    state::{multisig::Multisig, perpetuals::Perpetuals},
};
use solana_program_test::ProgramTestContext;
use solana_sdk::signer::{keypair::Keypair, Signer};

use crate::{pda, utils::get_account};

pub async fn test_init(
    program_test_ctx: &mut ProgramTestContext,
    upgrade_authority: &Keypair,
    params: InitParams,
    multisig_signers: &[&Keypair],
) {
    // ==== WHEN ==============================================================
    let perpetuals_program_data = pda::get_program_data_pda().0;
    let (multisig_pda, multisig_bump) = pda::get_multisig_pda();
    let (transfer_authority_pda, transfer_authority_bump) = pda::get_transfer_authority_pda();
    let (perpetuals_pda, perpetuals_bump) = pda::get_perpetuals_pda();

    let accounts_meta = {
        let accounts = perpetuals::accounts::Init {
            upgrade_authority: upgrade_authority.pubkey(),
            multisig: multisig_pda,
            transfer_authority: transfer_authority_pda,
            perpetuals: perpetuals_pda,
            perpetuals_program: perpetuals::ID,
            perpetuals_program_data,
            system_program: anchor_lang::system_program::ID,
            token_program: anchor_spl::token::ID,
        };

        let mut accounts_meta = accounts.to_account_metas(None);

        for signer in multisig_signers {
            accounts_meta.push(AccountMeta {
                pubkey: signer.pubkey(),
                is_signer: true,
                is_writable: false,
            });
        }

        accounts_meta
    };

    let arguments = perpetuals::instruction::Init { params };

    let ix = solana_sdk::instruction::Instruction {
        program_id: perpetuals::id(),
        accounts: accounts_meta,
        data: arguments.data(),
    };

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[ix],
        Some(&upgrade_authority.pubkey()),
        &[&[upgrade_authority], multisig_signers].concat(),
        program_test_ctx.last_blockhash,
    );

    program_test_ctx
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    // ==== THEN ==============================================================
    let perpetuals_account = get_account::<Perpetuals>(program_test_ctx, perpetuals_pda).await;

    // Assert permissions
    {
        let p = perpetuals_account.permissions;

        assert_eq!(p.allow_swap, params.allow_swap);
        assert_eq!(p.allow_add_liquidity, params.allow_add_liquidity);
        assert_eq!(p.allow_remove_liquidity, params.allow_remove_liquidity);
        assert_eq!(p.allow_open_position, params.allow_open_position);
        assert_eq!(p.allow_close_position, params.allow_close_position);
        assert_eq!(p.allow_pnl_withdrawal, params.allow_pnl_withdrawal);
        assert_eq!(
            p.allow_collateral_withdrawal,
            params.allow_collateral_withdrawal
        );
        assert_eq!(p.allow_size_change, params.allow_size_change);
    }

    assert_eq!(
        perpetuals_account.transfer_authority_bump,
        transfer_authority_bump
    );
    assert_eq!(perpetuals_account.perpetuals_bump, perpetuals_bump);

    let multisig_account = get_account::<Multisig>(program_test_ctx, multisig_pda).await;

    // Assert multisig
    {
        assert_eq!(multisig_account.bump, multisig_bump);
        assert_eq!(multisig_account.min_signatures, params.min_signatures);

        // Check signers
        {
            let mut i = 0;
            for signer in multisig_signers {
                assert_eq!(multisig_account.signers[i], signer.pubkey());

                i += 1;
            }
        }
    }
}
