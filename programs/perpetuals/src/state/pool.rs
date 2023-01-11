use std::{ops::Add, process::exit};

use anchor_spl::token;

use {
    crate::{
        error::PerpetualsError,
        state::{
            custody::{Custody, FeesMode},
            math,
            oracle::OraclePrice,
            perpetuals::Perpetuals,
            position::{Position, Side},
        },
    },
    anchor_lang::prelude::*,
};

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct PoolToken {
    pub custody: Pubkey,

    // ratios have implied BPS_DECIMALS decimals
    pub target_ratio: u64,
    pub min_ratio: u64,
    pub max_ratio: u64,
}

#[account]
#[derive(Default, Debug)]
pub struct Pool {
    pub name: String,
    pub tokens: Vec<PoolToken>,
    pub aum_usd: u128,

    pub bump: u8,
    pub lp_token_bump: u8,
    pub inception_time: i64,
}

/// Token Pool
/// All returned prices are scaled to PRICE_DECIMALS.
/// All returned amounts are scaled to corresponding custody decimals.
///
impl Pool {
    pub const LEN: usize = 8 + std::mem::size_of::<Pool>();

    pub fn get_token_id(&self, custody: &Pubkey) -> Result<usize> {
        self.tokens
            .iter()
            .position(|&k| k.custody == *custody)
            .ok_or(PerpetualsError::UnsupportedToken.into())
    }

    pub fn get_entry_price(
        &self,
        token_price: &OraclePrice,
        token_ema_price: &OraclePrice,
        side: Side,
        custody: &Custody,
    ) -> Result<u64> {
        let price = self.get_price(
            token_price,
            token_ema_price,
            side,
            custody.pricing.trade_spread_long,
            custody.pricing.trade_spread_short,
        )?;
        Ok(price
            .scale_to_exponent(-(Perpetuals::PRICE_DECIMALS as i32))?
            .price)
    }

    pub fn get_entry_fee(
        &self,
        token_id: usize,
        collateral: u64,
        size: u64,
        side: Side,
        custody: &Custody,
        token_price: &OraclePrice,
    ) -> Result<u64> {
        let collateral_fee =
            self.get_add_liquidity_fee(token_id, collateral, custody, token_price)?;
        let size_fee = Self::get_fee_amount(custody.fees.open_position, size)?;
        math::checked_add(collateral_fee, size_fee)
    }

    pub fn get_exit_price(
        &self,
        position: &Position,
        token_price: &OraclePrice,
        token_ema_price: &OraclePrice,
        custody: &Custody,
    ) -> Result<u64> {
        let price = self.get_price(
            token_price,
            token_ema_price,
            if position.side == Side::Long {
                Side::Short
            } else {
                Side::Long
            },
            custody.pricing.trade_spread_long,
            custody.pricing.trade_spread_short,
        )?;
        Ok(price
            .scale_to_exponent(-(Perpetuals::PRICE_DECIMALS as i32))?
            .price)
    }

    pub fn get_exit_fee(
        &self,
        position: &Position,
        collateral: u64,
        size: u64,
        custody: &Custody,
        token_price: &OraclePrice,
    ) -> Result<u64> {
        let collateral_fee = self.get_remove_liquidity_fee(
            position.token_id as usize,
            collateral,
            custody,
            token_price,
        )?;
        let size_fee = Self::get_fee_amount(custody.fees.close_position, size)?;
        math::checked_add(collateral_fee, size_fee)
    }

