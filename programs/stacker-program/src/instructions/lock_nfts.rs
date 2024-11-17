use crate::{
    errors::ChangeWalletStateError, errors::LockNFTErrors, events::LockAssetEvent, prelude::*,
};
use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    types::UpdateAuthority,
};

pub fn lock_nft<'info>(
    ctx: Context<'_, '_, '_, 'info, LockNFT<'info>>,
    duration: i64,
    seed_id: String,
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
        ctx.remaining_accounts.len() <= 10,
        LockNFTErrors::MaxNFTArrayLengthFound
    );
    let mut current_err: Option<LockNFTErrors> = None;
    for account in ctx.remaining_accounts.iter() {
        let asset = BaseAssetV1::from_bytes(account.data.borrow_mut().as_ref());
        if asset.is_err() {
            msg!("NFT Parsing Error: {}", asset.err().unwrap().kind());
            current_err = Some(LockNFTErrors::InvalidNFTAccountsPassed);
            break;
        } else {
            let nft = asset.unwrap();
            let nft_authority = nft.update_authority.clone();
            if let UpdateAuthority::Collection(c) = nft_authority {
                if c != ctx.accounts.stacker_metadata.collection_mint {
                    current_err = Some(LockNFTErrors::UnsupportedCollection);
                    break;
                } else if nft.owner != ctx.accounts.user.key() {
                    current_err = Some(LockNFTErrors::NotNFTOwner);
                    break;
                }
            } else {
                current_err = Some(LockNFTErrors::UnsupportedCollection);
                break;
            }
        }
    }
    if current_err.is_some() {
        // Error
        Err(current_err.unwrap().into())
    } else {
        for nft_acc in ctx.remaining_accounts.iter() {
            // Try to tranfer NFT to escrow PDA
            mpl_core::instructions::TransferV1Cpi {
                asset: nft_acc,
                collection: Some(ctx.accounts.collection.as_ref()),
                payer: &ctx.accounts.user.to_account_info(),
                authority: Some(&ctx.accounts.user.to_account_info()),
                new_owner: &ctx.accounts.user_stack_pda.to_account_info(),
                system_program: None,
                log_wrapper: None,
                __program: &ctx.accounts.mpl_core,
                __args: mpl_core::instructions::TransferV1InstructionArgs {
                    compression_proof: None,
                },
            }
            .invoke()?;
            ctx.accounts.user_stack_pda.stacked_nfts += 1;
        }
        let nft_pubkeys = ctx
            .remaining_accounts
            .iter()
            .map(|c| {
                return c.key().clone();
            })
            .collect();
        let lock_asset_metadata = &mut ctx.accounts.stack_asset_metadata;
        lock_asset_metadata.total_amount = ctx.remaining_accounts.len() as u64;
        lock_asset_metadata.stacked_at = ctx.accounts.clock.unix_timestamp;
        lock_asset_metadata.is_nft = true;
        lock_asset_metadata.is_cleared = false;
        lock_asset_metadata.duration = duration;
        lock_asset_metadata.asset_keys = nft_pubkeys;
        lock_asset_metadata.remaining_amount = ctx.remaining_accounts.len() as u64;
        emit!(LockAssetEvent {
            amount: ctx.remaining_accounts.len() as u64,
            locked_at: ctx.accounts.clock.unix_timestamp,
            locked_by: ctx.accounts.user.key(),
            lock_asset_metadata: lock_asset_metadata.key(),
            duration,
            nft_keys: lock_asset_metadata.asset_keys.clone(),
            seeds_index: seed_id,
            is_nft: true
        });
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(duration: i64, seed_id: String)]
pub struct LockNFT<'info> {
    #[account(
        seeds = [STACKER.as_bytes(), b"metadata"],
        bump,
    )]
    pub stacker_metadata: Account<'info, StackerMetadata>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [STACKER.as_bytes(), user.key().as_ref()],
        bump
    )]
    pub user_stack_pda: Account<'info, UserStackPda>,

    #[account(
        init,
        payer = user,
        space = DISCREMENATOR+StackAssetMetadata::INIT_SPACE,
        seeds = [STACKER.as_bytes(), b"nft", user.key().as_ref(), seed_id.as_bytes()],
        bump
    )]
    pub stack_asset_metadata: Account<'info, StackAssetMetadata>,

    pub collection: Account<'info, BaseCollectionV1>,

    /// CHECK: Checked in mpl-core.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,

    /// The SPL Noop program.
    /// CHECK: Checked in mpl-core.
    // pub log_wrapper: Option<AccountInfo<'info>>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}
