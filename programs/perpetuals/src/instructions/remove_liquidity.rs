//! RemoveLiquidity instruction handler

use {
    crate::{
        error::PerpetualsError,
        math,
        state::{custody::Custody, oracle::OraclePrice, perpetuals::Perpetuals, pool::Pool},
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
    solana_program::program_error::ProgramError,
};

#[derive(Accounts)]
#[instruction(params: RemoveLiquidityParams)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = receiving_account.mint == custody.mint,
        has_one = owner
    )]
    pub receiving_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = lp_token_account.mint == lp_token_mint.key(),
        has_one = owner
    )]
    pub lp_token_account: Box<Account<'info, TokenAccount>>,

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
        seeds = [b"custody",
                 pool.key().as_ref(),
                 custody.mint.as_ref()],
        bump = custody.bump
    )]
    pub custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the returned token
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
        seeds = [b"lp_token_mint",
                 pool.key().as_ref()],
        bump = pool.lp_token_bump
    )]
    pub lp_token_mint: Box<Account<'info, Mint>>,

    token_program: Program<'info, Token>,
    // remaining accounts:
    //   pool.tokens.len() - 1 custody accounts except receiving (write, unsigned)
    //   pool.tokens.len() - 1 custody oracles except receiving (write, unsigned)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveLiquidityParams {
    lp_amount: u64,
}

pub fn remove_liquidity(
    ctx: Context<RemoveLiquidity>,
    params: &RemoveLiquidityParams,
) -> Result<()> {
    // check permissions
    msg!("Check permissions");
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let custody = ctx.accounts.custody.as_mut();
    require!(
        perpetuals.permissions.allow_remove_liquidity && custody.permissions.allow_remove_liquidity,
        PerpetualsError::InstructionNotAllowed
    );

    // validate inputs
    msg!("Validate inputs");
    if params.lp_amount == 0 {
        return Err(ProgramError::InvalidArgument.into());
    }
    let pool = ctx.accounts.pool.as_mut();
    if pool.tokens.len() > 1 && ctx.remaining_accounts.len() < (pool.tokens.len() - 1) * 2 {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    }
    let token_id = pool.get_token_id(&custody.key())?;

    // compute assets under management
    msg!("Compute assets under management");
    let curtime = perpetuals.get_time()?;
    let token_price = OraclePrice::new_from_oracle(
        custody.oracle.oracle_type,
        &custody.to_account_info(),
        custody.oracle.max_price_error,
        custody.oracle.max_price_age_sec,
        curtime,
    )?;
    let mut pool_amount_usd: u128 =
        token_price.get_asset_amount_usd(custody.assets.owned, custody.decimals)? as u128;
    pool_amount_usd = math::checked_add(
        pool_amount_usd,
        pool.get_assets_under_management(&ctx.remaining_accounts, token_id, curtime)?,
    )?;

    // compute amount of tokens to return
    let remove_amount_usd = math::checked_as_u64(math::checked_div(
        math::checked_mul(pool_amount_usd as u128, params.lp_amount as u128)?,
        ctx.accounts.lp_token_mint.supply as u128,
    )?)?;
    let mut remove_amount = token_price.get_token_amount(remove_amount_usd, custody.decimals)?;

    // calculate fee
    let fee = pool.get_remove_liquidity_fee(token_id)?;
    let fee_amount = fee.get_fee_amount(remove_amount)?;
    msg!("Collected fee: {}", fee_amount);

    remove_amount = math::checked_sub(remove_amount, fee_amount)?;
    msg!("Amount out: {}", remove_amount);

    // check pool constraints
    msg!("Check pool constraints");
    require!(
        pool.check_amount_out(token_id, remove_amount)?,
        PerpetualsError::PoolAmountLimit
    );

    // transfer tokens
    msg!("Transfer tokens");
    perpetuals.transfer_tokens(
        ctx.accounts.custody_token_account.to_account_info(),
        ctx.accounts.receiving_account.to_account_info(),
        ctx.accounts.transfer_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        remove_amount,
    )?;

    // burn lp tokens
    msg!("Burn LP tokens");
    perpetuals.burn_tokens(
        ctx.accounts.lp_token_mint.to_account_info(),
        ctx.accounts.lp_token_account.to_account_info(),
        ctx.accounts.transfer_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        params.lp_amount,
    )?;

    // update pool stats
    msg!("Update pool stats");
    custody.collected_fees.remove_liquidity = math::checked_add(
        custody.collected_fees.remove_liquidity,
        token_price.get_asset_amount_usd(fee_amount, custody.decimals)?,
    )?;
    custody.volume_stats.remove_liquidity =
        math::checked_add(custody.volume_stats.remove_liquidity, remove_amount_usd)?;

    custody.assets.fees = math::checked_add(custody.assets.fees, fee_amount)?;
    custody.assets.owned = math::checked_sub(custody.assets.owned, remove_amount)?;

    Ok(())
}
