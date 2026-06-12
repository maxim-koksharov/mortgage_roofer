use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Currency::Usd => write!(f, "USD"),
            Currency::Eur => write!(f, "EUR"),
        }
    }
}

impl FromStr for Currency {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Currency::Usd),
            "EUR" => Ok(Currency::Eur),
            _ => Err(format!("Unknown currency: {}", s)),
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

impl fmt::Display for EuriborTenor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for EuriborTenor {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1m" => Ok(EuriborTenor::OneMonth),
            "3m" => Ok(EuriborTenor::ThreeMonths),
            "6m" => Ok(EuriborTenor::SixMonths),
            "12m" => Ok(EuriborTenor::TwelveMonths),
            _ => Err(format!("Unknown tenor: {}", s)),
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

impl fmt::Display for PaymentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentType::Annuity => write!(f, "Annuity"),
            PaymentType::Diff => write!(f, "Diff"),
        }
    }
}

impl FromStr for PaymentType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "annuity" | "annuitet" => Ok(PaymentType::Annuity),
            "diff" => Ok(PaymentType::Diff),
            _ => Err(format!("Unknown payment type: {}", s)),
        }
    }
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

impl fmt::Display for PrepaymentEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrepaymentEffect::ReduceTerm => write!(f, "ReduceTerm"),
            PrepaymentEffect::ReducePayment => write!(f, "ReducePayment"),
        }
    }
}

impl FromStr for PrepaymentEffect {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ReduceTerm" => Ok(PrepaymentEffect::ReduceTerm),
            "ReducePayment" => Ok(PrepaymentEffect::ReducePayment),
            _ => Err(format!("Unknown prepayment effect: {}", s)),
        }
    }
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

impl LoanParams {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.amount <= 0.0 {
            errors.push("Loan amount must be positive".to_string());
        }
        if self.amount > 100_000_000.0 {
            errors.push("Loan amount exceeds maximum (100,000,000)".to_string());
        }
        if self.term_years == 0 {
            errors.push("Loan term must be at least 1 year".to_string());
        }
        if self.term_years > 50 {
            errors.push("Loan term exceeds maximum (50 years)".to_string());
        }

        match &self.rate_mode {
            RateMode::Fix { rate, spread } => {
                if *rate < 0.0 {
                    errors.push("Interest rate cannot be negative".to_string());
                }
                if *rate > 100.0 {
                    errors.push("Interest rate exceeds 100%".to_string());
                }
                if *spread < 0.0 {
                    errors.push("Spread cannot be negative".to_string());
                }
            }
            RateMode::Euribor { spread, .. } => {
                if *spread < 0.0 {
                    errors.push("Spread cannot be negative".to_string());
                }
            }
            RateMode::Mixed {
                fix_years,
                fix_rate,
                fix_spread,
                euribor_spread,
                ..
            } => {
                if *fix_years <= 0.0 {
                    errors.push("Fixed period must be positive".to_string());
                }
                if *fix_rate < 0.0 {
                    errors.push("Fixed rate cannot be negative".to_string());
                }
                if *fix_spread < 0.0 {
                    errors.push("Fixed spread cannot be negative".to_string());
                }
                if !self.same_spread && *euribor_spread < 0.0 {
                    errors.push("Euribor spread cannot be negative".to_string());
                }
            }
        }

        for (i, prep) in self.prepayments.iter().enumerate() {
            if prep.amount <= 0.0 {
                errors.push(format!("Prepayment #{} amount must be positive", i + 1));
            }
            if prep.date < self.start_date {
                errors.push(format!(
                    "Prepayment #{} date ({}) is before loan start date ({})",
                    i + 1,
                    prep.date,
                    self.start_date
                ));
            }
        }

        errors
    }
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

/// Yearly aggregate of payments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlySummary {
    pub year: i32,
    pub total_payment: f64,
    pub total_principal: f64,
    pub total_interest: f64,
    pub payments_count: usize,
    pub ending_balance: f64,
}

impl LoanResult {
    pub fn yearly_summaries(&self) -> Vec<YearlySummary> {
        use chrono::Datelike;
        let mut summaries: Vec<YearlySummary> = Vec::new();

        for p in &self.payments {
            let year = p.date.year();
            if let Some(last) = summaries.last_mut() {
                if last.year == year {
                    last.total_payment += p.payment;
                    last.total_principal += p.principal;
                    last.total_interest += p.interest;
                    last.payments_count += 1;
                    last.ending_balance = p.remaining_balance;
                    continue;
                }
            }
            summaries.push(YearlySummary {
                year,
                total_payment: p.payment,
                total_principal: p.principal,
                total_interest: p.interest,
                payments_count: 1,
                ending_balance: p.remaining_balance,
            });
        }

        summaries
    }
}
