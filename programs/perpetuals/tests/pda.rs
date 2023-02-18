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
