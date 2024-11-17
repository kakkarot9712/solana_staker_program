use anchor_lang::prelude::*;

#[event]
pub struct LockAssetEvent {
    pub locked_at: i64,
    pub duration: i64,
    pub locked_by: Pubkey,
    pub seeds_index: String,
    pub lock_asset_metadata: Pubkey,
    pub nft_keys: Vec<Pubkey>,
    pub amount: u64,
    pub is_nft: bool,
}

#[event]
pub struct UnlockAssetsEvent {
    pub unlocked_at: i64,
    pub taxed_amount: u64,
    pub lock_asset_metadata: Pubkey,
    pub nft_keys: Vec<Pubkey>,
    pub amount: u64,
    pub is_nft: bool,
}
