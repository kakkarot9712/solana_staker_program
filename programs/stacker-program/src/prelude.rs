use anchor_lang::prelude::*;

// Frequently Used Seeds/Constants in Instructions
pub const DISCREMENATOR: usize = 8;
pub const STACKER: &str = "stacker";
// pub const ESCROW_PDA_SIGNER_SEEDS: &[&[&[u8]]] = &[&[ESCROW.as_bytes(), &[bump_seed]]];

// Frequently Used Accounts in Instructions
#[account]
#[derive(InitSpace)]
pub struct StackerMetadata {
    pub decimals: u8,
    pub mint: Pubkey,
    pub collection_mint: Pubkey,
    pub owner: Pubkey,
    pub is_disabled: bool,
    pub reward_wallet: Pubkey,
    pub force_unlock_tax: u8
}

#[account]
#[derive(InitSpace)]
pub struct StackAssetMetadata {
    pub duration: i64,
    pub stacked_at: i64,
    #[max_len(10)]
    pub asset_keys: Vec<Pubkey>,
    pub is_nft: bool,
    pub total_amount: u64,
    pub remaining_amount: u64,
    pub is_cleared: bool,
}

#[account]
#[derive(InitSpace)]
pub struct UserStackPda {
    pub stacked_tokens: u64,
    pub stacked_nfts: u32,
    pub is_disabled: bool
}
