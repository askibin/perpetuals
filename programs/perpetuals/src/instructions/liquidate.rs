//! Liquidate instruction handler

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
};

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = receiving_account.mint == custody.mint,
        constraint = receiving_account.owner == position.owner
    )]
    pub receiving_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = rewards_receiving_account.mint == custody.mint,
        constraint = rewards_receiving_account.owner == signer.key()
    )]
    pub rewards_receiving_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: empty PDA, authority for token accounts
    #[account(
        seeds = [b"transfer_authority"],
        bump = perpetuals.transfer_authority_bump
    )]
    pub transfer_authority: AccountInfo<'info>,

    #[account(
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
        seeds = [b"position",
                 position.owner.as_ref(),
                 pool.key().as_ref(),
                 custody.key().as_ref(),
                 &[position.side as u8]],
        bump
    )]
    pub position: Box<Account<'info, Position>>,

    #[account(
        mut,
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
        mut,
        seeds = [b"custody_token_account",
                 pool.key().as_ref(),
                 custody.mint.as_ref()],
        bump = custody.token_account_bump
    )]
    pub custody_token_account: Box<Account<'info, TokenAccount>>,

    token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct LiquidateParams {}

pub fn liquidate(ctx: Context<Liquidate>, _params: &LiquidateParams) -> Result<()> {
    // check permissions
    msg!("Check permissions");
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let custody = ctx.accounts.custody.as_mut();
    require!(
        perpetuals.permissions.allow_close_position && custody.permissions.allow_close_position,
        PerpetualsError::InstructionNotAllowed
    );

    let position = ctx.accounts.position.as_mut();
    let pool = ctx.accounts.pool.as_mut();
    let token_id = pool.get_token_id(&custody.key())?;

    // check if position can be liquidated
    let curtime = perpetuals.get_time()?;
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
    require!(
        pool.check_leverage(position, &token_price, &token_ema_price, &custody, false)?,
        PerpetualsError::InvalidPositionState
    );

    // compute exit price
    let exit_price = pool.get_exit_price(position, &token_price, &token_ema_price, &custody)?;
    msg!("Exit price: {}", exit_price);

    // compute amount to close
    let size = token_price.get_token_amount(position.size_usd, custody.decimals)?;
    let close_amount =
        pool.get_close_amount(&position, &token_price, &token_ema_price, &custody, size)?;

    // compute fee
    let fee_amount = pool.get_exit_fee(position, close_amount, size, &custody, &token_price)?;
    msg!("Collected fee: {}", fee_amount);

    // check collateral balance
    let transfer_amount = math::checked_sub(close_amount, fee_amount)?;
    msg!("Amount out: {}", transfer_amount);

    // pay accumulated interest
    let interest_debt = math::checked_add(
        pool.get_interest_amount(position, &custody, curtime)?,
        position.interest_debt_usd,
    )?;

    // update position
    msg!("Update position");
    position.time = perpetuals.get_time()?;
    //position.size = math::checked_sub(position.size, params.size)?;
    //position.collateral_usd = math::checked_sub(position.collateral_usd, params.collateral)?;

    // unlock pool funds
    pool.unlock_funds(transfer_amount, custody)?;

    // transfer tokens
    msg!("Transfer tokens");
    perpetuals.transfer_tokens(
        ctx.accounts.custody_token_account.to_account_info(),
        ctx.accounts.receiving_account.to_account_info(),
        ctx.accounts.transfer_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        transfer_amount,
    )?;

    // update custody stats
    msg!("Update custody stats");
    custody.collected_fees.close_position_usd = math::checked_add(
        custody.collected_fees.close_position_usd,
        token_price.get_asset_amount_usd(fee_amount, custody.decimals)?,
    )?;
    custody.volume_stats.close_position_usd = math::checked_add(
        custody.volume_stats.close_position_usd,
        token_price.get_asset_amount_usd(close_amount, custody.decimals)?,
    )?;

    //custody.assets.fees = math::checked_add(custody.assets.fees, fee_amount)?;

    if position.side == Side::Long {
        custody.trade_stats.oi_long_usd =
            math::checked_sub(custody.trade_stats.oi_long_usd, close_amount)?;
    } else {
        custody.trade_stats.oi_short_usd =
            math::checked_sub(custody.trade_stats.oi_short_usd, close_amount)?;
    }
    let pnl = 0;
    if pnl > 0 {
        custody.trade_stats.profit_usd = math::checked_add(custody.trade_stats.profit_usd, pnl)?;
    } else {
        custody.trade_stats.loss_usd = math::checked_add(custody.trade_stats.loss_usd, pnl)?;
    }

    Ok(())
}
