use chrono::NaiveDate;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use mortgage_core::models::*;
use mortgage_core::Calculator;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table},
    DefaultTerminal, Frame,
};
use std::fs;

#[derive(Debug, Clone, PartialEq)]
enum Screen {
    Form,
    Results,
    Popup(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Field {
    Amount,
    Term,
    Currency,
    PaymentType,
    RateMode,
    // Fix fields
    FixRate,
    FixSpread,
    // Euribor fields
    EuriborTenor,
    EuriborSpread,
    // Mixed fields
    MixedFixYears,
    MixedFixRate,
    MixedFixSpread,
    MixedEuriborTenor,
    MixedEuriborSpread,
    SameSpread,
    // Prepayment
    PrepaymentDate,
    PrepaymentAmount,
    PrepaymentEffect,
}

struct App {
    screen: Screen,
    fields: Vec<Field>,
    selected: usize,

    // Data
    amount: String,
    term: String,
    currency: usize,       // 0=EUR, 1=USD
    payment_type: usize,   // 0=Annuity, 1=Diff
    rate_mode: usize,      // 0=Fix, 1=Euribor, 2=Mixed
    fix_rate: String,
    fix_spread: String,
    euribor_tenor: usize,  // 0=1m,1=3m,2=6m,3=12m
    euribor_spread: String,
    mixed_fix_years: String,
    mixed_fix_rate: String,
    mixed_fix_spread: String,
    mixed_euribor_tenor: usize,
    mixed_euribor_spread: String,
    same_spread: bool,
    prepayment_date: String,
    prepayment_amount: String,
    prepayment_effect: usize, // 0=ReduceTerm, 1=ReducePayment

    // Results
    result: Option<LoanResult>,
    params: Option<LoanParams>,
    scroll_offset: usize,
    popup_msg: Option<String>,
}

impl App {
    fn new() -> Self {
        let mut app = Self {
            screen: Screen::Form,
            fields: vec![],
            selected: 0,
            amount: "185000".to_string(),
            term: "30".to_string(),
            currency: 0,
            payment_type: 0,
            rate_mode: 0,
            fix_rate: "3.6".to_string(),
            fix_spread: "0.0".to_string(),
            euribor_tenor: 2,
            euribor_spread: "1.0".to_string(),
            mixed_fix_years: "2".to_string(),
            mixed_fix_rate: "3.0".to_string(),
            mixed_fix_spread: "1.0".to_string(),
            mixed_euribor_tenor: 2,
            mixed_euribor_spread: "1.5".to_string(),
            same_spread: false,
            prepayment_date: "2027-01-01".to_string(),
            prepayment_amount: "20000".to_string(),
            prepayment_effect: 0,
            result: None,
            params: None,
            scroll_offset: 0,
            popup_msg: None,
        };
        app.rebuild_fields();
        app
    }

    fn rebuild_fields(&mut self) {
        let mut f = vec![
            Field::Amount,
            Field::Term,
            Field::Currency,
            Field::PaymentType,
            Field::RateMode,
        ];
        match self.rate_mode {
            0 => {
                f.push(Field::FixRate);
                f.push(Field::FixSpread);
            }
            1 => {
                f.push(Field::EuriborTenor);
                f.push(Field::EuriborSpread);
            }
            2 => {
                f.push(Field::MixedFixYears);
                f.push(Field::MixedFixRate);
                f.push(Field::MixedFixSpread);
                f.push(Field::MixedEuriborTenor);
                f.push(Field::MixedEuriborSpread);
                f.push(Field::SameSpread);
            }
            _ => {}
        }
        f.push(Field::PrepaymentDate);
        f.push(Field::PrepaymentAmount);
        f.push(Field::PrepaymentEffect);
        self.fields = f;
        if self.selected >= self.fields.len() {
            self.selected = self.fields.len().saturating_sub(1);
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while self.screen != Screen::Results || self.result.is_some() {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.screen {
                        Screen::Form => self.handle_form(key.code),
                        Screen::Results => self.handle_results(key.code),
                        Screen::Popup(_) => {
                            self.popup_msg = None;
                            self.screen = Screen::Results;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_form(&mut self, code: KeyCode) {
        match code {
            KeyCode::Tab => {
                self.selected = (self.selected + 1) % self.fields.len();
            }
            KeyCode::BackTab => {
                self.selected = (self.selected + self.fields.len() - 1) % self.fields.len();
            }
            KeyCode::Down => {
                self.selected = (self.selected + 1) % self.fields.len();
            }
            KeyCode::Up => {
                self.selected = (self.selected + self.fields.len() - 1) % self.fields.len();
            }
            KeyCode::Char(c) if c.is_numeric() || c == '.' || c == '-' => {
                self.edit_text(c);
            }
            KeyCode::Backspace => {
                self.backspace();
            }
            KeyCode::Left => self.cycle_enum(-1),
            KeyCode::Right => self.cycle_enum(1),
            KeyCode::Enter => {
                if let Err(e) = self.calculate() {
                    self.popup_msg = Some(format!("Error: {}", e));
                    self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
                } else {
                    self.screen = Screen::Results;
                }
            }
            KeyCode::Esc => {}
            _ => {}
        }
    }

    fn handle_results(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::Form;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if let Some(ref result) = self.result {
                    let csv = payments_to_csv(&result.payments);
                    let path = "/tmp/mortgage_tui_export.csv";
                    if let Err(e) = fs::write(path, csv) {
                        self.popup_msg = Some(format!("CSV export failed: {}", e));
                    } else {
                        self.popup_msg = Some(format!("Saved CSV to {}", path));
                    }
                    self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
                }
            }
            KeyCode::Down | KeyCode::PageDown => {
                if let Some(ref result) = self.result {
                    let max = result.payments.len().saturating_sub(1);
                    if self.scroll_offset < max {
                        self.scroll_offset += 1;
                    }
                }
            }
            KeyCode::Up | KeyCode::PageUp => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            _ => {}
        }
    }

    fn edit_text(&mut self, c: char) {
        match self.fields[self.selected] {
            Field::Amount => self.amount.push(c),
            Field::Term => self.term.push(c),
            Field::FixRate => self.fix_rate.push(c),
            Field::FixSpread => self.fix_spread.push(c),
            Field::EuriborSpread => self.euribor_spread.push(c),
            Field::MixedFixYears => self.mixed_fix_years.push(c),
            Field::MixedFixRate => self.mixed_fix_rate.push(c),
            Field::MixedFixSpread => self.mixed_fix_spread.push(c),
            Field::MixedEuriborSpread => self.mixed_euribor_spread.push(c),
            Field::PrepaymentDate => self.prepayment_date.push(c),
            Field::PrepaymentAmount => self.prepayment_amount.push(c),
            _ => {}
        }
    }

    fn backspace(&mut self) {
        match self.fields[self.selected] {
            Field::Amount => { self.amount.pop(); }
            Field::Term => { self.term.pop(); }
            Field::FixRate => { self.fix_rate.pop(); }
            Field::FixSpread => { self.fix_spread.pop(); }
            Field::EuriborSpread => { self.euribor_spread.pop(); }
            Field::MixedFixYears => { self.mixed_fix_years.pop(); }
            Field::MixedFixRate => { self.mixed_fix_rate.pop(); }
            Field::MixedFixSpread => { self.mixed_fix_spread.pop(); }
            Field::MixedEuriborSpread => { self.mixed_euribor_spread.pop(); }
            Field::PrepaymentDate => { self.prepayment_date.pop(); }
            Field::PrepaymentAmount => { self.prepayment_amount.pop(); }
            _ => {}
        }
    }

    fn cycle_enum(&mut self, delta: i8) {
        let current = self.fields[self.selected];
        match current {
            Field::Currency => {
                self.currency = ((self.currency as i8 + delta).rem_euclid(2)) as usize;
            }
            Field::PaymentType => {
                self.payment_type = ((self.payment_type as i8 + delta).rem_euclid(2)) as usize;
            }
            Field::RateMode => {
                let old = self.rate_mode;
                self.rate_mode = ((self.rate_mode as i8 + delta).rem_euclid(3)) as usize;
                if old != self.rate_mode {
                    self.rebuild_fields();
                }
            }
            Field::EuriborTenor => {
                self.euribor_tenor = ((self.euribor_tenor as i8 + delta).rem_euclid(4)) as usize;
            }
            Field::MixedEuriborTenor => {
                self.mixed_euribor_tenor = ((self.mixed_euribor_tenor as i8 + delta).rem_euclid(4)) as usize;
            }
            Field::SameSpread => {
                self.same_spread = !self.same_spread;
            }
            Field::PrepaymentEffect => {
                self.prepayment_effect = ((self.prepayment_effect as i8 + delta).rem_euclid(2)) as usize;
            }
            _ => {}
        }
    }

    fn calculate(&mut self) -> Result<(), String> {
        let amount = self.amount.parse::<f64>().map_err(|_| "Invalid amount")?;
        let term_years = self.term.parse::<u32>().map_err(|_| "Invalid term")?;
        let currency = if self.currency == 0 { Currency::Eur } else { Currency::Usd };
        let payment_type = if self.payment_type == 0 { PaymentType::Annuity } else { PaymentType::Diff };
        let start_date = chrono::Local::now().date_naive();

        let rate_mode = match self.rate_mode {
            0 => RateMode::Fix {
                rate: self.fix_rate.parse::<f64>().map_err(|_| "Invalid fix rate")?,
                spread: self.fix_spread.parse::<f64>().map_err(|_| "Invalid fix spread")?,
            },
            1 => RateMode::Euribor {
                tenor: tenor_from_idx(self.euribor_tenor),
                spread: self.euribor_spread.parse::<f64>().map_err(|_| "Invalid euribor spread")?,
            },
            2 => RateMode::Mixed {
                fix_years: self.mixed_fix_years.parse::<f64>().map_err(|_| "Invalid fix years")?,
                fix_rate: self.mixed_fix_rate.parse::<f64>().map_err(|_| "Invalid mixed fix rate")?,
                fix_spread: self.mixed_fix_spread.parse::<f64>().map_err(|_| "Invalid mixed fix spread")?,
                euribor_tenor: tenor_from_idx(self.mixed_euribor_tenor),
                euribor_spread: if self.same_spread {
                    self.mixed_fix_spread.parse::<f64>().map_err(|_| "Invalid spread")?
                } else {
                    self.mixed_euribor_spread.parse::<f64>().map_err(|_| "Invalid euribor spread")?
                },
            },
            _ => return Err("Unknown rate mode".to_string()),
        };

        let mut prepayments = vec![];
        let prep_date = NaiveDate::parse_from_str(&self.prepayment_date, "%Y-%m-%d")
            .map_err(|_| "Invalid prepayment date (YYYY-MM-DD)")?;
        let prep_amount = self.prepayment_amount.parse::<f64>().map_err(|_| "Invalid prepayment amount")?;
        let prep_effect = if self.prepayment_effect == 0 { PrepaymentEffect::ReduceTerm } else { PrepaymentEffect::ReducePayment };
        if prep_amount > 0.0 {
            prepayments.push(Prepayment {
                date: prep_date,
                amount: prep_amount,
                effect: prep_effect,
            });
        }

        let params = LoanParams {
            amount,
            term_years,
            payment_type,
            currency,
            start_date,
            rate_mode,
            same_spread: self.same_spread,
            euribor_curve: vec![],
            prepayments,
        };

        let result = Calculator::calculate(&params);
        self.params = Some(params);
        self.result = Some(result);
        self.scroll_offset = 0;
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.screen {
            Screen::Form => self.draw_form(frame),
            Screen::Results => self.draw_results(frame),
            Screen::Popup(_) => {
                self.draw_results(frame);
                if let Some(ref msg) = self.popup_msg {
                    self.draw_popup(frame, msg);
                }
            }
        }
    }

    fn draw_form(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let header = Paragraph::new("Mortgage Calculator — Tab:next  Enter:calc  ←→:toggle  Backspace:delete")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        let mut lines = vec![];
        for (i, field) in self.fields.iter().enumerate() {
            let is_sel = i == self.selected;
            let style = if is_sel {
                Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let (label, value) = match field {
                Field::Amount => ("Amount", self.amount.clone()),
                Field::Term => ("Term (yrs)", self.term.clone()),
                Field::Currency => ("Currency", if self.currency == 0 { "EUR".to_string() } else { "USD".to_string() }),
                Field::PaymentType => ("Payment type", if self.payment_type == 0 { "Annuity".to_string() } else { "Diff".to_string() }),
                Field::RateMode => ("Rate mode", match self.rate_mode { 0 => "Fix", 1 => "Euribor", _ => "Mixed" }.to_string()),
                Field::FixRate => ("Fix rate (%)", self.fix_rate.clone()),
                Field::FixSpread => ("Fix spread (%)", self.fix_spread.clone()),
                Field::EuriborTenor => ("Euribor tenor", tenor_name(self.euribor_tenor).to_string()),
                Field::EuriborSpread => ("Euribor spread (%)", self.euribor_spread.clone()),
                Field::MixedFixYears => ("Fixed years", self.mixed_fix_years.clone()),
                Field::MixedFixRate => ("Mixed fix rate (%)", self.mixed_fix_rate.clone()),
                Field::MixedFixSpread => ("Mixed fix spread (%)", self.mixed_fix_spread.clone()),
                Field::MixedEuriborTenor => ("Mixed euribor tenor", tenor_name(self.mixed_euribor_tenor).to_string()),
                Field::MixedEuriborSpread => ("Mixed euribor spread (%)", self.mixed_euribor_spread.clone()),
                Field::SameSpread => ("Same spread", if self.same_spread { "Yes".to_string() } else { "No".to_string() }),
                Field::PrepaymentDate => ("Prepayment date", self.prepayment_date.clone()),
                Field::PrepaymentAmount => ("Prepayment amount", self.prepayment_amount.clone()),
                Field::PrepaymentEffect => ("Prepayment effect", if self.prepayment_effect == 0 { "ReduceTerm".to_string() } else { "ReducePayment".to_string() }),
            };

            let hint = if is_sel {
                match field {
                    Field::Currency | Field::PaymentType | Field::RateMode | Field::EuriborTenor
                    | Field::MixedEuriborTenor | Field::SameSpread | Field::PrepaymentEffect => {
                        " [←→ toggle]"
                    }
                    _ => " [type]",
                }
            } else {
                ""
            };

            let line = Line::from(vec![
                Span::styled(format!("{:>22}: ", label), style),
                Span::styled(format!("{}{}", value, hint), style),
            ]);
            lines.push(line);
        }

        let form = Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title("Parameters"));
        frame.render_widget(form, chunks[1]);
    }

    fn draw_results(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(0)])
            .split(area);

        let summary_text = if let (Some(params), Some(result)) = (&self.params, &self.result) {
            let sym = params.currency.symbol();
            let mut lines = vec![
                Line::from(format!("Amount: {}{:.2} | Term: {} years | Payment: {:?} | Rate mode: {:?}",
                    sym, params.amount, params.term_years, params.payment_type, params.rate_mode)),
                Line::from(format!("Total principal: {}{:.2} | Total interest: {}{:.2} | Total paid: {}{:.2}",
                    sym, result.total_principal, sym, result.total_interest, sym, result.total_paid)),
            ];
            if let Some(mp) = result.monthly_payment {
                lines.push(Line::from(format!("Monthly payment: {}{:.2}", sym, mp)));
            }
            lines.push(Line::from(format!("Payments count: {}", result.payments.len())));
            if let Some(idx) = result.principal_exceeds_interest_at {
                lines.push(Line::from(format!("Principal > Interest at payment #{} ({})", idx + 1, result.payments[idx].date)));
            }
            Text::from(lines)
        } else {
            Text::from("No results yet.")
        };

        let summary = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("Summary — press 'S' to export CSV, Esc to return"));
        frame.render_widget(summary, chunks[0]);

        if let Some(ref result) = self.result {
            let header = Row::new(vec!["#", "Date", "Payment", "Principal", "Interest", "Balance"])
                .style(Style::default().fg(Color::Yellow));
            let rows: Vec<Row> = result
                .payments
                .iter()
                .enumerate()
                .skip(self.scroll_offset)
                .take(chunks[1].height as usize - 2)
                .map(|(i, p)| {
                    Row::new(vec![
                        (i + 1).to_string(),
                        p.date.to_string(),
                        format!("{:.2}", p.payment),
                        format!("{:.2}", p.principal),
                        format!("{:.2}", p.interest),
                        format!("{:.2}", p.remaining_balance),
                    ])
                })
                .collect();

            let table = Table::new(rows, [
                Constraint::Length(5),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Length(12),
            ])
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Payments (↑↓ scroll)"));

            frame.render_widget(table, chunks[1]);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            let mut scrollbar_state = ScrollbarState::new(result.payments.len())
                .position(self.scroll_offset);
            frame.render_stateful_widget(
                scrollbar,
                chunks[1].inner(Margin { vertical: 1, horizontal: 0 }),
                &mut scrollbar_state,
            );
        }
    }

    fn draw_popup(&self, frame: &mut Frame, msg: &str) {
        let area = frame.area();
        let popup_area = centered_rect(60, 20, area);
        frame.render_widget(Clear, popup_area);
        let popup = Paragraph::new(msg)
            .block(Block::default().borders(Borders::ALL).title("Message"))
            .alignment(Alignment::Center);
        frame.render_widget(popup, popup_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)])
        .split(popup_layout[1])[1]
}

fn tenor_name(idx: usize) -> &'static str {
    match idx {
        0 => "1m",
        1 => "3m",
        2 => "6m",
        _ => "12m",
    }
}

fn tenor_from_idx(idx: usize) -> EuriborTenor {
    match idx {
        0 => EuriborTenor::OneMonth,
        1 => EuriborTenor::ThreeMonths,
        2 => EuriborTenor::SixMonths,
        _ => EuriborTenor::TwelveMonths,
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

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
