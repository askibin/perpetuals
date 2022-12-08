//! Oracle price service handling

use {
    crate::{error::PerpetualsError, math, state},
    anchor_lang::prelude::*,
};

const ORACLE_EXPONENT_SCALE: i32 = -9;
const ORACLE_PRICE_SCALE: u64 = 1_000_000_000;
const ORACLE_MAX_PRICE: u64 = (1 << 28) - 1;

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum OracleType {
    None,
    Test,
    Pyth,
}

impl Default for OracleType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OraclePrice {
    pub price: u64,
    pub exponent: i32,
}

#[account]
#[derive(Default, Debug)]
pub struct TestOracle {
    pub price: u64,
    pub expo: i32,
    pub conf: u64,
    pub publish_time: i64,
}

impl TestOracle {
    pub const LEN: usize = 8 + std::mem::size_of::<TestOracle>();
}

#[allow(dead_code)]
impl OraclePrice {
    pub fn new(price: u64, exponent: i32) -> Self {
        Self { price, exponent }
    }

    pub fn new_from_token(amount_and_decimals: (u64, u8)) -> Self {
        Self {
            price: amount_and_decimals.0,
            exponent: -(amount_and_decimals.1 as i32),
        }
    }

    pub fn new_from_oracle(
        oracle_type: OracleType,
        oracle_account: &AccountInfo,
        max_price_error: f64,
        max_price_age_sec: u32,
        current_time: i64,
    ) -> Result<Self> {
        match oracle_type {
            OracleType::Test => Self::get_test_price(
                oracle_account,
                max_price_error,
                max_price_age_sec,
                current_time,
            ),
            OracleType::Pyth => Self::get_pyth_price(
                oracle_account,
                max_price_error,
                max_price_age_sec,
                current_time,
            ),
            _ => err!(PerpetualsError::UnsupportedOracle),
        }
    }

    // Converts token amount to USD using oracle price
    pub fn get_asset_value_usd(&self, token_amount: u64, token_decimals: u8) -> Result<f64> {
        if token_amount == 0 {
            return Ok(0.0);
        }
        let res = token_amount as f64
            * self.price as f64
            * math::checked_powi(
                10.0,
                math::checked_sub(self.exponent, token_decimals as i32)?,
            )?;
        if res.is_finite() {
            Ok(res)
        } else {
            err!(PerpetualsError::MathOverflow)
        }
    }

    /// Returns price with mantissa normalized to be less than ORACLE_MAX_PRICE
    pub fn normalize(&self) -> Result<OraclePrice> {
        let mut p = self.price;
        let mut e = self.exponent;

        while p > ORACLE_MAX_PRICE {
            p = math::checked_div(p, 10)?;
            e = math::checked_add(e, 1)?;
        }

        Ok(OraclePrice {
            price: p,
            exponent: e,
        })
    }

    pub fn checked_div(&self, other: &OraclePrice) -> Result<OraclePrice> {
        let base = self.normalize()?;
        let other = other.normalize()?;

        Ok(OraclePrice {
            price: math::checked_div(
                math::checked_mul(base.price, ORACLE_PRICE_SCALE)?,
                other.price,
            )?,
            exponent: math::checked_sub(
                math::checked_add(base.exponent, ORACLE_EXPONENT_SCALE)?,
                other.exponent,
            )?,
        })
    }

    pub fn checked_mul(&self, other: &OraclePrice) -> Result<OraclePrice> {
        Ok(OraclePrice {
            price: math::checked_mul(self.price, other.price)?,
            exponent: math::checked_add(self.exponent, other.exponent)?,
        })
    }

    pub fn scale_to_exponent(&self, target_exponent: i32) -> Result<OraclePrice> {
        if target_exponent == self.exponent {
            return Ok(*self);
        }
        let delta = math::checked_sub(target_exponent, self.exponent)?;
        if delta > 0 {
            Ok(OraclePrice {
                price: math::checked_div(self.price, math::checked_pow(10, delta as usize)?)?,
                exponent: target_exponent,
            })
        } else {
            Ok(OraclePrice {
                price: math::checked_mul(self.price, math::checked_pow(10, (-delta) as usize)?)?,
                exponent: target_exponent,
            })
        }
    }

    pub fn checked_as_f64(&self) -> Result<f64> {
        math::checked_float_mul(self.price as f64, math::checked_powi(10.0, self.exponent)?)
    }

    // private helpers
    fn get_test_price(
        test_price_info: &AccountInfo,
        max_price_error: f64,
        max_price_age_sec: u32,
        current_time: i64,
    ) -> Result<OraclePrice> {
        require!(
            !state::is_empty_account(test_price_info)?,
            PerpetualsError::InvalidOracleAccount
        );

        let oracle_acc = Account::<TestOracle>::try_from(test_price_info)?;

        let last_update_age_sec = math::checked_sub(current_time, oracle_acc.publish_time)?;
        if last_update_age_sec > max_price_age_sec as i64 {
            msg!("Error: Test oracle price is stale");
            return err!(PerpetualsError::StaleOraclePrice);
        }

        if oracle_acc.price == 0
            || math::checked_float_div(oracle_acc.conf as f64, oracle_acc.price as f64)?
                > max_price_error
        {
            msg!("Error: Test oracle price is out of bounds");
            return err!(PerpetualsError::InvalidOraclePrice);
        }

        Ok(OraclePrice {
            // price is i64 and > 0 per check above
            price: oracle_acc.price,
            exponent: oracle_acc.expo,
        })
    }

    fn get_pyth_price(
        pyth_price_info: &AccountInfo,
        max_price_error: f64,
        max_price_age_sec: u32,
        current_time: i64,
    ) -> Result<OraclePrice> {
        require!(
            !state::is_empty_account(pyth_price_info)?,
            PerpetualsError::InvalidOracleAccount
        );
        let price_feed = pyth_sdk_solana::load_price_feed_from_account_info(pyth_price_info)
            .map_err(|_| PerpetualsError::InvalidOracleAccount)?;
        let pyth_price = price_feed
            .get_current_price()
            .ok_or(PerpetualsError::InvalidOracleState)?;

        let last_update_age_sec = math::checked_sub(current_time, price_feed.publish_time)?;
        if last_update_age_sec > max_price_age_sec as i64 {
            msg!("Error: Pyth oracle price is stale");
            return err!(PerpetualsError::StaleOraclePrice);
        }

        if pyth_price.price <= 0
            || math::checked_float_div(pyth_price.conf as f64, pyth_price.price as f64)?
                > max_price_error
        {
            msg!("Error: Pyth oracle price is out of bounds");
            return err!(PerpetualsError::InvalidOraclePrice);
        }

        Ok(OraclePrice {
            // price is i64 and > 0 per check above
            price: pyth_price.price as u64,
            exponent: pyth_price.expo,
        })
    }
}
