//! SetPoolConfig instruction handler

use {
    crate::{
        error::PerpetualsError,
        state::{
            custody::Custody,
            multisig::{AdminInstruction, Multisig},
            oracle::OracleType,
        },
    },
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct SetPoolConfig<'info> {
    #[account()]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"multisig"],
        bump = multisig.load()?.bump
    )]
    pub multisig: AccountLoader<'info, Multisig>,

    #[account(
        mut,
        seeds = [b"custody",
                 custody.mint.as_ref()],
        bump = custody.bump
    )]
    pub custody: Box<Account<'info, Custody>>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SetPoolConfigParams {
    pub max_oracle_price_error: f64,
    pub max_oracle_price_age_sec: u32,
    pub oracle_type: OracleType,
    pub oracle_account: Pubkey,
}

pub fn set_pool_config<'info>(
    ctx: Context<'_, '_, '_, 'info, SetPoolConfig<'info>>,
    params: &SetPoolConfigParams,
) -> Result<u8> {
    // validate signatures
    let mut multisig = ctx.accounts.multisig.load_mut()?;

    let signatures_left = multisig.sign_multisig(
        &ctx.accounts.admin,
        &Multisig::get_account_infos(&ctx)[1..],
        &Multisig::get_instruction_data(AdminInstruction::SetPoolConfig, params)?,
    )?;
    if signatures_left > 0 {
        msg!(
            "Instruction has been signed but more signatures are required: {}",
            signatures_left
        );
        return Ok(signatures_left);
    }

    // update custody data
    let custody = ctx.accounts.custody.as_mut();
    custody.max_oracle_price_error = params.max_oracle_price_error;
    custody.max_oracle_price_age_sec = params.max_oracle_price_age_sec;
    custody.oracle_type = params.oracle_type;
    custody.oracle_account = params.oracle_account;

    //if !custody.validate() {
    //err!(PerpetualsError::InvalidCustodyConfig)
    //} else {
    Ok(0)
    //}
}
