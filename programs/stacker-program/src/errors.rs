use anchor_lang::prelude::*;

#[error_code]
pub enum InitializeErrors {
    #[msg("Force unlock tax must be between 0 and 100")]
    ForceUnlockTaxInvalid,
}

#[error_code]
pub enum LockTokenErrors {
    #[msg("Passed amount for locking token is invalid")]
    InvalidAmount,

    #[msg("Not enough token balance to lock in user's account. try to lower amount.")]
    InsufficientTokenBalance,

    #[msg("Invalid locking duration passed, which is probably in past.")]
    InvalidDuration,

    #[msg("Passed Token mint is not supported for Locking")]
    UnsupportedTokenMint,
}

#[error_code]
pub enum LockNFTErrors {
    #[msg("Signer is not owner of passed NFT")]
    NotNFTOwner,

    #[msg("Passed collection and NFT is not supported for Locking")]
    UnsupportedCollection,

    #[msg("Invalid Account(s) Passed")]
    InvalidNFTAccountsPassed,

    #[msg("Program Only supports locking/unlokcing of 10 NFTs at once")]
    MaxNFTArrayLengthFound,
}

#[error_code]
pub enum UnlockTokensErrors {
    #[msg("Locking period is not elipsed. Use force unlock to unlock tokens.")]
    LockDurationNotCompleted,

    #[msg("Tokens already released")]
    AlreadyReleased,

    #[msg("Passed Token mint is not supported for Unlocking")]
    UnsupportedTokenMint,

    #[msg("Passed reward account is invalid")]
    InavlidRewardAccountPassed,
}

#[error_code]
pub enum UnlockNFTErrors {
    #[msg("Locking period is not elipsed. Use force unlock to unlock NFTs.")]
    LockDurationNotCompleted,

    #[msg("NFTs already released")]
    AlreadyReleased,

    #[msg("Passed collection and NFT is not supported for Unlocking")]
    UnsupportedCollection,

    #[msg("NFT List passed in remaining accounts are invalid")]
    InvalidNFTAccountsPassed,
}

#[error_code]
pub enum ChangeWalletStateError {
    #[msg("You are not allowed to perform this change.")]
    NotAllowed,

    #[msg("Owner has disabled further interaction with your wallet right now. Try again later")]
    WalletDisabled,

    #[msg("Stacker program is disabled right now. try again later")]
    StackerProgramDisabled,
}
