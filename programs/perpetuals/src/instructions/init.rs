//! Init instruction handler

use {
    crate::{
        error::PerpetualsError,
        state::{multisig::Multisig, perpetuals::Perpetuals},
    },
    anchor_lang::prelude::*,
    anchor_spl::token::Token,
    solana_program::{program, program_error::ProgramError, sysvar},
};

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(mut)]
    pub upgrade_authority: Signer<'info>,

    #[account(
        init,
        payer = upgrade_authority,
        space = Multisig::LEN,
        seeds = [b"multisig"],
        bump
    )]
    pub multisig: AccountLoader<'info, Multisig>,

    /// CHECK: empty PDA, will be set as authority for token accounts
    #[account(
        init,
        payer = upgrade_authority,
        space = 0,
        seeds = [b"transfer_authority"],
        bump
    )]
    pub transfer_authority: AccountInfo<'info>,

    #[account(
        init,
        payer = upgrade_authority,
        space = Perpetuals::LEN,
        seeds = [b"perpetuals"],
        bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    // remaining accounts: 1 to Multisig::MAX_SIGNERS admin signers (read-only, unsigned)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitParams {
    pub min_signatures: u8,
}

pub fn init(ctx: Context<Init>, params: &InitParams) -> Result<()> {
    if !cfg!(feature = "test") {
        return err!(PerpetualsError::InvalidEnvironment);
    }

    // initialize multisig, this will fail if account is already initialized
    let mut multisig = ctx.accounts.multisig.load_init()?;

    multisig.set_signers(ctx.remaining_accounts, params.min_signatures)?;

    // record multisig PDA bump
    multisig.bump = *ctx
        .bumps
        .get("multisig")
        .ok_or(ProgramError::InvalidSeeds)?;

    // record perpetuals
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    perpetuals.transfer_authority_bump = *ctx
        .bumps
        .get("transfer_authority")
        .ok_or(ProgramError::InvalidSeeds)?;
    perpetuals.perpetuals_bump = *ctx
        .bumps
        .get("perpetuals")
        .ok_or(ProgramError::InvalidSeeds)?;
    perpetuals.inception_time = if cfg!(feature = "test") {
        0
    } else {
        perpetuals.get_time()?
    };

    if !perpetuals.validate() {
        return err!(PerpetualsError::InvalidPerpetualsConfig);
    }

    Ok(())
}
