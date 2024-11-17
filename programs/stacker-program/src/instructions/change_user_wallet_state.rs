use crate::errors::ChangeWalletStateError;
use crate::prelude::*;
use anchor_lang::prelude::*;

pub fn change_user_wallet_state(
    ctx: Context<ChangeUserWalletState>,
    new_state: bool,
) -> Result<()> {
    require!(
        !ctx.accounts.stacker_metadata.is_disabled,
        ChangeWalletStateError::StackerProgramDisabled
    );
    require!(
        ctx.accounts.owner.key() == ctx.accounts.stacker_metadata.owner,
        ChangeWalletStateError::NotAllowed
    );
    ctx.accounts.user_stack_pda.is_disabled = new_state;
    msg!("User wallet state changed successfully!");
    Ok(())
}

#[derive(Accounts)]
#[instruction(new_state: bool)]
pub struct ChangeUserWalletState<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [STACKER.as_bytes(), user.key().as_ref()],
        bump
    )]
    pub user_stack_pda: Account<'info, UserStackPda>,

    /// CHECK Not writing to this account
    pub user: UncheckedAccount<'info>,
    pub stacker_metadata: Account<'info, StackerMetadata>,
}
