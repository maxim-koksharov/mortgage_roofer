pub mod chart;

use iced::{
    Alignment, Element, Length, Theme,
    widget::{
        Column, Rule, button, checkbox, column, container, pick_list, row, scrollable, svg, text,
        text_input,
    },
};
use mortgage_core::models::*;
use mortgage_core::{Calculator, payments_to_csv};
use std::fs;

pub fn run() -> iced::Result {
    iced::application("Mortgage Calculator", update, view)
        .theme(|_| Theme::TokyoNightStorm)
        .run()
}

#[derive(Debug, Clone)]
pub enum Message {
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
    UpfrontCostChanged(String),
    UpfrontPercentChanged(String),
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
pub struct State {
    pub amount: String,
    pub term: String,
    pub start_date: String,
    pub rate: String,
    pub spread: String,
    pub currency: String,
    pub payment_type: String,
    pub rate_mode: String,
    pub euribor_tenor: String,
    pub euribor_spread: String,
    pub mixed_fix_years: String,
    pub mixed_fix_rate: String,
    pub mixed_fix_spread: String,
    pub mixed_euribor_tenor: String,
    pub mixed_euribor_spread: String,
    pub same_spread: bool,
    pub prepayment_date: String,
    pub prepayment_amount: String,
    pub prepayment_effect: String,
    pub prepayments: Vec<Prepayment>,
    pub upfront_cost: String,
    pub upfront_percent: String,
    pub rent: String,
    pub params: Option<LoanParams>,
    pub result: Option<LoanResult>,
    pub chart_svg: Option<String>,
    pub balance_svg: Option<String>,
    pub overlay_svg: Option<String>,
    pub active_tab: ViewTab,
    pub status: String,
    pub status_is_error: bool,
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
            upfront_cost: "0".to_string(),
            upfront_percent: "5".to_string(),
            rent: "900".to_string(),
            params: None,
            result: None,
            chart_svg: None,
            balance_svg: None,
            overlay_svg: None,
            active_tab: ViewTab::Table,
            status: String::new(),
            status_is_error: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ViewTab {
    #[default]
    Table,
    Chart,
    BalanceChart,
    OverlayChart,
    Yearly,
    Sensitivity,
    BreakEven,
}

pub fn update(state: &mut State, message: Message) {
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
        Message::UpfrontCostChanged(v) => state.upfront_cost = v,
        Message::UpfrontPercentChanged(v) => state.upfront_percent = v,
        Message::AddPrepayment => add_prepayment(state),
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
            generate_chart(state);
        }
        Message::ShowBalanceChart => {
            state.active_tab = ViewTab::BalanceChart;
            generate_balance_chart(state);
        }
        Message::ShowOverlayChart => {
            state.active_tab = ViewTab::OverlayChart;
            generate_overlay_chart(state);
        }
        Message::ShowYearly => state.active_tab = ViewTab::Yearly,
        Message::ShowSensitivity => state.active_tab = ViewTab::Sensitivity,
        Message::ShowBreakEven => state.active_tab = ViewTab::BreakEven,
        Message::RentChanged(v) => state.rent = v,
        Message::SaveSession => save_session_gui(state),
        Message::LoadSession => load_session_gui(state),
    }
}

fn add_prepayment(state: &mut State) {
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&state.prepayment_date, "%Y-%m-%d") {
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
                return;
            }
            state.status = "Prepayment amount must be positive".to_string();
        } else {
            state.status = "Invalid prepayment amount".to_string();
        }
    } else {
        state.status = "Invalid date format (YYYY-MM-DD)".to_string();
    }
    state.status_is_error = true;
}

fn input_row<'a>(label: &'a str, content: Element<'a, Message>) -> Element<'a, Message> {
    row![text(label).size(13).width(Length::Fixed(85.0)), content]
        .spacing(3)
        .align_y(Alignment::Center)
        .into()
}

fn section_header(title: &str) -> Element<'_, Message> {
    container(text(title).size(11))
        .padding(0)
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.2, 0.3, 0.4,
            ))),
            ..Default::default()
        })
        .into()
}

