use iced::{
    widget::{
        button, column, container, row, scrollable, svg, text, text_input, Column,
    },
    Alignment, Element, Length,
};
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
    Calculate,
    ExportCsv,
    ExportPdf,
    ShowTable,
    ShowChart,
}

#[derive(Debug, Clone, Default)]
struct State {
    amount: String,
    term: String,
    rate: String,
    spread: String,
    result: Option<LoanResult>,
    chart_svg: Option<String>,
    active_tab: ViewTab,
    status: String,
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
    let input_panel = column![
        text("Loan Parameters").size(20),
        row![
            text("Amount:").width(80),
            text_input("185000", &state.amount)
                .on_input(Message::AmountChanged)
                .width(120),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            text("Term (yrs):").width(80),
            text_input("30", &state.term)
                .on_input(Message::TermChanged)
                .width(120),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            text("Rate (%):").width(80),
            text_input("3.6", &state.rate)
                .on_input(Message::RateChanged)
                .width(120),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            text("Spread (%):").width(80),
            text_input("0.0", &state.spread)
                .on_input(Message::SpreadChanged)
                .width(120),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        button("Calculate").on_press(Message::Calculate),
        button("Export CSV").on_press(Message::ExportCsv),
        button("Export PDF").on_press(Message::ExportPdf),
    ]
    .spacing(10)
    .padding(20)
    .width(280);

    let results_panel: Element<Message> = if let Some(ref result) = state.result {
        let sym = "€";
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
    let rate = state.rate.parse::<f64>().unwrap_or(5.0);
    let spread = state.spread.parse::<f64>().unwrap_or(0.0);

    let params = LoanParams {
        amount,
        term_years,
        payment_type: PaymentType::Annuity,
        currency: Currency::Eur,
        start_date: chrono::Local::now().date_naive(),
        rate_mode: RateMode::Fix { rate, spread },
        same_spread: false,
        euribor_curve: vec![],
        prepayments: vec![],
    };

    state.result = Some(Calculator::calculate(&params));
    state.chart_svg = None;
    state.status = String::new();
}

fn generate_chart(state: &mut State) {
    if let Some(ref result) = state.result {
        state.chart_svg = Some(generate_balance_chart_svg(result));
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

        let sym = "€";
        write_line(&current_layer, &format!("Monthly Payment: {}{:.2}", sym, result.monthly_payment.unwrap_or(0.0)), y);
        y -= line_height;
        write_line(&current_layer, &format!("Total Principal: {}{:.2}", sym, result.total_principal), y);
        y -= line_height;
        write_line(&current_layer, &format!("Total Interest: {}{:.2}", sym, result.total_interest), y);
        y -= line_height;
        write_line(&current_layer, &format!("Total Paid: {}{:.2}", sym, result.total_paid), y);
        y -= line_height;
        write_line(&current_layer, &format!("Payments Count: {}", result.payments.len()), y);
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

        let path = "/tmp/mortgage_report.pdf";
        if let Ok(file) = fs::File::create(path) {
            let mut writer = BufWriter::new(file);
            if doc.save(&mut writer).is_ok() {
                state.status = format!("Saved PDF to {}", path);
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

fn generate_balance_chart_svg(result: &LoanResult) -> String {
    use plotters::prelude::*;

    let mut svg_data = String::new();
    {
        let root = SVGBackend::with_string(&mut svg_data, (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let months: Vec<usize> = (0..result.payments.len()).collect();
        let balances: Vec<f64> = result.payments.iter().map(|p| p.remaining_balance).collect();
        let principals: Vec<f64> = result.payments.iter().map(|p| p.principal).collect();
        let interests: Vec<f64> = result.payments.iter().map(|p| p.interest).collect();

        let max_y = balances
            .iter()
            .cloned()
            .chain(principals.iter().cloned())
            .chain(interests.iter().cloned())
            .fold(0.0, f64::max)
            * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Loan Balance & Payments", ("sans-serif", 30))
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0..months.len(), 0.0..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(LineSeries::new(
                months.iter().zip(balances.iter()).map(|(x, y)| (*x, *y)),
                &BLUE,
            ))
            .unwrap()
            .label("Balance");

        chart
            .draw_series(LineSeries::new(
                months.iter().zip(principals.iter()).map(|(x, y)| (*x, *y)),
                &GREEN,
            ))
            .unwrap()
            .label("Principal");

        chart
            .draw_series(LineSeries::new(
                months.iter().zip(interests.iter()).map(|(x, y)| (*x, *y)),
                &RED,
            ))
            .unwrap()
            .label("Interest");

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .draw()
            .unwrap();
    }

    svg_data
}
