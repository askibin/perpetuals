//! AddCollateral instruction handler

use {
    crate::{
        error::PerpetualsError,
        math,
        state::{
            custody::Custody, oracle::OraclePrice, perpetuals::Perpetuals, pool::Pool,
            position::Position,
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, TokenAccount},
    solana_program::program_error::ProgramError,
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

    token_program: Program<'info, Token>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddCollateralParams {
    collateral: u64,
}

pub fn add_collateral(ctx: Context<AddCollateral>, params: &AddCollateralParams) -> Result<()> {
    // validate inputs
    msg!("Validate inputs");
    if params.collateral == 0 {
        return Err(ProgramError::InvalidArgument.into());
    }
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let custody = ctx.accounts.custody.as_mut();
    let lock_custody = ctx.accounts.lock_custody.as_mut();
    let position = ctx.accounts.position.as_mut();
    let pool = ctx.accounts.pool.as_mut();
    let token_id = pool.get_token_id(&custody.key())?;
    require_keys_eq!(lock_custody.key(), position.lock_custody);

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

    let lock_token_price = OraclePrice::new_from_oracle(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
    )?;

    let lock_token_ema_price = OraclePrice::new_from_oracle_ema(
        custody.oracle.oracle_type,
        &ctx.accounts.custody_oracle_account.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
    )?;

    let lock_price = if lock_token_price < lock_token_ema_price {
        lock_token_price
    } else {
        lock_token_ema_price
    };

    // compute fee
    let fee_amount =
        pool.get_add_liquidity_fee(pool.get_token_id(&lock_custody.key())?, params.collateral, lock_custody, &lock_price)?;
    msg!("Collected fee: {}", fee_amount);

    // compute amount to transfer
    let transfer_amount = math::checked_add(params.collateral, fee_amount)?;
    let collateral_usd = lock_token_price.get_asset_amount_usd(params.collateral, lock_custody.decimals)?;
    msg!("Amount in: {}", transfer_amount);
    msg!("Collateral added in USD: {}", collateral_usd);

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
    position.update_time = perpetuals.get_time()?;
    position.collateral_usd = math::checked_add(position.collateral_usd, collateral_usd)?;
    position.collateral_amount = math::checked_add(position.collateral_amount, params.collateral)?;

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
        .wrapping_add(lock_token_price.get_asset_amount_usd(fee_amount, lock_custody.decimals)?);

    lock_custody.assets.collateral = math::checked_add(lock_custody.assets.collateral, params.collateral)?;
    lock_custody.assets.protocol_fees = math::checked_add(lock_custody.assets.protocol_fees, protocol_fee)?;

    Ok(())
}
