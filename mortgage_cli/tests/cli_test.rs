use std::process::Command;

fn cargo_bin() -> String {
    env!("CARGO_BIN_EXE_mortgage_cli").to_string()
}

#[test]
fn test_cli_basic_calculation() {
    let output = Command::new(cargo_bin())
        .args(&["-a", "100000", "-t", "10", "-r", "5"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Loan Summary"));
    assert!(stdout.contains("100000.00"));
}

#[test]
fn test_cli_diff_payment() {
    let output = Command::new(cargo_bin())
        .args(&["-a", "100000", "-t", "10", "-r", "5", "-p", "diff"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Diff"));
}

#[test]
fn test_cli_csv_output() {
    let output = Command::new(cargo_bin())
        .args(&["-a", "50000", "-t", "5", "-r", "3", "--format", "csv"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("date,payment,principal,interest,remaining_balance,applied_rate"));
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() > 1);
}

#[test]
fn test_cli_csv_file_output() {
    let output_path = "/tmp/mortgage_cli_test_output.csv";
    let output = Command::new(cargo_bin())
        .args(&["-a", "50000", "-t", "5", "-r", "3", "--output", output_path])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let content = std::fs::read_to_string(output_path).unwrap();
    assert!(content.contains("date,payment,principal,interest,remaining_balance,applied_rate"));
    std::fs::remove_file(output_path).ok();
}

#[test]
fn test_cli_json_config() {
    let config = r#"{
        "amount": 75000,
        "term_years": 5,
        "payment_type": "annuitet",
        "currency": "Eur",
        "start_date": "2025-01-01",
        "rate_mode": {"Fix": {"rate": 4.0, "spread": 0.5}},
        "same_spread": false,
        "euribor_curve": [],
        "prepayments": []
    }"#;
    let config_path = "/tmp/mortgage_cli_test_config.json";
    std::fs::write(config_path, config).unwrap();

    let output = Command::new(cargo_bin())
        .args(&["--config", config_path])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("75000.00"));
    std::fs::remove_file(config_path).ok();
}

#[test]
fn test_cli_yearly_summary() {
    let output = Command::new(cargo_bin())
        .args(&[
            "-a",
            "100000",
            "-t",
            "5",
            "-r",
            "5",
            "--yearly",
            "--start-date",
            "01-01-2025",
        ])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Yearly Summary"));
    assert!(stdout.contains("2025"));
    assert!(stdout.contains("2029"));
}

#[test]
fn test_cli_mixed_rate_mode() {
    let output = Command::new(cargo_bin())
        .args(&[
            "-a",
            "100000",
            "-t",
            "10",
            "--rate-mode",
            "mixed",
            "--fix-years",
            "2",
            "-r",
            "3",
            "--spread",
            "1",
            "--euribor-spread",
            "2",
        ])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Mixed"));
}

#[test]
fn test_cli_limit_option() {
    let output = Command::new(cargo_bin())
        .args(&["-a", "100000", "-t", "10", "-r", "5", "--limit", "5"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("more payments"));
}

#[test]
fn test_cli_currency_usd() {
    let output = Command::new(cargo_bin())
        .args(&["-a", "100000", "-t", "5", "-r", "5", "-u", "usd"])
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("$"));
}

#[test]
fn test_cli_invalid_config_file() {
    let output = Command::new(cargo_bin())
        .args(&["--config", "/tmp/nonexistent_config.json"])
        .output()
        .expect("Failed to run CLI");

    assert!(!output.status.success());
}