    pub fn get_close_amount(
        &self,
        position: &Position,
        token_price: &OraclePrice,
        token_ema_price: &OraclePrice,
        custody: &Custody,
        size: u64,
    ) -> Result<u64> {
        let (profit_usd, loss_usd) =
            self.get_pnl_usd(position, token_price, token_ema_price, custody)?;
        let unrealized_profit_usd = math::checked_add(position.unrealized_profit_usd, profit_usd)?;
        let unrealized_loss_usd = math::checked_add(position.unrealized_loss_usd, loss_usd)?;
        let mut available_amount_usd =
            math::checked_add(position.collateral_usd, unrealized_profit_usd)?;
        available_amount_usd = if unrealized_loss_usd < available_amount_usd {
            math::checked_sub(available_amount_usd, unrealized_loss_usd)?
        } else {
            0
        };
        let size_usd = token_price.get_asset_amount_usd(size, custody.decimals)?;
        let close_amount_usd = if size_usd >= position.size_usd {
            math::checked_as_u64(math::checked_div(
                math::checked_mul(available_amount_usd as u128, size_usd as u128)?,
                position.size_usd as u128,
            )?)?
        } else {
            available_amount_usd
        };
        token_price.get_token_amount(close_amount_usd, custody.decimals)
    }

    pub fn get_swap_price(
        &self,
        token_in_price: &OraclePrice,
        token_in_ema_price: &OraclePrice,
        token_out_price: &OraclePrice,
        token_out_ema_price: &OraclePrice,
        custody_in: &Custody,
    ) -> Result<OraclePrice> {
        let min_price = if token_in_price < token_in_ema_price {
            token_in_price
        } else {
            token_in_ema_price
        };
        let max_price = if token_out_price > token_out_ema_price {
            token_out_price
        } else {
            token_out_ema_price
        };
        let pair_price = min_price.checked_div(max_price)?;
        self.get_price(
            &pair_price,
            &pair_price,
            Side::Short,
            custody_in.pricing.swap_spread,
            custody_in.pricing.swap_spread,
        )
    }

    pub fn get_swap_amount(
        &self,
        token_id_in: usize,
        token_id_out: usize,
        token_in_price: &OraclePrice,
        token_in_ema_price: &OraclePrice,
        token_out_price: &OraclePrice,
        token_out_ema_price: &OraclePrice,
        custody_in: &Custody,
        custody_out: &Custody,
        amount_in: u64,
    ) -> Result<u64> {
        let swap_price = self.get_swap_price(
            token_in_price,
            token_in_ema_price,
            token_out_price,
            token_out_ema_price,
            custody_in,
        )?;
        math::checked_decimal_mul(
            amount_in,
            -(custody_in.decimals as i32),
            swap_price.price,
            swap_price.exponent,
            -(custody_out.decimals as i32),
        )
    }

    pub fn get_swap_fees(
        &self,
        token_id_in: usize,
        token_id_out: usize,
        amount_in: u64,
        amount_out: u64,
        custody_in: &Custody,
        token_price_in: &OraclePrice,
        custody_out: &Custody,
        token_price_out: &OraclePrice,
    ) -> Result<(u64, u64)> {
        let add_liquidity_fee =
            self.get_add_liquidity_fee(token_id_in, amount_in, custody_in, token_price_in)?;
        let remove_liquidity_fee =
            self.get_remove_liquidity_fee(token_id_out, amount_out, custody_out, token_price_out)?;
        let swap_fee = Self::get_fee_amount(custody_out.fees.swap, amount_out)?;
        Ok((
            add_liquidity_fee,
            math::checked_add(remove_liquidity_fee, swap_fee)?,
        ))
    }

    pub fn get_add_liquidity_fee(
        &self,
        token_id: usize,
        amount: u64,
        custody: &Custody,
        token_price: &OraclePrice,
    ) -> Result<u64> {
        self.get_fee(
            token_id,
            custody.fees.add_liquidity,
            amount,
            0u64,
            custody,
            token_price,
        )
    }

    pub fn get_remove_liquidity_fee(
        &self,
        token_id: usize,
        amount: u64,
        custody: &Custody,
        token_price: &OraclePrice,
    ) -> Result<u64> {
        self.get_fee(
            token_id,
            custody.fees.remove_liquidity,
            0u64,
            amount,
            custody,
            token_price,
        )
    }

    pub fn check_amount_in_out(
        &self,
        token_id: usize,
        amount_add: u64,
        amount_remove: u64,
        custody: &Custody,
        token_price: &OraclePrice,
    ) -> Result<bool> {
        let new_ratio = self.get_new_ratio(amount_add, amount_remove, custody, token_price)?;
        Ok(new_ratio <= self.tokens[token_id].max_ratio
            && new_ratio >= self.tokens[token_id].min_ratio)
    }

