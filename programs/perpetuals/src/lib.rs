//! Perpetuals program entrypoint

#![allow(clippy::result_large_err)]

mod error;
mod instructions;
mod math;
mod state;

use {anchor_lang::prelude::*, instructions::*};

solana_security_txt::security_txt! {
    name: "Perpetuals",
    project_url: "https://github.com/solana-labs/solana-program-library/tree/master/perpetuals",
    contacts: "email:solana.farms@protonmail.com",
    policy: "",
    preferred_languages: "en",
    auditors: ""
}

declare_id!("Psx1bVshnRYzG8PdDFVLfBrYCs9JMwf7gwNMQ3zapXf");

#[program]
pub mod perpetuals {
    use super::*;

    // admin instructions

    // test instructions

    pub fn test_init(ctx: Context<TestInit>, params: TestInitParams) -> Result<()> {
        instructions::test_init(ctx, &params)
    }

    // public instructions

    pub fn open_position(ctx: Context<OpenPosition>, params: OpenPositionParams) -> Result<()> {
        instructions::open_position(ctx, &params)
    }

    pub fn close_position(ctx: Context<ClosePosition>, params: ClosePositionParams) -> Result<()> {
        instructions::close_position(ctx, &params)
    }
}
