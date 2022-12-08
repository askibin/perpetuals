//! OpenPosition instruction handler

use {
    crate::{
        error::PerpetualsError,
        math,
        state::{
            custody::Custody,
            multisig::Multisig,
            perpetuals::Perpetuals,
            pool::Pool,
            position::{Position, Side},
        },
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, TokenAccount},
    solana_program::{program, program_error::ProgramError},
};

#[derive(Accounts)]
#[instruction(params: OpenPositionParams)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = funding_account.mint == collateral_custody.mint,
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
        init_if_needed,
        payer = owner,
        space = Position::LEN,
        seeds = [b"position",
                 owner.key().as_ref(),
                 pool.key().as_ref(),
                 collateral_custody.key().as_ref(),
                 &[params.side as u8]],
        bump
    )]
    pub position: Box<Account<'info, Position>>,

    #[account(
        seeds = [b"custody",
                 collateral_custody.mint.as_ref()],
        bump = collateral_custody.bump
    )]
    pub collateral_custody: Box<Account<'info, Custody>>,

    /// CHECK: oracle account for the collateral token
    #[account(
        constraint = custody_oracle_account.key() == collateral_custody.oracle_account
    )]
    pub custody_oracle_account: AccountInfo<'info>,

    #[account(
        mut,
        constraint = custody_token_account.key() == collateral_custody.token_account.key()
    )]
    pub custody_token_account: Box<Account<'info, TokenAccount>>,

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
    let position = ctx.accounts.position.as_mut();
    let pool = ctx.accounts.pool.as_mut();

    // validate inputs
    msg!("Validate inputs");
    if params.price == 0
        || (params.collateral == 0 && position.time == 0)
        || params.size == 0
        || params.collateral < params.size
        || params.side == Side::None
    {
        return Err(ProgramError::InvalidArgument.into());
    }
    let token_id = pool.get_token_id(&ctx.accounts.collateral_custody.key())?;

    // compute position price
    let position_price = pool.get_entry_price(token_id, params.side)?;
    msg!("Trade price: {}", position_price);
    if params.side == Side::Long {
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

    // compute fee
    let fee = pool.get_entry_fee(token_id, params.side, params.size)?;
    let fee_amount = fee.get_fee_amount(params.size)?;
    msg!("Trade fee: {}", fee_amount);

    // check user balance
    msg!("Check user balance");
    let transfer_amount = math::checked_add(params.collateral, fee_amount)?;
    if ctx.accounts.funding_account.amount < transfer_amount {
        return Err(ProgramError::InsufficientFunds.into());
    }

    // check pool constraints
    msg!("Check pool constraints");
    let amount_limit = pool.get_amount_limit(token_id)?;
    let new_pool_amount =
        math::checked_add(ctx.accounts.custody_token_account.amount, transfer_amount)?;
    require_gte!(
        amount_limit,
        new_pool_amount,
        PerpetualsError::MaxPoolAmount
    );

    if position.time == 0 {
        // init new position
        msg!("Initialize new position");
        position.owner = ctx.accounts.owner.key();
        position.pool = pool.key();
        position.token_id = token_id as u16;
        position.time = perpetuals.get_time()?;
        position.side = params.side;
        position.price = position_price;
        position.size = params.size;
        position.collateral = params.collateral;
        position.interest_debt = 0;
        position.unrealized_pnl = 0;
        position.bump = *ctx
            .bumps
            .get("position")
            .ok_or(ProgramError::InvalidSeeds)?;
    } else {
        // update existing position
        msg!("Update existing position");

        if ctx.accounts.owner.key() != position.owner {
            return Err(ProgramError::IllegalOwner.into());
        }

        // save accumulated interest nad pnl
        position.interest_debt =
            math::checked_add(pool.get_interest_amount(position)?, position.interest_debt)?;
        position.unrealized_pnl = math::checked_add(position.get_pnl()?, position.unrealized_pnl)?;

        position.time = perpetuals.get_time()?;
        position.price = position_price;
        position.size = math::checked_add(position.size, params.size)?;
        position.collateral = math::checked_add(position.collateral, params.collateral)?;
    }

    // check position risk
    msg!("Check position risks");
    require!(pool.check_leverage(position)?, PerpetualsError::MaxLeverage);

    // lock funds for potential profit payoff
    pool.lock_funds(params.size)?;

    // update user stats
    msg!("Update user stats");

    // update pool stats
    msg!("Update pool stats");

    // update protocol stats
    msg!("Update protocol stats");

    Ok(())
}
