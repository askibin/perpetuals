//! GetExitPriceAndFee instruction handler

use {
    crate::state::{
        custody::Custody,
        oracle::OraclePrice,
        perpetuals::{Perpetuals, PriceAndFee},
        pool::Pool,
        position::Position,
    },
    anchor_lang::prelude::*,
    solana_program::program_error::ProgramError,
};

#[derive(Accounts)]
pub struct GetExitPriceAndFee<'info> {
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
        seeds = [b"position",
                 position.owner.as_ref(),
                 pool.key().as_ref(),
                 custody.key().as_ref(),
                 &[position.side as u8]],
        bump
    )]
    pub position: Box<Account<'info, Position>>,

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
pub struct GetExitPriceAndFeeParams {
    size_usd: u64,
}

pub fn get_exit_price_and_fee(
    ctx: Context<GetExitPriceAndFee>,
    params: &GetExitPriceAndFeeParams,
) -> Result<PriceAndFee> {
    // validate inputs
    let position = &ctx.accounts.position;
    if params.size_usd == 0 || params.size_usd > position.size_usd {
        return Err(ProgramError::InvalidArgument.into());
    }
    let pool = &ctx.accounts.pool;
    let token_id = pool.get_token_id(&ctx.accounts.custody.key())?;

    // compute exit price
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
    let price = pool.get_exit_price(position, &token_price, &token_ema_price, &custody)?;

    // compute amount to close
    let size = token_price.get_token_amount(params.size_usd, custody.decimals)?;
    let close_amount =
        pool.get_close_amount(&position, &token_price, &token_ema_price, &custody, size)?;

    // compute fee
    let fee = pool.get_exit_fee(position, close_amount, size, &custody, &token_price)?;

    Ok(PriceAndFee { price, fee })
}
