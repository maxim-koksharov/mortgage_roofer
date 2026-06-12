use crate::models::Payment;

pub fn payments_to_csv(payments: &[Payment]) -> String {
    let mut lines = vec![
        "date,payment,principal,interest,remaining_balance,applied_rate".to_string(),
    ];
    for p in payments {
        lines.push(format!(
            "{},{:.2},{:.2},{:.2},{:.2},{:.4}",
            p.date, p.payment, p.principal, p.interest, p.remaining_balance, p.applied_rate
        ));
    }
    lines.join("\n") + "\n"
}
