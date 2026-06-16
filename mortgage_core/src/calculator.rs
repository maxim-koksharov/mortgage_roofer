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
///     upfront_cost: None,
///     upfront_percent: None,
///     down_payment: None,
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
        if let Err(errors) = params.validate() {
            return Err(MortgageError::CalculationError(errors.join("; ")));
        }

        let mut payments = Vec::new();
        let down_payment = params.down_payment.unwrap_or(0.0).min(params.amount);
        let mut balance = params.amount - down_payment;
        let mut current_date = params.start_date;
        let mut total_interest = 0.0;
        let mut total_principal = down_payment;

        let total_months = params.term_years * 12;
        let mut remaining_months = total_months;

        let mut prepayments = params.prepayments.clone();
        prepayments.sort_by_key(|p| p.date);

        let mut target_monthly: Option<f64> = None;
        let mut target_principal: Option<f64> = None;
        let mut prev_rate: Option<f64> = None;

        while balance > 0.01 && remaining_months > 0 {
            Self::apply_prepayments(
                &mut prepayments,
                &mut balance,
                &mut total_principal,
                &mut target_monthly,
                &mut target_principal,
                &mut remaining_months,
                current_date,
            );

            let rate = Self::annual_rate_for_date(current_date, params);
            let monthly_rate = rate / 12.0 / 100.0;

            if prev_rate.is_none_or(|r| (r - rate).abs() > 0.0001) {
                target_monthly = None;
            }
            prev_rate = Some(rate);

            let interest = balance * monthly_rate;
            let (principal, payment) = match params.payment_type {
                PaymentType::Annuity => {
                    let monthly = target_monthly.get_or_insert_with(|| {
                        Self::annuity_payment(balance, monthly_rate, remaining_months)
                    });
                    let principal = (*monthly - interest).min(balance);
                    let payment = principal + interest;
                    (principal, payment)
                }
                PaymentType::Diff => {
                    let principal = target_principal
                        .get_or_insert_with(|| balance / remaining_months as f64)
                        .min(balance);
                    let payment = principal + interest;
                    (principal, payment)
                }
            };

            balance -= principal;
            total_interest += interest;
            total_principal += principal;

            payments.push(Payment {
                payment,
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

        let principal_exceeds_interest_at = payments.iter().position(|p| p.principal > p.interest);

        let monthly_payment = match params.payment_type {
            PaymentType::Annuity => payments.first().map(|p| p.payment),
            _ => None,
        };

        Ok(LoanResult {
            monthly_payment,
            total_principal,
            total_interest,
            total_paid: total_principal + total_interest,
            payments,
            principal_exceeds_interest_at,
        })
    }

    fn annuity_payment(balance: f64, monthly_rate: f64, months: u32) -> f64 {
        if monthly_rate > 0.0 {
            let n = months as i32;
            let num = monthly_rate * (1.0 + monthly_rate).powi(n);
            let den = (1.0 + monthly_rate).powi(n) - 1.0;
            balance * num / den
        } else {
            balance / months as f64
        }
    }

    fn apply_prepayments(
        prepayments: &mut Vec<Prepayment>,
        balance: &mut f64,
        total_principal: &mut f64,
        target_monthly: &mut Option<f64>,
        target_principal: &mut Option<f64>,
        remaining_months: &mut u32,
        current_date: NaiveDate,
    ) {
        while let Some(prep) = prepayments.first() {
            if prep.date > current_date {
                break;
            }
            let prep = prepayments.remove(0);
            let prep_amount = prep.amount.min(*balance);
            *balance -= prep_amount;
            *total_principal += prep_amount;

            match prep.effect {
                PrepaymentEffect::ReduceTerm => {
                    if let Some(tp) = target_principal {
                        *remaining_months = ((*balance / *tp).ceil() as u32).max(1);
                    }
                }
                PrepaymentEffect::ReducePayment => {
                    *target_monthly = None;
                    *target_principal = None;
                }
            }
        }
    }

    pub(crate) fn annual_rate_for_date(date: NaiveDate, params: &LoanParams) -> f64 {
        match &params.rate_mode {
            RateMode::Fix { rate, spread } => rate + spread,
            RateMode::Euribor { spread, .. } => Self::euribor_for_date(date, params) + spread,
            RateMode::Mixed {
                fix_years,
                fix_rate,
                fix_spread,
                euribor_spread,
                ..
            } => {
                let fix_end = params
                    .start_date
                    .checked_add_months(Months::new((fix_years * 12.0).round() as u32))
                    .expect("Invalid date");
                if date < fix_end {
                    fix_rate + fix_spread
                } else {
                    let spread = if params.same_spread {
                        *fix_spread
                    } else {
                        *euribor_spread
                    };
                    Self::euribor_for_date(date, params) + spread
                }
            }
        }
    }

    /// Look up Euribor rate for a date.
    /// First check the manual curve, then fall back to auto-fetched (stub for now).
    fn euribor_for_date(date: NaiveDate, params: &LoanParams) -> f64 {
        params
            .euribor_curve
            .iter()
            .filter(|p| p.date_from <= date)
            .map(|p| p.rate)
            .next_back()
            .unwrap_or(0.0)
    }

    /// Reverse calculator: given a target monthly payment, annual rate (%) and term,
    /// returns the maximum affordable loan amount.
    ///
    /// For annuity: `P = M * ((1+r)^n - 1) / (r * (1+r)^n)`
    /// For diff:    `P = M * n / (1 + n * r)`  (first payment is the largest)
    ///
    /// where `r` = monthly rate (annual_rate / 12 / 100), `n` = total months.
    pub fn reverse_calculate(
        target_monthly: f64,
        annual_rate: f64,
        term_years: u32,
        payment_type: PaymentType,
    ) -> f64 {
        let monthly_rate = annual_rate / 12.0 / 100.0;
        let months = term_years * 12;

        if target_monthly <= 0.0 || months == 0 {
            return 0.0;
        }

        match payment_type {
            PaymentType::Annuity => {
                if monthly_rate > 0.0 {
                    let n = months as i32;
                    let pow = (1.0 + monthly_rate).powi(n);
                    target_monthly * (pow - 1.0) / (monthly_rate * pow)
                } else {
                    target_monthly * months as f64
                }
            }
            PaymentType::Diff => {
                if monthly_rate > 0.0 {
                    target_monthly * months as f64 / (1.0 + months as f64 * monthly_rate)
                } else {
                    target_monthly * months as f64
                }
            }
        }
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
            upfront_cost: None,
            upfront_percent: None,
            down_payment: None,
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
        assert!(result.is_ok());
    }

    #[test]
    fn test_reverse_annuity_roundtrip() {
        let params = default_params();
        let result = Calculator::calculate(&params).unwrap();
        let forward_monthly = result.monthly_payment.unwrap();
        let reverse_amount = Calculator::reverse_calculate(
            forward_monthly,
            5.0,
            params.term_years,
            PaymentType::Annuity,
        );
        assert!(
            (reverse_amount - params.amount).abs() < 1.0,
            "Roundtrip: forward={} reverse={}",
            params.amount,
            reverse_amount
        );
    }

    #[test]
    fn test_reverse_diff_roundtrip() {
        let mut params = default_params();
        params.payment_type = PaymentType::Diff;
        let result = Calculator::calculate(&params).unwrap();
        let first_payment = result.payments.first().unwrap().payment;
        let reverse_amount =
            Calculator::reverse_calculate(first_payment, 5.0, params.term_years, PaymentType::Diff);
        assert!(
            (reverse_amount - params.amount).abs() < 1.0,
            "Roundtrip diff: forward={} reverse={}",
            params.amount,
            reverse_amount
        );
    }

    #[test]
    fn test_reverse_annuity_zero_rate() {
        let amount = Calculator::reverse_calculate(1000.0, 0.0, 10, PaymentType::Annuity);
        assert!((amount - 120_000.0).abs() < 0.01, "Got {}", amount);
    }

    #[test]
    fn test_reverse_diff_zero_rate() {
        let amount = Calculator::reverse_calculate(1000.0, 0.0, 10, PaymentType::Diff);
        assert!((amount - 120_000.0).abs() < 0.01, "Got {}", amount);
    }

    #[test]
    fn test_reverse_zero_target() {
        let amount = Calculator::reverse_calculate(0.0, 5.0, 10, PaymentType::Annuity);
        assert_eq!(amount, 0.0);
    }

    #[test]
    fn test_reverse_zero_term() {
        let amount = Calculator::reverse_calculate(1000.0, 5.0, 0, PaymentType::Annuity);
        assert_eq!(amount, 0.0);
    }
}
