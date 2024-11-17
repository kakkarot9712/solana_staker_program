use anchor_lang::prelude::*;
use crate::errors::ChangeWalletStateError;
use crate::prelude::*;

pub fn change_wallet_state(ctx: Context<ChangeWalletState>, new_state: bool) -> Result<()> {
    require!(
        ctx.accounts.owner.key() == ctx.accounts.stacker_metadata.owner,
        ChangeWalletStateError::NotAllowed
    );
    ctx.accounts.stacker_metadata.is_disabled = new_state;
    msg!("Wallet state changed successfully");
    Ok(())
}

#[derive(Accounts)]
#[instruction(new_state: bool)]
pub struct ChangeWalletState<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [STACKER.as_bytes(), b"metadata"],
        bump
    )]
    pub stacker_metadata: Account<'info, StackerMetadata>,
}
