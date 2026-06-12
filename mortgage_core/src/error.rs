use thiserror::Error;

#[derive(Error, Debug)]
pub enum MortgageError {
    #[error("Invalid loan amount: must be positive")]
    InvalidAmount,

    #[error("Invalid loan term: must be at least 1 year")]
    InvalidTerm,

    #[error("Invalid interest rate: must be non-negative")]
    InvalidRate,

    #[error("Invalid date: {0}")]
    InvalidDate(String),

    #[error("Prepayment amount ({0}) exceeds remaining balance ({1})")]
    PrepaymentExceedsBalance(f64, f64),

    #[error("Euribor fetch failed: {0}")]
    EuriborFetchError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),
}

pub type Result<T> = std::result::Result<T, MortgageError>;
