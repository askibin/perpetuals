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


    #[account(
        seeds = [b"custody",
                 pool.key().as_ref(),
                 lock_custody.mint.as_ref()],
        bump = lock_custody.bump
    )]
    pub lock_custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = lock_custody_oracle_account.key() == lock_custody.oracle.oracle_account
    )]
    pub lock_custody_oracle_account: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetEntryPriceAndFeeParams {
    collateral: u64,
    size: u64,
    side: Side,
}

pub fn get_entry_price_and_fee(
    ctx: Context<GetEntryPriceAndFee>,
    params: &GetEntryPriceAndFeeParams,
) -> Result<PriceAndFee> {
    // validate inputs
    if params.collateral == 0 || params.size == 0 || params.side == Side::None {
        return Err(ProgramError::InvalidArgument.into());
    }
    let pool = &ctx.accounts.pool;
    let custody = ctx.accounts.custody.as_mut();
    let lock_custody = &ctx.accounts.lock_custody.as_mut();
    // let token_id = pool.get_token_id(&custody.key())?;

    // compute position price
    let curtime = ctx.accounts.perpetuals.get_time()?;

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

    let price = pool.get_entry_price(&token_price, &token_ema_price, params.side, custody)?;

    let lock_token_price = OraclePrice::new_from_oracle(
        lock_custody.oracle.oracle_type,
        &ctx.accounts.lock_custody_oracle_account.to_account_info(),
        lock_custody.oracle.max_price_error,
        lock_custody.oracle.max_price_age_sec,
        curtime,
    )?;

    let lock_token_ema_price = OraclePrice::new_from_oracle_ema(
        lock_custody.oracle.oracle_type,
        &ctx.accounts.lock_custody_oracle_account.to_account_info(),
        lock_custody.oracle.max_price_error,
        lock_custody.oracle.max_price_age_sec,
        curtime,
    )?;

    let lock_price = if lock_token_price < lock_token_ema_price {
        lock_token_price
    } else {
        lock_token_ema_price
    };

    // compute fee
    let fee = pool.get_entry_fee(
        pool.get_token_id(&lock_custody.key())?,
        params.collateral,
        params.size,
        custody,
        lock_custody,
        &lock_price,
    )?;

    Ok(PriceAndFee { price, fee })
}
