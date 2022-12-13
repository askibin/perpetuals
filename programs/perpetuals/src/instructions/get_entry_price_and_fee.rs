//! GetEntryPriceAndFee instruction handler

use {
    crate::state::{
        custody::Custody,
        oracle::OraclePrice,
        perpetuals::{Perpetuals, PriceAndFee},
        pool::Pool,
        position::{Position, Side},
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, TokenAccount},
    solana_program::{program, program_error::ProgramError},
};

#[derive(Accounts)]
pub struct GetEntryPriceAndFee<'info> {
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
pub struct GetEntryPriceAndFeeParams {
    size: u64,
    side: Side,
}

pub fn get_entry_price_and_fee(
    ctx: Context<GetEntryPriceAndFee>,
    params: &GetEntryPriceAndFeeParams,
) -> Result<PriceAndFee> {
    // validate inputs
    if params.size == 0 || params.side == Side::None {
        return Err(ProgramError::InvalidArgument.into());
    }
    let pool = &ctx.accounts.pool;
    let token_id = pool.get_token_id(&ctx.accounts.custody.key())?;

    // compute position price
    let curtime = ctx.accounts.perpetuals.get_time()?;
    let custody = ctx.accounts.custody.as_mut();
    let token_price = OraclePrice::new_from_oracle(
        custody.oracle_type,
        &ctx.accounts.custody.to_account_info(),
        custody.max_oracle_price_error,
        custody.max_oracle_price_age_sec,
        curtime,
    )?;
    let position_price = pool.get_entry_price(token_id, &token_price, params.side)?;

    // compute fee
    let fee = pool.get_entry_fee(token_id, params.side, params.size)?;
    let fee_amount = fee.get_fee_amount(params.size)?;

    Ok(PriceAndFee {
        price: position_price,
        fee: fee_amount,
    })
}
