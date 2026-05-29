use clap::{Parser, ValueEnum};
use mortgage_core::models::*;
use mortgage_core::Calculator;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
enum CliPaymentType {
    Annuity,
    Diff,
}

#[derive(Debug, Clone, ValueEnum)]
enum CliCurrency {
    Usd,
    Eur,
}

#[derive(Debug, Clone, ValueEnum)]
enum CliRateMode {
    Fix,
    Euribor,
    Mixed,
}

#[derive(Debug, Clone, ValueEnum)]
enum CliEuriborTenor {
    #[value(name = "1m")]
    OneMonth,
    #[value(name = "3m")]
    ThreeMonths,
    #[value(name = "6m")]
    SixMonths,
    #[value(name = "12m")]
    TwelveMonths,
}

#[derive(Parser, Debug)]
#[command(name = "mortgage_cli")]
#[command(about = "Mortgage calculator CLI")]
struct Args {
    /// Loan amount
    #[arg(short, long)]
    amount: Option<f64>,

    /// Loan term in years
    #[arg(short, long)]
    term: Option<u32>,

    /// Payment type: annuity or diff
    #[arg(short, long, value_enum)]
    payment_type: Option<CliPaymentType>,

    /// Currency: usd or eur
    #[arg(short = 'u', long, value_enum)]
    currency: Option<CliCurrency>,

    /// Start date (YYYY-MM-DD)
    #[arg(long)]
    start_date: Option<String>,

    /// Rate mode: fix, euribor, or mixed
    #[arg(long, value_enum)]
    rate_mode: Option<CliRateMode>,

    /// Base rate (for fix) or fix rate (for mixed)
    #[arg(short, long)]
    rate: Option<f64>,

    /// Bank spread
    #[arg(long)]
    spread: Option<f64>,

    /// Fixed period in years (for mixed mode)
    #[arg(long)]
    fix_years: Option<f64>,

    /// Euribor tenor (for euribor or mixed mode)
    #[arg(long, value_enum)]
    euribor_tenor: Option<CliEuriborTenor>,

    /// Euribor spread (for euribor or mixed mode)
    #[arg(long)]
    euribor_spread: Option<f64>,

    /// Use same spread for fixed and euribor periods
    #[arg(long)]
    same_spread: bool,

    /// Path to JSON config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Output file path (CSV)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format: table or csv
    #[arg(long, default_value = "table")]
    format: String,

    /// Number of payments to display in table mode
    #[arg(long, default_value = "24")]
    limit: usize,
}

fn main() {
    let args = Args::parse();

    let params = if let Some(config_path) = args.config {
        let content = fs::read_to_string(&config_path)
            .unwrap_or_else(|e| panic!("Failed to read config file: {}", e));
        serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse config JSON: {}", e))
    } else {
        build_params_from_args(&args)
    };

    let result = Calculator::calculate(&params);

    if let Some(output_path) = args.output {
        let csv = payments_to_csv(&result.payments);
        fs::write(&output_path, csv)
            .unwrap_or_else(|e| panic!("Failed to write output file: {}", e));
        println!("Saved {} payments to {}", result.payments.len(), output_path.display());
    } else if args.format == "csv" {
        print!("{}", payments_to_csv(&result.payments));
    } else {
        print_summary(&params, &result);
        print_table(&result.payments, args.limit);
    }
}

