use {
    crate::state::{
        oracle::OracleType,
        perpetuals::{Fee, Permissions},
    },
    anchor_lang::prelude::*,
};

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct Fees {
    pub swap: Fee,
    pub add_liquidity: Fee,
    pub remove_liquidity: Fee,
    pub open_position: Fee,
    pub close_position: Fee,
    pub liquidation: Fee,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct FeesStats {
    pub swap: u64,
    pub add_liquidity: u64,
    pub remove_liquidity: u64,
    pub open_position: u64,
    pub close_position: u64,
    pub liquidation: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct VolumeStats {
    pub swap: u64,
    pub add_liquidity: u64,
    pub remove_liquidity: u64,
    pub open_position: u64,
    pub close_position: u64,
    pub liquidation: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct TradeStats {
    pub profit: u64,
    pub loss: u64,
    pub oi_long: u64,
    pub oi_short: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct Assets {
    pub collateral: u64,
    pub fees: u64,
    pub owned: u64,
    pub locked: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct OracleParams {
    pub oracle_account: Pubkey,
    pub oracle_type: OracleType,
    pub max_price_error: f64,
    pub max_price_age_sec: u32,
}

#[account]
#[derive(Default, Debug)]
pub struct Custody {
    pub token_account: Pubkey,
    pub mint: Pubkey,
    pub decimals: u8,
    pub oracle: OracleParams,
    pub permissions: Permissions,
    pub fees: Fees,

    pub assets: Assets,
    pub collected_fees: FeesStats,
    pub volume_stats: VolumeStats,
    pub trade_stats: TradeStats,

    pub bump: u8,
    pub token_account_bump: u8,
}

impl Custody {
    pub const LEN: usize = 8 + std::mem::size_of::<Custody>();

    pub fn validate(&self) -> bool {
        matches!(self.oracle.oracle_type, OracleType::None)
            || (self.oracle.oracle_account != Pubkey::default()
                && self.oracle.max_price_error >= 0.0)
    }
}
