//! GetEntryPriceAndFee instruction handler

use {
    crate::state::{
        custody::Custody,
        oracle::OraclePrice,
        perpetuals::{Perpetuals, PriceAndFee},
        pool::Pool,
        position::Side,
    },
    anchor_lang::prelude::*,
    solana_program::program_error::ProgramError,
};

#[derive(Accounts)]
pub struct GetEntryPriceAndFee<'info> {
    #[account()]
    pub signer: Signer<'info>,

    #[account(
        seeds = [b"perpetuals"],
        bump = perpetuals.perpetuals_bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,

    #[account(
        seeds = [b"pool",
                 pool.name.as_bytes()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        seeds = [b"custody",
                 pool.key().as_ref(),
                 custody.mint.as_ref()],
        bump = custody.bump
    )]
    pub custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = custody_oracle_account.key() == custody.oracle.oracle_account
    )]
    pub custody_oracle_account: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetEntryPriceAndFeeParams {
    collateral: u64,
    size_usd: u64,
    side: Side,
}

pub fn get_entry_price_and_fee(
    ctx: Context<GetEntryPriceAndFee>,
    params: &GetEntryPriceAndFeeParams,
) -> Result<PriceAndFee> {
    // validate inputs
    if params.size_usd == 0 || params.side == Side::None {
        return Err(ProgramError::InvalidArgument.into());
    }
    let pool = &ctx.accounts.pool;
    let token_id = pool.get_token_id(&ctx.accounts.custody.key())?;

    // compute position price
    let curtime = ctx.accounts.perpetuals.get_time()?;
    let custody = ctx.accounts.custody.as_mut();
    let token_price = OraclePrice::new_from_oracle(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
    )?;
    let token_ema_price = OraclePrice::new_from_oracle_ema(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
    )?;
    let price = pool.get_entry_price(&token_price, &token_ema_price, params.side, &custody)?;

    // compute fee
    let size = token_price.get_token_amount(params.size_usd, custody.decimals)?;
    let fee = pool.get_entry_fee(
        0,
        params.collateral,
        size,
        params.side,
        &custody,
        &token_price,
    )?;

    Ok(PriceAndFee { price, fee })
}