fn compact_input<'a>(
    placeholder: &str,
    value: &'a str,
    msg: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    text_input(placeholder, value)
        .on_input(msg)
        .padding(0)
        .size(13)
        .width(Length::Fill)
        .into()
}

pub fn view(state: &State) -> Element<'_, Message> {
    let input_panel = column![
        view_loan_section(state),
        Rule::horizontal(1),
        view_rate_section(state),
        Rule::horizontal(1),
        view_prepay_section(state),
        Rule::horizontal(1),
        view_actions_section(state),
    ]
    .width(Length::FillPortion(1));

    let results_panel = container(view_results_panel(state)).width(Length::FillPortion(3));

    container(row![input_panel, results_panel].spacing(10).padding(2))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn view_loan_section(state: &State) -> Element<'_, Message> {
    let currencies = vec!["EUR".to_string(), "USD".to_string()];
    let payment_types = vec!["Annuity".to_string(), "Diff".to_string()];

    column![
        section_header("Loan"),
        input_row(
            "Amount:",
            validated_input(
                "185000",
                &state.amount,
                Message::AmountChanged,
                state.amount.parse::<f64>().is_ok()
            )
        ),
        input_row(
            "Term:",
            validated_input(
                "30",
                &state.term,
                Message::TermChanged,
                state.term.parse::<u32>().is_ok()
            )
        ),
        input_row(
            "Start:",
            validated_input(
                "2025-01-01",
                &state.start_date,
                Message::StartDateChanged,
                chrono::NaiveDate::parse_from_str(&state.start_date, "%Y-%m-%d").is_ok()
            )
        ),
        input_row(
            "Curr:",
            pick_list(
                currencies,
                Some(state.currency.clone()),
                Message::CurrencyChanged
            )
            .text_size(13)
            .into()
        ),
        input_row(
            "Type:",
            pick_list(
                payment_types,
                Some(state.payment_type.clone()),
                Message::PaymentTypeChanged
            )
            .text_size(13)
            .into()
        ),
    ]
    .spacing(0)
    .padding(0)
    .into()
}

fn view_rate_section(state: &State) -> Element<'_, Message> {
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

    let mut fields: Vec<Element<'_, Message>> = vec![section_header("Rate")];
    fields.push(input_row(
        "Mode:",
        pick_list(
            rate_modes,
            Some(state.rate_mode.clone()),
            Message::RateModeChanged,
        )
        .text_size(13)
        .into(),
    ));

    match state.rate_mode.as_str() {
        "Fix" => {
            fields.push(input_row(
                "Rate:",
                compact_input("3.6", &state.rate, Message::RateChanged),
            ));
            fields.push(input_row(
                "Spread:",
                compact_input("0.0", &state.spread, Message::SpreadChanged),
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
                .text_size(13)
                .into(),
            ));
            fields.push(input_row(
                "Spread:",
                compact_input("1.0", &state.euribor_spread, Message::EuriborSpreadChanged),
            ));
        }
        "Mixed" => {
            fields.push(input_row(
                "Fix yrs:",
                compact_input("2", &state.mixed_fix_years, Message::MixedFixYearsChanged),
            ));
            fields.push(input_row(
                "Fix rate:",
                compact_input("3.0", &state.mixed_fix_rate, Message::MixedFixRateChanged),
            ));
            fields.push(input_row(
                "Fix spr:",
                compact_input(
                    "1.0",
                    &state.mixed_fix_spread,
                    Message::MixedFixSpreadChanged,
                ),
            ));
            fields.push(input_row(
                "Euri tnr:",
                pick_list(
                    tenors.clone(),
                    Some(state.mixed_euribor_tenor.clone()),
                    Message::MixedEuriborTenorChanged,
                )
                .text_size(13)
                .into(),
            ));
            if !state.same_spread {
                fields.push(input_row(
                    "Euri spr:",
                    compact_input(
                        "1.5",
                        &state.mixed_euribor_spread,
                        Message::MixedEuriborSpreadChanged,
                    ),
                ));
            }
            fields.push(input_row(
                "Same spr:",
                checkbox("", state.same_spread)
                    .on_toggle(Message::SameSpreadToggled)
                    .into(),
            ));
        }
        _ => {}
    }
    Column::from_vec(fields).spacing(0).padding(0).into()
}

