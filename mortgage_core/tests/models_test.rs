use chrono::NaiveDate;
use mortgage_core::models::*;

fn sample_params() -> LoanParams {
    LoanParams {
        amount: 200_000.0,
        term_years: 20,
        payment_type: PaymentType::Annuity,
        currency: Currency::Eur,
        start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        rate_mode: RateMode::Mixed {
            fix_years: 2.0,
            fix_rate: 2.5,
            fix_spread: 1.0,
            euribor_tenor: EuriborTenor::SixMonths,
            euribor_spread: 1.5,
        },
        same_spread: false,
        euribor_curve: vec![EuriborPoint {
            date_from: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            rate: 3.0,
        }],
        prepayments: vec![Prepayment {
            date: NaiveDate::from_ymd_opt(2028, 1, 1).unwrap(),
            amount: 50_000.0,
            effect: PrepaymentEffect::ReduceTerm,
        }],
        upfront_cost: None,
        upfront_percent: None,
    }
}

#[test]
fn test_currency_serde_roundtrip() {
    let currencies = vec![Currency::Usd, Currency::Eur];
    for c in currencies {
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: Currency = serde_json::from_str(&json).unwrap();
        assert_eq!(c, deserialized);
    }
}

#[test]
fn test_payment_type_serde_roundtrip() {
    let types = vec![PaymentType::Annuity, PaymentType::Diff];
    for t in types {
        let json = serde_json::to_string(&t).unwrap();
        let deserialized: PaymentType = serde_json::from_str(&json).unwrap();
        assert_eq!(t, deserialized);
    }
}

#[test]
fn test_euribor_tenor_serde_roundtrip() {
    let tenors = vec![
        EuriborTenor::OneMonth,
        EuriborTenor::ThreeMonths,
        EuriborTenor::SixMonths,
        EuriborTenor::TwelveMonths,
    ];
    for t in tenors {
        let json = serde_json::to_string(&t).unwrap();
        let deserialized: EuriborTenor = serde_json::from_str(&json).unwrap();
        assert_eq!(t, deserialized);
    }
}

#[test]
fn test_prepayment_effect_serde_roundtrip() {
    let effects = vec![
        PrepaymentEffect::ReduceTerm,
        PrepaymentEffect::ReducePayment,
    ];
    for e in effects {
        let json = serde_json::to_string(&e).unwrap();
        let deserialized: PrepaymentEffect = serde_json::from_str(&json).unwrap();
        assert_eq!(e, deserialized);
    }
}

#[test]
fn test_rate_mode_fix_serde_roundtrip() {
    let mode = RateMode::Fix {
        rate: 5.0,
        spread: 1.0,
    };
    let json = serde_json::to_string(&mode).unwrap();
    let deserialized: RateMode = serde_json::from_str(&json).unwrap();
    match deserialized {
        RateMode::Fix { rate, spread } => {
            assert_eq!(rate, 5.0);
            assert_eq!(spread, 1.0);
        }
        _ => panic!("Expected Fix rate mode"),
    }
}

#[test]
fn test_rate_mode_euribor_serde_roundtrip() {
    let mode = RateMode::Euribor {
        tenor: EuriborTenor::ThreeMonths,
        spread: 2.0,
    };
    let json = serde_json::to_string(&mode).unwrap();
    let deserialized: RateMode = serde_json::from_str(&json).unwrap();
    match deserialized {
        RateMode::Euribor { tenor, spread } => {
            assert_eq!(tenor, EuriborTenor::ThreeMonths);
            assert_eq!(spread, 2.0);
        }
        _ => panic!("Expected Euribor rate mode"),
    }
}

#[test]
fn test_rate_mode_mixed_serde_roundtrip() {
    let mode = RateMode::Mixed {
        fix_years: 3.0,
        fix_rate: 2.5,
        fix_spread: 1.0,
        euribor_tenor: EuriborTenor::SixMonths,
        euribor_spread: 1.5,
    };
    let json = serde_json::to_string(&mode).unwrap();
    let deserialized: RateMode = serde_json::from_str(&json).unwrap();
    match deserialized {
        RateMode::Mixed {
            fix_years,
            fix_rate,
            fix_spread,
            euribor_tenor,
            euribor_spread,
        } => {
            assert_eq!(fix_years, 3.0);
            assert_eq!(fix_rate, 2.5);
            assert_eq!(fix_spread, 1.0);
            assert_eq!(euribor_tenor, EuriborTenor::SixMonths);
            assert_eq!(euribor_spread, 1.5);
        }
        _ => panic!("Expected Mixed rate mode"),
    }
}

#[test]
fn test_prepayment_serde_roundtrip() {
    let prep = Prepayment {
        date: NaiveDate::from_ymd_opt(2028, 6, 1).unwrap(),
        amount: 25_000.0,
        effect: PrepaymentEffect::ReducePayment,
    };
    let json = serde_json::to_string(&prep).unwrap();
    let deserialized: Prepayment = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.date, prep.date);
    assert_eq!(deserialized.amount, prep.amount);
    assert_eq!(deserialized.effect, prep.effect);
}