    pub fn get_interest_amount(
        &self,
        position: &Position,
        custody: &Custody,
        curtime: i64,
    ) -> Result<u64> {
        let time_diff = if curtime > position.update_time {
            math::checked_sub(curtime, position.update_time)? as u128
        } else {
            return Ok(0);
        };

        let interest_per_sec = math::checked_div(
            math::checked_mul(
                custody.pricing.interest_per_sec as u128,
                position.size_usd as u128,
            )?,
            Perpetuals::BPS_POWER,
        )?;
        let interest_per_sec = math::checked_decimal_mul(
            custody.pricing.interest_per_sec,
            -(Perpetuals::BPS_DECIMALS as i32),
            position.size_usd,
            -(custody.decimals as i32),
            -(custody.decimals as i32),
        )?;

        math::checked_as_u64(math::checked_mul(time_diff, interest_per_sec as u128)?)
    }

    pub fn get_leverage(
        &self,
        position: &Position,
        token_price: &OraclePrice,
        token_ema_price: &OraclePrice,
        custody: &Custody,
    ) -> Result<u64> {
        let (profit_usd, loss_usd) =
            self.get_pnl_usd(position, token_price, token_ema_price, custody)?;
        let current_margin_usd = if profit_usd > 0 {
            math::checked_add(position.collateral_usd, profit_usd)?
        } else if loss_usd <= position.collateral_usd {
            math::checked_sub(position.collateral_usd, loss_usd)?
        } else {
            0
        };
        if current_margin_usd > 0 {
            math::checked_as_u64(math::checked_div(
                math::checked_mul(position.size_usd as u128, Perpetuals::BPS_POWER)?,
                current_margin_usd as u128,
            )?)
        } else {
            Ok(u64::MAX)
        }
    }

    pub fn get_initial_leverage(&self, position: &Position) -> Result<u64> {
        math::checked_as_u64(math::checked_div(
            math::checked_mul(position.size_usd as u128, Perpetuals::BPS_POWER)?,
            position.collateral_usd as u128,
        )?)
    }

    pub fn check_leverage(
        &self,
        position: &Position,
        token_price: &OraclePrice,
        token_ema_price: &OraclePrice,
        custody: &Custody,
        initial: bool,
    ) -> Result<bool> {
        let current_leverage =
            self.get_leverage(position, token_price, token_ema_price, custody)?;
        Ok(current_leverage <= custody.pricing.max_leverage
            && (!initial || current_leverage >= custody.pricing.min_initial_leverage))
    }

    pub fn get_liquidation_price(
        &self,
        position: &Position,
        token_price: &OraclePrice,
        custody: &Custody,
    ) -> Result<u64> {
        let exit_fee = self.get_exit_fee(
            position,
            position.collateral_usd,
            position.size_usd,
            custody,
            token_price,
        )?;
        let max_loss_usd = math::checked_as_u64(math::checked_add(
            math::checked_div(
                math::checked_mul(position.size_usd as u128, Perpetuals::BPS_POWER)?,
                custody.pricing.max_leverage as u128,
            )?,
            exit_fee as u128,
        )?)?;
        let initial_leverage = self.get_initial_leverage(position)?;

        let token_price_dec = token_price
            .scale_to_exponent(-(Perpetuals::PRICE_DECIMALS as i32))?
            .price;
        let max_price_diff = if max_loss_usd >= position.collateral_usd {
            math::checked_div(
                math::checked_mul(
                    math::checked_sub(max_loss_usd, position.collateral_usd)?,
                    token_price_dec,
                )?,
                initial_leverage,
            )?
        } else {
            math::checked_div(
                math::checked_mul(
                    math::checked_sub(position.collateral_usd, max_loss_usd)?,
                    token_price_dec,
                )?,
                initial_leverage,
            )?
        };

        if position.side == Side::Long {
            if max_loss_usd >= position.collateral_usd {
                math::checked_add(position.price, max_price_diff)
            } else {
                math::checked_sub(position.price, max_price_diff)
            }
        } else {
            if max_loss_usd >= position.collateral_usd {
                math::checked_sub(position.price, max_price_diff)
            } else {
                math::checked_add(position.price, max_price_diff)
            }
        }
    }

