pub mod calculator;
pub mod euribor;
pub mod models;

pub use calculator::Calculator;
pub use models::*;

use chrono::{Months, NaiveDate};
use rust_decimal::prelude::ToPrimitive;

// Legacy structs kept for backward compatibility during migration.
// New code should use models::LoanParams and calculator::Calculator.

#[derive(Debug, Clone)]
pub struct Loan {
    amount: f64,            // total loan sum
    interest_rate: f64,     // annual interest rate in percentage
    term_years: u32,        // total years
    payment_type: String,   // annuitet/diff
    start_date: Option<NaiveDate>,
}

impl Loan {
    pub fn new(
        amount: f64,
        interest_rate: f64,
        term_years: u32,
        payment_type: String,
        start_date: Option<NaiveDate>,
    ) -> Self {
        Loan {
            amount,
            interest_rate,
            term_years,
            payment_type,
            start_date,
        }
    }

    fn calculate_annuity(&self) -> LoanResultLegacy {
        let monthly_rate = self.interest_rate / 12.0 / 100.0;
        let term_months = self.term_years * 12;
        let numerator = monthly_rate * (1.0 + monthly_rate).powi(term_months as i32);
        let denominator = (1.0 + monthly_rate).powi(term_months as i32) - 1.0;
        let monthly_payment = self.amount * numerator / denominator;

        let mut balance = self.amount;
        let mut total_interest = 0.0;
        let mut payments = Vec::new();
        let mut current_date = self.start_date;

        for _ in 1..=term_months {
            let interest = balance * monthly_rate;
            let principal = monthly_payment - interest;
            balance -= principal;
            total_interest += interest;

            payments.push(PaymentLegacy {
                payment: monthly_payment.to_f64().unwrap(),
                date: current_date,
                principal: principal.to_f64().unwrap(),
                interest: interest.to_f64().unwrap(),
                remaining_balance: balance.to_f64().unwrap(),
            });

            current_date = Some(
                current_date
                    .expect("REASON")
                    .checked_add_months(Months::new(1))
                    .expect("Invalid date"),
            );
        }

        LoanResultLegacy {
            monthly_payment: Some(monthly_payment),
            total_interest,
            payments,
        }
    }

    fn calculate_diff(&self) -> LoanResultLegacy {
        let monthly_rate = self.interest_rate / 12.0 / 100.0;
        let term_months = self.term_years * 12;
        let principal_part = self.amount / term_months as f64;

        let mut balance = self.amount;
        let mut total_interest = 0.0;
        let mut payments = Vec::new();
        let mut current_date = self.start_date;

        for _ in 1..=term_months {
            let interest = balance * monthly_rate;
            let payment = principal_part + interest;
            balance -= principal_part;
            total_interest += interest;

            payments.push(PaymentLegacy {
                payment: payment.to_f64().unwrap(),
                date: current_date,
                principal: principal_part.to_f64().unwrap(),
                interest: interest.to_f64().unwrap(),
                remaining_balance: balance.to_f64().unwrap(),
            });

            current_date = Some(
                current_date
                    .expect("REASON")
                    .checked_add_months(Months::new(1))
                    .expect("Invalid date"),
            );
        }

        LoanResultLegacy {
            monthly_payment: None, // No fixed monthly payment
            total_interest,
            payments,
        }
    }

    pub fn calculate_loan(&self) -> LoanResultLegacy {
        match self.payment_type.as_str() {
            "annuitet" => self.calculate_annuity(),
            "diff" => self.calculate_diff(),
            _ => panic!("Unknown payment type"),
        }
    }
}

#[derive(Debug)]
pub struct PaymentLegacy {
    pub payment: f64,              // current payment amount
    pub date: Option<NaiveDate>,   // payment date
    pub principal: f64,            // pay for principal sum
    pub interest: f64,           // pay for interest sum
    pub remaining_balance: f64,
}

#[derive(Debug)]
pub struct LoanResultLegacy {
    pub monthly_payment: Option<f64>, // monthly payment amount
    pub total_interest: f64,          // total interest paid
    pub payments: Vec<PaymentLegacy>,   // vector of payments
}