#[test]
fn test_euribor_point_serde_roundtrip() {
    let point = EuriborPoint {
        date_from: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
        rate: 3.5,
    };
    let json = serde_json::to_string(&point).unwrap();
    let deserialized: EuriborPoint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.date_from, point.date_from);
    assert_eq!(deserialized.rate, point.rate);
}

#[test]
fn test_loan_params_full_serde_roundtrip() {
    let params = sample_params();
    let json = serde_json::to_string_pretty(&params).unwrap();
    let deserialized: LoanParams = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.amount, params.amount);
    assert_eq!(deserialized.term_years, params.term_years);
    assert_eq!(deserialized.payment_type, params.payment_type);
    assert_eq!(deserialized.currency, params.currency);
    assert_eq!(deserialized.start_date, params.start_date);
    assert_eq!(deserialized.same_spread, params.same_spread);
    assert_eq!(deserialized.euribor_curve.len(), params.euribor_curve.len());
    assert_eq!(deserialized.prepayments.len(), params.prepayments.len());
}

#[test]
fn test_loan_result_serde_roundtrip() {
    use mortgage_core::Calculator;

    let params = LoanParams {
        amount: 100_000.0,
        term_years: 5,
        payment_type: PaymentType::Annuity,
        currency: Currency::Usd,
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
    };

    let result = Calculator::calculate(&params).unwrap();
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: LoanResult = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.payments.len(), result.payments.len());
    assert!((deserialized.total_paid - result.total_paid).abs() < 0.01);
    assert_eq!(
        deserialized.principal_exceeds_interest_at,
        result.principal_exceeds_interest_at
    );
}

#[test]
fn test_display_currency() {
    assert_eq!(Currency::Usd.to_string(), "USD");
    assert_eq!(Currency::Eur.to_string(), "EUR");
}

#[test]
fn test_display_payment_type() {
    assert_eq!(PaymentType::Annuity.to_string(), "Annuity");
    assert_eq!(PaymentType::Diff.to_string(), "Diff");
}

#[test]
fn test_display_euribor_tenor() {
    assert_eq!(EuriborTenor::OneMonth.to_string(), "1m");
    assert_eq!(EuriborTenor::ThreeMonths.to_string(), "3m");
    assert_eq!(EuriborTenor::SixMonths.to_string(), "6m");
    assert_eq!(EuriborTenor::TwelveMonths.to_string(), "12m");
}

#[test]
fn test_display_prepayment_effect() {
    assert_eq!(PrepaymentEffect::ReduceTerm.to_string(), "ReduceTerm");
    assert_eq!(PrepaymentEffect::ReducePayment.to_string(), "ReducePayment");
}

#[test]
fn test_from_str_currency() {
    assert_eq!("USD".parse::<Currency>().unwrap(), Currency::Usd);
    assert_eq!("eur".parse::<Currency>().unwrap(), Currency::Eur);
    assert_eq!("EUR".parse::<Currency>().unwrap(), Currency::Eur);
    assert!("GBP".parse::<Currency>().is_err());
}

#[test]
fn test_from_str_payment_type() {
    assert_eq!(
        "annuity".parse::<PaymentType>().unwrap(),
        PaymentType::Annuity
    );
    assert_eq!(
        "annuitet".parse::<PaymentType>().unwrap(),
        PaymentType::Annuity
    );
    assert_eq!("diff".parse::<PaymentType>().unwrap(), PaymentType::Diff);
    assert!("unknown".parse::<PaymentType>().is_err());
}

#[test]
fn test_from_str_euribor_tenor() {
    assert_eq!(
        "1m".parse::<EuriborTenor>().unwrap(),
        EuriborTenor::OneMonth
    );
    assert_eq!(
        "3m".parse::<EuriborTenor>().unwrap(),
        EuriborTenor::ThreeMonths
    );
    assert_eq!(
        "6m".parse::<EuriborTenor>().unwrap(),
        EuriborTenor::SixMonths
    );
    assert_eq!(
        "12m".parse::<EuriborTenor>().unwrap(),
        EuriborTenor::TwelveMonths
    );
    assert!("24m".parse::<EuriborTenor>().is_err());
}

#[test]
fn test_from_str_prepayment_effect() {
    assert_eq!(
        "ReduceTerm".parse::<PrepaymentEffect>().unwrap(),
        PrepaymentEffect::ReduceTerm
    );
    assert_eq!(
        "ReducePayment".parse::<PrepaymentEffect>().unwrap(),
        PrepaymentEffect::ReducePayment
    );
    assert!("Unknown".parse::<PrepaymentEffect>().is_err());
}
