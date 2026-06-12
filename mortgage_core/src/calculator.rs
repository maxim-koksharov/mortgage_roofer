use crate::error::MortgageError;
use crate::models::*;
use chrono::{Months, NaiveDate};

/// Calculates a loan schedule from parameters.
///
/// # Examples
///
/// ```
/// use mortgage_core::{Calculator, LoanParams, PaymentType, Currency, RateMode};
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
/// assert!(result.monthly_payment.is_some());
/// assert_eq!(result.payments.len(), 120);
/// ```
pub struct Calculator;

impl Calculator {
    /// Calculates a loan schedule from the given parameters.
    ///
    /// Returns `Err` if validation fails (e.g., negative amount, zero term).
    pub fn calculate(params: &LoanParams) -> Result<LoanResult, MortgageError> {
        let errors = params.validate();
        if !errors.is_empty() {
            return Err(MortgageError::CalculationError(errors.join("; ")));
        }

        let mut payments = Vec::new();
        let mut balance = params.amount;
        let mut current_date = params.start_date;
        let mut total_interest = 0.0;
        let mut total_principal = 0.0;

        let total_months = params.term_years * 12;

        let mut prepayments = params.prepayments.clone();
        prepayments.sort_by_key(|p| p.date);

        let effective_rate = |date: NaiveDate| -> f64 { Self::annual_rate_for_date(date, params) };

        if params.payment_type == PaymentType::Annuity {
            let mut remaining_months = total_months;
            let mut target_monthly_payment: Option<f64> = None;

            while balance > 0.01 && remaining_months > 0 {
                while let Some(prep) = prepayments.first() {
                    if prep.date == current_date {
                        let prep = prepayments.remove(0);
                        let prep_amount = prep.amount.min(balance);
                        balance -= prep_amount;
                        total_principal += prep_amount;

                        match prep.effect {
                            PrepaymentEffect::ReduceTerm => {}
                            PrepaymentEffect::ReducePayment => {
                                target_monthly_payment = None;
                            }
                        }
                    } else {
                        break;
                    }
                }

                let rate = effective_rate(current_date);
                let monthly_rate = rate / 12.0 / 100.0;

                let monthly_payment = match target_monthly_payment {
                    Some(mp) => mp,
                    None => {
                        let mp = if monthly_rate > 0.0 {
                            let num =
                                monthly_rate * (1.0 + monthly_rate).powi(remaining_months as i32);
                            let den = (1.0 + monthly_rate).powi(remaining_months as i32) - 1.0;
                            balance * num / den
                        } else {
                            balance / remaining_months as f64
                        };
                        target_monthly_payment = Some(mp);
                        mp
                    }
                };

                let interest = balance * monthly_rate;
                let principal = (monthly_payment - interest).min(balance);
                let actual_payment = principal + interest;

                balance -= principal;
                total_interest += interest;
                total_principal += principal;

                payments.push(Payment {
                    payment: actual_payment,
                    date: current_date,
                    principal,
                    interest,
                    remaining_balance: balance.max(0.0),
                    applied_rate: rate,
                });

                current_date = current_date
                    .checked_add_months(Months::new(1))
                    .ok_or_else(|| MortgageError::InvalidDate("Date overflow".to_string()))?;
                remaining_months -= 1;
            }

            let first_principal_gt_interest =
                payments.iter().position(|p| p.principal > p.interest);

            let fixed_monthly = if payments.is_empty() {
                None
            } else {
                let initial_rate = effective_rate(params.start_date);
                let mr = initial_rate / 12.0 / 100.0;
                if mr > 0.0 {
                    let n = total_months as i32;
                    let num = mr * (1.0 + mr).powi(n);
                    let den = (1.0 + mr).powi(n) - 1.0;
                    Some(params.amount * num / den)
                } else {
                    Some(params.amount / total_months as f64)
                }
            };

            Ok(LoanResult {
                monthly_payment: fixed_monthly,
                total_principal,
                total_interest,
                total_paid: total_principal + total_interest,
                payments,
                principal_exceeds_interest_at: first_principal_gt_interest,
            })
        } else {
            let mut remaining_months = total_months;
            let mut target_principal: Option<f64> = None;

            while balance > 0.01 && remaining_months > 0 {
                while let Some(prep) = prepayments.first() {
                    if prep.date == current_date {
                        let prep = prepayments.remove(0);
                        let prep_amount = prep.amount.min(balance);
                        balance -= prep_amount;
                        total_principal += prep_amount;

                        match prep.effect {
                            PrepaymentEffect::ReduceTerm => {
                                if let Some(tp) = target_principal {
                                    remaining_months = ((balance / tp).ceil() as u32).max(1);
                                }
                            }
                            PrepaymentEffect::ReducePayment => {
                                target_principal = None;
                            }
                        }
                    } else {
                        break;
                    }
                }

                let rate = effective_rate(current_date);
                let monthly_rate = rate / 12.0 / 100.0;

                let principal_part = match target_principal {
                    Some(tp) => tp.min(balance),
                    None => {
                        let tp = balance / remaining_months as f64;
                        target_principal = Some(tp);
                        tp.min(balance)
                    }
                };
                let interest = balance * monthly_rate;
                let payment = principal_part + interest;

                balance -= principal_part;
                total_interest += interest;
                total_principal += principal_part;

                payments.push(Payment {
                    payment,
                    date: current_date,
                    principal: principal_part,
                    interest,
                    remaining_balance: balance.max(0.0),
                    applied_rate: rate,
                });

                current_date = current_date
                    .checked_add_months(Months::new(1))
                    .ok_or_else(|| MortgageError::InvalidDate("Date overflow".to_string()))?;
                remaining_months -= 1;
            }

            let first_principal_gt_interest =
                payments.iter().position(|p| p.principal > p.interest);

            Ok(LoanResult {
                monthly_payment: None,
                total_principal,
                total_interest,
                total_paid: total_principal + total_interest,
                payments,
                principal_exceeds_interest_at: first_principal_gt_interest,
            })
        }
    }