    pub fn get_pnl_usd(
        &self,
        position: &Position,
        token_price: &OraclePrice,
        token_ema_price: &OraclePrice,
        custody: &Custody,
    ) -> Result<(u64, u64)> {
        let size = token_price.get_token_amount(position.size_usd, custody.decimals)?;
        let collateral = token_price.get_token_amount(position.collateral_usd, custody.decimals)?;
        let exit_price = self.get_exit_price(position, token_price, token_ema_price, custody)?;
        let exit_fee = self.get_exit_fee(position, collateral, size, custody, token_price)?;
        let exit_fee_usd = token_price.get_asset_amount_usd(exit_fee, custody.decimals)?;

        let (price_diff_profit, price_diff_loss) = if position.side == Side::Long {
            if exit_price > position.price {
                (math::checked_sub(exit_price, position.price)?, 0u64)
            } else {
                (0u64, math::checked_sub(position.price, exit_price)?)
            }
        } else {
            if exit_price < position.price {
                (math::checked_sub(position.price, exit_price)?, 0u64)
            } else {
                (0u64, math::checked_sub(exit_price, position.price)?)
            }
        };

        let position_leverage = self.get_initial_leverage(position)?;
        if price_diff_profit > 0 {
            let potential_profit_usd = math::checked_as_u64(math::checked_mul(
                price_diff_profit as u128,
                position_leverage as u128,
            )?)?;

            if potential_profit_usd >= exit_fee_usd {
                Ok((math::checked_sub(potential_profit_usd, exit_fee_usd)?, 0u64))
            } else {
                Ok((0u64, math::checked_sub(exit_fee_usd, potential_profit_usd)?))
            }
        } else {
            let loss_usd = math::checked_mul(price_diff_loss, position_leverage)?;

            Ok((0u64, math::checked_add(loss_usd, exit_fee_usd)?))
        }
    }

    pub fn lock_funds(&self, amount: u64, custody: &mut Custody) -> Result<()> {
        custody.assets.locked = math::checked_add(custody.assets.locked, amount)?;
        if custody.assets.owned < custody.assets.locked {
            Err(ProgramError::InsufficientFunds.into())
        } else {
            Ok(())
        }
    }

    pub fn unlock_funds(&self, amount: u64, custody: &mut Custody) -> Result<()> {
        if amount > custody.assets.locked {
            custody.assets.locked = 0;
        } else {
            custody.assets.locked = math::checked_sub(custody.assets.locked, amount)?;
        }
        Ok(())
    }

    pub fn get_assets_under_management_usd<'a>(
        &self,
        accounts: &[AccountInfo<'a>],
        skip_token_id: usize,
        curtime: i64,
    ) -> Result<u128> {
        let mut pool_amount_usd: u128 = 0;
        let mut acc_idx: usize = 0;
        for (idx, &token) in self.tokens.iter().enumerate() {
            if idx != skip_token_id {
                let oracle_idx = acc_idx + self.tokens.len() - 1;
                if oracle_idx >= accounts.len() {
                    return Err(ProgramError::NotEnoughAccountKeys.into());
                }
                require_keys_eq!(accounts[acc_idx].key(), token.custody);
                let custody = Account::<Custody>::try_from(&accounts[acc_idx])?;
                require_keys_eq!(accounts[oracle_idx].key(), custody.oracle.oracle_account);
                let token_price = OraclePrice::new_from_oracle(
                    custody.oracle.oracle_type,
                    &accounts[oracle_idx],
                    custody.oracle.max_price_error,
                    custody.oracle.max_price_age_sec,
                    curtime,
                )?;
                let token_amount_usd =
                    token_price.get_asset_amount_usd(custody.assets.owned, custody.decimals)?;

                pool_amount_usd = math::checked_add(pool_amount_usd, token_amount_usd as u128)?;
                acc_idx += 1;
            }
        }
        Ok(pool_amount_usd)
    }

