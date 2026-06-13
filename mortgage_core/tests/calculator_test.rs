use chrono::NaiveDate;
use mortgage_core::Calculator;
use mortgage_core::models::*;

fn base_params() -> LoanParams {
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
    }
}

#[test]
fn test_very_small_amount() {
    let mut params = base_params();
    params.amount = 1.0;
    let result = Calculator::calculate(&params).unwrap();
    assert_eq!(result.payments.len(), 120);
    assert!((result.total_principal - 1.0).abs() < 0.01);
}

#[test]
fn test_very_large_amount() {
    let mut params = base_params();
    params.amount = 10_000_000.0;
    let result = Calculator::calculate(&params).unwrap();
    assert!((result.total_principal - 10_000_000.0).abs() < 1.0);
}

#[test]
fn test_one_month_term() {
    let mut params = base_params();
    params.term_years = 0;
    let result = Calculator::calculate(&params);
    assert!(result.is_err());
}

#[test]
fn test_one_year_term() {
    let mut params = base_params();
    params.term_years = 1;
    let result = Calculator::calculate(&params).unwrap();
    assert_eq!(result.payments.len(), 12);
}

#[test]
fn test_fifty_year_term() {
    let mut params = base_params();
    params.term_years = 50;
    let result = Calculator::calculate(&params).unwrap();
    assert_eq!(result.payments.len(), 600);
}

#[test]
fn test_very_high_rate() {
    let mut params = base_params();
    params.rate_mode = RateMode::Fix {
        rate: 50.0,
        spread: 0.0,
    };
    let result = Calculator::calculate(&params).unwrap();
    assert!(result.total_interest > result.total_principal);
}

#[test]
fn test_diff_with_zero_rate() {
    let mut params = base_params();
    params.payment_type = PaymentType::Diff;
    params.rate_mode = RateMode::Fix {
        rate: 0.0,
        spread: 0.0,
    };
    let result = Calculator::calculate(&params).unwrap();
    assert_eq!(result.total_interest, 0.0);
    let expected_monthly = 100_000.0 / 120.0;
    assert!((result.payments[0].payment - expected_monthly).abs() < 0.01);
    assert!((result.payments[119].payment - expected_monthly).abs() < 0.01);
}

#[test]
fn test_prepayment_exceeds_balance() {
    let mut params = base_params();
    params.prepayments.push(Prepayment {
        date: NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        amount: 200_000.0,
        effect: PrepaymentEffect::ReduceTerm,
    });
    let result = Calculator::calculate(&params).unwrap();
    assert!((result.total_principal - 100_000.0).abs() < 1.0);
    assert!(result.payments.len() < 120);
}

#[test]
fn test_multiple_prepayments() {
    let mut params = base_params();
    params.prepayments.push(Prepayment {
        date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        amount: 10_000.0,
        effect: PrepaymentEffect::ReduceTerm,
    });
    params.prepayments.push(Prepayment {
        date: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
        amount: 10_000.0,
        effect: PrepaymentEffect::ReducePayment,
    });
    params.prepayments.push(Prepayment {
        date: NaiveDate::from_ymd_opt(2028, 1, 1).unwrap(),
        amount: 10_000.0,
        effect: PrepaymentEffect::ReduceTerm,
    });
    let result = Calculator::calculate(&params).unwrap();
    assert!((result.total_principal - 100_000.0).abs() < 1.0);
    assert!(result.payments.len() < 120);
}

#[test]
fn test_prepayment_at_start() {
    let mut params = base_params();
    params.prepayments.push(Prepayment {
        date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        amount: 50_000.0,
        effect: PrepaymentEffect::ReduceTerm,
    });
    let result = Calculator::calculate(&params).unwrap();
    assert!((result.total_principal - 100_000.0).abs() < 1.0);
}

#[test]
fn test_prepayment_at_end() {
    let mut params = base_params();
    params.prepayments.push(Prepayment {
        date: NaiveDate::from_ymd_opt(2034, 12, 1).unwrap(),
        amount: 10_000.0,
        effect: PrepaymentEffect::ReduceTerm,
    });
    let result = Calculator::calculate(&params).unwrap();
    assert!((result.total_principal - 100_000.0).abs() < 1.0);
}

#[test]
fn test_balance_never_negative() {
    let mut params = base_params();
    params.prepayments.push(Prepayment {
        date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        amount: 50_000.0,
        effect: PrepaymentEffect::ReduceTerm,
    });
    let result = Calculator::calculate(&params).unwrap();
    for p in &result.payments {
        assert!(
            p.remaining_balance >= 0.0,
            "Balance went negative: {}",
            p.remaining_balance
        );
    }
}

#[test]
fn test_total_paid_equals_principal_plus_interest() {
    let params = base_params();
    let result = Calculator::calculate(&params).unwrap();
    assert!((result.total_paid - (result.total_principal + result.total_interest)).abs() < 0.01);
}

#[test]
fn test_diff_first_payment_highest() {
    let mut params = base_params();
    params.payment_type = PaymentType::Diff;
    let result = Calculator::calculate(&params).unwrap();
    for i in 1..result.payments.len() {
        assert!(
            result.payments[i].payment <= result.payments[i - 1].payment + 0.01,
            "Diff payment should decrease: {} > {}",
            result.payments[i].payment,
            result.payments[i - 1].payment
        );
    }
}

#[test]
fn test_annuity_constant_payment() {
    let params = base_params();
    let result = Calculator::calculate(&params).unwrap();
    let expected = result.monthly_payment.unwrap();
    for (i, p) in result.payments.iter().enumerate() {
        if i < result.payments.len() - 1 {
            assert!(
                (p.payment - expected).abs() < 0.01,
                "Annuity payment should be constant: {} vs {}",
                p.payment,
                expected
            );
        }
    }
}

#[test]
fn test_validation_amount_too_large() {
    let mut params = base_params();
    params.amount = 200_000_000.0;
    let result = Calculator::calculate(&params);
    assert!(result.is_err());
}

#[test]
fn test_validation_term_too_large() {
    let mut params = base_params();
    params.term_years = 100;
    let result = Calculator::calculate(&params);
    assert!(result.is_err());
}

#[test]
fn test_yearly_summaries_basic() {
    let params = base_params();
    let result = Calculator::calculate(&params).unwrap();
    let summaries = result.yearly_summaries();
    assert_eq!(summaries.len(), 10);
    assert_eq!(summaries[0].year, 2025);
    assert_eq!(summaries[9].year, 2034);
    for s in &summaries {
        assert_eq!(s.payments_count, 12);
    }
}

#[test]
fn test_yearly_summaries_totals_match() {
    let params = base_params();
    let result = Calculator::calculate(&params).unwrap();
    let summaries = result.yearly_summaries();
    let total_payment: f64 = summaries.iter().map(|s| s.total_payment).sum();
    let total_principal: f64 = summaries.iter().map(|s| s.total_principal).sum();
    let total_interest: f64 = summaries.iter().map(|s| s.total_interest).sum();
    assert!((total_payment - result.total_paid).abs() < 0.01);
    assert!((total_principal - result.total_principal).abs() < 0.01);
    assert!((total_interest - result.total_interest).abs() < 0.01);
}
