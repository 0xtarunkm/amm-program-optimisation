use anchor_lang::error_code;

#[error_code]
pub enum AmmError {
    #[msg("Fees should not be more than 100%")]
    InvalidFee,
}
