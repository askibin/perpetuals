use {crate::math, anchor_lang::prelude::*, anchor_spl::token::Transfer};

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Default, Debug)]
pub struct Fee {
    numerator: u64,
    denominator: u64,
}

#[account]
#[derive(Default, Debug)]
pub struct Perpetuals {
    pub transfer_authority_bump: u8,
    pub perpetuals_bump: u8,
    // time of inception, also used as current wall clock time for testing
    pub inception_time: i64,
}

impl Fee {
    pub fn is_zero(&self) -> bool {
        self.numerator == 0
    }

    pub fn get_fee_amount(&self, amount: u64) -> Result<u64> {
        if self.is_zero() {
            return Ok(0);
        }
        math::checked_as_u64(math::checked_ceil_div(
            math::checked_mul(amount as u128, self.numerator as u128)?,
            self.denominator as u128,
        )?)
    }
}

impl anchor_lang::Id for Perpetuals {
    fn id() -> Pubkey {
        crate::ID
    }
}

impl Perpetuals {
    pub const LEN: usize = 8 + std::mem::size_of::<Perpetuals>();

    pub fn validate(&self) -> bool {
        true
    }

    #[cfg(feature = "test")]
    pub fn get_time(&self) -> Result<i64> {
        Ok(self.inception_time)
    }

    #[cfg(not(feature = "test"))]
    pub fn get_time(&self) -> Result<i64> {
        let time = solana_program::sysvar::clock::Clock::get()?.unix_timestamp;
        if time > 0 {
            Ok(time)
        } else {
            Err(ProgramError::InvalidAccountData.into())
        }
    }

    pub fn transfer_tokens<'info>(
        &self,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        let authority_seeds: &[&[&[u8]]] =
            &[&[b"transfer_authority", &[self.transfer_authority_bump]]];

        let context = CpiContext::new(
            token_program,
            Transfer {
                from,
                to,
                authority,
            },
        )
        .with_signer(authority_seeds);

        anchor_spl::token::transfer(context, amount)
    }
}
