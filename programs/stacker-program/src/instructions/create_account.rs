use crate::errors::ChangeWalletStateError;
use crate::prelude::*;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};

pub fn create_locker_account(ctx: Context<CreateUserAccount>) -> Result<()> {
    require!(
        !ctx.accounts.stacker_metadata.is_disabled,
        ChangeWalletStateError::StackerProgramDisabled
    );
    let user_stack_pda = &mut ctx.accounts.user_stack_pda;
    user_stack_pda.stacked_nfts = 0;
    user_stack_pda.stacked_tokens = 0;
    user_stack_pda.is_disabled = false;
    msg!("User Stack Account Created Successfully!");
    Ok(())
}

#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [STACKER.as_bytes(), b"metadata"],
        bump
    )]
    pub stacker_metadata: Account<'info, StackerMetadata>,

    #[account(
        init,
        seeds = [STACKER.as_bytes(), user.key().as_ref()],
        bump,
        payer = user,
        space = DISCREMENATOR + UserStackPda::INIT_SPACE,
    )]
    pub user_stack_pda: Account<'info, UserStackPda>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user_stack_pda,
    )]
    pub user_stack_pda_ata: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
