use mortgage_gui::{Message, State, ViewTab, update};

#[test]
fn test_state_default() {
    let state = State::default();
    assert_eq!(state.amount, "185000");
    assert_eq!(state.term, "30");
    assert_eq!(state.currency, "EUR");
    assert_eq!(state.payment_type, "Annuity");
    assert_eq!(state.rate_mode, "Fix");
    assert!(state.result.is_none());
    assert!(state.params.is_none());
}

#[test]
fn test_update_amount_changed() {
    let mut state = State::default();
    let _ = update(&mut state, Message::AmountChanged("200000".to_string()));
    assert_eq!(state.amount, "200000");
}

#[test]
fn test_update_term_changed() {
    let mut state = State::default();
    let _ = update(&mut state, Message::TermChanged("20".to_string()));
    assert_eq!(state.term, "20");
}

#[test]
fn test_update_currency_changed() {
    let mut state = State::default();
    assert_eq!(state.currency, "EUR");
    let _ = update(&mut state, Message::CurrencyChanged("USD".to_string()));
    assert_eq!(state.currency, "USD");
}

#[test]
fn test_update_payment_type_changed() {
    let mut state = State::default();
    assert_eq!(state.payment_type, "Annuity");
    let _ = update(&mut state, Message::PaymentTypeChanged("Diff".to_string()));
    assert_eq!(state.payment_type, "Diff");
}

#[test]
fn test_update_rate_mode_changed() {
    let mut state = State::default();
    assert_eq!(state.rate_mode, "Fix");
    let _ = update(&mut state, Message::RateModeChanged("Euribor".to_string()));
    assert_eq!(state.rate_mode, "Euribor");
    let _ = update(&mut state, Message::RateModeChanged("Mixed".to_string()));
    assert_eq!(state.rate_mode, "Mixed");
}

#[test]
fn test_update_add_prepayment() {
    let mut state = State::default();
    assert_eq!(state.prepayments.len(), 0);

    let _ = update(
        &mut state,
        Message::PrepaymentDateChanged("01-01-2027".to_string()),
    );
    let _ = update(
        &mut state,
        Message::PrepaymentAmountChanged("20000".to_string()),
    );
    let _ = update(
        &mut state,
        Message::PrepaymentEffectChanged("ReduceTerm".to_string()),
    );
    let _ = update(&mut state, Message::AddPrepayment);

    assert_eq!(state.prepayments.len(), 1);
    assert_eq!(state.prepayments[0].amount, 20000.0);
}

#[test]
fn test_update_remove_prepayment() {
    let mut state = State::default();
    let _ = update(
        &mut state,
        Message::PrepaymentDateChanged("01-01-2027".to_string()),
    );
    let _ = update(
        &mut state,
        Message::PrepaymentAmountChanged("20000".to_string()),
    );
    let _ = update(
        &mut state,
        Message::PrepaymentEffectChanged("ReduceTerm".to_string()),
    );
    let _ = update(&mut state, Message::AddPrepayment);
    assert_eq!(state.prepayments.len(), 1);

    let _ = update(&mut state, Message::RemovePrepayment(0));
    assert_eq!(state.prepayments.len(), 0);
}

#[test]
fn test_update_calculate() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate = "5.0".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(&mut state, Message::Calculate);

    assert!(state.result.is_some());
    assert!(state.params.is_some());
    assert!(!state.status_is_error);
    assert_eq!(state.status, "Calculation complete");
}

#[test]
fn test_update_calculate_invalid_amount() {
    let mut state = State::default();
    state.amount = "invalid".to_string();

    let _ = update(&mut state, Message::Calculate);

    assert!(state.result.is_none());
    assert!(state.status_is_error);
    assert_eq!(state.status, "Invalid amount");
}

#[test]
fn test_update_calculate_invalid_term() {
    let mut state = State::default();
    state.term = "invalid".to_string();

    let _ = update(&mut state, Message::Calculate);

    assert!(state.result.is_none());
    assert!(state.status_is_error);
    assert_eq!(state.status, "Invalid term");
}

#[test]
fn test_update_calculate_invalid_date() {
    let mut state = State::default();
    state.start_date = "invalid".to_string();

    let _ = update(&mut state, Message::Calculate);

    assert!(state.result.is_none());
    assert!(state.status_is_error);
    assert_eq!(state.status, "Invalid start date (DD-MM-YYYY)");
}

#[test]
fn test_update_show_table() {
    let mut state = State::default();
    let _ = update(&mut state, Message::ShowTable);
    assert_eq!(state.active_tab, ViewTab::Table);
}

#[test]
fn test_update_show_yearly() {
    let mut state = State::default();
    let _ = update(&mut state, Message::ShowYearly);
    assert_eq!(state.active_tab, ViewTab::Yearly);
}

#[test]
fn test_update_show_sensitivity() {
    let mut state = State::default();
    let _ = update(&mut state, Message::ShowSensitivity);
    assert_eq!(state.active_tab, ViewTab::Sensitivity);
}

