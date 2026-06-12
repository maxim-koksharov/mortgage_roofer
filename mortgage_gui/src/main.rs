mod chart;

use iced::{
    Alignment, Element, Length, Theme,
    widget::{
        Column, button, checkbox, column, container, pick_list, row, scrollable, svg, text,
        text_input,
    },
};
use image::open as image_open;
use mortgage_core::models::*;
use mortgage_core::{Calculator, payments_to_csv};
use std::fs;

pub fn main() -> iced::Result {
    iced::application("Mortgage Calculator", update, view)
        .theme(|_| Theme::TokyoNightStorm)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    AmountChanged(String),
    TermChanged(String),
    StartDateChanged(String),
    RateChanged(String),
    SpreadChanged(String),
    CurrencyChanged(String),
    PaymentTypeChanged(String),
    RateModeChanged(String),
    EuriborTenorChanged(String),
    EuriborSpreadChanged(String),
    MixedFixYearsChanged(String),
    MixedFixRateChanged(String),
    MixedFixSpreadChanged(String),
    MixedEuriborTenorChanged(String),
    MixedEuriborSpreadChanged(String),
    SameSpreadToggled(bool),
    PrepaymentDateChanged(String),
    PrepaymentAmountChanged(String),
    PrepaymentEffectChanged(String),
    AddPrepayment,
    RemovePrepayment(usize),
    Calculate,
    ExportCsv,
    ExportPdf,
    ShowTable,
    ShowChart,
    ShowBalanceChart,
    ShowOverlayChart,
    ShowYearly,
    ShowSensitivity,
    ShowBreakEven,
    RentChanged(String),
    SaveSession,
    LoadSession,
}

