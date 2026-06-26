use clap::{Parser, ValueEnum};
use mortgage_core::euribor::EuriborCache;
use mortgage_core::models::*;
use mortgage_core::{Calculator, payments_to_csv};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, ValueEnum)]
enum CliRateMode {
    Fix,
    Euribor,
    Mixed,
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
    #[arg(short, long)]
    payment_type: Option<String>,

    /// Currency: usd or eur
    #[arg(short = 'u', long)]
    currency: Option<String>,

    /// Start date (DD-MM-YYYY)
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
    #[arg(long)]
    euribor_tenor: Option<String>,

    /// Euribor spread (for euribor or mixed mode)
    #[arg(long)]
    euribor_spread: Option<f64>,

    /// Use same spread for fixed and euribor periods
    #[arg(long)]
    same_spread: bool,

    /// Upfront costs for break-even analysis (fixed amount)
    #[arg(long)]
    upfront_cost: Option<f64>,

    /// Upfront costs for break-even analysis (percent of loan amount)
    #[arg(long)]
    upfront_percent: Option<f64>,

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

    /// Show yearly summary instead of monthly table
    #[arg(long)]
    yearly: bool,

    /// Prepayment in format "DD-MM-YYYY:amount:effect" (effect: ReduceTerm or ReducePayment). Can be repeated.
    #[arg(long = "prepayment", num_args = 1)]
    prepayments: Vec<String>,

    /// Save session to JSON file after calculation
    #[arg(long)]
    save: Option<PathBuf>,

    /// Load session from JSON file (overrides other args)
    #[arg(long)]
    load: Option<PathBuf>,

    /// Show rate sensitivity analysis with given deltas (comma-separated, e.g. "-1,-0.5,0,0.5,1")
    #[arg(long)]
    sensitivity: Option<String>,

    /// Show break-even analysis vs rent (monthly rent amount)
    #[arg(long)]
    break_even_rent: Option<f64>,
}

fn main() {
    let args = Args::parse();

    let (params, result) = if let Some(load_path) = args.load {
        let session = mortgage_core::load_session(&load_path)
            .unwrap_or_else(|e| panic!("Failed to load session: {}", e));
        (session.params, session.result)
    } else {
        let mut params = if let Some(config_path) = args.config {
            let content = fs::read_to_string(&config_path)
                .unwrap_or_else(|e| panic!("Failed to read config file: {}", e));
            serde_json::from_str(&content)
                .unwrap_or_else(|e| panic!("Failed to parse config JSON: {}", e))
        } else {
            build_params_from_args(&args)
        };

        for prep_str in &args.prepayments {
            params.prepayments.push(parse_prepayment(prep_str));
        }

        let result = match Calculator::calculate(&params) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Calculation error: {}", e);
                std::process::exit(1);
            }
        };
        (params, result)
    };

    if let Some(save_path) = args.save {
        mortgage_core::save_session(&save_path, &params, &result)
            .unwrap_or_else(|e| panic!("Failed to save session: {}", e));
        println!("Session saved to {}", save_path.display());
    }

    if let Some(sensitivity_str) = &args.sensitivity {
        let deltas: Vec<f64> = sensitivity_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        let points = mortgage_core::sensitivity_analysis(&params, &deltas);
        print_sensitivity(&points);
    }

    if let Some(rent) = args.break_even_rent {
        let be = mortgage_core::break_even_analysis(&params, rent);
        print_break_even(&be);
    }

    if let Some(output_path) = args.output {
        let csv = payments_to_csv(&result.payments);
        fs::write(&output_path, csv)
            .unwrap_or_else(|e| panic!("Failed to write output file: {}", e));
        println!(
            "Saved {} payments to {}",
            result.payments.len(),
            output_path.display()
        );
    } else if args.format == "csv" {
        print!("{}", payments_to_csv(&result.payments));
    } else {
        print_summary(&params, &result);
        if args.yearly {
            print_yearly(&result);
        } else {
            print_table(&result.payments, args.limit);
        }
    }
}

fn parse_prepayment(prep_str: &str) -> Prepayment {
    let parts: Vec<&str> = prep_str.split(':').collect();
    if parts.len() != 3 {
        eprintln!(
            "Invalid prepayment format: {}. Use DD-MM-YYYY:amount:effect",
            prep_str
        );
        std::process::exit(1);
    }
    let date = chrono::NaiveDate::parse_from_str(parts[0], "%d-%m-%Y")
        .unwrap_or_else(|_| panic!("Invalid prepayment date: {}", parts[0]));
    let amount: f64 = parts[1]
        .parse()
        .unwrap_or_else(|_| panic!("Invalid prepayment amount: {}", parts[1]));
    let effect = parts[2].parse::<PrepaymentEffect>().unwrap_or_else(|_| {
        panic!(
            "Invalid prepayment effect: {}. Use ReduceTerm or ReducePayment",
            parts[2]
        )
    });
    Prepayment {
        date,
        amount,
        effect,
    }
}

fn parse_or<T: FromStr>(value: Option<&String>, default: T) -> T
where
    T::Err: std::fmt::Display,
{
    match value {
        Some(s) => s
            .parse::<T>()
            .unwrap_or_else(|e| panic!("Invalid value '{}': {}", s, e)),
        None => default,
    }
}

