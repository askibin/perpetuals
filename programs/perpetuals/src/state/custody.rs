use {
    crate::{
        math,
        state::{
            oracle::OracleType,
            perpetuals::{Permissions, Perpetuals},
        },
    },
    anchor_lang::prelude::*,
};

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum FeesMode {
    Fixed,
    Linear,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct Fees {
    pub mode: FeesMode,
    // fees have implied BPS_DECIMALS decimals
    pub max_increase: u64,
    pub max_decrease: u64,
    pub swap: u64,
    pub add_liquidity: u64,
    pub remove_liquidity: u64,
    pub open_position: u64,
    pub close_position: u64,
    pub liquidation: u64,
    pub protocol_share: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct FeesStats {
    pub swap_usd: u64,
    pub add_liquidity_usd: u64,
    pub remove_liquidity_usd: u64,
    pub open_position_usd: u64,
    pub close_position_usd: u64,
    pub liquidation_usd: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct VolumeStats {
    pub swap_usd: u64,
    pub add_liquidity_usd: u64,
    pub remove_liquidity_usd: u64,
    pub open_position_usd: u64,
    pub close_position_usd: u64,
    pub liquidation_usd: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct TradeStats {
    pub profit_usd: u64,
    pub loss_usd: u64,
    // open interest
    pub oi_long_usd: u64,
    pub oi_short_usd: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct Assets {
    // collateral debt
    pub collateral: u64,
    // protocol_fees are part of the collected fees that is reserved for the protocol
    pub protocol_fees: u64,
    // owned = total_assets - collateral + collected_fees - protocol_fees
    pub owned: u64,
    // locked funds for pnl payoff
    pub locked: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct OracleParams {
    pub oracle_account: Pubkey,
    pub oracle_type: OracleType,
    pub max_price_error: u64,
    pub max_price_age_sec: u32,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct PricingParams {
    pub use_ema: bool,
    // pricing params have implied BPS_DECIMALS decimals
    pub trade_spread_long: u64,
    pub trade_spread_short: u64,
    pub swap_spread: u64,
    pub min_initial_leverage: u64,
    pub max_leverage: u64,
    // max_user_profit = position_size * max_payoff_mult
    pub max_payoff_mult: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct BorrowRateParams {
    // borrow rate params have implied RATE_DECIMALS decimals
    pub base_rate: u64,
    pub slope1: u64,
    pub slope2: u64,
    pub optimal_utilization: u64,
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct BorrowRateState {
    // borrow rates have implied RATE_DECIMALS decimals
    pub current_rate: u64,
    pub rate_sum: u128,
    pub last_update: i64,
}

#[account]
#[derive(Default, Debug)]
pub struct Custody {
    // static parameters
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub token_account: Pubkey,
    pub decimals: u8,
    pub is_stable: bool,
    pub oracle: OracleParams,
    pub pricing: PricingParams,
    pub permissions: Permissions,
    pub fees: Fees,
    pub borrow_rate: BorrowRateParams,

    // dynamic variables
    pub assets: Assets,
    pub collected_fees: FeesStats,
    pub volume_stats: VolumeStats,
    pub trade_stats: TradeStats,
    pub borrow_rate_state: BorrowRateState,

    // bumps for address validation
    pub bump: u8,
    pub token_account_bump: u8,
}

#[account]
#[derive(Default, Debug)]
pub struct DeprecatedCustody {
    pub pool: Pubkey,
    pub mint: Pubkey,
    pub token_account: Pubkey,
    pub decimals: u8,
    pub is_stable: bool,
    pub oracle: OracleParams,
    pub pricing: PricingParams,
    pub permissions: Permissions,
    pub fees: Fees,
    pub borrow_rate: u64,
    pub borrow_rate_sum: u64,

    pub assets: Assets,
    pub collected_fees: FeesStats,
    pub volume_stats: VolumeStats,
    pub trade_stats: TradeStats,

    pub bump: u8,
    pub token_account_bump: u8,
}

impl Default for FeesMode {
    fn default() -> Self {
        Self::Linear
    }
}

impl Fees {
    pub fn validate(&self) -> bool {
        self.max_decrease as u128 <= Perpetuals::BPS_POWER
            && self.swap as u128 <= Perpetuals::BPS_POWER
            && self.add_liquidity as u128 <= Perpetuals::BPS_POWER
            && self.remove_liquidity as u128 <= Perpetuals::BPS_POWER
            && self.open_position as u128 <= Perpetuals::BPS_POWER
            && self.close_position as u128 <= Perpetuals::BPS_POWER
            && self.liquidation as u128 <= Perpetuals::BPS_POWER
            && self.protocol_share as u128 <= Perpetuals::BPS_POWER
    }
}

impl OracleParams {
    pub fn validate(&self) -> bool {
        self.oracle_type == OracleType::None || self.oracle_account != Pubkey::default()
    }
}

impl PricingParams {
    pub fn validate(&self) -> bool {
        (self.min_initial_leverage as u128) >= Perpetuals::BPS_POWER
            && self.min_initial_leverage <= self.max_leverage
            && (self.trade_spread_long as u128) < Perpetuals::BPS_POWER
            && (self.trade_spread_short as u128) < Perpetuals::BPS_POWER
            && (self.swap_spread as u128) < Perpetuals::BPS_POWER
            && self.max_payoff_mult > 0
    }
}

impl BorrowRateParams {
    pub fn validate(&self) -> bool {
        self.optimal_utilization > 0 && (self.optimal_utilization as u128) <= Perpetuals::RATE_POWER
    }
}

impl Custody {
    pub const LEN: usize = 8 + std::mem::size_of::<Custody>();

    pub fn validate(&self) -> bool {
        self.token_account != Pubkey::default()
            && self.mint != Pubkey::default()
            && self.oracle.validate()
            && self.pricing.validate()
            && self.fees.validate()
            && self.borrow_rate.validate()
    }

    pub fn update_borrow_rate(&mut self, curtime: i64) -> Result<()> {
        // if current_utilization < optimal_utilization:
        //   rate = base_rate + (current_utilization / optimal_utilization) * slope1
        // else:
        //   rate = base_rate + slope1 + (current_utilization - optimal_utilization) / (1 - optimal_utilization) * slope2

        if curtime <= self.borrow_rate_state.last_update {
            return Ok(());
        }

        let current_utilization = math::checked_div(
            math::checked_mul(self.assets.locked as u128, Perpetuals::RATE_POWER)?,
            self.assets.owned as u128,
        )?;

        let hourly_rate = if current_utilization < (self.borrow_rate.optimal_utilization as u128)
            || (self.borrow_rate.optimal_utilization as u128) >= Perpetuals::RATE_POWER
        {
            math::checked_div(
                math::checked_mul(current_utilization, self.borrow_rate.slope1 as u128)?,
                self.borrow_rate.optimal_utilization as u128,
            )?
        } else {
            math::checked_add(
                self.borrow_rate.slope1 as u128,
                math::checked_div(
                    math::checked_mul(
                        math::checked_sub(
                            current_utilization,
                            self.borrow_rate.optimal_utilization as u128,
                        )?,
                        self.borrow_rate.slope2 as u128,
                    )?,
                    Perpetuals::RATE_POWER - self.borrow_rate.optimal_utilization as u128,
                )?,
            )?
        };
        let hourly_rate = math::checked_add(
            math::checked_as_u64(hourly_rate)?,
            self.borrow_rate.base_rate,
        )?;

        let rate_per_second = math::checked_div(hourly_rate, 3600)?;
        let rate_sum = math::checked_mul(
            math::checked_sub(curtime, self.borrow_rate_state.last_update)? as u128,
            rate_per_second as u128,
        )?;

        self.borrow_rate_state.current_rate = hourly_rate;
        self.borrow_rate_state.rate_sum = rate_sum;
        self.borrow_rate_state.last_update = curtime;

        Ok(())
    }
}

impl DeprecatedCustody {
    pub const LEN: usize = 8 + std::mem::size_of::<DeprecatedCustody>();
}
