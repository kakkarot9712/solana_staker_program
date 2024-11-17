use crate::{
    errors::{ChangeWalletStateError, LockTokenErrors},
    events::LockAssetEvent,
    prelude::*,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

pub fn lock_tokens(
    ctx: Context<LockTokens>,
    amount: u64,
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
        ctx.accounts.user_ata.mint == ctx.accounts.stacker_metadata.mint,
        LockTokenErrors::UnsupportedTokenMint
    );
    require!(amount > 0, LockTokenErrors::InvalidAmount);
    require!(
        ctx.accounts.user_ata.amount >= amount,
        LockTokenErrors::InsufficientTokenBalance
    );
    require!(duration > 0, LockTokenErrors::InvalidDuration);
    // Try to transfer tokens into our account now
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                authority: ctx.accounts.user.to_account_info(),
                from: ctx.accounts.user_ata.to_account_info(),
                to: ctx.accounts.user_stack_pda_ata.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.mint.decimals,
    )?;
    let lock_asset_metadata = &mut ctx.accounts.stack_asset_metadata;
    lock_asset_metadata.total_amount = amount;
    lock_asset_metadata.stacked_at = ctx.accounts.clock.unix_timestamp;
    lock_asset_metadata.is_nft = false;
    lock_asset_metadata.is_cleared = false;
    lock_asset_metadata.duration = duration;
    lock_asset_metadata.asset_keys = vec![ctx.accounts.mint.key()];
    lock_asset_metadata.remaining_amount = amount;

    emit!(LockAssetEvent {
        amount,
        locked_at: ctx.accounts.clock.unix_timestamp,
        locked_by: ctx.accounts.user.key(),
        lock_asset_metadata: lock_asset_metadata.key(),
        duration,
        nft_keys: vec![],
        seeds_index: seed_id,
        is_nft: false
    });

    // ctx.accounts.user_stack_pda.token_seed_index += 1;
    ctx.accounts.user_stack_pda.stacked_tokens += amount;
    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, duration: i64, seed_id: String)]
pub struct LockTokens<'info> {
    #[account(
        seeds = [STACKER.as_bytes(), b"metadata"],
        bump,
    )]
    pub stacker_metadata: Account<'info, StackerMetadata>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user
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
        associated_token::mint = mint,
        associated_token::authority = user_stack_pda,
    )]
    pub user_stack_pda_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        seeds = [STACKER.as_bytes(), b"token", user.key().as_ref(), seed_id.as_bytes()],
        bump,
        payer = user,
        space = DISCREMENATOR + StackAssetMetadata::INIT_SPACE
    )]
    pub stack_asset_metadata: Account<'info, StackAssetMetadata>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>,
}
