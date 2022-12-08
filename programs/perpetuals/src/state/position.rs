use anchor_lang::prelude::*;

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum Side {
    None,
    Long,
    Short,
}

impl Default for Side {
    fn default() -> Self {
        Self::None
    }
}

#[account]
#[derive(Default, Debug)]
pub struct Position {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub token_id: u16,

    pub time: i64,
    pub side: Side,
    pub price: u64,
    pub size: u64,
    pub collateral: u64,
    pub interest_debt: u64,
    pub unrealized_pnl: u64,

    pub bump: u8,
}

impl Position {
    pub const LEN: usize = 8 + std::mem::size_of::<Position>();

    pub fn get_pnl(&self) -> Result<u64> {
        Ok(0)
    }
}
