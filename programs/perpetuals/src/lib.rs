//! Perpetuals program entrypoint

#![allow(clippy::result_large_err)]

mod error;
mod instructions;
mod math;
mod state;

use {anchor_lang::prelude::*, instructions::*, state::perpetuals::PriceAndFee};

solana_security_txt::security_txt! {
    name: "Perpetuals",
    project_url: "https://github.com/solana-labs/solana-program-library/tree/master/perpetuals",
    contacts: "email:solana.farms@protonmail.com",
    policy: "",
    preferred_languages: "en",
    auditors: ""
}

declare_id!("Psx1bVshnRYzG8PdDFVLfBrYCs9JMwf7gwNMQ3zapXf");

#[program]
pub mod perpetuals {
    use super::*;

    // admin instructions
    pub fn init(ctx: Context<Init>, params: InitParams) -> Result<()> {
        instructions::init(ctx, &params)
    }

    pub fn init_pool<'info>(
        ctx: Context<'_, '_, '_, 'info, InitPool<'info>>,
        params: InitPoolParams,
    ) -> Result<u8> {
        instructions::init_pool(ctx, &params)
    }

    pub fn delete_pool<'info>(
        ctx: Context<'_, '_, '_, 'info, DeletePool<'info>>,
        params: DeletePoolParams,
    ) -> Result<u8> {
        instructions::delete_pool(ctx, &params)
    }

    pub fn add_token<'info>(
        ctx: Context<'_, '_, '_, 'info, AddToken<'info>>,
        params: AddTokenParams,
    ) -> Result<u8> {
        instructions::add_token(ctx, &params)
    }

    pub fn remove_token<'info>(
        ctx: Context<'_, '_, '_, 'info, RemoveToken<'info>>,
        params: RemoveTokenParams,
    ) -> Result<u8> {
        instructions::remove_token(ctx, &params)
    }

    pub fn set_admin_signers<'info>(
        ctx: Context<'_, '_, '_, 'info, SetAdminSigners<'info>>,
        params: SetAdminSignersParams,
    ) -> Result<u8> {
        instructions::set_admin_signers(ctx, &params)
    }

    pub fn set_pool_config<'info>(
        ctx: Context<'_, '_, '_, 'info, SetPoolConfig<'info>>,
        params: SetPoolConfigParams,
    ) -> Result<u8> {
        instructions::set_pool_config(ctx, &params)
    }

    pub fn set_token_config<'info>(
        ctx: Context<'_, '_, '_, 'info, SetTokenConfig<'info>>,
        params: SetTokenConfigParams,
    ) -> Result<u8> {
        instructions::set_token_config(ctx, &params)
    }

    pub fn set_permissions<'info>(
        ctx: Context<'_, '_, '_, 'info, SetPermissions<'info>>,
        params: SetPermissionsParams,
    ) -> Result<u8> {
        instructions::set_permissions(ctx, &params)
    }

    pub fn withdraw_fees<'info>(
        ctx: Context<'_, '_, '_, 'info, WithdrawFees<'info>>,
        params: WithdrawFeesParams,
    ) -> Result<u8> {
        instructions::withdraw_fees(ctx, &params)
    }

    // test instructions

    pub fn test_init(ctx: Context<TestInit>, params: TestInitParams) -> Result<()> {
        instructions::test_init(ctx, &params)
    }

    pub fn set_test_oracle_price<'info>(
        ctx: Context<'_, '_, '_, 'info, SetTestOraclePrice<'info>>,
        params: SetTestOraclePriceParams,
    ) -> Result<u8> {
        instructions::set_test_oracle_price(ctx, &params)
    }

    pub fn set_test_time<'info>(
        ctx: Context<'_, '_, '_, 'info, SetTestTime<'info>>,
        params: SetTestTimeParams,
    ) -> Result<u8> {
        instructions::set_test_time(ctx, &params)
    }

    // public instructions

    pub fn swap(ctx: Context<Swap>, params: SwapParams) -> Result<()> {
        instructions::swap(ctx, &params)
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, params: AddLiquidityParams) -> Result<()> {
        instructions::add_liquidity(ctx, &params)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        params: RemoveLiquidityParams,
    ) -> Result<()> {
        instructions::remove_liquidity(ctx, &params)
    }

    pub fn open_position(ctx: Context<OpenPosition>, params: OpenPositionParams) -> Result<()> {
        instructions::open_position(ctx, &params)
    }

    pub fn close_position(ctx: Context<ClosePosition>, params: ClosePositionParams) -> Result<()> {
        instructions::close_position(ctx, &params)
    }

    pub fn get_entry_price_and_fee(
        ctx: Context<GetEntryPriceAndFee>,
        params: GetEntryPriceAndFeeParams,
    ) -> Result<PriceAndFee> {
        instructions::get_entry_price_and_fee(ctx, &params)
    }

    pub fn get_exit_price_and_fee(
        ctx: Context<GetExitPriceAndFee>,
        params: GetExitPriceAndFeeParams,
    ) -> Result<PriceAndFee> {
        instructions::get_exit_price_and_fee(ctx, &params)
    }

    pub fn get_liquidation_price(
        ctx: Context<GetLiquidationPrice>,
        params: GetLiquidationPriceParams,
    ) -> Result<u64> {
        instructions::get_liquidation_price(ctx, &params)
    }

    pub fn get_swap_amount_and_fee(ctx: Context<Swap>, params: SwapParams) -> Result<AmountAndFee> {
        instructions::get_swap_amount_and_fee(ctx, &params)
    }
}
