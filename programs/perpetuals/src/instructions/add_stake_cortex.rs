//! AddStakeCortex instruction handler

use anchor_spl::token::TokenAccount;

use crate::math;

use {
    crate::{
        error::PerpetualsError,
        state::{cortex::Cortex, multisig::Multisig, perpetuals::Perpetuals},
    },
    anchor_lang::prelude::*,
    anchor_spl::token::{Mint, Token},
    solana_program::program_error::ProgramError,
};

#[derive(Accounts)]
pub struct AddStakeCortex<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        token::mint = lm_token_mint,
        has_one = owner
    )]
    pub funding_account: Box<Account<'info, TokenAccount>>,

    // lm_token_staking vault
    #[account(
        mut,
        token::mint = lm_token_mint,
        seeds = [b"stake_token_account"],
        bump = cortex.stake_token_account_bump
    )]
    pub stake_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: empty PDA, authority for token accounts
    #[account(
        seeds = [b"transfer_authority"],
        bump = perpetuals.transfer_authority_bump
    )]
    pub transfer_authority: AccountInfo<'info>,

    #[account(
        seeds = [b"cortex"],
        bump = cortex.bump
    )]
    pub cortex: Box<Account<'info, Cortex>>,

    #[account(
        seeds = [b"perpetuals"],
        bump = perpetuals.perpetuals_bump
    )]
    pub perpetuals: Box<Account<'info, Perpetuals>>,

    #[account(
        mut,
        seeds = [b"lm_token_mint"],
        bump = cortex.lm_token_bump
    )]
    pub lm_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [b"redeemable_token_mint",
                 lm_token_mint.key().as_ref()],
        bump = cortex.redeemable_token_bump
    )]
    pub redeemable_token_mint: Box<Account<'info, Mint>>,

    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    // remaining accounts: 1 to Multisig::MAX_SIGNERS admin signers (read-only, unsigned)
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct AddStakeCortexParams {
    pub amount: u64,
}

pub fn add_stake_cortex(ctx: Context<AddStakeCortex>, params: &AddStakeCortexParams) -> Result<()> {
    // validate inputs
    msg!("Validate inputs");
    if params.amount == 0 {
        return Err(ProgramError::InvalidArgument.into());
    }

    // transfer tokens
    msg!("Transfer tokens");
    let perpetuals = ctx.accounts.perpetuals.as_mut();
    perpetuals.transfer_tokens_from_user(
        ctx.accounts.funding_account.to_account_info(),
        ctx.accounts.stake_token_account.to_account_info(),
        ctx.accounts.owner.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        params.amount,
    )?;

    // compute amount of lp tokens to mint
    let pool_amount = ctx.accounts.stake_token_account.amount;
    let redeemable_amount = if pool_amount == 0 {
        params.amount
    } else {
        math::checked_as_u64(math::checked_div(
            math::checked_mul(
                params.amount as u128,
                ctx.accounts.redeemable_token_mint.supply as u128,
            )?,
            pool_amount.into(),
        )?)?
    };
    msg!("Reedemable tokens to mint: {}", redeemable_amount);

    // mint redeemable tokens
    perpetuals.mint_tokens(
        ctx.accounts.redeemable_token_mint.to_account_info(),
        ctx.accounts.stake_token_account.to_account_info(),
        ctx.accounts.transfer_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        redeemable_amount,
    )?;

    // transfer staking amount to vault
    Ok(())
}