fn view_prepay_section(state: &State) -> Element<'_, Message> {
    let effects = vec!["ReduceTerm".to_string(), "ReducePayment".to_string()];

    let mut fields: Vec<Element<'_, Message>> = vec![section_header("Prepay")];
    fields.push(input_row(
        "Date:",
        compact_input(
            "2027-01-01",
            &state.prepayment_date,
            Message::PrepaymentDateChanged,
        ),
    ));
    fields.push(input_row(
        "Amt:",
        compact_input(
            "20000",
            &state.prepayment_amount,
            Message::PrepaymentAmountChanged,
        ),
    ));
    fields.push(input_row(
        "Effect:",
        pick_list(
            effects,
            Some(state.prepayment_effect.clone()),
            Message::PrepaymentEffectChanged,
        )
        .text_size(13)
        .into(),
    ));
    fields.push(
        button(" +Add ")
            .padding(0)
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
                .size(12)
                .width(Length::Fill),
                button(" X")
                    .padding(0)
                    .on_press(Message::RemovePrepayment(i)),
            ]
            .spacing(5)
            .align_y(Alignment::Center)
            .into(),
        );
    }
    Column::from_vec(fields).spacing(0).padding(0).into()
}

fn view_actions_section(state: &State) -> Element<'_, Message> {
    let _ = state; // unused, kept for API consistency
    column![
        section_header("Actions"),
        row![
            button(" Calc ").padding(0).on_press(Message::Calculate),
            button(" CSV ").padding(0).on_press(Message::ExportCsv),
            button(" PDF ").padding(0).on_press(Message::ExportPdf),
        ]
        .spacing(3),
        row![
            button(" Save ").padding(0).on_press(Message::SaveSession),
            button(" Load ").padding(0).on_press(Message::LoadSession),
        ]
        .spacing(3),
    ]
    .spacing(0)
    .padding(0)
    .into()
}

fn view_results_panel(state: &State) -> Element<'_, Message> {
    let Some(ref result) = state.result else {
        return container(text("Enter parameters and press Calculate"))
            .padding(10)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    };

    let sym = if state.currency == "USD" { "$" } else { "€" };
    let summary = container(
        column![
            text("Results").size(16),
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
        .spacing(1),
    )
    .padding(4)
    .width(Length::Fill);

    let tabs = view_tabs(state);
    let content = view_tab_content(state, result);
    let status_bar = view_status_bar(state);

    column![summary, tabs, content, status_bar]
        .spacing(4)
        .padding(4)
        .into()
}

fn view_tabs(state: &State) -> Element<'_, Message> {
    let tab = |label, tab, active_tab| {
        button(label)
            .padding(2)
            .style(if active_tab == tab {
                button::primary
            } else {
                button::secondary
            })
            .on_press(match tab {
                ViewTab::Table => Message::ShowTable,
                ViewTab::Chart => Message::ShowChart,
                ViewTab::BalanceChart => Message::ShowBalanceChart,
                ViewTab::OverlayChart => Message::ShowOverlayChart,
                ViewTab::Yearly => Message::ShowYearly,
                ViewTab::Sensitivity => Message::ShowSensitivity,
                ViewTab::BreakEven => Message::ShowBreakEven,
            })
    };

    row![
        tab("Table", ViewTab::Table, state.active_tab),
        tab("Stacked", ViewTab::Chart, state.active_tab),
        tab("Balance", ViewTab::BalanceChart, state.active_tab),
        tab("Overlay", ViewTab::OverlayChart, state.active_tab),
        tab("Yearly", ViewTab::Yearly, state.active_tab),
        tab("Sensitivity", ViewTab::Sensitivity, state.active_tab),
        tab("Break-Even", ViewTab::BreakEven, state.active_tab),
    ]
    .spacing(3)
    .into()
}