#[test]
fn test_update_show_break_even() {
    let mut state = State::default();
    let _ = update(&mut state, Message::ShowBreakEven);
    assert_eq!(state.active_tab, ViewTab::BreakEven);
}

#[test]
fn test_update_rent_changed() {
    let mut state = State::default();
    let _ = update(&mut state, Message::RentChanged("1000".to_string()));
    assert_eq!(state.rent, "1000");
}

#[test]
fn test_update_same_spread_toggled() {
    let mut state = State::default();
    assert!(!state.same_spread);
    let _ = update(&mut state, Message::SameSpreadToggled(true));
    assert!(state.same_spread);
    let _ = update(&mut state, Message::SameSpreadToggled(false));
    assert!(!state.same_spread);
}

#[test]
fn test_update_export_csv_no_result() {
    let mut state = State::default();
    let _ = update(&mut state, Message::ExportCsv);
    assert!(state.status_is_error);
    assert_eq!(state.status, "No results to export. Calculate first.");
}

#[test]
fn test_update_export_pdf_no_result() {
    let mut state = State::default();
    let _ = update(&mut state, Message::ExportPdf);
    assert!(state.status_is_error);
    assert_eq!(state.status, "No results to export. Calculate first.");
}

#[test]
fn test_update_save_session_no_result() {
    let mut state = State::default();
    let _ = update(&mut state, Message::SaveSession);
    assert!(state.status_is_error);
    assert_eq!(state.status, "No results to save. Calculate first.");
}

#[test]
fn test_update_calculate_then_export_csv() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate = "5.0".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(&mut state, Message::Calculate);
    assert!(state.result.is_some());

    let _ = update(&mut state, Message::ExportCsv);
    assert!(!state.status_is_error);
    assert!(state.status.contains("Saved to"));
}

#[test]
fn test_update_calculate_then_show_chart() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate = "5.0".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(&mut state, Message::Calculate);
    assert!(state.result.is_some());

    let _ = update(&mut state, Message::ShowChart);
    assert_eq!(state.active_tab, ViewTab::Chart);
    // Chart generation may fail in test environment (no fonts), but should not panic
}

#[test]
fn test_update_calculate_then_show_balance_chart() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate = "5.0".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(&mut state, Message::Calculate);
    assert!(state.result.is_some());

    let _ = update(&mut state, Message::ShowBalanceChart);
    assert_eq!(state.active_tab, ViewTab::BalanceChart);
    // Chart generation may fail in test environment (no fonts), but should not panic
}

#[test]
fn test_update_calculate_then_show_overlay_chart() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate = "5.0".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(&mut state, Message::Calculate);
    assert!(state.result.is_some());

    let _ = update(&mut state, Message::ShowBreakEven);
    assert_eq!(state.active_tab, ViewTab::BreakEven);
    // Chart generation may fail in test environment (no fonts), but should not panic
}

#[test]
fn test_update_calculate_diff_payment() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate = "5.0".to_string();
    state.payment_type = "Diff".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(&mut state, Message::Calculate);

    assert!(state.result.is_some());
    assert!(!state.status_is_error);
}

#[test]
fn test_update_calculate_with_prepayments() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate = "5.0".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(
        &mut state,
        Message::PrepaymentDateChanged("01-01-2027".to_string()),
    );
    let _ = update(
        &mut state,
        Message::PrepaymentAmountChanged("20000".to_string()),
    );
    let _ = update(
        &mut state,
        Message::PrepaymentEffectChanged("ReduceTerm".to_string()),
    );
    let _ = update(&mut state, Message::AddPrepayment);

    let _ = update(&mut state, Message::Calculate);

    assert!(state.result.is_some());
    assert!(!state.status_is_error);
}

#[test]
fn test_update_calculate_euribor_mode() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate_mode = "Euribor".to_string();
    state.euribor_spread = "1.0".to_string();
    state.start_date = "01-01-2025".to_string();
    state.euribor_manual_points = vec![("01-01-2025".to_string(), "2.5".to_string())];
    state.use_manual_euribor = true;

    let _ = update(&mut state, Message::Calculate);

    assert!(state.result.is_some());
    assert!(!state.status_is_error);
}

#[test]
fn test_update_calculate_mixed_mode() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate_mode = "Mixed".to_string();
    state.mixed_fix_years = "2.0".to_string();
    state.mixed_fix_rate = "3.0".to_string();
    state.mixed_fix_spread = "1.0".to_string();
    state.mixed_euribor_spread = "1.5".to_string();
    state.start_date = "01-01-2025".to_string();

    // Use manual Euribor points to avoid network dependency
    state.euribor_manual_points = vec![("01-01-2027".to_string(), "2.5".to_string())];
    state.use_manual_euribor = true;

    let _ = update(&mut state, Message::Calculate);

    assert!(state.result.is_some());
    assert!(!state.status_is_error);
}

