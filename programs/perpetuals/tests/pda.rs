use solana_sdk::pubkey::Pubkey;

pub fn get_multisig_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&["multisig".as_ref()], &perpetuals::id())
}

pub fn get_transfer_authority_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&["transfer_authority".as_ref()], &perpetuals::id())
}

pub fn get_perpetuals_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&["perpetuals".as_ref()], &perpetuals::id())
}

pub fn get_program_data_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&perpetuals::id().as_ref()],
        &solana_program::bpf_loader_upgradeable::id(),
    )
}

pub fn get_pool_pda(name: String) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["pool".as_ref(), name.as_bytes()], &perpetuals::id())
}

pub fn get_lp_token_mint_pda(pool_pda: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&["lp_token_mint".as_ref(), pool_pda.as_ref()], &perpetuals::id())
}
