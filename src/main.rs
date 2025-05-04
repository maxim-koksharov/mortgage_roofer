pub mod mortgage_logic;

fn main() {
    let loan = mortgage_logic::Loan::new(
        100000.0,
        5.0,
        15,
        "annuitet".to_string(),
        chrono::NaiveDate::from_ymd_opt(2023, 1, 1),
    );

    let result = loan.calculate_annuity();

    println!("Monthly Payment: {:?}", result.monthly_payment);
    println!("Total Interest: {:?}", result.total_interest);
    for payment in result.payments {
        println!(
            "Payment: {:?}, Date: {:?}, Principal: {:?}, Interest: {:?}, Remaining Balance: {:?}",
            payment.payment, payment.date, payment.principal, payment.interest, payment.remaining_balance
        );
    }
}
