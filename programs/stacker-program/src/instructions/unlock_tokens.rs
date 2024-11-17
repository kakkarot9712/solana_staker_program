use crate::errors::{ChangeWalletStateError, UnlockTokensErrors};
use crate::events::UnlockAssetsEvent;
use crate::prelude::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

pub fn unlock_tokens(ctx: Context<UnlockTokens>, _seed_id: String, force: bool) -> Result<()> {
    require!(
        !ctx.accounts.user_stack_pda.is_disabled,
        ChangeWalletStateError::WalletDisabled
    );
    // Check if transactions are allowed
    require!(
        !ctx.accounts.stacker_metadata.is_disabled,
        ChangeWalletStateError::StackerProgramDisabled
    );

    // Check if asset mint is supported
    require!(
        ctx.accounts.stacker_metadata.mint.key() == ctx.accounts.mint.key(),
        UnlockTokensErrors::UnsupportedTokenMint
    );

    // Check if assets are already released
    require!(
        !ctx.accounts.stack_asset_metadata.is_cleared,
        UnlockTokensErrors::AlreadyReleased
    );

    require!(
        ctx.accounts.reward_account.key() == ctx.accounts.stacker_metadata.reward_wallet,
        UnlockTokensErrors::InavlidRewardAccountPassed
    );

    // Check if duration is completed
    let current_time = ctx.accounts.clock.unix_timestamp;
    let locking_time_required =
        ctx.accounts.stack_asset_metadata.duration + ctx.accounts.stack_asset_metadata.stacked_at;
    let is_unlockable = current_time > locking_time_required;

    let mut releasable_amount = ctx.accounts.stack_asset_metadata.total_amount;
    let mut tax_amount: u64 = 0;

    if force && !is_unlockable {
        // Deduct tax from releasable amount
        tax_amount =
            releasable_amount * ctx.accounts.stacker_metadata.force_unlock_tax as u64 / 100;
        releasable_amount -= tax_amount;
        msg!(
            "Release is taxable {}, {}, {}",
            tax_amount,
            releasable_amount,
            ctx.accounts.stack_asset_metadata.total_amount
        );
    } else {
        require!(is_unlockable, UnlockTokensErrors::LockDurationNotCompleted);
    }

    // Try to release tokens to user
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                authority: ctx.accounts.user_stack_pda.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.user_stacker_pda_ata.to_account_info(),
                to: ctx.accounts.user_ata.to_account_info(),
            },
            &[&[
                STACKER.as_bytes(),
                ctx.accounts.user.key().as_ref(),
                &[ctx.bumps.user_stack_pda],
            ]],
        ),
        releasable_amount,
        ctx.accounts.mint.decimals,
    )?;

    if tax_amount > 0 {
        // Transfer tax amount to reward wallet
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    authority: ctx.accounts.user_stack_pda.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.user_stacker_pda_ata.to_account_info(),
                    to: ctx.accounts.reward_account_ata.to_account_info(),
                },
                &[&[
                    STACKER.as_bytes(),
                    ctx.accounts.user.key().as_ref(),
                    &[ctx.bumps.user_stack_pda],
                ]],
            ),
            tax_amount,
            ctx.accounts.mint.decimals,
        )?;
    }

    // update lock_asset_metadata account
    let lock_asset_metadta = &mut ctx.accounts.stack_asset_metadata;
    lock_asset_metadta.remaining_amount = 0;
    lock_asset_metadta.is_cleared = true;
    ctx.accounts.user_stack_pda.stacked_tokens -= lock_asset_metadta.total_amount;
    emit!(UnlockAssetsEvent {
        amount: releasable_amount,
        unlocked_at: ctx.accounts.clock.unix_timestamp,
        is_nft: false,
        taxed_amount: tax_amount,
        lock_asset_metadata: ctx.accounts.stack_asset_metadata.key(),
        nft_keys: Vec::new(),
    });
    Ok(())
}

#[derive(Accounts)]
#[instruction(seed_id: String, force: bool)]
pub struct UnlockTokens<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [STACKER.as_bytes(), b"metadata"],
        bump
    )]
    pub stacker_metadata: Account<'info, StackerMetadata>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user_stack_pda
    )]
    pub user_stacker_pda_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = reward_account
    )]
    pub reward_account_ata: Account<'info, TokenAccount>,

    /// CHECK Not writing to this account
    pub reward_account: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [STACKER.as_bytes(), user.key().as_ref()],
        bump
    )]
    pub user_stack_pda: Account<'info, UserStackPda>,

    #[account(
        mut,
        seeds = [STACKER.as_bytes(), b"token", user.key().as_ref(), seed_id.as_bytes()],
        bump
    )]
    pub stack_asset_metadata: Account<'info, StackAssetMetadata>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
}
