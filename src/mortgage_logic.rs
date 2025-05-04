use chrono::{Months, NaiveDate};
// use rust_decimal::Decimal;
// use rust_decimal::prelude::FromPrimitive;
use rust_decimal::prelude::ToPrimitive;
// use rust_decimal_macros::dec;

#[derive(Debug, Clone)]
pub struct Loan {
    amount: f64, // total loan sum
    interest_rate: f64, // annual interest rate in percentage
    term_years: u32, // total years
    payment_type: String, // annuitet/diff
    start_date: Option<NaiveDate>,
}

#[derive(Debug)]
pub struct Payment {
    pub payment: f64, // current payment amount
    pub date: Option<NaiveDate>, // payment date
    pub principal: f64, // pay for principal sum
    pub interest: f64, // pay for interest sum
    pub remaining_balance: f64,
}

#[derive(Debug)]
pub struct LoanResult {
    pub monthly_payment: Option<f64>, // monthly payment amount
    pub total_interest: f64, // total interest paid
    pub payments: Vec<Payment>, // vector of payments
}

impl Loan {
    pub fn new(amount: f64, interest_rate: f64, term_years: u32, payment_type: String, start_date: Option<NaiveDate>) -> Self {
        Loan {
            amount,
            interest_rate,
            term_years,
            payment_type,
            start_date,
        }
    }

    pub fn calculate_annuity(&self) -> LoanResult {
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

            payments.push(Payment {
                payment: monthly_payment.to_f64().unwrap(),
                date: current_date,
                principal: principal.to_f64().unwrap(),
                interest: interest.to_f64().unwrap(),
                remaining_balance: balance.to_f64().unwrap(),
            });

            current_date = Some(current_date.expect("REASON").checked_add_months(Months::new(1)).expect("Invalid date"));
        }

        LoanResult {
            monthly_payment: Some(monthly_payment),
            total_interest: total_interest,
            payments,
        }
    }

    pub fn calculate_diff(&self) -> LoanResult {
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

            payments.push(Payment {
                payment: payment.to_f64().unwrap(),
                date: current_date,
                principal: principal_part.to_f64().unwrap(),
                interest: interest.to_f64().unwrap(),
                remaining_balance: balance.to_f64().unwrap(),
            });

            current_date = Some(current_date.expect("REASON").checked_add_months(Months::new(1)).expect("Invalid date"));
        }

        LoanResult {
            monthly_payment: None, // No fixed monthly payment
            total_interest: total_interest,
            payments,
        }
    }
}

impl Payment {
    fn new(payment: f64, date: Option<NaiveDate>, principal: f64, interest: f64, remaining_balance: f64) -> Self {
        Payment {
            payment,
            date,
            principal,
            interest,
            remaining_balance,
        }
    }
}

impl LoanResult {
    fn new(monthly_payment: Option<f64>, total_interest: f64, payments: Vec<Payment>) -> Self {
        LoanResult {
            monthly_payment,
            total_interest,
            payments,
        }
    }
}