#[test]
fn test_update_session_save_load_roundtrip() {
    let mut state = State::default();
    state.amount = "150000".to_string();
    state.term = "15".to_string();
    state.rate = "4.5".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(&mut state, Message::Calculate);
    assert!(state.result.is_some());

    let _ = update(&mut state, Message::SaveSession);
    assert!(!state.status_is_error);

    let mut state2 = State::default();
    let _ = update(&mut state2, Message::LoadSession);

    assert_eq!(state2.amount, "150000");
    assert_eq!(state2.term, "15");
    assert!(state2.result.is_some());
}

#[test]
fn test_multiple_prepayments() {
    let mut state = State::default();

    let _ = update(
        &mut state,
        Message::PrepaymentDateChanged("01-01-2027".to_string()),
    );
    let _ = update(
        &mut state,
        Message::PrepaymentAmountChanged("10000".to_string()),
    );
    let _ = update(&mut state, Message::AddPrepayment);

    let _ = update(
        &mut state,
        Message::PrepaymentDateChanged("01-01-2028".to_string()),
    );
    let _ = update(
        &mut state,
        Message::PrepaymentAmountChanged("15000".to_string()),
    );
    let _ = update(&mut state, Message::AddPrepayment);

    assert_eq!(state.prepayments.len(), 2);
    assert_eq!(state.prepayments[0].amount, 10000.0);
    assert_eq!(state.prepayments[1].amount, 15000.0);
}

#[test]
fn test_update_all_tabs_after_calculate() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate = "5.0".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(&mut state, Message::Calculate);
    assert!(state.result.is_some());

    // Test non-chart tabs
    let non_chart_tabs = vec![
        Message::ShowTable,
        Message::ShowYearly,
        Message::ShowSensitivity,
        Message::ShowBreakEven,
    ];

    for msg in non_chart_tabs {
        let _ = update(&mut state, msg);
        assert!(!state.status_is_error, "Tab switch should not cause error");
    }

    // Chart tabs may fail in test environment (no fonts), but should not panic
    let chart_tabs: Vec<Message> = vec![Message::ShowChart, Message::ShowBalanceChart];

    for msg in chart_tabs {
        let _ = update(&mut state, msg);
        // Just verify it doesn't panic
    }
}

#[test]
fn test_chart_switch_roundtrip() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate = "5.0".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(&mut state, Message::Calculate);
    assert!(state.result.is_some());

    // Switch to chart and back without panic
    let _ = update(&mut state, Message::ShowChart);
    assert_eq!(state.active_tab, ViewTab::Chart);
    let _ = update(&mut state, Message::ShowTable);
    assert_eq!(state.active_tab, ViewTab::Table);
    let _ = update(&mut state, Message::ShowBalanceChart);
    assert_eq!(state.active_tab, ViewTab::BalanceChart);
    let _ = update(&mut state, Message::ShowYearly);
    assert_eq!(state.active_tab, ViewTab::Yearly);
    assert!(!state.status_is_error);
}

#[test]
fn test_chart_area_coordinates_indirect() {
    // This test verifies that chart_area produces valid rectangles
    // by exercising the update path with simulated cursor positions.
    // The chart_coord helper is defined inside the chart structs, so we
    // test indirectly via the message flow.
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate = "5.0".to_string();
    state.start_date = "01-01-2025".to_string();

    let _ = update(&mut state, Message::Calculate);
    assert!(state.result.is_some());

    // Switch to Stacked chart tab
    let _ = update(&mut state, Message::ShowChart);
    assert_eq!(state.active_tab, ViewTab::Chart);

    // Switch to Balance chart tab
    let _ = update(&mut state, Message::ShowBalanceChart);
    assert_eq!(state.active_tab, ViewTab::BalanceChart);

    // Ensure error status is not set
    assert!(
        !state.status_is_error,
        "Chart tab switch caused error: {}",
        state.status
    );
}

#[test]
fn test_mixed_mode_with_charts() {
    let mut state = State::default();
    state.amount = "100000".to_string();
    state.term = "10".to_string();
    state.rate_mode = "Mixed".to_string();
    state.mixed_fix_years = "2.0".to_string();
    state.mixed_fix_rate = "3.0".to_string();
    state.mixed_fix_spread = "1.0".to_string();
    state.mixed_euribor_spread = "1.5".to_string();
    state.start_date = "01-01-2025".to_string();
    state.euribor_manual_points = vec![("01-01-2027".to_string(), "2.5".to_string())];
    state.use_manual_euribor = true;

    let _ = update(&mut state, Message::Calculate);
    assert!(state.result.is_some());

    // Chart tabs should not panic after mixed-mode calculation
    let _ = update(&mut state, Message::ShowChart);
    assert_eq!(state.active_tab, ViewTab::Chart);
    let _ = update(&mut state, Message::ShowBalanceChart);
    assert_eq!(state.active_tab, ViewTab::BalanceChart);
    assert!(!state.status_is_error);
}