#[derive(Debug, Clone)]
struct State {
    amount: String,
    term: String,
    start_date: String,
    rate: String,
    spread: String,
    currency: String,
    payment_type: String,
    rate_mode: String,
    euribor_tenor: String,
    euribor_spread: String,
    mixed_fix_years: String,
    mixed_fix_rate: String,
    mixed_fix_spread: String,
    mixed_euribor_tenor: String,
    mixed_euribor_spread: String,
    same_spread: bool,
    prepayment_date: String,
    prepayment_amount: String,
    prepayment_effect: String,
    prepayments: Vec<Prepayment>,
    rent: String,
    params: Option<LoanParams>,
    result: Option<LoanResult>,
    chart_svg: Option<String>,
    balance_svg: Option<String>,
    active_tab: ViewTab,
    status: String,
    status_is_error: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            amount: "185000".to_string(),
            term: "30".to_string(),
            start_date: chrono::Local::now()
                .date_naive()
                .format("%Y-%m-%d")
                .to_string(),
            rate: "3.6".to_string(),
            spread: "0.0".to_string(),
            currency: "EUR".to_string(),
            payment_type: "Annuity".to_string(),
            rate_mode: "Fix".to_string(),
            euribor_tenor: "6m".to_string(),
            euribor_spread: "1.0".to_string(),
            mixed_fix_years: "2".to_string(),
            mixed_fix_rate: "3.0".to_string(),
            mixed_fix_spread: "1.0".to_string(),
            mixed_euribor_tenor: "6m".to_string(),
            mixed_euribor_spread: "1.5".to_string(),
            same_spread: false,
            prepayment_date: "2027-01-01".to_string(),
            prepayment_amount: "20000".to_string(),
            prepayment_effect: "ReduceTerm".to_string(),
            prepayments: vec![],
            rent: "900".to_string(),
            params: None,
            result: None,
            chart_svg: None,
            balance_svg: None,
            active_tab: ViewTab::Table,
            status: String::new(),
            status_is_error: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum ViewTab {
    #[default]
    Table,
    Chart,
    BalanceChart,
    OverlayChart,
    Yearly,
    Sensitivity,
    BreakEven,
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::AmountChanged(v) => state.amount = v,
        Message::TermChanged(v) => state.term = v,
        Message::StartDateChanged(v) => state.start_date = v,
        Message::RateChanged(v) => state.rate = v,
        Message::SpreadChanged(v) => state.spread = v,
        Message::CurrencyChanged(v) => state.currency = v,
        Message::PaymentTypeChanged(v) => state.payment_type = v,
        Message::RateModeChanged(v) => state.rate_mode = v,
        Message::EuriborTenorChanged(v) => state.euribor_tenor = v,
        Message::EuriborSpreadChanged(v) => state.euribor_spread = v,
        Message::MixedFixYearsChanged(v) => state.mixed_fix_years = v,
        Message::MixedFixRateChanged(v) => state.mixed_fix_rate = v,
        Message::MixedFixSpreadChanged(v) => state.mixed_fix_spread = v,
        Message::MixedEuriborTenorChanged(v) => state.mixed_euribor_tenor = v,
        Message::MixedEuriborSpreadChanged(v) => state.mixed_euribor_spread = v,
        Message::SameSpreadToggled(v) => state.same_spread = v,
        Message::PrepaymentDateChanged(v) => state.prepayment_date = v,
        Message::PrepaymentAmountChanged(v) => state.prepayment_amount = v,
        Message::PrepaymentEffectChanged(v) => state.prepayment_effect = v,
        Message::AddPrepayment => {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(&state.prepayment_date, "%Y-%m-%d")
            {
                if let Ok(amount) = state.prepayment_amount.parse::<f64>() {
                    if amount > 0.0 {
                        let effect = if state.prepayment_effect == "ReducePayment" {
                            PrepaymentEffect::ReducePayment
                        } else {
                            PrepaymentEffect::ReduceTerm
                        };
                        state.prepayments.push(Prepayment {
                            date,
                            amount,
                            effect,
                        });
                        state.prepayment_amount = "0".to_string();
                        state.status = format!("Added prepayment #{}", state.prepayments.len());
                        state.status_is_error = false;
                    } else {
                        state.status = "Prepayment amount must be positive".to_string();
                        state.status_is_error = true;
                    }
                } else {
                    state.status = "Invalid prepayment amount".to_string();
                    state.status_is_error = true;
                }
            } else {
                state.status = "Invalid date format (YYYY-MM-DD)".to_string();
                state.status_is_error = true;
            }
        }
        Message::RemovePrepayment(idx) => {
            if idx < state.prepayments.len() {
                state.prepayments.remove(idx);
                state.status = format!("Removed prepayment. {} remaining", state.prepayments.len());
                state.status_is_error = false;
            }
        }
        Message::Calculate => calculate(state),
        Message::ExportCsv => export_csv(state),
        Message::ExportPdf => export_pdf(state),
        Message::ShowTable => state.active_tab = ViewTab::Table,
        Message::ShowChart => {
            state.active_tab = ViewTab::Chart;
            if state.chart_svg.is_none() && state.result.is_some() {
                generate_chart(state);
            }
        }
        Message::ShowBalanceChart => {
            state.active_tab = ViewTab::BalanceChart;
            if state.balance_svg.is_none() && state.result.is_some() {
                generate_balance_chart(state);
            }
        }
        Message::ShowOverlayChart => {
            state.active_tab = ViewTab::OverlayChart;
        }
        Message::ShowYearly => state.active_tab = ViewTab::Yearly,
        Message::ShowSensitivity => state.active_tab = ViewTab::Sensitivity,
        Message::ShowBreakEven => state.active_tab = ViewTab::BreakEven,
        Message::RentChanged(v) => state.rent = v,
        Message::SaveSession => save_session_gui(state),
        Message::LoadSession => load_session_gui(state),
    }
}

fn input_row<'a>(label: &'a str, content: Element<'a, Message>) -> Element<'a, Message> {
    row![text(label).width(Length::Fixed(130.0)), content,]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}

fn section_header(title: &str) -> Element<'_, Message> {
    container(text(title).size(16))
        .padding(5)
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.2, 0.3, 0.4,
            ))),
            ..Default::default()
        })
        .into()
}