    fn annual_rate_for_date(date: NaiveDate, params: &LoanParams) -> f64 {
        match &params.rate_mode {
            RateMode::Fix { rate, spread } => rate + spread,
            RateMode::Euribor { tenor: _, spread } => {
                let euribor = Self::euribor_for_date(date, params);
                euribor + spread
            }
            RateMode::Mixed {
                fix_years,
                fix_rate,
                fix_spread,
                euribor_tenor: _,
                euribor_spread,
            } => {
                let fix_end = params
                    .start_date
                    .checked_add_months(Months::new((fix_years * 12.0) as u32))
                    .expect("Invalid date");
                if date < fix_end {
                    fix_rate + fix_spread
                } else {
                    let euribor = Self::euribor_for_date(date, params);
                    let spread = if params.same_spread {
                        *fix_spread
                    } else {
                        *euribor_spread
                    };
                    euribor + spread
                }
            }
        }
    }

    /// Look up Euribor rate for a date.
    /// First check the manual curve, then fall back to auto-fetched (stub for now).
    fn euribor_for_date(date: NaiveDate, params: &LoanParams) -> f64 {
        // Search manual curve for the latest point before or on this date.
        let mut applicable: Option<f64> = None;
        for point in &params.euribor_curve {
            if point.date_from <= date {
                applicable = Some(point.rate);
            }
        }
        if let Some(rate) = applicable {
            return rate;
        }

        // Fallback: stub — will be replaced by fetched ECB data.
        // For now, return 0.0 so the caller can see data is missing.
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn default_params() -> LoanParams {
        LoanParams {
            amount: 100_000.0,
            term_years: 10,
            payment_type: PaymentType::Annuity,
            currency: Currency::Eur,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            rate_mode: RateMode::Fix {
                rate: 5.0,
                spread: 0.0,
            },
            same_spread: false,
            euribor_curve: vec![],
            prepayments: vec![],
        }
    }

    #[test]
    fn test_annuity_basic() {
        let params = default_params();
        let result = Calculator::calculate(&params).unwrap();
        assert!(result.monthly_payment.is_some());
        assert!((result.total_principal - params.amount).abs() < 1.0);
        assert!(result.total_interest > 0.0);
        assert_eq!(result.payments.len(), 120);
    }

    #[test]
    fn test_diff_basic() {
        let mut params = default_params();
        params.payment_type = PaymentType::Diff;
        let result = Calculator::calculate(&params).unwrap();
        assert!(result.monthly_payment.is_none());
        assert!((result.total_principal - params.amount).abs() < 1.0);
        assert!(result.total_interest > 0.0);
        assert_eq!(result.payments.len(), 120);
    }

    #[test]
    fn test_prepayment_reduce_term() {
        let mut params = default_params();
        params.prepayments.push(Prepayment {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            amount: 20_000.0,
            effect: PrepaymentEffect::ReduceTerm,
        });
        let result = Calculator::calculate(&params).unwrap();
        assert!(
            result.payments.len() < 120,
            "Expected <120 payments, got {}",
            result.payments.len()
        );
        assert!((result.total_principal - params.amount).abs() < 1.0);
    }

    #[test]
    fn test_prepayment_reduce_payment() {
        let mut params = default_params();
        params.prepayments.push(Prepayment {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            amount: 20_000.0,
            effect: PrepaymentEffect::ReducePayment,
        });
        let result = Calculator::calculate(&params).unwrap();
        let len = result.payments.len();
        assert!(
            len >= 118 && len <= 122,
            "Expected ~120 payments, got {}",
            len
        );
        assert!((result.total_principal - params.amount).abs() < 1.0);
    }

    #[test]
    fn test_mixed_rate() {
        let mut params = default_params();
        params.rate_mode = RateMode::Mixed {
            fix_years: 1.0,
            fix_rate: 3.0,
            fix_spread: 1.0,
            euribor_tenor: EuriborTenor::SixMonths,
            euribor_spread: 2.0,
        };
        params.euribor_curve = vec![EuriborPoint {
            date_from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            rate: 4.0,
        }];
        let result = Calculator::calculate(&params).unwrap();
        assert_eq!(result.payments[0].applied_rate, 4.0);
        let p2026 = result
            .payments
            .iter()
            .find(|p| p.date.year() == 2026)
            .expect("Should have 2026 payments");
        assert_eq!(p2026.applied_rate, 6.0);
    }

    #[test]
    fn test_same_spread() {
        let mut params = default_params();
        params.rate_mode = RateMode::Mixed {
            fix_years: 1.0,
            fix_rate: 3.0,
            fix_spread: 1.5,
            euribor_tenor: EuriborTenor::SixMonths,
            euribor_spread: 2.0,
        };
        params.same_spread = true;
        params.euribor_curve = vec![EuriborPoint {
            date_from: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            rate: 4.0,
        }];
        let result = Calculator::calculate(&params).unwrap();
        let p2026 = result
            .payments
            .iter()
            .find(|p| p.date.year() == 2026)
            .expect("Should have 2026 payments");
        assert_eq!(p2026.applied_rate, 5.5);
    }

    #[test]
    fn test_zero_rate() {
        let mut params = default_params();
        params.rate_mode = RateMode::Fix {
            rate: 0.0,
            spread: 0.0,
        };
        let result = Calculator::calculate(&params).unwrap();
        assert_eq!(result.total_interest, 0.0);
        assert!((result.total_principal - params.amount).abs() < 0.01);
        let mp = result.monthly_payment.unwrap();
        assert!((mp - params.amount / 120.0).abs() < 0.01);
    }

    #[test]
    fn test_principal_exceeds_interest() {
        let mut params = default_params();
        params.rate_mode = RateMode::Fix {
            rate: 5.0,
            spread: 0.0,
        };
        let result = Calculator::calculate(&params).unwrap();
        assert!(result.principal_exceeds_interest_at.is_some());
    }

    #[test]
    fn test_validation_invalid_amount() {
        let mut params = default_params();
        params.amount = -100.0;
        let result = Calculator::calculate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_zero_term() {
        let mut params = default_params();
        params.term_years = 0;
        let result = Calculator::calculate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_negative_rate() {
        let mut params = default_params();
        params.rate_mode = RateMode::Fix {
            rate: -5.0,
            spread: 0.0,
        };
        let result = Calculator::calculate(&params);
        assert!(result.is_err());
    }
}
