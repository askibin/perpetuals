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
        constraint = receiving_account.mint == custody.mint,
        constraint = receiving_account.owner == *signer.owner
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
    require!(
        !pool.check_leverage(position)?,
        PerpetualsError::InvalidPositionState
    );

    // compute exit price
    let curtime = perpetuals.get_time()?;
    let token_price = OraclePrice::new_from_oracle(
        custody.oracle.oracle_type,
        &custody.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
    )?;
    let exit_price = pool.get_exit_price(position, &token_price)?;
    msg!("Exit price: {}", exit_price);

    // compute amount to close
    let unrealized_pnl = math::checked_add(position.unrealized_pnl, pool.get_pnl(position)?)?;
    let available_amount = math::checked_add(position.collateral, unrealized_pnl)?;
    let close_amount = available_amount;

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
        token_price.get_asset_amount_usd(close_amount, custody.decimals)?,
    )?;

    custody.assets.fees = math::checked_add(custody.assets.fees, fee_amount)?;

    if position.side == Side::Long {
        custody.trade_stats.oi_long = math::checked_sub(custody.trade_stats.oi_long, close_amount)?;
    } else {
        custody.trade_stats.oi_short =
            math::checked_sub(custody.trade_stats.oi_short, close_amount)?;
    }
    let pnl = 0;
    if pnl > 0 {
        custody.trade_stats.profit = math::checked_add(custody.trade_stats.profit, pnl)?;
    } else {
        custody.trade_stats.loss = math::checked_add(custody.trade_stats.loss, pnl)?;
    }

    Ok(())
}