    pub fn get_fee_amount(fee: u64, amount: u64) -> Result<u64> {
        if fee == 0 || amount == 0 {
            return Ok(0);
        }
        math::checked_as_u64(math::checked_ceil_div(
            math::checked_mul(amount as u128, fee as u128)?,
            Perpetuals::BPS_POWER,
        )?)
    }

    // private helpers
    fn get_new_ratio(
        &self,
        amount_add: u64,
        amount_remove: u64,
        custody: &Custody,
        token_price: &OraclePrice,
    ) -> Result<u64> {
        let new_aum_usd = token_price.get_asset_amount_usd(
            math::checked_sub(
                math::checked_add(custody.assets.owned, amount_add)?,
                amount_remove,
            )?,
            custody.decimals,
        )? as u128;
        math::checked_as_u64(math::checked_div(
            math::checked_mul(new_aum_usd, Perpetuals::BPS_POWER)?,
            self.aum_usd,
        )?)
    }

    fn get_price(
        &self,
        token_price: &OraclePrice,
        token_ema_price: &OraclePrice,
        side: Side,
        spread_long: u64,
        spread_short: u64,
    ) -> Result<OraclePrice> {
        if side == Side::Long {
            let max_price = if token_price > token_ema_price {
                token_price
            } else {
                token_ema_price
            };
            Ok(OraclePrice {
                price: math::checked_add(
                    max_price.price,
                    math::checked_decimal_ceil_mul(
                        max_price.price,
                        max_price.exponent,
                        spread_long,
                        -(Perpetuals::BPS_DECIMALS as i32),
                        max_price.exponent,
                    )?,
                )?,
                exponent: max_price.exponent,
            })
        } else {
            let min_price = if token_price > token_ema_price {
                token_price
            } else {
                token_ema_price
            };
            Ok(OraclePrice {
                price: math::checked_sub(
                    min_price.price,
                    math::checked_decimal_mul(
                        min_price.price,
                        min_price.exponent,
                        spread_short,
                        -(Perpetuals::BPS_DECIMALS as i32),
                        min_price.exponent,
                    )?,
                )?,
                exponent: min_price.exponent,
            })
        }
    }

    fn get_fee(
        &self,
        token_id: usize,
        base_fee: u64,
        amount_add: u64,
        amount_remove: u64,
        custody: &Custody,
        token_price: &OraclePrice,
    ) -> Result<u64> {
        if custody.fees.mode == FeesMode::Fixed {
            return Self::get_fee_amount(base_fee, std::cmp::max(amount_add, amount_remove));
        }
        let token = &self.tokens[token_id];
        let new_ratio = self.get_new_ratio(amount_add, amount_remove, custody, token_price)?;
        let max_fee_change = math::checked_div(
            math::checked_mul(
                math::checked_sub(custody.fees.max_change as u128, Perpetuals::BPS_POWER)?,
                custody.fees.open_position as u128,
            )?,
            Perpetuals::BPS_POWER,
        )?;

        let fee = if new_ratio >= token.target_ratio {
            math::checked_add(
                custody.fees.open_position,
                math::checked_as_u64(math::checked_ceil_div(
                    math::checked_mul(
                        math::checked_sub(
                            std::cmp::min(token.max_ratio, new_ratio),
                            token.target_ratio,
                        )? as u128,
                        max_fee_change,
                    )?,
                    math::checked_sub(token.max_ratio, token.target_ratio)? as u128,
                )?)?,
            )?
        } else {
            math::checked_sub(
                custody.fees.open_position,
                math::checked_as_u64(math::checked_ceil_div(
                    math::checked_mul(
                        math::checked_sub(
                            token.target_ratio,
                            std::cmp::max(token.min_ratio, new_ratio),
                        )? as u128,
                        max_fee_change,
                    )?,
                    math::checked_sub(token.target_ratio, token.min_ratio)? as u128,
                )?)?,
            )?
        };

        Self::get_fee_amount(fee, std::cmp::max(amount_add, amount_remove))
    }
}
