pub mod mortgage_logic;

fn main() {
    let loan = mortgage_logic::Loan::new(
        185000.0,
        3.6,
        30,
        "annuitet".to_string(),
        chrono::NaiveDate::from_ymd_opt(2025, 5, 12),
    );

    let result = loan.calculate_loan();

    println!("Monthly Payment: {:?}", result.monthly_payment);
    println!("Total Interest: {:?}", result.total_interest);
    for payment in result.payments {
        println!(
            "Payment: {:?}, Date: {:?}, Principal: {:?}, Interest: {:?}, Remaining Balance: {:?}",
            payment.payment, payment.date, payment.principal, payment.interest, payment.remaining_balance
        );
    }
}
