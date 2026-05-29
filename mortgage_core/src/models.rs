use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Currency for formatting only; does not affect calculation logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    Usd,
    Eur,
}

impl Currency {
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::Usd => "$",
            Currency::Eur => "€",
        }
    }
}

/// Euribor tenor (maturity period).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EuriborTenor {
    #[serde(rename = "1m")]
    OneMonth,
    #[serde(rename = "3m")]
    ThreeMonths,
    #[serde(rename = "6m")]
    SixMonths,
    #[serde(rename = "12m")]
    TwelveMonths,
}

impl Default for EuriborTenor {
    fn default() -> Self {
        EuriborTenor::SixMonths
    }
}

impl EuriborTenor {
    pub fn as_str(&self) -> &'static str {
        match self {
            EuriborTenor::OneMonth => "1m",
            EuriborTenor::ThreeMonths => "3m",
            EuriborTenor::SixMonths => "6m",
            EuriborTenor::TwelveMonths => "12m",
        }
    }
}

/// Payment schedule type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentType {
    #[serde(rename = "annuitet")]
    Annuity,
    #[serde(rename = "diff")]
    Diff,
}

/// How the interest rate is determined over the loan life.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateMode {
    /// Fixed rate for the entire term.
    Fix {
        rate: f64,
        spread: f64,
    },
    /// Euribor-linked rate for the entire term.
    Euribor {
        tenor: EuriborTenor,
        spread: f64,
    },
    /// Fixed for an initial period, then switches to Euribor+spread.
    Mixed {
        fix_years: f64,
        fix_rate: f64,
        fix_spread: f64,
        euribor_tenor: EuriborTenor,
        euribor_spread: f64,
    },
}

/// Effect of a prepayment on the loan schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrepaymentEffect {
    /// Reduce the remaining term; monthly payment stays roughly the same.
    ReduceTerm,
    /// Reduce the monthly payment; term stays the same.
    ReducePayment,
}

/// A single prepayment event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prepayment {
    pub date: NaiveDate,
    pub amount: f64,
    pub effect: PrepaymentEffect,
}

/// User-defined Euribor curve point: from this date onward use the given rate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EuriborPoint {
    pub date_from: NaiveDate,
    pub rate: f64,
}

/// Full set of loan parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanParams {
    pub amount: f64,
    pub term_years: u32,
    pub payment_type: PaymentType,
    pub currency: Currency,
    pub start_date: NaiveDate,
    pub rate_mode: RateMode,
    /// If true, the spread from the fixed period is reused for the Euribor period.
    pub same_spread: bool,
    /// Optional manual Euribor curve (overrides fetched values).
    pub euribor_curve: Vec<EuriborPoint>,
    /// Optional prepayments.
    pub prepayments: Vec<Prepayment>,
}

/// A single monthly payment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub payment: f64,
    pub date: NaiveDate,
    pub principal: f64,
    pub interest: f64,
    pub remaining_balance: f64,
    /// The effective annual rate (in %) used for this payment.
    pub applied_rate: f64,
}

/// Result of a loan schedule calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanResult {
    /// Fixed monthly payment for annuity; None for diff.
    pub monthly_payment: Option<f64>,
    pub total_principal: f64,
    pub total_interest: f64,
    pub total_paid: f64,
    pub payments: Vec<Payment>,
    /// Index of the first payment where principal > interest, if any.
    pub principal_exceeds_interest_at: Option<usize>,
}