fn build_params_from_args(args: &Args) -> LoanParams {
    let amount = args.amount.unwrap_or(100_000.0);
    let term_years = args.term.unwrap_or(10);
    let payment_type = parse_or(args.payment_type.as_ref(), PaymentType::Annuity);
    let currency = parse_or(args.currency.as_ref(), Currency::Eur);
    let start_date = args
        .start_date
        .as_ref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%d-%m-%Y").ok())
        .unwrap_or_else(|| chrono::Local::now().date_naive());
    let euribor_tenor = parse_or(args.euribor_tenor.as_ref(), EuriborTenor::SixMonths);

    let rate_mode = match args.rate_mode.as_ref().unwrap_or(&CliRateMode::Fix) {
        CliRateMode::Fix => RateMode::Fix {
            rate: args.rate.unwrap_or(5.0),
            spread: args.spread.unwrap_or(0.0),
        },
        CliRateMode::Euribor => RateMode::Euribor {
            tenor: euribor_tenor,
            spread: args.spread.unwrap_or(0.0),
        },
        CliRateMode::Mixed => RateMode::Mixed {
            fix_years: args.fix_years.unwrap_or(1.0),
            fix_rate: args.rate.unwrap_or(3.0),
            fix_spread: args.spread.unwrap_or(1.0),
            euribor_tenor,
            euribor_spread: args.euribor_spread.unwrap_or(2.0),
        },
    };

    let euribor_curve = match &rate_mode {
        RateMode::Euribor { tenor, .. }
        | RateMode::Mixed {
            euribor_tenor: tenor,
            ..
        } => {
            let curve_start = if let RateMode::Mixed { fix_years, .. } = &rate_mode {
                start_date
                    .checked_add_months(chrono::Months::new((fix_years * 12.0).round() as u32))
                    .unwrap_or(start_date)
            } else {
                start_date
            };
            let today = chrono::Local::now().date_naive();
            let ecb_end = today;
            let ecb_start = curve_start.min(
                today
                    .checked_sub_months(chrono::Months::new(3))
                    .unwrap_or(today),
            );
            eprintln!("Loading Euribor {} historical data...", tenor);
            let mut cache = EuriborCache::default();
            match cache.fetch_historical(*tenor, ecb_start, ecb_end) {
                Ok(points) => {
                    eprintln!(
                        "Auto-fetched {} Euribor {} historical points",
                        points.len(),
                        tenor
                    );
                    points
                }
                Err(e) => {
                    eprintln!("Warning: Euribor fetch failed ({}), using empty curve", e);
                    vec![]
                }
            }
        }
        _ => vec![],
    };

    LoanParams {
        amount,
        term_years,
        payment_type,
        currency,
        start_date,
        rate_mode,
        same_spread: args.same_spread,
        euribor_curve,
        prepayments: vec![],
        upfront_cost: args.upfront_cost,
        upfront_percent: args.upfront_percent,
        down_payment: None,
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
        println!(
            "Principal > Interest at payment #{} ({})",
            idx + 1,
            result.payments[idx].date
        );
    }
    println!();
}

fn print_table(payments: &[Payment], limit: usize) {
    println!(
        "{:>5} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "#", "Date", "Payment", "Principal", "Interest", "Balance"
    );
    println!("{}", "-".repeat(70));
    for (i, p) in payments.iter().take(limit).enumerate() {
        println!(
            "{:>5} {:>12} {:>12.2} {:>12.2} {:>12.2} {:>12.2}",
            i + 1,
            p.date,
            p.payment,
            p.principal,
            p.interest,
            p.remaining_balance
        );
    }
    if payments.len() > limit {
        println!("... ({} more payments)", payments.len() - limit);
    }
}

fn print_yearly(result: &LoanResult) {
    let summaries = result.yearly_summaries();
    println!("=== Yearly Summary ===");
    println!(
        "{:>6} {:>14} {:>14} {:>14} {:>6} {:>14}",
        "Year", "Payment", "Principal", "Interest", "Months", "Balance"
    );
    println!("{}", "-".repeat(72));
    for s in &summaries {
        println!(
            "{:>6} {:>14.2} {:>14.2} {:>14.2} {:>6} {:>14.2}",
            s.year,
            s.total_payment,
            s.total_principal,
            s.total_interest,
            s.payments_count,
            s.ending_balance
        );
    }
}

fn print_sensitivity(points: &[mortgage_core::SensitivityPoint]) {
    println!("=== Rate Sensitivity Analysis ===");
    println!(
        "{:>10} {:>10} {:>14} {:>14} {:>14}",
        "Delta", "Rate %", "Monthly", "Interest", "Total Paid"
    );
    println!("{}", "-".repeat(66));
    for p in points {
        let monthly = p
            .monthly_payment
            .map(|m| format!("{:.2}", m))
            .unwrap_or_else(|| "N/A".to_string());
        println!(
            "{:>+10.2} {:>10.2} {:>14} {:>14.2} {:>14.2}",
            p.rate_delta, p.effective_rate, monthly, p.total_interest, p.total_paid
        );
    }
}

fn print_break_even(be: &mortgage_core::BreakEvenResult) {
    println!("=== Break-Even vs Rent ===");
    println!("Monthly rent:      {:.2}", be.monthly_rent);
    println!("Monthly mortgage:  {:.2}", be.monthly_cost);
    println!("Upfront costs:     {:.2}", be.upfront_costs);
    println!("Total interest:    {:.2}", be.total_interest);
    if let (Some(months), Some(years)) = (be.break_even_months, be.break_even_years) {
        println!("Break-even:        {} months ({:.1} years)", months, years);
    }
    println!("Note:              {}", be.explanation);
}
