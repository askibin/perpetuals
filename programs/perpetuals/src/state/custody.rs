use {crate::state::oracle::OracleType, anchor_lang::prelude::*};

#[account]
#[derive(Default, Debug)]
pub struct Custody {
    pub token_account: Pubkey,
    pub collected_fees: u64,
    pub mint: Pubkey,
    pub decimals: u8,
    pub max_oracle_price_error: f64,
    pub max_oracle_price_age_sec: u32,
    pub oracle_type: OracleType,
    pub oracle_account: Pubkey,
    pub bump: u8,
}

impl Custody {
    pub const LEN: usize = 8 + std::mem::size_of::<Custody>();

    pub fn validate(&self) -> bool {
        matches!(self.oracle_type, OracleType::None)
            || (self.oracle_account != Pubkey::default() && self.max_oracle_price_error >= 0.0)
    }
}
