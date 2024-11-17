use anchor_lang::prelude::*;
use instructions::*;

pub mod errors;
pub mod events;
pub mod instructions;
pub mod prelude;

declare_id!("9mjoNzkJ4or7VhtCAnFFVmSZnoXvEvXQZ63tq5qLb5Fx");

#[program]
pub mod stacker_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, force_unlock_tax: u8) -> Result<()> {
        instructions::initialize(ctx, force_unlock_tax)
    }

    pub fn create_locker_account(ctx: Context<CreateUserAccount>) -> Result<()> {
        instructions::create_locker_account(ctx)
    }

    pub fn lock_tokens(
        ctx: Context<LockTokens>,
        amount: u64,
        duration: i64,
        seed_id: String,
    ) -> Result<()> {
        instructions::lock_tokens(ctx, amount, duration, seed_id)
    }

    pub fn lock_nft<'info>(
        ctx: Context<'_, '_, '_, 'info, LockNFT<'info>>,
        duration: i64,
        seed_id: String,
    ) -> Result<()> {
        instructions::lock_nft(ctx, duration, seed_id)
    }

    pub fn unlock_tokens(ctx: Context<UnlockTokens>, seed_id: String, force: bool) -> Result<()> {
        instructions::unlock_tokens(ctx, seed_id, force)
    }

    pub fn unlock_nfts<'info>(
        ctx: Context<'_, '_, '_, 'info, UnlockNfts<'info>>,
        seed_id: String,
    ) -> Result<()> {
        instructions::unlock_nfts(ctx, seed_id)
    }

    pub fn change_wallet_state(ctx: Context<ChangeWalletState>, new_state: bool) -> Result<()> {
        instructions::change_wallet_state(ctx, new_state)
    }
}
