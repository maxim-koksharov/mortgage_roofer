use crate::calculator::Calculator;
use crate::models::*;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SensitivityPoint {
    pub rate_delta: f64,
    pub effective_rate: f64,
    pub monthly_payment: Option<f64>,
    pub total_interest: f64,
    pub total_paid: f64,
}

pub fn sensitivity_analysis(params: &LoanParams, deltas: &[f64]) -> Vec<SensitivityPoint> {
    let mut results = Vec::new();
    let base_rate = match &params.rate_mode {
        RateMode::Fix { rate, spread } => rate + spread,
        RateMode::Euribor { .. } => Calculator::annual_rate_for_date(params.start_date, params),
        RateMode::Mixed {
            fix_rate,
            fix_spread,
            ..
        } => fix_rate + fix_spread,
    };

    for &delta in deltas {
        let new_rate = (base_rate + delta).max(0.0);
        let mut new_params = params.clone();
        match &params.rate_mode {
            RateMode::Fix { spread, .. } => {
                new_params.rate_mode = RateMode::Fix {
                    rate: (new_rate - spread).max(0.0),
                    spread: *spread,
                };
            }
            RateMode::Euribor { spread, tenor } => {
                for point in &mut new_params.euribor_curve {
                    point.rate = (point.rate + delta).max(0.0);
                }
                new_params.rate_mode = RateMode::Euribor {
                    tenor: *tenor,
                    spread: *spread,
                };
            }
            RateMode::Mixed {
                fix_rate,
                fix_spread,
                euribor_tenor,
                euribor_spread,
                fix_years,
            } => {
                for point in &mut new_params.euribor_curve {
                    point.rate = (point.rate + delta).max(0.0);
                }
                new_params.rate_mode = RateMode::Mixed {
                    fix_years: *fix_years,
                    fix_rate: (fix_rate + delta).max(0.0),
                    fix_spread: *fix_spread,
                    euribor_tenor: *euribor_tenor,
                    euribor_spread: *euribor_spread,
                };
            }
        }

        if let Ok(result) = Calculator::calculate(&new_params) {
            results.push(SensitivityPoint {
                rate_delta: delta,
                effective_rate: new_rate,
                monthly_payment: result.monthly_payment,
                total_interest: result.total_interest,
                total_paid: result.total_paid,
            });
        }
    }
    results
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BreakEvenResult {
    pub monthly_rent: f64,
    pub monthly_cost: f64,
    pub break_even_months: Option<u32>,
    pub break_even_years: Option<f64>,
    pub total_interest: f64,
    pub upfront_costs: f64,
    pub explanation: String,
}

fn upfront_costs(params: &LoanParams) -> f64 {
    params
        .upfront_cost
        .unwrap_or_else(|| params.amount * params.upfront_percent.unwrap_or(0.0) / 100.0)
}

pub fn break_even_analysis(params: &LoanParams, monthly_rent: f64) -> BreakEvenResult {
    let Ok(result) = Calculator::calculate(params) else {
        return BreakEvenResult {
            monthly_rent,
            monthly_cost: 0.0,
            break_even_months: None,
            break_even_years: None,
            total_interest: 0.0,
            upfront_costs: upfront_costs(params),
            explanation: "Invalid loan parameters.".to_string(),
        };
    };

    let monthly_cost = result
        .monthly_payment
        .or_else(|| result.payments.first().map(|p| p.payment))
        .unwrap_or(0.0);
    let upfront = upfront_costs(params);

    if monthly_cost >= monthly_rent {
        return BreakEvenResult {
            monthly_rent,
            monthly_cost,
            break_even_months: None,
            break_even_years: None,
            total_interest: result.total_interest,
            upfront_costs: upfront,
            explanation: format!(
                "Monthly mortgage ({:.2}) >= rent ({:.2}). Buying costs more per month.",
                monthly_cost, monthly_rent
            ),
        };
    }

    let monthly_savings = monthly_rent - monthly_cost;
    let equity_buildup_per_month = result.payments.first().map(|p| p.principal).unwrap_or(0.0);
    let effective_monthly_benefit = monthly_savings + equity_buildup_per_month;

    if effective_monthly_benefit <= 0.0 {
        return BreakEvenResult {
            monthly_rent,
            monthly_cost,
            break_even_months: None,
            break_even_years: None,
            total_interest: result.total_interest,
            upfront_costs: upfront,
            explanation: "No positive monthly benefit found.".to_string(),
        };
    }

    let break_even_months = (upfront / effective_monthly_benefit).ceil() as u32;
    let break_even_years = break_even_months as f64 / 12.0;

    BreakEvenResult {
        monthly_rent,
        monthly_cost,
        break_even_months: Some(break_even_months),
        break_even_years: Some(break_even_years),
        total_interest: result.total_interest,
        upfront_costs: upfront,
        explanation: format!(
            "Upfront: {:.2}, monthly savings: {:.2}, equity buildup: {:.2}. Break-even in {} months ({:.1} years).",
            upfront, monthly_savings, equity_buildup_per_month, break_even_months, break_even_years
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

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
    fn test_break_even_default_upfront_percent() {
        let mut params = default_params();
        params.upfront_percent = Some(5.0);
        let be = break_even_analysis(&params, 1500.0);
        assert!(be.break_even_months.is_some());
        assert!((be.upfront_costs - 5_000.0).abs() < 0.01);
    }

    #[test]
    fn test_break_even_fixed_upfront_cost() {
        let mut params = default_params();
        params.upfront_cost = Some(10_000.0);
        params.upfront_percent = Some(5.0);
        let be = break_even_analysis(&params, 1500.0);
        assert!((be.upfront_costs - 10_000.0).abs() < 0.01);
    }

    #[test]
    fn test_break_even_mortgage_higher_than_rent() {
        let params = default_params();
        let be = break_even_analysis(&params, 500.0);
        assert!(be.break_even_months.is_none());
    }
}
