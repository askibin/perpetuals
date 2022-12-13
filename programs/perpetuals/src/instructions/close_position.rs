//! ClosePosition instruction handler

use {
    crate::{
        error::PerpetualsError,
        math,
        state::{
            custody::Custody,
            oracle::OraclePrice,
            perpetuals::Perpetuals,
            pool::Pool,
            position::{Position, Side},
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, TokenAccount},
    solana_program::program_error::ProgramError,
};

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = receiving_account.mint == custody.mint,
        has_one = owner
    )]
    pub receiving_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: empty PDA, authority for token accounts
    #[account(
        seeds = [b"transfer_authority"],
        bump = perpetuals.transfer_authority_bump
    )]
    pub transfer_authority: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"perpetuals"],
        bump = perpetuals.perpetuals_bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,

    #[account(
        mut,
        seeds = [b"pool",
                 pool.name.as_bytes()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        has_one = owner,
        seeds = [b"position",
                 owner.key().as_ref(),
                 pool.key().as_ref(),
                 custody.key().as_ref(),
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

    #[account(
        mut,
        constraint = custody_token_account.key() == custody.token_account.key()
    )]
    pub custody_token_account: Box<Account<'info, TokenAccount>>,

    token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ClosePositionParams {
    price: u64,
    size: u64,
    collateral_only: u64,
    size_only: u64,
    profit_only: u64,
}

pub fn close_position(ctx: Context<ClosePosition>, params: &ClosePositionParams) -> Result<()> {
    // check permissions
    msg!("Check permissions");
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    require!(
        perpetuals.permissions.allow_close_position,
        PerpetualsError::InstructionNotAllowed
    );

    // validate inputs
    msg!("Validate inputs");
    if params.price == 0 || params.size == 0 {
        return Err(ProgramError::InvalidArgument.into());
    }
    let position = ctx.accounts.position.as_mut();
    let pool = ctx.accounts.pool.as_mut();
    let token_id = pool.get_token_id(&ctx.accounts.custody.key())?;

    // compute exit price
    let curtime = perpetuals.get_time()?;
    let custody = ctx.accounts.custody.as_mut();
    let token_price = OraclePrice::new_from_oracle(
        custody.oracle_type,
        &ctx.accounts.custody.to_account_info(),
        custody.max_oracle_price_error,
        custody.max_oracle_price_age_sec,
        curtime,
    )?;
    let exit_price = pool.get_exit_price(position, &token_price)?;
    msg!("Exit price: {}", exit_price);
    if position.side == Side::Long {
        require_gte!(exit_price, params.price, PerpetualsError::MaxPriceSlippage);
    } else {
        require_gte!(params.price, exit_price, PerpetualsError::MaxPriceSlippage);
    }

    // compute amount to close
    let unrealized_pnl = math::checked_add(position.unrealized_pnl, pool.get_pnl(position)?)?;
    let available_amount = math::checked_add(position.collateral, unrealized_pnl)?;
    let close_amount = math::checked_as_u64(math::checked_div(
        math::checked_mul(available_amount as u128, params.size as u128)?,
        1000000u128,
    )?)?;

    // compute fee
    let fee = pool.get_exit_fee(position)?;
    let fee_amount = fee.get_fee_amount(close_amount)?;
    msg!("Collected fee: {}", fee_amount);

    // check collateral balance
    let transfer_amount = math::checked_sub(close_amount, fee_amount)?;
    msg!("Amount out: {}", transfer_amount);

    // check pool constraints
    msg!("Check pool constraints");
    require!(
        pool.check_amount_out(token_id, transfer_amount)?,
        PerpetualsError::PoolAmountLimit
    );

    // pay accumulated interest
    let interest_debt =
        math::checked_add(pool.get_interest_amount(position)?, position.interest_debt)?;

    // update position
    msg!("Update position");
    position.time = perpetuals.get_time()?;
    //position.size = math::checked_sub(position.size, params.size)?;
    //position.collateral = math::checked_sub(position.collateral, params.collateral)?;

    // check position risk
    msg!("Check position risks");
    require!(pool.check_leverage(position)?, PerpetualsError::MaxLeverage);

    // unlock pool funds
    pool.unlock_funds(transfer_amount)?;

    // transfer tokens
    msg!("Transfer tokens");
    perpetuals.transfer_tokens(
        ctx.accounts.custody_token_account.to_account_info(),
        ctx.accounts.receiving_account.to_account_info(),
        ctx.accounts.transfer_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        transfer_amount,
    )?;

    // update pool stats
    msg!("Update pool stats");
    custody.collected_fees.close_position = math::checked_add(
        custody.collected_fees.close_position,
        token_price.get_asset_amount_usd(fee_amount, custody.decimals)?,
    )?;
    custody.volume_stats.close_position = math::checked_add(
        custody.volume_stats.close_position,
        token_price.get_asset_amount_usd(params.size, custody.decimals)?,
    )?;

    custody.fee_amount = math::checked_add(custody.fee_amount, fee_amount)?;

    if position.side == Side::Long {
        custody.trade_stats.oi_long = math::checked_sub(custody.trade_stats.oi_long, params.size)?;
    } else {
        custody.trade_stats.oi_short =
            math::checked_sub(custody.trade_stats.oi_short, params.size)?;
    }
    if pnl > 0 {
        custody.trade_stats.profit = math::checked_add(custody.trade_stats.profit, pnl)?;
    } else {
        custody.trade_stats.loss = math::checked_add(custody.trade_stats.loss, pnl)?;
    }

    Ok(())
}
