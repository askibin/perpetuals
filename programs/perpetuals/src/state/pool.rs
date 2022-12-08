use {
    crate::{
        error::PerpetualsError,
        state::{
            perpetuals::Fee,
            position::{Position, Side},
        },
    },
    anchor_lang::prelude::*,
};

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct Token {
    pub ratio: u64,
    pub custody: Pubkey,
}

#[account]
#[derive(Default, Debug)]
pub struct Pool {
    pub name: String,
    pub tokens: Vec<Token>,

    pub bump: u8,
}

impl Pool {
    pub const LEN: usize = 8 + std::mem::size_of::<Pool>();

    pub fn get_token_id(&self, custody: &Pubkey) -> Result<usize> {
        self.tokens
            .iter()
            .position(|&k| k.custody == *custody)
            .ok_or(PerpetualsError::UnsupportedToken.into())
    }

    pub fn get_entry_price(&self, token_id: usize, side: Side) -> Result<u64> {
        Ok(0)
    }

    pub fn get_exit_price(&self, position: &Position) -> Result<u64> {
        Ok(0)
    }

    pub fn get_entry_fee(&self, token_id: usize, side: Side, size: u64) -> Result<Fee> {
        Ok(Fee::default())
    }

    pub fn get_exit_fee(&self, position: &Position) -> Result<Fee> {
        Ok(Fee::default())
    }

    pub fn get_amount_limit(&self, token_id: usize) -> Result<u64> {
        Ok(0)
    }

    pub fn get_interest_amount(&self, position: &Position) -> Result<u64> {
        Ok(0)
    }

    pub fn check_leverage(&self, position: &Position) -> Result<bool> {
        Ok(true)
    }

    pub fn lock_funds(&self, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn unlock_funds(&self, amount: u64) -> Result<()> {
        Ok(())
    }
}
