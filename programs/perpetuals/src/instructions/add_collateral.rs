//! AddCollateral instruction handler

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
#[instruction(params: AddCollateralParams)]
pub struct AddCollateral<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = funding_account.mint == custody.mint,
        has_one = owner
    )]
    pub funding_account: Box<Account<'info, TokenAccount>>,

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
        has_one = owner,
        seeds = [b"position",
                 owner.key().as_ref(),
                 pool.key().as_ref(),
                 custody.key().as_ref(),
                 &[position.side as u8]],
        bump = position.bump
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
pub struct AddCollateralParams {
    pub price: u64,
    pub collateral: u64,
    pub size: u64,
}

pub fn add_collateral(ctx: Context<AddCollateral>, params: &AddCollateralParams) -> Result<()> {
    // check permissions
    msg!("Check permissions");
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let custody = ctx.accounts.custody.as_mut();
    if params.size > 0 {
        require!(
            perpetuals.permissions.allow_open_position && custody.permissions.allow_open_position,
            PerpetualsError::InstructionNotAllowed
        );
    }

    // validate inputs
    msg!("Validate inputs");
    if params.collateral == 0 && (params.price == 0 || params.size == 0) {
        return Err(ProgramError::InvalidArgument.into());
    }
    let position = ctx.accounts.position.as_mut();
    let pool = ctx.accounts.pool.as_mut();
    let token_id = pool.get_token_id(&custody.key())?;

    // compute position price
    let curtime = perpetuals.get_time()?;

    let token_price = OraclePrice::new_from_oracle(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
        false,
    )?;

    let token_ema_price = OraclePrice::new_from_oracle(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
        custody.pricing.use_ema,
    )?;

    let position_price =
        pool.get_entry_price(&token_price, &token_ema_price, position.side, custody)?;
    msg!("Entry price: {}", position_price);

    if params.size > 0 {
        if position.side == Side::Long {
            require_gte!(
                params.price,
                position_price,
                PerpetualsError::MaxPriceSlippage
            );
        } else {
            require_gte!(
                position_price,
                params.price,
                PerpetualsError::MaxPriceSlippage
            );
        }
    }

    // compute fee
    let mut fee_amount = pool.get_entry_fee(
        token_id,
        params.collateral,
        params.size,
        custody,
        &token_price,
    )?;
    // collect interest fee and reset cumulative_interest_snapshot
    if params.size > 0 {
        let interest_usd = custody.get_interest_amount_usd(position, curtime)?;
        let interest_amount = token_price.get_token_amount(interest_usd, custody.decimals)?;

        fee_amount = fee_amount + interest_amount;
        // remove position here cause borrow fees collected
        // will be reopen later
        custody.remove_position(position, curtime)?;
    }
    msg!("Collected fee: {}", fee_amount);

    // compute amount to transfer
    let transfer_amount = math::checked_add(params.collateral, fee_amount)?;
    msg!("Amount in: {}", transfer_amount);

    // check pool constraints
    msg!("Check pool constraints");
    let protocol_fee = Pool::get_fee_amount(custody.fees.protocol_share, fee_amount)?;
    let deposit_amount = math::checked_sub(transfer_amount, protocol_fee)?;
    require!(
        pool.check_token_ratio(token_id, deposit_amount, 0, custody, &token_price)?,
        PerpetualsError::TokenRatioOutOfRange
    );

    // update existing position
    msg!("Update existing position");
    let size_usd = token_price.get_asset_amount_usd(params.size, custody.decimals)?;
    let collateral_usd = token_price.get_asset_amount_usd(params.collateral, custody.decimals)?;
    msg!("params Collateral added in USD: {}", params.collateral);
    msg!("Collateral added in USD: {}", collateral_usd);
    let additional_locked_amount = math::checked_as_u64(math::checked_div(
        math::checked_mul(params.size as u128, custody.pricing.max_payoff_mult as u128)?,
        Perpetuals::BPS_POWER,
    )?)?;

    position.update_time = curtime;

    if params.size > 0 {
        // (current size * price + new size * new price) /
        position.price = math::checked_as_u64(math::checked_div(
            math::checked_add(
                math::checked_mul(position.size_usd as u128, position.price as u128)?,
                math::checked_mul(params.size as u128, position_price as u128)?,
            )?,
            math::checked_add(position.size_usd as u128, params.size as u128)?,
        )?)?;
        position.size_usd = math::checked_add(position.size_usd, size_usd)?;
        position.locked_amount =
            math::checked_add(position.locked_amount, additional_locked_amount)?;
        position.cumulative_interest_snapshot = custody.get_cumulative_interest(curtime)?;
    }

    if params.collateral > 0 {
        position.collateral_usd = math::checked_add(position.collateral_usd, collateral_usd)?;
        position.collateral_amount =
            math::checked_add(position.collateral_amount, params.collateral)?;
    }

    // check position risk
    msg!("Check position risks");
    require!(
        pool.check_leverage(
            token_id,
            position,
            &token_price,
            &token_ema_price,
            custody,
            curtime,
            false
        )?,
        PerpetualsError::MaxLeverage
    );

    // lock funds for potential profit payoff
    custody.lock_funds(additional_locked_amount)?;

    // transfer tokens
    msg!("Transfer tokens");
    perpetuals.transfer_tokens_from_user(
        ctx.accounts.funding_account.to_account_info(),
        ctx.accounts.custody_token_account.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        transfer_amount,
    )?;

    // update custody stats
    msg!("Update custody stats");
    custody.collected_fees.open_position_usd = custody
        .collected_fees
        .open_position_usd
        .wrapping_add(token_price.get_asset_amount_usd(fee_amount, custody.decimals)?);
    custody.volume_stats.open_position_usd = custody
        .volume_stats
        .open_position_usd
        .wrapping_add(size_usd);

    custody.assets.collateral = math::checked_add(custody.assets.collateral, params.collateral)?;
    custody.assets.protocol_fees = math::checked_add(custody.assets.protocol_fees, protocol_fee)?;

    if position.side == Side::Long {
        custody.trade_stats.oi_long_usd =
            math::checked_add(custody.trade_stats.oi_long_usd, size_usd)?;
    } else {
        custody.trade_stats.oi_short_usd =
            math::checked_add(custody.trade_stats.oi_short_usd, size_usd)?;
    }

    if params.size > 0 {
        custody.add_position(position, curtime)?;
    } else if params.collateral > 0 {
        custody.add_collateral(position.side, collateral_usd)?;
    }

    custody.update_borrow_rate(curtime)?;

    Ok(())
}
