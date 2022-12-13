use {crate::state::oracle::OracleType, anchor_lang::prelude::*};

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct CollectedFees {
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

#[account]
#[derive(Default, Debug)]
pub struct Custody {
    pub token_account: Pubkey,
    pub mint: Pubkey,
    pub decimals: u8,
    pub max_oracle_price_error: f64,
    pub max_oracle_price_age_sec: u32,
    pub oracle_type: OracleType,
    pub oracle_account: Pubkey,

    pub collateral_amount: u64,
    pub fee_amount: u64,
    pub owned_amount: u64,
    pub locked_amount: u64,

    pub collected_fees: CollectedFees,
    pub volume_stats: VolumeStats,
    pub trade_stats: TradeStats,

    pub bump: u8,
}

impl Custody {
    pub const LEN: usize = 8 + std::mem::size_of::<Custody>();

    pub fn validate(&self) -> bool {
        matches!(self.oracle_type, OracleType::None)
            || (self.oracle_account != Pubkey::default() && self.max_oracle_price_error >= 0.0)
    }
}
