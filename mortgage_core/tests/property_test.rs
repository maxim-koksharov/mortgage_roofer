use chrono::NaiveDate;
use mortgage_core::Calculator;
use mortgage_core::models::*;
use proptest::prelude::*;

fn valid_params_strategy() -> impl Strategy<Value = LoanParams> {
    (1000.0f64..1_000_000.0, 1u32..30, 0.0f64..20.0, 0.0f64..5.0).prop_map(
        |(amount, term_years, rate, spread)| LoanParams {
            amount,
            term_years,
            payment_type: PaymentType::Annuity,
            currency: Currency::Eur,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            rate_mode: RateMode::Fix { rate, spread },
            same_spread: false,
            euribor_curve: vec![],
            prepayments: vec![],
            upfront_cost: None,
            upfront_percent: None,
            down_payment: None,
        },
    )
}

proptest! {
    #[test]
    fn prop_total_principal_equals_amount(params in valid_params_strategy()) {
        let result = Calculator::calculate(&params).unwrap();
        prop_assert!(
            (result.total_principal - params.amount).abs() < 1.0,
            "total_principal {} != amount {}",
            result.total_principal,
            params.amount
        );
    }

    #[test]
    fn prop_total_paid_equals_sum(params in valid_params_strategy()) {
        let result = Calculator::calculate(&params).unwrap();
        let sum: f64 = result.payments.iter().map(|p| p.payment).sum();
        prop_assert!(
            (sum - result.total_paid).abs() < 1.0,
            "sum of payments {} != total_paid {}",
            sum,
            result.total_paid
        );
    }

    #[test]
    fn prop_balance_non_negative(params in valid_params_strategy()) {
        let result = Calculator::calculate(&params).unwrap();
        for p in &result.payments {
            prop_assert!(p.remaining_balance >= -0.01, "negative balance: {}", p.remaining_balance);
        }
    }

    #[test]
    fn prop_final_balance_near_zero(params in valid_params_strategy()) {
        let result = Calculator::calculate(&params).unwrap();
        if let Some(last) = result.payments.last() {
            prop_assert!(last.remaining_balance < 1.0, "final balance: {}", last.remaining_balance);
        }
    }

    #[test]
    fn prop_payments_count_matches_term(params in valid_params_strategy()) {
        let result = Calculator::calculate(&params).unwrap();
        let expected = params.term_years * 12;
        prop_assert!(
            result.payments.len() <= expected as usize,
            "payments {} > expected {}",
            result.payments.len(),
            expected
        );
    }

    #[test]
    fn prop_interest_non_negative(params in valid_params_strategy()) {
        let result = Calculator::calculate(&params).unwrap();
        prop_assert!(result.total_interest >= 0.0);
        for p in &result.payments {
            prop_assert!(p.interest >= -0.01, "negative interest: {}", p.interest);
        }
    }

    #[test]
    fn prop_principal_non_negative(params in valid_params_strategy()) {
        let result = Calculator::calculate(&params).unwrap();
        for p in &result.payments {
            prop_assert!(p.principal >= -0.01, "negative principal: {}", p.principal);
        }
    }

    #[test]
    fn prop_payment_equals_principal_plus_interest(params in valid_params_strategy()) {
        let result = Calculator::calculate(&params).unwrap();
        for p in &result.payments {
            prop_assert!(
                (p.payment - (p.principal + p.interest)).abs() < 0.01,
                "payment {} != principal {} + interest {}",
                p.payment, p.principal, p.interest
            );
        }
    }
}
