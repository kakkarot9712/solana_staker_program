use crate::errors::{ChangeWalletStateError, UnlockNFTErrors};
use crate::events::UnlockAssetsEvent;
use crate::prelude::*;
use anchor_lang::prelude::*;
use mpl_core::accounts::BaseCollectionV1;

// TODO: Debug NFT Already Unlocked Issue
pub fn unlock_nfts<'info>(
    ctx: Context<'_, '_, '_, 'info, UnlockNfts<'info>>,
    _seed_id: String,
) -> Result<()> {
    require!(
        !ctx.accounts.user_stack_pda.is_disabled,
        ChangeWalletStateError::WalletDisabled
    );
    require!(
        !ctx.accounts.stacker_metadata.is_disabled,
        ChangeWalletStateError::StackerProgramDisabled
    );
    require!(
        !ctx.accounts.stack_asset_metadata.is_cleared,
        UnlockNFTErrors::AlreadyReleased
    );
    require!(
        ctx.accounts.collection.key() == ctx.accounts.stacker_metadata.collection_mint.key(),
        UnlockNFTErrors::UnsupportedCollection
    );
    // Check if duration is completed
    let current_time = ctx.accounts.clock.unix_timestamp;
    let locking_time_required =
        ctx.accounts.stack_asset_metadata.duration + ctx.accounts.stack_asset_metadata.stacked_at;
    require!(
        current_time > locking_time_required,
        UnlockNFTErrors::LockDurationNotCompleted
    );

    // Validate all NFT Addresses
    let nft_keys_to_unlock = &ctx.accounts.stack_asset_metadata.asset_keys;
    require!(
        nft_keys_to_unlock.len() == ctx.remaining_accounts.len(),
        UnlockNFTErrors::InvalidNFTAccountsPassed
    );
    let nft_key_iter = nft_keys_to_unlock.iter();
    let mut current_error: Option<UnlockNFTErrors> = None;

    for account in nft_key_iter {
        if ctx
            .remaining_accounts
            .iter()
            .find(|a| a.key() == account.key())
            .is_none()
        {
            current_error = Some(UnlockNFTErrors::InvalidNFTAccountsPassed);
            break;
        }
    }

    if let Some(e) = current_error {
        Err(e.into())
    } else {
        for nft in ctx.remaining_accounts.iter() {
            mpl_core::instructions::TransferV1Cpi {
                asset: nft,
                collection: Some(ctx.accounts.collection.as_ref()),
                payer: ctx.accounts.user.to_account_info().as_ref(),
                authority: Some(ctx.accounts.user_stack_pda.to_account_info().as_ref()),
                new_owner: ctx.accounts.user.to_account_info().as_ref(),
                system_program: None,
                log_wrapper: None,
                __program: ctx.accounts.mpl_core.as_ref(),
                __args: mpl_core::instructions::TransferV1InstructionArgs {
                    compression_proof: None,
                },
            }
            .invoke_signed(&[&[
                STACKER.as_bytes(),
                ctx.accounts.user.key().as_ref(),
                &[ctx.bumps.user_stack_pda],
            ]])?;
            ctx.accounts.user_stack_pda.stacked_nfts -= 1;
        }
        ctx.accounts.stack_asset_metadata.is_cleared = true;
        ctx.accounts.stack_asset_metadata.remaining_amount = 0;
        emit!(UnlockAssetsEvent {
            amount: ctx.accounts.stack_asset_metadata.total_amount,
            unlocked_at: ctx.accounts.clock.unix_timestamp,
            is_nft: true,
            taxed_amount: 0,
            lock_asset_metadata: ctx.accounts.stack_asset_metadata.key(),
            nft_keys: ctx.accounts.stack_asset_metadata.asset_keys.clone(),
        });
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(seed_id: String)]
pub struct UnlockNfts<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [STACKER.as_bytes(), b"metadata"],
        bump
    )]
    pub stacker_metadata: Account<'info, StackerMetadata>,

    #[account(
        mut,
        seeds = [STACKER.as_bytes(), b"nft", user.key().as_ref(), seed_id.as_bytes()],
        bump
    )]
    pub stack_asset_metadata: Account<'info, StackAssetMetadata>,

    #[account(
        mut,
        seeds = [STACKER.as_bytes(), user.key().as_ref()],
        bump
    )]
    pub user_stack_pda: Account<'info, UserStackPda>,
    pub collection: Account<'info, BaseCollectionV1>,
    pub clock: Sysvar<'info, Clock>,

    /// CHECK: Checked in mpl-core.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
}
