use crate::errors::InitializeErrors;
use crate::prelude::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use mpl_core::accounts::BaseCollectionV1;

pub fn initialize(ctx: Context<Initialize>, force_unlock_tax: u8) -> Result<()> {
    require!(
        force_unlock_tax <= 100,
        InitializeErrors::ForceUnlockTaxInvalid
    );
    let stacker_metadata = &mut ctx.accounts.stacker_metadata;
    stacker_metadata.mint = ctx.accounts.mint.key();
    stacker_metadata.decimals = ctx.accounts.mint.decimals;
    stacker_metadata.collection_mint = ctx.accounts.collection.key();
    stacker_metadata.owner = ctx.accounts.initializer.key();
    stacker_metadata.is_disabled = false;
    stacker_metadata.force_unlock_tax = force_unlock_tax;
    stacker_metadata.reward_wallet = ctx.accounts.reward_wallet.key();

    msg!("Escrow Initialized Successfully!");
    Ok(())
}

#[derive(Accounts)]
#[instruction(force_unlock_tax: u8)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [STACKER.as_bytes(), b"metadata"],
        bump,
        payer = initializer,
        space = DISCREMENATOR + StackerMetadata::INIT_SPACE,
    )]
    pub stacker_metadata: Account<'info, StackerMetadata>,

    /// CHECK: Not writing to this wallet
    pub reward_wallet: UncheckedAccount<'info>,

    #[account(mut)]
    pub initializer: Signer<'info>,

    pub collection: Account<'info, BaseCollectionV1>,
    pub system_program: Program<'info, System>,
    pub mint: Account<'info, Mint>,
}
