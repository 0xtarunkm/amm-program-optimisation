use anchor_lang::error_code;
#[error_code]
pub enum AmmError {
    #[msg("Fees should not be more than 100%")]
    InvalidFee,
    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,
    #[msg("Pool is locked")]
    PoolLocked,
    #[msg("Invalid mint account")]
    InvalidMint,
    #[msg("Insufficient liquidity in the pool")]
    InsufficientLiquidity,
    #[msg("invalid token amount it cannot be zero")]
    TokenNonZero,
}
