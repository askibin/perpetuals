//! GetLiquidationPrice instruction handler

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
#[instruction(params: GetLiquidationPriceParams)]
pub struct GetLiquidationPrice<'info> {
    #[account()]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"pool",
                 pool.name.as_bytes()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        seeds = [b"position",
                 owner.key().as_ref(),
                 pool.key().as_ref(),
                 pool.tokens[position.token_id as usize].custody.as_ref(),
                 &[position.side as u8]],
        bump
    )]
    pub position: Box<Account<'info, Position>>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct GetLiquidationPriceParams {}

pub fn get_liquidation_price(
    ctx: Context<GetLiquidationPrice>,
    params: &GetLiquidationPriceParams,
) -> Result<u64> {
    ctx.accounts
        .pool
        .get_liquidation_price(&ctx.accounts.position)
}
