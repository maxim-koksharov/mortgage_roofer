
use iced::{
    widget::{
        button, checkbox, column, container, pick_list, row, scrollable, svg, text, text_input, Column,
    },
    Alignment, Element, Length,
};
use image::open as image_open;
use mortgage_core::models::*;
use mortgage_core::Calculator;
use std::fs;

pub fn main() -> iced::Result {
    iced::run("Mortgage Calculator", update, view)
}

#[derive(Debug, Clone)]
enum Message {
    AmountChanged(String),
    TermChanged(String),
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
    Calculate,
    ExportCsv,
    ExportPdf,
    ShowTable,
    ShowChart,
}

#[derive(Debug, Clone)]
struct State {
    amount: String,
    term: String,
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
    result: Option<LoanResult>,
    chart_svg: Option<String>,
    active_tab: ViewTab,
    status: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            amount: "185000".to_string(),
            term: "30".to_string(),
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
            result: None,
            chart_svg: None,
            active_tab: ViewTab::Table,
            status: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum ViewTab {
    #[default]
    Table,
    Chart,
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::AmountChanged(v) => state.amount = v,
        Message::TermChanged(v) => state.term = v,
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
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let currencies = vec!["EUR".to_string(), "USD".to_string()];
    let payment_types = vec!["Annuity".to_string(), "Diff".to_string()];
    let rate_modes = vec!["Fix".to_string(), "Euribor".to_string(), "Mixed".to_string()];
    let tenors = vec!["1m".to_string(), "3m".to_string(), "6m".to_string(), "12m".to_string()];
    let effects = vec!["ReduceTerm".to_string(), "ReducePayment".to_string()];

    let mut input_fields: Vec<Element<'_, Message>> = vec![
        text("Loan Parameters").size(20).into(),
        row![
            text("Amount:").width(Length::Fixed(120.0)),
            text_input("185000", &state.amount)
                .on_input(Message::AmountChanged)
                .width(Length::Fixed(150.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
        row![
            text("Term (yrs):").width(Length::Fixed(120.0)),
            text_input("30", &state.term)
                .on_input(Message::TermChanged)
                .width(Length::Fixed(150.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
        row![
            text("Currency:").width(Length::Fixed(120.0)),
            pick_list(currencies, Some(state.currency.clone()), Message::CurrencyChanged),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
        row![
            text("Payment:").width(Length::Fixed(120.0)),
            pick_list(payment_types, Some(state.payment_type.clone()), Message::PaymentTypeChanged),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
        row![
            text("Rate mode:").width(Length::Fixed(120.0)),
            pick_list(rate_modes, Some(state.rate_mode.clone()), Message::RateModeChanged),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
    ];

    match state.rate_mode.as_str() {
        "Fix" => {
            input_fields.push(
                row![
                    text("Rate (%):").width(Length::Fixed(120.0)),
                    text_input("3.6", &state.rate)
                        .on_input(Message::RateChanged)
                        .width(Length::Fixed(150.0)),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into(),
            );
            input_fields.push(
                row![
                    text("Spread (%):").width(Length::Fixed(120.0)),
                    text_input("0.0", &state.spread)
                        .on_input(Message::SpreadChanged)
                        .width(Length::Fixed(150.0)),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into(),
            );
        }
        "Euribor" => {
            input_fields.push(
                row![
                    text("Tenor:").width(Length::Fixed(120.0)),
                    pick_list(tenors.clone(), Some(state.euribor_tenor.clone()), Message::EuriborTenorChanged),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into(),
            );
            input_fields.push(
                row![
                    text("Spread (%):").width(Length::Fixed(120.0)),
                    text_input("1.0", &state.euribor_spread)
                        .on_input(Message::EuriborSpreadChanged)
                        .width(Length::Fixed(150.0)),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into(),
            );
        }
        "Mixed" => {
            input_fields.push(
                row![
                    text("Fixed years:").width(Length::Fixed(120.0)),
                    text_input("2", &state.mixed_fix_years)
                        .on_input(Message::MixedFixYearsChanged)
                        .width(Length::Fixed(150.0)),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into(),
            );
            input_fields.push(
                row![
                    text("Fix rate (%):").width(Length::Fixed(120.0)),
                    text_input("3.0", &state.mixed_fix_rate)
                        .on_input(Message::MixedFixRateChanged)
                        .width(Length::Fixed(150.0)),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into(),
            );
            input_fields.push(
                row![
                    text("Fix spread (%):").width(Length::Fixed(120.0)),
                    text_input("1.0", &state.mixed_fix_spread)
                        .on_input(Message::MixedFixSpreadChanged)
                        .width(Length::Fixed(150.0)),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into(),
            );
            input_fields.push(
                row![
                    text("Euribor tenor:").width(Length::Fixed(120.0)),
                    pick_list(tenors.clone(), Some(state.mixed_euribor_tenor.clone()), Message::MixedEuriborTenorChanged),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into(),
            );
            if !state.same_spread {
                input_fields.push(
                    row![
                        text("Euribor spr (%):").width(Length::Fixed(120.0)),
                        text_input("1.5", &state.mixed_euribor_spread)
                            .on_input(Message::MixedEuriborSpreadChanged)
                            .width(Length::Fixed(150.0)),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .into(),
                );
            }
            input_fields.push(
                row![
                    text("Same spread:").width(Length::Fixed(120.0)),
                    checkbox("", state.same_spread)
                        .on_toggle(Message::SameSpreadToggled),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into(),
            );
        }
        _ => {}
    }

    // Prepayment
    input_fields.push(text("Prepayment").size(16).into());
    input_fields.push(
        row![
            text("Date:").width(Length::Fixed(120.0)),
            text_input("2027-01-01", &state.prepayment_date)
                .on_input(Message::PrepaymentDateChanged)
                .width(Length::Fixed(150.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
    );
    input_fields.push(
        row![
            text("Amount:").width(Length::Fixed(120.0)),
            text_input("20000", &state.prepayment_amount)
                .on_input(Message::PrepaymentAmountChanged)
                .width(Length::Fixed(150.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
    );
    input_fields.push(
        row![
            text("Effect:").width(Length::Fixed(120.0)),
            pick_list(effects, Some(state.prepayment_effect.clone()), Message::PrepaymentEffectChanged),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .into(),
    );

    input_fields.push(
        row![
            button("Calculate").on_press(Message::Calculate),
            button("Export CSV").on_press(Message::ExportCsv),
            button("Export PDF").on_press(Message::ExportPdf),
        ]
        .spacing(10)
        .into(),
    );

    let input_panel = Column::from_vec(input_fields)
        .spacing(8)
        .padding(20)
        .width(300);

    let results_panel: Element<Message> = if let Some(ref result) = state.result {
        let sym = if state.currency == "USD" { "$" } else { "€" };
        let summary = column![
            text("Results").size(20),
            text(format!("Monthly: {}{:.2}", sym, result.monthly_payment.unwrap_or(0.0))),
            text(format!("Total Principal: {}{:.2}", sym, result.total_principal)),
            text(format!("Total Interest: {}{:.2}", sym, result.total_interest)),
            text(format!("Total Paid: {}{:.2}", sym, result.total_paid)),
            text(format!("Payments: {}", result.payments.len())),
            if let Some(idx) = result.principal_exceeds_interest_at {
                text(format!("Principal > Interest at payment #{} ({})", idx + 1, result.payments[idx].date))
            } else {
                text("")
            },
            text(&state.status),
        ]
        .spacing(5)
        .padding(10);

        let tabs = row![
            button("Table").on_press(Message::ShowTable),
            button("Chart").on_press(Message::ShowChart),
        ]
        .spacing(10);

        let content: Element<Message> = if state.active_tab == ViewTab::Table {
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
        } else {
            if let Some(ref svg_str) = state.chart_svg {
                let _ = fs::write("/tmp/mortgage_chart.svg", svg_str);
                let handle = svg::Handle::from_path("/tmp/mortgage_chart.svg");
                svg(handle).width(Length::Fill).height(Length::Fill).into()
            } else {
                text("Chart not available").into()
            }
        };

        column![summary, tabs, content].spacing(10).padding(10).into()
    } else {
        text("Enter parameters and press Calculate").into()
    };

    let main_layout = row![input_panel, results_panel]
        .spacing(20)
        .padding(20);

    container(main_layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn calculate(state: &mut State) {
    let amount = state.amount.parse::<f64>().unwrap_or(100_000.0);
    let term_years = state.term.parse::<u32>().unwrap_or(10);
    let currency = if state.currency == "USD" { Currency::Usd } else { Currency::Eur };
    let payment_type = if state.payment_type == "Diff" { PaymentType::Diff } else { PaymentType::Annuity };
    let start_date = chrono::Local::now().date_naive();

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
        _ => RateMode::Fix { rate: 3.6, spread: 0.0 },
    };

    let mut prepayments = vec![];
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&state.prepayment_date, "%Y-%m-%d") {
        let amount = state.prepayment_amount.parse::<f64>().unwrap_or(0.0);
        if amount > 0.0 {
            let effect = if state.prepayment_effect == "ReducePayment" {
                PrepaymentEffect::ReducePayment
            } else {
                PrepaymentEffect::ReduceTerm
            };
            prepayments.push(Prepayment { date, amount, effect });
        }
    }

    let params = LoanParams {
        amount,
        term_years,
        payment_type,
        currency,
        start_date,
        rate_mode,
        same_spread: state.same_spread,
        euribor_curve: vec![],
        prepayments,
    };

    state.result = Some(Calculator::calculate(&params));
    state.chart_svg = None;
    state.status = String::new();
}

fn generate_chart(state: &mut State) {
    if let Some(ref result) = state.result {
        state.chart_svg = Some(generate_stacked_bar_chart_svg(result));
    }
}

fn export_csv(state: &mut State) {
    if let Some(ref result) = state.result {
        let csv = payments_to_csv(&result.payments);
        if let Err(e) = fs::write("/tmp/mortgage_payments.csv", csv) {
            state.status = format!("Export failed: {}", e);
        } else {
            state.status = "Saved to /tmp/mortgage_payments.csv".to_string();
        }
    }
}

fn export_pdf(state: &mut State) {
    use printpdf::*;
    use std::io::BufWriter;

    if let Some(ref result) = state.result {
        let (doc, page1, layer1) = PdfDocument::new(
            "Mortgage Report",
            Mm(210.0),
            Mm(297.0),
            "Layer 1",
        );
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .expect("font");

        let mut y = Mm(280.0);
        let line_height = Mm(6.0);

        let write_line = |layer: &PdfLayerReference, text: &str, y: Mm| {
            layer.use_text(text, 10.0, Mm(20.0), y, &font);
        };

        write_line(&current_layer, "Mortgage Loan Report", y);
        y -= line_height * 2.0;

        let sym = if state.currency == "USD" { "$" } else { "€" };
        write_line(&current_layer, &format!("Monthly Payment: {}{:.2}", sym, result.monthly_payment.unwrap_or(0.0)), y);
        y -= line_height;
        write_line(&current_layer, &format!("Total Principal: {}{:.2}", sym, result.total_principal), y);
        y -= line_height;
        write_line(&current_layer, &format!("Total Interest: {}{:.2}", sym, result.total_interest), y);
        y -= line_height;
        write_line(&current_layer, &format!("Total Paid: {}{:.2}", sym, result.total_paid), y);
        y -= line_height;
        write_line(&current_layer, &format!("Payments Count: {}", result.payments.len()), y);
        if let Some(idx) = result.principal_exceeds_interest_at {
            y -= line_height;
            write_line(&current_layer, &format!("Principal > Interest at payment #{} ({})", idx + 1, result.payments[idx].date), y);
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

        // Add chart page
        let png_bytes = generate_chart_png(result);
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
                state.status = format!("Saved PDF with chart to {}", path);
            } else {
                state.status = "PDF save failed".to_string();
            }
        } else {
            state.status = "PDF file creation failed".to_string();
        }
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

fn parse_tenor(s: &str) -> EuriborTenor {
    match s {
        "1m" => EuriborTenor::OneMonth,
        "3m" => EuriborTenor::ThreeMonths,
        "12m" => EuriborTenor::TwelveMonths,
        _ => EuriborTenor::SixMonths,
    }
}

fn generate_stacked_bar_chart_svg(result: &LoanResult) -> String {
    use plotters::prelude::*;

    let mut svg_data = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_data, (900, 500)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let n = result.payments.len();
        let max_y = result.payments
            .iter()
            .map(|p| p.principal + p.interest)
            .fold(0.0, f64::max)
            * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Principal vs Interest (Stacked)", ("sans-serif", 28))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        let bar_width = 0.8;
        for (idx, p) in result.payments.iter().enumerate() {
            let x = idx as f64;
            chart.draw_series(std::iter::once(
                Rectangle::new(
                    [(x - bar_width / 2.0, 0.0), (x + bar_width / 2.0, p.principal)],
                    GREEN.filled(),
                ),
            )).unwrap();
            chart.draw_series(std::iter::once(
                Rectangle::new(
                    [(x - bar_width / 2.0, p.principal), (x + bar_width / 2.0, p.principal + p.interest)],
                    RED.filled(),
                ),
            )).unwrap();
        }

        // Marker for principal > interest crossing
        if let Some(cross_idx) = result.principal_exceeds_interest_at {
            let cross_payment = &result.payments[cross_idx];
            chart.draw_series(std::iter::once(
                Circle::new((cross_idx as f64, cross_payment.principal + cross_payment.interest), 6, BLUE.filled()),
            )).unwrap();
            chart.draw_series(std::iter::once(
                Text::new(
                    format!("Cross #{} ({})", cross_idx + 1, cross_payment.date),
                    (cross_idx as f64, max_y * 0.92),
                    ("sans-serif", 12).into_font().color(&BLUE),
                ),
            )).unwrap();
        }

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .draw()
            .unwrap();
    }

    svg_data
}

fn generate_chart_png(result: &LoanResult) -> Vec<u8> {
    use plotters::prelude::*;
    use image::ImageEncoder;

    let width = 900u32;
    let height = 500u32;
    let mut raw = vec![0u8; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut raw, (width, height)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let n = result.payments.len();
        let max_y = result.payments
            .iter()
            .map(|p| p.principal + p.interest)
            .fold(0.0, f64::max)
            * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Principal vs Interest (Stacked)", ("sans-serif", 28))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .build_cartesian_2d(0.0..n as f64, 0.0..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        let bar_width = 0.8;
        for (idx, p) in result.payments.iter().enumerate() {
            let x = idx as f64;
            chart.draw_series(std::iter::once(
                Rectangle::new(
                    [(x - bar_width / 2.0, 0.0), (x + bar_width / 2.0, p.principal)],
                    GREEN.filled(),
                ),
            )).unwrap();
            chart.draw_series(std::iter::once(
                Rectangle::new(
                    [(x - bar_width / 2.0, p.principal), (x + bar_width / 2.0, p.principal + p.interest)],
                    RED.filled(),
                ),
            )).unwrap();
        }

        if let Some(cross_idx) = result.principal_exceeds_interest_at {
            let cross_payment = &result.payments[cross_idx];
            chart.draw_series(std::iter::once(
                Circle::new((cross_idx as f64, cross_payment.principal + cross_payment.interest), 6, BLUE.filled()),
            )).unwrap();
            chart.draw_series(std::iter::once(
                Text::new(
                    format!("Cross #{} ({})", cross_idx + 1, cross_payment.date),
                    (cross_idx as f64, max_y * 0.92),
                    ("sans-serif", 12).into_font().color(&BLUE),
                ),
            )).unwrap();
        }

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .draw()
            .unwrap();
    }

    // Encode raw RGB to PNG
    let mut png_buf = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut png_buf);
        image::codecs::png::PngEncoder::new(&mut cursor)
            .write_image(&raw, width, height, image::ColorType::Rgb8)
            .unwrap();
    }
    png_buf
}