fn view_tab_content<'a>(state: &'a State, result: &'a LoanResult) -> Element<'a, Message> {
    match state.active_tab {
        ViewTab::Table => view_table_tab(result),
        ViewTab::Chart => view_chart_tab(
            &state.chart_svg,
            "/tmp/mortgage_chart.svg",
            "Chart not available",
        ),
        ViewTab::BalanceChart => view_chart_tab(
            &state.balance_svg,
            "/tmp/mortgage_balance.svg",
            "Balance chart not available",
        ),
        ViewTab::OverlayChart => view_chart_tab(
            &state.overlay_svg,
            "/tmp/mortgage_overlay.svg",
            "Overlay chart not available",
        ),
        ViewTab::Yearly => view_yearly_tab(result),
        ViewTab::Sensitivity => view_sensitivity_tab(state),
        ViewTab::BreakEven => view_break_even_tab(state),
    }
}

fn view_table_tab(result: &LoanResult) -> Element<'_, Message> {
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
        table_rows.push(
            row![
                text(format!("{}", i + 1)).width(Length::Fixed(40.0)),
                text(p.date.to_string()).width(Length::Fixed(100.0)),
                text(format!("{:.2}", p.payment)).width(Length::Fixed(100.0)),
                text(format!("{:.2}", p.principal)).width(Length::Fixed(100.0)),
                text(format!("{:.2}", p.interest)).width(Length::Fixed(100.0)),
                text(format!("{:.2}", p.remaining_balance)).width(Length::Fixed(100.0)),
            ]
            .spacing(5)
            .into(),
        );
    }

    scrollable(container(Column::from_vec(table_rows).spacing(2)).padding(4)).into()
}

fn view_chart_tab<'a>(
    svg_opt: &'a Option<String>,
    path: &'a str,
    fallback: &'a str,
) -> Element<'a, Message> {
    if let Some(svg_str) = svg_opt {
        let _ = fs::write(path, svg_str);
        let handle = svg::Handle::from_path(path);
        svg(handle).width(Length::Fill).height(Length::Fill).into()
    } else {
        text(fallback).into()
    }
}

fn view_yearly_tab(result: &LoanResult) -> Element<'_, Message> {
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
        rows.push(
            row![
                text(format!("{}", s.year)).width(Length::Fixed(60.0)),
                text(format!("{:.2}", s.total_payment)).width(Length::Fixed(110.0)),
                text(format!("{:.2}", s.total_principal)).width(Length::Fixed(110.0)),
                text(format!("{:.2}", s.total_interest)).width(Length::Fixed(110.0)),
                text(format!("{}", s.payments_count)).width(Length::Fixed(60.0)),
                text(format!("{:.2}", s.ending_balance)).width(Length::Fixed(110.0)),
            ]
            .spacing(5)
            .into(),
        );
    }

    scrollable(container(Column::from_vec(rows).spacing(2)).padding(4)).into()
}

fn view_sensitivity_tab(state: &State) -> Element<'_, Message> {
    let Some(ref params) = state.params else {
        return text("Calculate first to see sensitivity analysis").into();
    };

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
        rows.push(
            row![
                text(format!("{:+.2}", p.rate_delta)).width(Length::Fixed(60.0)),
                text(format!("{:.2}", p.effective_rate)).width(Length::Fixed(80.0)),
                text(monthly).width(Length::Fixed(110.0)),
                text(format!("{:.2}", p.total_interest)).width(Length::Fixed(110.0)),
                text(format!("{:.2}", p.total_paid)).width(Length::Fixed(110.0)),
            ]
            .spacing(5)
            .into(),
        );
    }

    scrollable(container(Column::from_vec(rows).spacing(2)).padding(4)).into()
}

