// Contains dummy values usable in tests, made to reduce boilerplate

use anchor_lang::prelude::Pubkey;
use perpetuals::{state::{
    custody::{Fees, FeesMode, PricingParams, OracleParams},
    perpetuals::Permissions, oracle::OracleType,
}, instructions::InitParams};

pub fn permissions_full() -> Permissions {
    Permissions {
        allow_swap: true,
        allow_add_liquidity: true,
        allow_remove_liquidity: true,
        allow_open_position: true,
        allow_close_position: true,
        allow_pnl_withdrawal: true,
        allow_collateral_withdrawal: true,
        allow_size_change: true,
    }
}

pub fn fees_linear_regular() -> Fees {
    Fees {
        mode: FeesMode::Linear,
        max_increase: 20_000,
        max_decrease: 5_000,
        swap: 100,
        add_liquidity: 100,
        remove_liquidity: 100,
        open_position: 100,
        close_position: 100,
        liquidation: 100,
        protocol_share: 10,
    }
}

pub fn pricing_params_regular(use_ema: bool) -> PricingParams {
    PricingParams {
        use_ema,
        trade_spread_long: 100,
        trade_spread_short: 100,
        swap_spread: 200,
        min_initial_leverage: 10_000,
        max_leverage: 1_000_000,
        max_payoff_mult: 10,
    }
}

pub fn oracle_params_regular(oracle_account: Pubkey) -> OracleParams {
    OracleParams {
        oracle_account,
        oracle_type: OracleType::Test,
        max_price_error: 1_000_000,
        max_price_age_sec: 30,
    }
}

pub fn init_params_permissions_full(min_signatures: u8) -> InitParams {
    InitParams {
        min_signatures,
        allow_swap: true,
        allow_add_liquidity: true,
        allow_remove_liquidity: true,
        allow_open_position: true,
        allow_close_position: true,
        allow_pnl_withdrawal: true,
        allow_collateral_withdrawal: true,
        allow_size_change: true,
    }
}