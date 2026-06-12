use crate::models::Payment;

/// Converts a list of payments to CSV format.
///
/// # Examples
///
/// ```
/// use mortgage_core::{Calculator, LoanParams, PaymentType, Currency, RateMode, payments_to_csv};
/// use chrono::NaiveDate;
///
/// let params = LoanParams {
///     amount: 100_000.0,
///     term_years: 1,
///     payment_type: PaymentType::Annuity,
///     currency: Currency::Eur,
///     start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
///     rate_mode: RateMode::Fix { rate: 5.0, spread: 0.0 },
///     same_spread: false,
///     euribor_curve: vec![],
///     prepayments: vec![],
///     upfront_cost: None,
///     upfront_percent: None,
/// };
///
/// let result = Calculator::calculate(&params).unwrap();
/// let csv = payments_to_csv(&result.payments);
/// assert!(csv.contains("date,payment,principal,interest,remaining_balance,applied_rate"));
/// ```
pub fn payments_to_csv(payments: &[Payment]) -> String {
    let mut lines =
        vec!["date,payment,principal,interest,remaining_balance,applied_rate".to_string()];
    for p in payments {
        lines.push(format!(
            "{},{:.2},{:.2},{:.2},{:.2},{:.4}",
            p.date, p.payment, p.principal, p.interest, p.remaining_balance, p.applied_rate
        ));
    }
    lines.join("\n") + "\n"
}
