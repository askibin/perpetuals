//! ClosePosition instruction handler

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
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        constraint = receiving_account.mint == collateral_custody.mint,
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
                 collateral_custody.key().as_ref(),
                 &[position.side as u8]],
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
pub struct ClosePositionParams {
    price: u64,
    amount_percent: u64,
}

pub fn close_position(ctx: Context<ClosePosition>, params: &ClosePositionParams) -> Result<()> {
    // check permissions
    msg!("Check permissions");
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    let position = ctx.accounts.position.as_mut();
    let pool = ctx.accounts.pool.as_mut();

    // validate inputs
    msg!("Validate inputs");
    if params.price == 0 || params.amount_percent == 0 {
        return Err(ProgramError::InvalidArgument.into());
    }
    let token_id = pool.get_token_id(&ctx.accounts.collateral_custody.key())?;

    // compute exit price
    let exit_price = pool.get_exit_price(position)?;
    msg!("Close price: {}", exit_price);
    if position.side == Side::Long {
        require_gte!(exit_price, params.price, PerpetualsError::MaxPriceSlippage);
    } else {
        require_gte!(params.price, exit_price, PerpetualsError::MaxPriceSlippage);
    }

    // compute amount to close
    let unrealized_pnl = math::checked_add(position.unrealized_pnl, position.get_pnl()?)?;
    let available_amount = math::checked_add(position.collateral, unrealized_pnl)?;
    let close_amount = math::checked_as_u64(math::checked_div(
        math::checked_mul(available_amount as u128, params.amount_percent as u128)?,
        1000000u128,
    )?)?;

    // compute fee
    let fee = pool.get_exit_fee(position)?;
    let fee_amount = fee.get_fee_amount(close_amount)?;
    msg!("Close fee: {}", fee_amount);

    // check collateral balance
    msg!("Check collateral balance");
    let transfer_amount = math::checked_sub(close_amount, fee_amount)?;
    if ctx.accounts.custody_token_account.amount < transfer_amount {
        return Err(ProgramError::InsufficientFunds.into());
    }

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

    // update user stats
    msg!("Update user stats");

    // update pool stats
    msg!("Update pool stats");

    // update protocol stats
    msg!("Update protocol stats");

    Ok(())
}
