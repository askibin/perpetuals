//! AddCollateral instruction handler

use {
    crate::{
        error::PerpetualsError,
        math,
        state::{
            cortex::Cortex, custody::Custody, oracle::OraclePrice, perpetuals::Perpetuals,
            pool::Pool, position::Position,
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
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

    #[account(
        mut,
        constraint = lm_token_account.mint == lm_token_mint.key(),
        has_one = owner
    )]
    pub lm_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: empty PDA, authority for token accounts
    #[account(
        seeds = [b"transfer_authority"],
        bump = perpetuals.transfer_authority_bump
    )]
    pub transfer_authority: AccountInfo<'info>,

    #[account(
        seeds = [b"cortex"],
        bump = cortex.cortex_bump
    )]
    pub cortex: Box<Account<'info, Cortex>>,

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

    #[account(
        mut,
        seeds = [b"lm_token_mint"],
        bump = cortex.lm_token_bump
    )]
    pub lm_token_mint: Box<Account<'info, Mint>>,

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

    // compute fee
    let fee_amount =
        pool.get_add_liquidity_fee(token_id, params.collateral, custody, &token_price)?;
    msg!("Collected fee: {}", fee_amount);

    // compute amount to transfer
    let transfer_amount = math::checked_add(params.collateral, fee_amount)?;
    let collateral_usd = token_price.get_asset_amount_usd(params.collateral, custody.decimals)?;
    msg!("Amount in: {}", transfer_amount);
    msg!("Collateral added in USD: {}", collateral_usd);

    // compute amount of lm token to mint
    let lm_rewards_amount = ctx.accounts.cortex.get_lm_rewards_amount(fee_amount)?;

    // mint lm tokens
    perpetuals.mint_tokens(
        ctx.accounts.lm_token_mint.to_account_info(),
        ctx.accounts.lm_token_account.to_account_info(),
        ctx.accounts.transfer_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        lm_rewards_amount,
    )?;
    msg!("Amount LM rewards out: {}", lm_rewards_amount);

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

    custody.distributed_rewards.open_position_lm = custody
        .distributed_rewards
        .open_position_lm
        .wrapping_add(lm_rewards_amount);

    custody.assets.collateral = math::checked_add(custody.assets.collateral, params.collateral)?;
    custody.assets.protocol_fees = math::checked_add(custody.assets.protocol_fees, protocol_fee)?;

    Ok(())
}
