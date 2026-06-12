use crate::error::MortgageError;
use crate::models::{LoanParams, LoanResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// A saved calculation session containing both parameters and results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub params: LoanParams,
    pub result: LoanResult,
}

/// Saves a calculation session to a JSON file.
///
/// # Examples
///
/// ```no_run
/// use mortgage_core::{Calculator, LoanParams, PaymentType, Currency, RateMode, save_session};
/// use chrono::NaiveDate;
///
/// let params = LoanParams {
///     amount: 100_000.0,
///     term_years: 10,
///     payment_type: PaymentType::Annuity,
///     currency: Currency::Eur,
///     start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
///     rate_mode: RateMode::Fix { rate: 5.0, spread: 0.0 },
///     same_spread: false,
///     euribor_curve: vec![],
///     prepayments: vec![],
/// };
///
/// let result = Calculator::calculate(&params).unwrap();
/// save_session("/tmp/session.json", &params, &result).unwrap();
/// ```
pub fn save_session<P: AsRef<Path>>(
    path: P,
    params: &LoanParams,
    result: &LoanResult,
) -> Result<(), MortgageError> {
    let session = Session {
        params: params.clone(),
        result: result.clone(),
    };
    let json = serde_json::to_string_pretty(&session)
        .map_err(|e| MortgageError::ConfigError(format!("Serialization failed: {}", e)))?;
    fs::write(path, json)
        .map_err(|e| MortgageError::ConfigError(format!("File write failed: {}", e)))?;
    Ok(())
}

pub fn load_session<P: AsRef<Path>>(path: P) -> Result<Session, MortgageError> {
    let content = fs::read_to_string(path)
        .map_err(|e| MortgageError::ConfigError(format!("File read failed: {}", e)))?;
    let session: Session = serde_json::from_str(&content)
        .map_err(|e| MortgageError::ConfigError(format!("JSON parse failed: {}", e)))?;
    Ok(session)
}
