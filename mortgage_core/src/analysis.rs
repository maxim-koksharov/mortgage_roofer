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
        RateMode::Euribor { spread, .. } => *spread,
        RateMode::Mixed {
            fix_rate,
            fix_spread,
            ..
        } => fix_rate + fix_spread,
    };

    for &delta in deltas {
        let new_rate = (base_rate + delta).max(0.0);
        let new_params = match &params.rate_mode {
            RateMode::Fix { spread, .. } => {
                let mut p = params.clone();
                p.rate_mode = RateMode::Fix {
                    rate: (new_rate - spread).max(0.0),
                    spread: *spread,
                };
                p
            }
            _ => params.clone(),
        };

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
    pub explanation: String,
}

pub fn break_even_analysis(params: &LoanParams, monthly_rent: f64) -> BreakEvenResult {
    let result = Calculator::calculate(params).unwrap_or(LoanResult {
        monthly_payment: None,
        total_principal: 0.0,
        total_interest: 0.0,
        total_paid: 0.0,
        payments: vec![],
        principal_exceeds_interest_at: None,
    });

    let monthly_cost = result.monthly_payment.unwrap_or(0.0);

    if monthly_cost >= monthly_rent {
        return BreakEvenResult {
            monthly_rent,
            monthly_cost,
            break_even_months: None,
            break_even_years: None,
            total_interest: result.total_interest,
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
            explanation: "No positive monthly benefit found.".to_string(),
        };
    }

    let upfront_costs = params.amount * 0.05;
    let break_even_months = (upfront_costs / effective_monthly_benefit).ceil() as u32;
    let break_even_years = break_even_months as f64 / 12.0;

    BreakEvenResult {
        monthly_rent,
        monthly_cost,
        break_even_months: Some(break_even_months),
        break_even_years: Some(break_even_years),
        total_interest: result.total_interest,
        explanation: format!(
            "Monthly savings: {:.2}, equity buildup: {:.2}. Break-even in {} months ({:.1} years).",
            monthly_savings, equity_buildup_per_month, break_even_months, break_even_years
        ),
    }
}