fn view(state: &State) -> Element<'_, Message> {
    let currencies = vec!["EUR".to_string(), "USD".to_string()];
    let payment_types = vec!["Annuity".to_string(), "Diff".to_string()];
    let rate_modes = vec![
        "Fix".to_string(),
        "Euribor".to_string(),
        "Mixed".to_string(),
    ];
    let tenors = vec![
        "1m".to_string(),
        "3m".to_string(),
        "6m".to_string(),
        "12m".to_string(),
    ];
    let effects = vec!["ReduceTerm".to_string(), "ReducePayment".to_string()];

    let amount_valid = state.amount.parse::<f64>().is_ok();
    let term_valid = state.term.parse::<u32>().is_ok();
    let date_valid = chrono::NaiveDate::parse_from_str(&state.start_date, "%Y-%m-%d").is_ok();

    let loan_section = column![
        section_header("Loan Parameters"),
        input_row(
            "Amount:",
            validated_input(
                "185000",
                &state.amount,
                Message::AmountChanged,
                amount_valid
            )
        ),
        input_row(
            "Term (years):",
            validated_input("30", &state.term, Message::TermChanged, term_valid)
        ),
        input_row(
            "Start date:",
            validated_input(
                "2025-01-01",
                &state.start_date,
                Message::StartDateChanged,
                date_valid
            )
        ),
        input_row(
            "Currency:",
            pick_list(
                currencies,
                Some(state.currency.clone()),
                Message::CurrencyChanged
            )
            .into()
        ),
        input_row(
            "Payment type:",
            pick_list(
                payment_types,
                Some(state.payment_type.clone()),
                Message::PaymentTypeChanged
            )
            .into()
        ),
    ]
    .spacing(6)
    .padding(10);

    let rate_section = {
        let mut fields: Vec<Element<'_, Message>> = vec![section_header("Rate Configuration")];
        fields.push(input_row(
            "Rate mode:",
            pick_list(
                rate_modes,
                Some(state.rate_mode.clone()),
                Message::RateModeChanged,
            )
            .into(),
        ));

        match state.rate_mode.as_str() {
            "Fix" => {
                fields.push(input_row(
                    "Rate (%):",
                    text_input("3.6", &state.rate)
                        .on_input(Message::RateChanged)
                        .width(Length::Fill)
                        .into(),
                ));
                fields.push(input_row(
                    "Spread (%):",
                    text_input("0.0", &state.spread)
                        .on_input(Message::SpreadChanged)
                        .width(Length::Fill)
                        .into(),
                ));
            }
            "Euribor" => {
                fields.push(input_row(
                    "Tenor:",
                    pick_list(
                        tenors.clone(),
                        Some(state.euribor_tenor.clone()),
                        Message::EuriborTenorChanged,
                    )
                    .into(),
                ));
                fields.push(input_row(
                    "Spread (%):",
                    text_input("1.0", &state.euribor_spread)
                        .on_input(Message::EuriborSpreadChanged)
                        .width(Length::Fill)
                        .into(),
                ));
            }
            "Mixed" => {
                fields.push(input_row(
                    "Fixed years:",
                    text_input("2", &state.mixed_fix_years)
                        .on_input(Message::MixedFixYearsChanged)
                        .width(Length::Fill)
                        .into(),
                ));
                fields.push(input_row(
                    "Fix rate (%):",
                    text_input("3.0", &state.mixed_fix_rate)
                        .on_input(Message::MixedFixRateChanged)
                        .width(Length::Fill)
                        .into(),
                ));
                fields.push(input_row(
                    "Fix spread (%):",
                    text_input("1.0", &state.mixed_fix_spread)
                        .on_input(Message::MixedFixSpreadChanged)
                        .width(Length::Fill)
                        .into(),
                ));
                fields.push(input_row(
                    "Euribor tenor:",
                    pick_list(
                        tenors.clone(),
                        Some(state.mixed_euribor_tenor.clone()),
                        Message::MixedEuriborTenorChanged,
                    )
                    .into(),
                ));
                if !state.same_spread {
                    fields.push(input_row(
                        "Euribor spr (%):",
                        text_input("1.5", &state.mixed_euribor_spread)
                            .on_input(Message::MixedEuriborSpreadChanged)
                            .width(Length::Fill)
                            .into(),
                    ));
                }
                fields.push(input_row(
                    "Same spread:",
                    checkbox("", state.same_spread)
                        .on_toggle(Message::SameSpreadToggled)
                        .into(),
                ));
            }
            _ => {}
        }
        Column::from_vec(fields).spacing(6).padding(10)
    };

    let prepay_section = {
        let mut fields: Vec<Element<'_, Message>> = vec![section_header("Prepayments")];
        fields.push(input_row(
            "Date:",
            text_input("2027-01-01", &state.prepayment_date)
                .on_input(Message::PrepaymentDateChanged)
                .width(Length::Fill)
                .into(),
        ));
        fields.push(input_row(
            "Amount:",
            text_input("20000", &state.prepayment_amount)
                .on_input(Message::PrepaymentAmountChanged)
                .width(Length::Fill)
                .into(),
        ));
        fields.push(input_row(
            "Effect:",
            pick_list(
                effects,
                Some(state.prepayment_effect.clone()),
                Message::PrepaymentEffectChanged,
            )
            .into(),
        ));
        fields.push(
            button("  + Add Prepayment  ")
                .on_press(Message::AddPrepayment)
                .into(),
        );

        for (i, prep) in state.prepayments.iter().enumerate() {
            fields.push(
                row![
                    text(format!(
                        "  #{}: {} {:.0} {}",
                        i + 1,
                        prep.date,
                        prep.amount,
                        prep.effect
                    ))
                    .width(Length::Fill),
                    button(" X ").on_press(Message::RemovePrepayment(i)),
                ]
                .spacing(5)
                .align_y(Alignment::Center)
                .into(),
            );
        }
        Column::from_vec(fields).spacing(6).padding(10)
    };

    let actions_section = column![
        section_header("Actions"),
        row![
            button("  Calculate  ").on_press(Message::Calculate),
            button("  Export CSV  ").on_press(Message::ExportCsv),
            button("  Export PDF  ").on_press(Message::ExportPdf),
        ]
        .spacing(8),
        row![
            button("  Save Session  ").on_press(Message::SaveSession),
            button("  Load Session  ").on_press(Message::LoadSession),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .padding(10);

    let input_panel = scrollable(
        column![loan_section, rate_section, prepay_section, actions_section].spacing(10),
    )
    .width(Length::Fixed(380.0));

    let results_panel: Element<Message> = if let Some(ref result) = state.result {
        let sym = if state.currency == "USD" { "$" } else { "€" };
        let summary = container(
            column![
                text("Results").size(18),
                text(format!(
                    "Monthly: {}{:.2}",
                    sym,
                    result.monthly_payment.unwrap_or(0.0)
                )),
                text(format!(
                    "Total Principal: {}{:.2}",
                    sym, result.total_principal
                )),
                text(format!(
                    "Total Interest: {}{:.2}",
                    sym, result.total_interest
                )),
                text(format!("Total Paid: {}{:.2}", sym, result.total_paid)),
                text(format!("Payments: {}", result.payments.len())),
                if let Some(idx) = result.principal_exceeds_interest_at {
                    text(format!(
                        "Principal > Interest at #{} ({})",
                        idx + 1,
                        result.payments[idx].date
                    ))
                } else {
                    text("")
                },
            ]
            .spacing(4),
        )
        .padding(10)
        .width(Length::Fill);

        let tab_style = |active: bool| {
            if active {
                button::primary
            } else {
                button::secondary
            }
        };

        let tabs = row![
            button("Table")
                .style(tab_style(state.active_tab == ViewTab::Table))
                .on_press(Message::ShowTable),
            button("Stacked")
                .style(tab_style(state.active_tab == ViewTab::Chart))
                .on_press(Message::ShowChart),
            button("Balance")
                .style(tab_style(state.active_tab == ViewTab::BalanceChart))
                .on_press(Message::ShowBalanceChart),
            button("Overlay")
                .style(tab_style(state.active_tab == ViewTab::OverlayChart))
                .on_press(Message::ShowOverlayChart),
            button("Yearly")
                .style(tab_style(state.active_tab == ViewTab::Yearly))
                .on_press(Message::ShowYearly),
            button("Sensitivity")
                .style(tab_style(state.active_tab == ViewTab::Sensitivity))
                .on_press(Message::ShowSensitivity),
            button("Break-Even")
                .style(tab_style(state.active_tab == ViewTab::BreakEven))
                .on_press(Message::ShowBreakEven),
        ]
        .spacing(6);

        let content: Element<Message> = match state.active_tab {
            ViewTab::Table => {
                let table_header = row![
                    text("#").width(Length::Fixed(40.0)),
                    text("Date").width(Length::Fixed(100.0)),
                    text("Payment").width(Length::Fixed(100.0)),
                    text("Principal").width(Length::Fixed(100.0)),
                    text("Interest").width(Length::Fixed(100.0)),
                    text("Balance").width(Length::Fixed(100.0)),
                ]
                .spacing(5);

                let mut table_rows: Vec<Element<Message>> = vec![table_header.into()];
                for (i, p) in result.payments.iter().enumerate() {
                    let r = row![
                        text(format!("{}", i + 1)).width(Length::Fixed(40.0)),
                        text(p.date.to_string()).width(Length::Fixed(100.0)),
                        text(format!("{:.2}", p.payment)).width(Length::Fixed(100.0)),
                        text(format!("{:.2}", p.principal)).width(Length::Fixed(100.0)),
                        text(format!("{:.2}", p.interest)).width(Length::Fixed(100.0)),
                        text(format!("{:.2}", p.remaining_balance)).width(Length::Fixed(100.0)),
                    ]
                    .spacing(5);
                    table_rows.push(r.into());
                }

                let table = Column::from_vec(table_rows).spacing(2);
                scrollable(container(table).padding(10)).into()
            }
            ViewTab::Chart => {
                if let Some(ref svg_str) = state.chart_svg {
                    let _ = fs::write("/tmp/mortgage_chart.svg", svg_str);
                    let handle = svg::Handle::from_path("/tmp/mortgage_chart.svg");
                    svg(handle).width(Length::Fill).height(Length::Fill).into()
                } else {
                    text("Chart not available").into()
                }
            }
            ViewTab::BalanceChart => {
                if let Some(ref svg_str) = state.balance_svg {
                    let _ = fs::write("/tmp/mortgage_balance.svg", svg_str);
                    let handle = svg::Handle::from_path("/tmp/mortgage_balance.svg");
                    svg(handle).width(Length::Fill).height(Length::Fill).into()
                } else {
                    text("Balance chart not available").into()
                }
            }
            ViewTab::OverlayChart => {
                if let Some(ref result) = state.result {
                    let svg_str = chart::generate_overlay_chart_svg(result);
                    let _ = fs::write("/tmp/mortgage_overlay.svg", &svg_str);
                    let handle = svg::Handle::from_path("/tmp/mortgage_overlay.svg");
                    svg(handle).width(Length::Fill).height(Length::Fill).into()
                } else {
                    text("Overlay chart not available").into()
                }
            }
            ViewTab::Yearly => {
                let summaries = result.yearly_summaries();
                let header = row![
                    text("Year").width(Length::Fixed(60.0)),
                    text("Payment").width(Length::Fixed(110.0)),
                    text("Principal").width(Length::Fixed(110.0)),
                    text("Interest").width(Length::Fixed(110.0)),
                    text("Months").width(Length::Fixed(60.0)),
                    text("Balance").width(Length::Fixed(110.0)),
                ]
                .spacing(5);

                let mut rows: Vec<Element<Message>> = vec![header.into()];
                for s in &summaries {
                    let r = row![
                        text(format!("{}", s.year)).width(Length::Fixed(60.0)),
                        text(format!("{:.2}", s.total_payment)).width(Length::Fixed(110.0)),
                        text(format!("{:.2}", s.total_principal)).width(Length::Fixed(110.0)),
                        text(format!("{:.2}", s.total_interest)).width(Length::Fixed(110.0)),
                        text(format!("{}", s.payments_count)).width(Length::Fixed(60.0)),
                        text(format!("{:.2}", s.ending_balance)).width(Length::Fixed(110.0)),
                    ]
                    .spacing(5);
                    rows.push(r.into());
                }

                let table = Column::from_vec(rows).spacing(2);
                scrollable(container(table).padding(10)).into()
            }
            ViewTab::Sensitivity => {
                if let Some(ref params) = state.params {
                    let deltas = vec![-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0];
                    let points = mortgage_core::sensitivity_analysis(params, &deltas);
                    let header = row![
                        text("Delta").width(Length::Fixed(60.0)),
                        text("Rate %").width(Length::Fixed(80.0)),
                        text("Monthly").width(Length::Fixed(110.0)),
                        text("Interest").width(Length::Fixed(110.0)),
                        text("Total Paid").width(Length::Fixed(110.0)),
                    ]
                    .spacing(5);

                    let mut rows: Vec<Element<Message>> = vec![header.into()];
                    for p in &points {
                        let monthly = p
                            .monthly_payment
                            .map(|m| format!("{:.2}", m))
                            .unwrap_or_else(|| "N/A".to_string());
                        let r = row![
                            text(format!("{:+.2}", p.rate_delta)).width(Length::Fixed(60.0)),
                            text(format!("{:.2}", p.effective_rate)).width(Length::Fixed(80.0)),
                            text(monthly).width(Length::Fixed(110.0)),
                            text(format!("{:.2}", p.total_interest)).width(Length::Fixed(110.0)),
                            text(format!("{:.2}", p.total_paid)).width(Length::Fixed(110.0)),
                        ]
                        .spacing(5);
                        rows.push(r.into());
                    }

                    let table = Column::from_vec(rows).spacing(2);
                    scrollable(container(table).padding(10)).into()
                } else {
                    text("Calculate first to see sensitivity analysis").into()
                }
            }
            ViewTab::BreakEven => {
                let rent = state.rent.parse::<f64>().unwrap_or(0.0);
                if let Some(ref params) = state.params {
                    if rent > 0.0 {
                        let be = mortgage_core::break_even_analysis(params, rent);
                        let content = column![
                            text("Break-Even vs Rent").size(16),
                            text(format!("Monthly rent:      {:.2}", be.monthly_rent)),
                            text(format!("Monthly mortgage:  {:.2}", be.monthly_cost)),
                            text(format!("Total interest:    {:.2}", be.total_interest)),
                            text(""),
                            if let (Some(months), Some(years)) =
                                (be.break_even_months, be.break_even_years)
                            {
                                text(format!(
                                    "Break-even:        {} months ({:.1} years)",
                                    months, years
                                ))
                            } else {
                                text("Break-even:        N/A")
                            },
                            text(""),
                            text(be.explanation.clone()),
                            text(""),
                            input_row(
                                "Monthly rent:",
                                text_input("900", &state.rent)
                                    .on_input(Message::RentChanged)
                                    .width(Length::Fixed(150.0))
                                    .into()
                            ),
                        ]
                        .spacing(4);
                        container(content).padding(10).into()
                    } else {
                        column![
                            text("Enter monthly rent for break-even analysis"),
                            input_row(
                                "Monthly rent:",
                                text_input("900", &state.rent)
                                    .on_input(Message::RentChanged)
                                    .width(Length::Fixed(150.0))
                                    .into()
                            ),
                        ]
                        .spacing(10)
                        .padding(10)
                        .into()
                    }
                } else {
                    text("Calculate first to see break-even analysis").into()
                }
            }
        };

        let status_bar = container(text(&state.status))
            .padding(8)
            .width(Length::Fill)
            .style(|_theme: &Theme| {
                if state.status_is_error {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.5, 0.1, 0.1,
                        ))),
                        ..Default::default()
                    }
                } else if !state.status.is_empty() {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(
                            0.1, 0.4, 0.1,
                        ))),
                        ..Default::default()
                    }
                } else {
                    container::Style::default()
                }
            });

        column![summary, tabs, content, status_bar]
            .spacing(10)
            .padding(10)
            .into()
    } else {
        container(text("Enter parameters and press Calculate"))
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    };

    let main_layout = row![input_panel, results_panel].spacing(10).padding(10);

    container(main_layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn validated_input(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'static,
    valid: bool,
) -> Element<'static, Message> {
    let input = text_input(placeholder, value)
        .on_input(on_input)
        .width(Length::Fill);
    if valid {
        input.into()
    } else {
        container(input)
            .style(|_theme: &Theme| container::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(0.8, 0.2, 0.2),
                    width: 2.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}

fn calculate(state: &mut State) {
    let amount = match state.amount.parse::<f64>() {
        Ok(v) => v,
        Err(_) => {
            state.status = "Invalid amount".to_string();
            state.status_is_error = true;
            return;
        }
    };
    let term_years = match state.term.parse::<u32>() {
        Ok(v) => v,
        Err(_) => {
            state.status = "Invalid term".to_string();
            state.status_is_error = true;
            return;
        }
    };
    let currency = if state.currency == "USD" {
        Currency::Usd
    } else {
        Currency::Eur
    };
    let payment_type = if state.payment_type == "Diff" {
        PaymentType::Diff
    } else {
        PaymentType::Annuity
    };
    let start_date = match chrono::NaiveDate::parse_from_str(&state.start_date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            state.status = "Invalid start date (YYYY-MM-DD)".to_string();
            state.status_is_error = true;
            return;
        }
    };

    let rate_mode = match state.rate_mode.as_str() {
        "Fix" => RateMode::Fix {
            rate: state.rate.parse::<f64>().unwrap_or(3.6),
            spread: state.spread.parse::<f64>().unwrap_or(0.0),
        },
        "Euribor" => RateMode::Euribor {
            tenor: parse_tenor(&state.euribor_tenor),
            spread: state.euribor_spread.parse::<f64>().unwrap_or(1.0),
        },
        "Mixed" => RateMode::Mixed {
            fix_years: state.mixed_fix_years.parse::<f64>().unwrap_or(2.0),
            fix_rate: state.mixed_fix_rate.parse::<f64>().unwrap_or(3.0),
            fix_spread: state.mixed_fix_spread.parse::<f64>().unwrap_or(1.0),
            euribor_tenor: parse_tenor(&state.mixed_euribor_tenor),
            euribor_spread: if state.same_spread {
                state.mixed_fix_spread.parse::<f64>().unwrap_or(1.0)
            } else {
                state.mixed_euribor_spread.parse::<f64>().unwrap_or(1.5)
            },
        },
        _ => RateMode::Fix {
            rate: 3.6,
            spread: 0.0,
        },
    };

    let params = LoanParams {
        amount,
        term_years,
        payment_type,
        currency,
        start_date,
        rate_mode,
        same_spread: state.same_spread,
        euribor_curve: vec![],
        prepayments: state.prepayments.clone(),
    };

    match Calculator::calculate(&params) {
        Ok(result) => {
            state.params = Some(params);
            state.result = Some(result);
            state.chart_svg = None;
            state.balance_svg = None;
            state.status = "Calculation complete".to_string();
            state.status_is_error = false;
        }
        Err(e) => {
            state.status = format!("Error: {}", e);
            state.status_is_error = true;
        }
    }
}

fn generate_chart(state: &mut State) {
    if let Some(ref result) = state.result {
        state.chart_svg = Some(chart::generate_stacked_bar_chart_svg(result));
    }
}

fn generate_balance_chart(state: &mut State) {
    if let Some(ref result) = state.result {
        state.balance_svg = Some(chart::generate_balance_line_chart_svg(result));
    }
}

fn save_session_gui(state: &mut State) {
    if let Some(ref result) = state.result {
        let amount = state.amount.parse::<f64>().unwrap_or(100_000.0);
        let term_years = state.term.parse::<u32>().unwrap_or(10);
        let currency = if state.currency == "USD" {
            Currency::Usd
        } else {
            Currency::Eur
        };
        let payment_type = if state.payment_type == "Diff" {
            PaymentType::Diff
        } else {
            PaymentType::Annuity
        };
        let start_date = chrono::NaiveDate::parse_from_str(&state.start_date, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let rate_mode = RateMode::Fix {
            rate: state.rate.parse::<f64>().unwrap_or(3.6),
            spread: state.spread.parse::<f64>().unwrap_or(0.0),
        };
        let params = LoanParams {
            amount,
            term_years,
            payment_type,
            currency,
            start_date,
            rate_mode,
            same_spread: state.same_spread,
            euribor_curve: vec![],
            prepayments: state.prepayments.clone(),
        };
        match mortgage_core::save_session("/tmp/mortgage_session.json", &params, result) {
            Ok(()) => {
                state.status = "Session saved to /tmp/mortgage_session.json".to_string();
                state.status_is_error = false;
            }
            Err(e) => {
                state.status = format!("Save failed: {}", e);
                state.status_is_error = true;
            }
        }
    } else {
        state.status = "No results to save. Calculate first.".to_string();
        state.status_is_error = true;
    }
}

fn load_session_gui(state: &mut State) {
    match mortgage_core::load_session("/tmp/mortgage_session.json") {
        Ok(session) => {
            state.amount = format!("{}", session.params.amount);
            state.term = format!("{}", session.params.term_years);
            state.start_date = session.params.start_date.format("%Y-%m-%d").to_string();
            state.currency = match session.params.currency {
                Currency::Usd => "USD".to_string(),
                Currency::Eur => "EUR".to_string(),
            };
            state.payment_type = match session.params.payment_type {
                PaymentType::Annuity => "Annuity".to_string(),
                PaymentType::Diff => "Diff".to_string(),
            };
            state.prepayments = session.params.prepayments;
            state.result = Some(session.result);
            state.chart_svg = None;
            state.balance_svg = None;
            state.status = "Session loaded successfully".to_string();
            state.status_is_error = false;
        }
        Err(e) => {
            state.status = format!("Load failed: {}", e);
            state.status_is_error = true;
        }
    }
}

fn export_csv(state: &mut State) {
    if let Some(ref result) = state.result {
        let csv = payments_to_csv(&result.payments);
        if let Err(e) = fs::write("/tmp/mortgage_payments.csv", csv) {
            state.status = format!("Export failed: {}", e);
            state.status_is_error = true;
        } else {
            state.status = "Saved to /tmp/mortgage_payments.csv".to_string();
            state.status_is_error = false;
        }
    } else {
        state.status = "No results to export. Calculate first.".to_string();
        state.status_is_error = true;
    }
}

fn export_pdf(state: &mut State) {
    use printpdf::*;
    use std::io::BufWriter;

    if let Some(ref result) = state.result {
        let (doc, page1, layer1) =
            PdfDocument::new("Mortgage Report", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let font = doc.add_builtin_font(BuiltinFont::Helvetica).expect("font");

        let mut y = Mm(280.0);
        let line_height = Mm(6.0);

        let write_line = |layer: &PdfLayerReference, text: &str, y: Mm| {
            layer.use_text(text, 10.0, Mm(20.0), y, &font);
        };

        write_line(&current_layer, "Mortgage Loan Report", y);
        y -= line_height * 2.0;

        let sym = if state.currency == "USD" { "$" } else { "€" };
        write_line(
            &current_layer,
            &format!(
                "Monthly Payment: {}{:.2}",
                sym,
                result.monthly_payment.unwrap_or(0.0)
            ),
            y,
        );
        y -= line_height;
        write_line(
            &current_layer,
            &format!("Total Principal: {}{:.2}", sym, result.total_principal),
            y,
        );
        y -= line_height;
        write_line(
            &current_layer,
            &format!("Total Interest: {}{:.2}", sym, result.total_interest),
            y,
        );
        y -= line_height;
        write_line(
            &current_layer,
            &format!("Total Paid: {}{:.2}", sym, result.total_paid),
            y,
        );
        y -= line_height;
        write_line(
            &current_layer,
            &format!("Payments Count: {}", result.payments.len()),
            y,
        );
        if let Some(idx) = result.principal_exceeds_interest_at {
            y -= line_height;
            write_line(
                &current_layer,
                &format!(
                    "Principal > Interest at payment #{} ({})",
                    idx + 1,
                    result.payments[idx].date
                ),
                y,
            );
        }
        y -= line_height * 2.0;

        write_line(&current_layer, "Payment Schedule (first 60):", y);
        y -= line_height;

        for (i, p) in result.payments.iter().take(60).enumerate() {
            let line = format!(
                "{:>3} | {} | {:>10.2} | {:>10.2} | {:>10.2} | {:>12.2}",
                i + 1,
                p.date,
                p.payment,
                p.principal,
                p.interest,
                p.remaining_balance
            );
            write_line(&current_layer, &line, y);
            y -= line_height;
        }

        let png_bytes = chart::generate_stacked_bar_chart_png(result);
        let png_path = "/tmp/mortgage_chart.png";
        let _ = fs::write(png_path, &png_bytes);
        let dynamic_image = image_open(png_path).expect("PNG open");
        let chart_image = Image::from_dynamic_image(&dynamic_image);
        let (page2, layer2) = doc.add_page(Mm(210.0), Mm(297.0), "Chart Layer");
        let chart_layer = doc.get_page(page2).get_layer(layer2);
        chart_image.add_to_layer(
            chart_layer,
            ImageTransform {
                translate_x: Some(Mm(20.0)),
                translate_y: Some(Mm(120.0)),
                dpi: Some(150.0),
                ..Default::default()
            },
        );

        let path = "/tmp/mortgage_report.pdf";
        if let Ok(file) = fs::File::create(path) {
            let mut writer = BufWriter::new(file);
            if doc.save(&mut writer).is_ok() {
                state.status = format!("Saved PDF to {}", path);
                state.status_is_error = false;
            } else {
                state.status = "PDF save failed".to_string();
                state.status_is_error = true;
            }
        } else {
            state.status = "PDF file creation failed".to_string();
            state.status_is_error = true;
        }
    } else {
        state.status = "No results to export. Calculate first.".to_string();
        state.status_is_error = true;
    }
}

fn parse_tenor(s: &str) -> EuriborTenor {
    match s {
        "1m" => EuriborTenor::OneMonth,
        "3m" => EuriborTenor::ThreeMonths,
        "12m" => EuriborTenor::TwelveMonths,
        _ => EuriborTenor::SixMonths,
    }
}
