//! AddLiquidity instruction handler

use {
    crate::{
        error::PerpetualsError,
        math,
        state::{
            custody::Custody,
            multisig::Multisig,
            oracle::OraclePrice,
            perpetuals::Perpetuals,
            pool::Pool,
            position::{Position, Side},
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token, TokenAccount},
    solana_program::{program, program_error::ProgramError},
};

#[derive(Accounts)]
#[instruction(params: AddLiquidityParams)]
pub struct AddLiquidity<'info> {
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
        seeds = [b"custody",
                 custody.mint.as_ref()],
        bump = custody.bump
    )]
    pub custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the receiving token
    #[account(
        constraint = custody_oracle_account.key() == custody.oracle_account
    )]
    pub custody_oracle_account: AccountInfo<'info>,

    #[account(
        mut,
        constraint = custody_token_account.key() == custody.token_account.key()
    )]
    pub custody_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = lp_token_mint.key() == pool.lp_token
    )]
    pub lp_token_mint: Box<Account<'info, Mint>>,

    token_program: Program<'info, Token>,
    // remaining accounts:
    //   pool.tokens.len()-1 custody accounts except receiving (read-only, unsigned)
    //   pool.tokens.len()-1 custody oracles except receiving (read-only, unsigned)
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddLiquidityParams {
    amount: u64,
}

pub fn add_liquidity(ctx: Context<AddLiquidity>, params: &AddLiquidityParams) -> Result<()> {
    // check permissions
    msg!("Check permissions");
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    require!(
        perpetuals.permissions.allow_add_liquidity,
        PerpetualsError::InstructionNotAllowed
    );

    // validate inputs
    msg!("Validate inputs");
    if params.amount == 0 {
        return Err(ProgramError::InvalidArgument.into());
    }
    let pool = ctx.accounts.pool.as_mut();
    let token_id = pool.get_token_id(&ctx.accounts.custody.key())?;

    // calculate fee
    let fee = pool.get_add_liquidity_fee(token_id)?;
    let fee_amount = fee.get_fee_amount(params.amount)?;
    msg!("Collected fee: {}", fee_amount);

    // check pool constraints
    msg!("Check pool constraints");
    require!(
        pool.check_amount_in(token_id, params.amount)?,
        PerpetualsError::PoolAmountLimit
    );

    // transfer tokens
    msg!("Transfer tokens");
    perpetuals.transfer_tokens_from_user(
        ctx.accounts.funding_account.to_account_info(),
        ctx.accounts.custody_token_account.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        params.amount,
    )?;

    // compute assets under management
    msg!("Compute assets under management");
    let curtime = perpetuals.get_time()?;
    let custody = ctx.accounts.custody.as_mut();
    let token_price = OraclePrice::new_from_oracle(
        custody.oracle_type,
        &ctx.accounts.custody.to_account_info(),
        custody.max_oracle_price_error,
        custody.max_oracle_price_age_sec,
        curtime,
    )?;
    let mut pool_amount_usd: u128 =
        token_price.get_asset_amount_usd(custody.owned_amount, custody.decimals)? as u128;
    pool_amount_usd = math::checked_add(
        pool_amount_usd,
        pool.get_assets_under_management(&ctx.remaining_accounts, token_id, curtime)?,
    )?;

    // compute amount of lp tokens to mint
    let no_fee_amount = math::checked_sub(params.amount, fee_amount)?;
    require_gte!(
        no_fee_amount,
        1,
        PerpetualsError::InsufficientAmountReturned
    );
    let token_amount_usd = token_price.get_asset_amount_usd(no_fee_amount, custody.decimals)?;
    let lp_amount = if pool_amount_usd == 0 {
        token_amount_usd
    } else {
        math::checked_as_u64(math::checked_div(
            math::checked_mul(
                token_amount_usd as u128,
                ctx.accounts.lp_token_mint.supply as u128,
            )?,
            pool_amount_usd,
        )?)?
    };
    msg!("LP tokens to mint: {}", lp_amount);

    // mint lp tokens
    perpetuals.mint_tokens(
        ctx.accounts.lp_token_mint.to_account_info(),
        ctx.accounts.lp_token_account.to_account_info(),
        ctx.accounts.transfer_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        lp_amount,
    )?;

    // update pool stats
    msg!("Update pool stats");
    custody.collected_fees.add_liquidity = math::checked_add(
        custody.collected_fees.add_liquidity,
        token_price.get_asset_amount_usd(fee_amount, custody.decimals)?,
    )?;
    custody.volume_stats.add_liquidity = math::checked_add(
        custody.volume_stats.add_liquidity,
        token_price.get_asset_amount_usd(params.amount, custody.decimals)?,
    )?;

    custody.fee_amount = math::checked_add(custody.fee_amount, fee_amount)?;
    custody.owned_amount = math::checked_add(custody.owned_amount, params.amount)?;

    Ok(())
}
