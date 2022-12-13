//! GetExitPriceAndFee instruction handler

use {
    crate::{
        error::PerpetualsError,
        math,
        state::{
            custody::Custody,
            oracle::OraclePrice,
            perpetuals::{Perpetuals, PriceAndFee},
            pool::Pool,
            position::{Position, Side},
        },
    },
    anchor_lang::prelude::*,
    solana_program::program_error::ProgramError,
};

#[derive(Accounts)]
pub struct GetExitPriceAndFee<'info> {
    #[account()]
    pub owner: Signer<'info>,

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
                 owner.key().as_ref(),
                 pool.key().as_ref(),
                 collateral_custody.key().as_ref(),
                 &[position.side as u8]],
        bump
    )]
    pub position: Box<Account<'info, Position>>,

    #[account(
        seeds = [b"custody",
                 custody.mint.as_ref()],
        bump = custody.bump
    )]
    pub custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = custody_oracle_account.key() == custody.oracle_account
    )]
    pub custody_oracle_account: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetExitPriceAndFeeParams {
    size: u64,
}

pub fn get_exit_price_and_fee(
    ctx: Context<GetExitPriceAndFee>,
    params: &GetExitPriceAndFeeParams,
) -> Result<PriceAndFee> {
    // validate inputs
    if params.size == 0 {
        return Err(ProgramError::InvalidArgument.into());
    }
    let position = &ctx.accounts.position;
    let pool = &ctx.accounts.pool;
    let token_id = pool.get_token_id(&ctx.accounts.custody.key())?;

    // compute exit price
    let curtime = ctx.accounts.perpetuals.get_time()?;
    let custody = ctx.accounts.custody.as_mut();
    let token_price = OraclePrice::new_from_oracle(
        custody.oracle_type,
        &ctx.accounts.custody.to_account_info(),
        custody.max_oracle_price_error,
        custody.max_oracle_price_age_sec,
        curtime,
    )?;
    let exit_price = pool.get_exit_price(position, &token_price)?;

    // compute amount to close
    let unrealized_pnl = math::checked_add(position.unrealized_pnl, position.get_pnl()?)?;
    let available_amount = math::checked_add(position.collateral, unrealized_pnl)?;
    let close_amount = math::checked_as_u64(math::checked_div(
        math::checked_mul(available_amount as u128, params.size as u128)?,
        1000000u128,
    )?)?;

    // compute fee
    let fee = pool.get_exit_fee(position)?;
    let fee_amount = fee.get_fee_amount(close_amount)?;

    Ok(PriceAndFee {
        price: exit_price,
        fee: fee_amount,
    })
}
