//! OpenPosition instruction handler

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
#[instruction(params: OpenPositionParams)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = funding_account.mint == lock_custody.mint,
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
        init,
        payer = owner,
        space = Position::LEN,
        seeds = [b"position",
                 owner.key().as_ref(),
                 pool.key().as_ref(),
                 custody.key().as_ref(),
                 &[params.side as u8]],
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

    #[account(
        mut,
        seeds = [b"custody_token_account",
                 pool.key().as_ref(),
                 lock_custody.mint.as_ref()],
        bump = lock_custody.token_account_bump
    )]
    pub lock_custody_token_account: Box<Account<'info, TokenAccount>>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct OpenPositionParams {
    price: u64,
    collateral: u64,
    size: u64,
    side: Side,
}

pub fn open_position(ctx: Context<OpenPosition>, params: &OpenPositionParams) -> Result<()> {
    // check permissions
    msg!("Check permissions");
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let custody = ctx.accounts.custody.as_mut();
    let lock_custody = ctx.accounts.lock_custody.as_mut();

    require!(
        perpetuals.permissions.allow_open_position && custody.permissions.allow_open_position,
        PerpetualsError::InstructionNotAllowed
    );

    // validate inputs
    msg!("Validate inputs");
    if params.price == 0 || params.collateral == 0 || params.size == 0 || params.side == Side::None
    {
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
    )?;

    let token_ema_price = OraclePrice::new_from_oracle_ema(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
    )?;

    let position_price =
        pool.get_entry_price(&token_price, &token_ema_price, params.side, custody)?;
    msg!("Entry price: {}", position_price);

    
    let locked_amount = if params.side == Side::Long {
        require_gte!(
            params.price,
            position_price,
            PerpetualsError::MaxPriceSlippage
        );
        require_keys_eq!(custody.key(), lock_custody.key());
            
        math::checked_div(
            math::checked_mul(params.size as u128, custody.pricing.max_payoff_mult as u128)?,
            Perpetuals::BPS_POWER,
        )?
    } else {
        require_gte!(
            position_price,
            params.price,
            PerpetualsError::MaxPriceSlippage
        );
        require!(lock_custody.is_stable, PerpetualsError::InvalidCustodyAccountToLock);
        let price = if token_price < token_ema_price {
            token_price
        } else {
            token_ema_price
        };

        math::checked_div(
            math::checked_mul(
                math::checked_mul(params.size as u128, price.price as u128)?,
                custody.pricing.max_payoff_mult as u128)?,
            Perpetuals::BPS_POWER,
        )?
    };


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
    let fee_amount = pool.get_entry_fee(
        pool.get_token_id(&lock_custody.key())?,
        params.collateral,
        params.size,
        custody,
        lock_custody,
        &lock_price,
    )?;
    msg!("Collected fee: {}", fee_amount);

    // compute amount to transfer
    let transfer_amount = math::checked_add(params.collateral, fee_amount)?;
    msg!("Amount in: {}", transfer_amount);

    // check pool constraints
    msg!("Check pool constraints");
    let protocol_fee = Pool::get_fee_amount(lock_custody.fees.protocol_share, fee_amount)?;
    let deposit_amount = math::checked_sub(transfer_amount, protocol_fee)?;
    require!(
        pool.check_token_ratio(token_id, deposit_amount, 0, lock_custody, &token_price)?,
        PerpetualsError::TokenRatioOutOfRange
    );

    // init new position
    msg!("Initialize new position");
    let size_usd = token_price.get_asset_amount_usd(params.size, custody.decimals)?;
    let collateral_usd = lock_price.get_asset_amount_usd(params.collateral, lock_custody.decimals)?;

    position.owner = ctx.accounts.owner.key();
    position.pool = pool.key();
    position.custody = custody.key();
    position.lock_custody = lock_custody.key();
    position.open_time = perpetuals.get_time()?;
    position.update_time = 0;
    position.side = params.side;
    position.price = position_price;
    position.size_usd = size_usd;
    position.collateral_usd = collateral_usd;
    position.unrealized_profit_usd = 0;
    position.unrealized_loss_usd = 0;
    position.cumulative_interest_snapshot = custody.get_cumulative_interest(curtime)?;
    position.locked_amount = math::checked_as_u64(locked_amount)?;
    position.collateral_amount = params.collateral;
    position.bump = *ctx
        .bumps
        .get("position")
        .ok_or(ProgramError::InvalidSeeds)?;

    // check position risk
    msg!("Check position risks");
    require!(
        position.locked_amount > 0,
        PerpetualsError::InsufficientAmountReturned
    );
    require!(
        pool.check_leverage(
            token_id,
            position,
            &token_price,
            &token_ema_price,
            custody,
            curtime,
            true
        )?,
        PerpetualsError::MaxLeverage
    );

    // lock funds for potential profit payoff
    pool.lock_funds(position.locked_amount, lock_custody)?;

    // transfer tokens
    msg!("Transfer tokens");
    perpetuals.transfer_tokens_from_user(
        ctx.accounts.funding_account.to_account_info(),
        ctx.accounts.lock_custody_token_account.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        transfer_amount,
    )?;

    // update custody stats
    msg!("Update custody stats");
    
    lock_custody.collected_fees.open_position_usd = lock_custody
        .collected_fees
        .open_position_usd
        .wrapping_add(lock_price.get_asset_amount_usd(fee_amount, lock_custody.decimals)?);
        
    lock_custody.assets.collateral = math::checked_add(lock_custody.assets.collateral, params.collateral)?;
    lock_custody.assets.protocol_fees = math::checked_add(lock_custody.assets.protocol_fees, protocol_fee)?;

    custody.volume_stats.open_position_usd = custody
        .volume_stats
        .open_position_usd
        .wrapping_add(size_usd);

    if params.side == Side::Long {
        custody.trade_stats.oi_long_usd =
            math::checked_add(custody.trade_stats.oi_long_usd, size_usd)?;
    } else {
        custody.trade_stats.oi_short_usd =
            math::checked_add(custody.trade_stats.oi_short_usd, size_usd)?;
    }

    custody.update_borrow_rate(curtime)?;

    Ok(())
}
