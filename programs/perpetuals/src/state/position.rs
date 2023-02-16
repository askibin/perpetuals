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

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum CollateralChange {
    None,
    Add,
    Remove,
}

impl Default for CollateralChange {
    fn default() -> Self {
        Self::None
    }
}

#[account]
#[derive(Default, Debug)]
pub struct Position {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub custody: Pubkey,

    pub open_time: i64,
    pub update_time: i64,
    pub side: Side,
    pub price: u64,
    pub size_usd: u64,
    pub collateral_usd: u64,
    pub unrealized_profit_usd: u64,
    pub unrealized_loss_usd: u64,
    pub borrow_rate_sum: u128,
    pub locked_amount: u64,
    pub collateral_amount: u64,

    pub bump: u8,
}

impl Position {
    pub const LEN: usize = 8 + std::mem::size_of::<Position>();
}
