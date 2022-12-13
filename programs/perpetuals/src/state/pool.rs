use {
    crate::{
        error::PerpetualsError,
        state::{
            math,
            oracle::OraclePrice,
            perpetuals::Fee,
            position::{Position, Side},
        },
    },
    anchor_lang::prelude::*,
};

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct Token {
    pub custody: Pubkey,
    pub ratio: u64,
}

#[account]
#[derive(Default, Debug)]
pub struct Pool {
    pub name: String,
    pub tokens: Vec<Token>,
    pub lp_token: Pubkey,

    pub bump: u8,
    pub inception_time: i64,
}

impl Pool {
    pub const LEN: usize = 8 + std::mem::size_of::<Pool>();
    pub const LP_DECIMALS: u8 = OraclePrice::USD_DECIMALS;

    pub fn get_token_id(&self, custody: &Pubkey) -> Result<usize> {
        self.tokens
            .iter()
            .position(|&k| k.custody == *custody)
            .ok_or(PerpetualsError::UnsupportedToken.into())
    }

    pub fn get_entry_price(
        &self,
        token_id: usize,
        token_price: &OraclePrice,
        side: Side,
    ) -> Result<u64> {
        Ok(0)
    }

    pub fn get_exit_price(&self, position: &Position, token_price: &OraclePrice) -> Result<u64> {
        Ok(0)
    }

    pub fn get_entry_fee(&self, token_id: usize, side: Side, size: u64) -> Result<Fee> {
        Ok(Fee::default())
    }

    pub fn get_exit_fee(&self, position: &Position) -> Result<Fee> {
        Ok(Fee::default())
    }

    pub fn get_swap_amount(
        &self,
        token_id_in: usize,
        token_id_out: usize,
        token_in_price: &OraclePrice,
        token_out_price: &OraclePrice,
        amount: u64,
    ) -> Result<u64> {
        Ok(0)
    }

    pub fn get_swap_fee(&self, token_id_in: usize, token_id_out: usize) -> Result<Fee> {
        Ok(Fee::default())
    }

    pub fn get_add_liquidity_fee(&self, token_id: usize) -> Result<Fee> {
        Ok(Fee::default())
    }

    pub fn get_remove_liquidity_fee(&self, token_id: usize) -> Result<Fee> {
        Ok(Fee::default())
    }

    pub fn get_amount_limit(&self, token_id: usize) -> Result<u64> {
        Ok(0)
    }

    pub fn check_amount_in(&self, token_id: usize, amount: u64) -> Result<bool> {
        let amount_limit = self.get_amount_limit(token_id)?;
        let new_pool_amount = math::checked_add(self.tokens[token_id].total_amount, amount)?;
        Ok(amount_limit >= new_pool_amount)
    }

    pub fn check_amount_out(&self, token_id: usize, amount: u64) -> Result<bool> {
        // TODO
        let new_pool_amount = math::checked_sub(self.tokens[token_id].total_amount, amount)?;
        Ok(true)
    }

    pub fn get_interest_amount(&self, position: &Position) -> Result<u64> {
        Ok(0)
    }

    pub fn get_leverage(&self, position: &Position) -> Result<u64> {
        Ok(0)
    }

    pub fn check_leverage(&self, position: &Position) -> Result<bool> {
        Ok(true)
    }

    pub fn get_liquidation_price(&self, position: &Position) -> Result<u64> {
        Ok(0)
    }

    pub fn get_pnl(&self, position: &Position) -> Result<u64> {
        Ok(0)
    }

    pub fn lock_funds(&self, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn unlock_funds(&self, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn get_assets_under_management<'a>(
        &self,
        accounts: &[AccountInfo<'a>],
        skip_token_id: usize,
        curtime: i64,
    ) -> Result<u128> {
        if self.tokens.len() > 1 && accounts.len() < (self.tokens.len() - 1) * 2 {
            return Err(ProgramError::NotEnoughAccountKeys.into());
        }
        let pool_amount_usd: u128 = 0;
        for (idx, &token) in self.tokens.iter().enumerate() {
            if idx != skip_token_id {
                require_keys_eq!(accounts[idx].key(), token.custody);
                let custody = Account::<Custody>::try_from(&accounts[idx])?;
                require_keys_eq!(
                    accounts[idx + self.tokens.len() - 1].key(),
                    custody.oracle_account
                );
                let token_price = OraclePrice::new_from_oracle(
                    custody.oracle_type,
                    &accounts[idx + self.tokens.len() - 1],
                    custody.max_oracle_price_error,
                    custody.max_oracle_price_age_sec,
                    curtime,
                )?;
                let token_amount_usd =
                    token_price.get_asset_amount_usd(custody.owned_amount, custody.decimals)?;

                pool_amount_usd = math::checked_add(pool_amount_usd, token_amount_usd as u128)?;
            }
        }
        Ok(pool_amount_usd)
    }
}