fn build_params_from_args(args: &Args) -> LoanParams {
    let amount = args.amount.unwrap_or(100_000.0);
    let term_years = args.term.unwrap_or(10);
    let payment_type = match args.payment_type.as_ref().unwrap_or(&CliPaymentType::Annuity) {
        CliPaymentType::Annuity => PaymentType::Annuity,
        CliPaymentType::Diff => PaymentType::Diff,
    };
    let currency = match args.currency.as_ref().unwrap_or(&CliCurrency::Eur) {
        CliCurrency::Usd => Currency::Usd,
        CliCurrency::Eur => Currency::Eur,
    };
    let start_date = args
        .start_date
        .as_ref()
        .and_then(|s| s.parse::<chrono::NaiveDate>().ok())
        .unwrap_or_else(|| chrono::Local::now().date_naive());

    let rate_mode = match args.rate_mode.as_ref().unwrap_or(&CliRateMode::Fix) {
        CliRateMode::Fix => RateMode::Fix {
            rate: args.rate.unwrap_or(5.0),
            spread: args.spread.unwrap_or(0.0),
        },
        CliRateMode::Euribor => RateMode::Euribor {
            tenor: args
                .euribor_tenor
                .as_ref()
                .map(|t| match t {
                    CliEuriborTenor::OneMonth => EuriborTenor::OneMonth,
                    CliEuriborTenor::ThreeMonths => EuriborTenor::ThreeMonths,
                    CliEuriborTenor::SixMonths => EuriborTenor::SixMonths,
                    CliEuriborTenor::TwelveMonths => EuriborTenor::TwelveMonths,
                })
                .unwrap_or_default(),
            spread: args.spread.unwrap_or(0.0),
        },
        CliRateMode::Mixed => RateMode::Mixed {
            fix_years: args.fix_years.unwrap_or(1.0),
            fix_rate: args.rate.unwrap_or(3.0),
            fix_spread: args.spread.unwrap_or(1.0),
            euribor_tenor: args
                .euribor_tenor
                .as_ref()
                .map(|t| match t {
                    CliEuriborTenor::OneMonth => EuriborTenor::OneMonth,
                    CliEuriborTenor::ThreeMonths => EuriborTenor::ThreeMonths,
                    CliEuriborTenor::SixMonths => EuriborTenor::SixMonths,
                    CliEuriborTenor::TwelveMonths => EuriborTenor::TwelveMonths,
                })
                .unwrap_or_default(),
            euribor_spread: args.euribor_spread.unwrap_or(2.0),
        },
    };

    LoanParams {
        amount,
        term_years,
        payment_type,
        currency,
        start_date,
        rate_mode,
        same_spread: args.same_spread,
        euribor_curve: vec![],
        prepayments: vec![],
    }
}

fn print_summary(params: &LoanParams, result: &LoanResult) {
    let sym = params.currency.symbol();
    println!("=== Loan Summary ===");
    println!("Amount:        {}{:.2}", sym, params.amount);
    println!("Term:          {} years", params.term_years);
    println!("Payment type:  {:?}", params.payment_type);
    println!("Rate mode:     {:?}", params.rate_mode);
    if let Some(mp) = result.monthly_payment {
        println!("Monthly pay:   {}{:.2}", sym, mp);
    }
    println!("Total principal: {}{:.2}", sym, result.total_principal);
    println!("Total interest:  {}{:.2}", sym, result.total_interest);
    println!("Total paid:      {}{:.2}", sym, result.total_paid);
    println!("Payments count:  {}", result.payments.len());
    if let Some(idx) = result.principal_exceeds_interest_at {
        println!("Principal > Interest at payment #{} ({})", idx + 1, result.payments[idx].date);
    }
    println!();
}

fn print_table(payments: &[Payment], limit: usize) {
    println!("{:>5} {:>12} {:>12} {:>12} {:>12} {:>12}",
             "#", "Date", "Payment", "Principal", "Interest", "Balance");
    println!("{}", "-".repeat(70));
    for (i, p) in payments.iter().take(limit).enumerate() {
        println!("{:>5} {:>12} {:>12.2} {:>12.2} {:>12.2} {:>12.2}",
                 i + 1,
                 p.date,
                 p.payment,
                 p.principal,
                 p.interest,
                 p.remaining_balance);
    }
    if payments.len() > limit {
        println!("... ({} more payments)", payments.len() - limit);
    }
}

fn payments_to_csv(payments: &[Payment]) -> String {
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