fn view_break_even_tab(state: &State) -> Element<'_, Message> {
    let Some(ref params) = state.params else {
        return text("Calculate first to see break-even analysis").into();
    };

    let rent = state.rent.parse::<f64>().unwrap_or(0.0);
    let content = if rent > 0.0 {
        let mut params = params.clone();
        params.upfront_cost = state.upfront_cost.parse::<f64>().ok().filter(|&v| v != 0.0);
        params.upfront_percent = state
            .upfront_percent
            .parse::<f64>()
            .ok()
            .filter(|&v| v != 0.0);
        let be = mortgage_core::break_even_analysis(&params, rent);
        column![
            text("Break-Even vs Rent").size(16),
            text(format!("Monthly rent:      {:.2}", be.monthly_rent)),
            text(format!("Monthly mortgage:  {:.2}", be.monthly_cost)),
            text(format!("Upfront costs:     {:.2}", be.upfront_costs)),
            text(format!("Total interest:    {:.2}", be.total_interest)),
            text(""),
            if let (Some(months), Some(years)) = (be.break_even_months, be.break_even_years) {
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
            input_row(
                "Upfront cost:",
                text_input("0", &state.upfront_cost)
                    .on_input(Message::UpfrontCostChanged)
                    .width(Length::Fixed(150.0))
                    .into()
            ),
            input_row(
                "Upfront %:",
                text_input("5", &state.upfront_percent)
                    .on_input(Message::UpfrontPercentChanged)
                    .width(Length::Fixed(150.0))
                    .into()
            ),
        ]
        .spacing(2)
    } else {
        column![
            text("Enter monthly rent for break-even analysis"),
            input_row(
                "Rent:",
                text_input("900", &state.rent)
                    .on_input(Message::RentChanged)
                    .width(Length::Fill)
                    .into(),
            ),
        ]
        .spacing(2)
    };

    container(content).padding(4).into()
}

fn view_status_bar(state: &State) -> Element<'_, Message> {
    container(text(&state.status))
        .padding(4)
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
        })
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
        .padding(0)
        .size(13)
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
        upfront_cost: state.upfront_cost.parse::<f64>().ok().filter(|&v| v != 0.0),
        upfront_percent: state
            .upfront_percent
            .parse::<f64>()
            .ok()
            .filter(|&v| v != 0.0),
    };

    match Calculator::calculate(&params) {
        Ok(result) => {
            state.params = Some(params);
            state.result = Some(result);
            state.chart_svg = None;
            state.balance_svg = None;
            state.overlay_svg = None;
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
    if let Some(ref result) = state.result
        && let Ok(svg) = chart::generate_stacked_bar_chart_svg(result)
    {
        state.chart_svg = Some(svg);
    }
}

fn generate_balance_chart(state: &mut State) {
    if let Some(ref result) = state.result
        && let Ok(svg) = chart::generate_balance_line_chart_svg(result)
    {
        state.balance_svg = Some(svg);
    }
}

fn generate_overlay_chart(state: &mut State) {
    if let Some(ref result) = state.result
        && let Ok(svg) = chart::generate_overlay_chart_svg(result)
    {
        state.overlay_svg = Some(svg);
    }
}

fn save_session_gui(state: &mut State) {
    if let (Some(params), Some(result)) = (&state.params, &state.result) {
        match mortgage_core::save_session("/tmp/mortgage_session.json", params, result) {
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
            state.prepayments = session.params.prepayments.clone();
            state.params = Some(session.params);
            state.result = Some(session.result);
            state.chart_svg = None;
            state.balance_svg = None;
            state.overlay_svg = None;
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
    use ::image::open as image_open;
    use printpdf::*;
    use std::io::BufWriter;

    if let Some(ref result) = state.result {
        let (doc, page1, layer1) =
            PdfDocument::new("Mortgage Report", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let font = match doc.add_builtin_font(BuiltinFont::Helvetica) {
            Ok(f) => f,
            Err(e) => {
                state.status = format!("Font error: {}", e);
                state.status_is_error = true;
                return;
            }
        };

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

        let png_bytes = match chart::generate_stacked_bar_chart_png(result) {
            Ok(bytes) => bytes,
            Err(e) => {
                state.status = format!("PNG generation error: {}", e);
                state.status_is_error = true;
                return;
            }
        };
        let png_path = "/tmp/mortgage_chart.png";
        if let Err(e) = fs::write(png_path, &png_bytes) {
            state.status = format!("PNG write error: {}", e);
            state.status_is_error = true;
            return;
        }

        let dynamic_image = match image_open(png_path) {
            Ok(img) => img,
            Err(e) => {
                state.status = format!("PNG open error: {}", e);
                state.status_is_error = true;
                return;
            }
        };
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
    s.parse().unwrap_or(EuriborTenor::SixMonths)
}
