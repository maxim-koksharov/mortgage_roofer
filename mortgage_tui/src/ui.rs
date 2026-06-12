use crate::app::{AnalysisView, App, Field, Screen, tenor_name};
use mortgage_core::payments_to_csv;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table,
    },
};
use std::fs;

impl App {
    pub fn draw(&mut self, frame: &mut Frame) {
        match self.screen.clone() {
            Screen::Form => self.draw_form(frame),
            Screen::Results => self.draw_results(frame),
            Screen::Help => self.draw_help(frame),
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

        let header = Paragraph::new(
            "Mortgage Calculator — Tab:next  Enter:calc  ←→:toggle  Backspace:delete",
        )
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        let mut lines = vec![];
        for (i, field) in self.fields.iter().enumerate() {
            let is_sel = i == self.selected;
            let style = if is_sel {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let (label, value) = match field {
                Field::Amount => ("Amount", self.amount.clone()),
                Field::Term => ("Term (yrs)", self.term.clone()),
                Field::StartDate => ("Start date", self.start_date.clone()),
                Field::Currency => (
                    "Currency",
                    if self.currency == 0 {
                        "EUR".to_string()
                    } else {
                        "USD".to_string()
                    },
                ),
                Field::PaymentType => (
                    "Payment type",
                    if self.payment_type == 0 {
                        "Annuity".to_string()
                    } else {
                        "Diff".to_string()
                    },
                ),
                Field::RateMode => (
                    "Rate mode",
                    match self.rate_mode {
                        0 => "Fix",
                        1 => "Euribor",
                        _ => "Mixed",
                    }
                    .to_string(),
                ),
                Field::FixRate => ("Fix rate (%)", self.fix_rate.clone()),
                Field::FixSpread => ("Fix spread (%)", self.fix_spread.clone()),
                Field::EuriborTenor => {
                    ("Euribor tenor", tenor_name(self.euribor_tenor).to_string())
                }
                Field::EuriborSpread => ("Euribor spread (%)", self.euribor_spread.clone()),
                Field::EuriborFetchButton => {
                    let count = self.euribor_curve.len();
                    ("Euribor curve", format!("[Enter to fetch, {} pts]", count))
                }
                Field::EuriborDate => ("Euribor date", self.euribor_date.clone()),
                Field::EuriborRate => ("Euribor rate (%)", self.euribor_rate.clone()),
                Field::AddEuriborPoint => (
                    "Add Euribor pt",
                    "[Enter to add, Del=remove last]".to_string(),
                ),
                Field::MixedFixYears => ("Fixed years", self.mixed_fix_years.clone()),
                Field::MixedFixRate => ("Mixed fix rate (%)", self.mixed_fix_rate.clone()),
                Field::MixedFixSpread => ("Mixed fix spread (%)", self.mixed_fix_spread.clone()),
                Field::MixedEuriborTenor => (
                    "Mixed euribor tenor",
                    tenor_name(self.mixed_euribor_tenor).to_string(),
                ),
                Field::MixedEuriborSpread => (
                    "Mixed euribor spread (%)",
                    self.mixed_euribor_spread.clone(),
                ),
                Field::SameSpread => (
                    "Same spread",
                    if self.same_spread {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    },
                ),
                Field::PrepaymentDate => ("Prepayment date", self.prepayment_date.clone()),
                Field::PrepaymentAmount => ("Prepayment amount", self.prepayment_amount.clone()),
                Field::PrepaymentEffect => (
                    "Prepayment effect",
                    if self.prepayment_effect == 0 {
                        "ReduceTerm".to_string()
                    } else {
                        "ReducePayment".to_string()
                    },
                ),
                Field::AddPrepayment => {
                    let count = self.prepayments.len();
                    ("Add prepayment", format!("[Enter to add, {} saved]", count))
                }
                Field::UpfrontCost => ("Upfront cost", self.upfront_cost.clone()),
                Field::UpfrontPercent => ("Upfront %", self.upfront_percent.clone()),
            };

            let hint = if is_sel {
                match field {
                    Field::Currency
                    | Field::PaymentType
                    | Field::RateMode
                    | Field::EuriborTenor
                    | Field::MixedEuriborTenor
                    | Field::SameSpread
                    | Field::PrepaymentEffect => " [←→ toggle]",
                    Field::AddPrepayment => " [Enter=add, Del=remove last]",
                    Field::EuriborFetchButton => " [Enter=fetch]",
                    Field::EuriborDate => " [YYYY-MM-DD]",
                    Field::EuriborRate => " [type rate]",
                    Field::AddEuriborPoint => " [Enter=add, Del=remove last]",
                    Field::StartDate => " [YYYY-MM-DD]",
                    Field::UpfrontCost => " [fixed amount or 0]",
                    Field::UpfrontPercent => " [percent or 0]",
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
                Line::from(format!(
                    "Amount: {}{:.2} | Term: {} years | Payment: {:?} | Rate mode: {:?}",
                    sym, params.amount, params.term_years, params.payment_type, params.rate_mode
                )),
                Line::from(format!(
                    "Total principal: {}{:.2} | Total interest: {}{:.2} | Total paid: {}{:.2}",
                    sym, result.total_principal, sym, result.total_interest, sym, result.total_paid
                )),
            ];
            if let Some(mp) = result.monthly_payment {
                lines.push(Line::from(format!("Monthly payment: {}{:.2}", sym, mp)));
            }
            lines.push(Line::from(format!(
                "Payments count: {}",
                result.payments.len()
            )));
            if let Some(idx) = result.principal_exceeds_interest_at {
                lines.push(Line::from(format!(
                    "Principal > Interest at payment #{} ({})",
                    idx + 1,
                    result.payments[idx].date
                )));
            }
            Text::from(lines)
        } else {
            Text::from("No results yet.")
        };

        let summary = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("Summary — 'S':CSV 'Y':Yearly 'R':Sensitivity 'B':Break-even 'W':Save 'L':Load Esc:back"));
        frame.render_widget(summary, chunks[0]);

        if let Some(ref result) = self.result {
            if let Some(ref analysis) = self.show_analysis {
                match analysis {
                    AnalysisView::Sensitivity(points) => {
                        let header =
                            Row::new(vec!["Delta", "Rate %", "Monthly", "Interest", "Total Paid"])
                                .style(Style::default().fg(Color::Yellow));
                        let rows: Vec<Row> = points
                            .iter()
                            .skip(self.scroll_offset)
                            .take(chunks[1].height as usize - 2)
                            .map(|p| {
                                let monthly = p
                                    .monthly_payment
                                    .map(|m| format!("{:.2}", m))
                                    .unwrap_or_else(|| "N/A".to_string());
                                Row::new(vec![
                                    format!("{:+.2}", p.rate_delta),
                                    format!("{:.2}", p.effective_rate),
                                    monthly,
                                    format!("{:.2}", p.total_interest),
                                    format!("{:.2}", p.total_paid),
                                ])
                            })
                            .collect();

                        let table = Table::new(
                            rows,
                            [
                                Constraint::Length(10),
                                Constraint::Length(10),
                                Constraint::Length(14),
                                Constraint::Length(14),
                                Constraint::Length(14),
                            ],
                        )
                        .header(header)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Rate Sensitivity (↑↓ scroll)"),
                        );

                        frame.render_widget(table, chunks[1]);
                    }
                    AnalysisView::BreakEven(be) => {
                        let text = Text::from(vec![
                            Line::from(format!("Monthly rent:      {:.2}", be.monthly_rent)),
                            Line::from(format!("Monthly mortgage:  {:.2}", be.monthly_cost)),
                            Line::from(format!("Upfront costs:     {:.2}", be.upfront_costs)),
                            Line::from(format!("Total interest:    {:.2}", be.total_interest)),
                            Line::from(""),
                            if let (Some(months), Some(years)) =
                                (be.break_even_months, be.break_even_years)
                            {
                                Line::from(format!(
                                    "Break-even:        {} months ({:.1} years)",
                                    months, years
                                ))
                            } else {
                                Line::from("Break-even:        N/A")
                            },
                            Line::from(""),
                            Line::from(be.explanation.clone()),
                        ]);
                        let para = Paragraph::new(text).block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Break-Even vs Rent"),
                        );
                        frame.render_widget(para, chunks[1]);
                    }
                }
            } else if self.show_yearly {
                let summaries = result.yearly_summaries();
                let header = Row::new(vec![
                    "Year",
                    "Payment",
                    "Principal",
                    "Interest",
                    "Months",
                    "Balance",
                ])
                .style(Style::default().fg(Color::Yellow));
                let rows: Vec<Row> = summaries
                    .iter()
                    .skip(self.scroll_offset)
                    .take(chunks[1].height as usize - 2)
                    .map(|s| {
                        Row::new(vec![
                            s.year.to_string(),
                            format!("{:.2}", s.total_payment),
                            format!("{:.2}", s.total_principal),
                            format!("{:.2}", s.total_interest),
                            s.payments_count.to_string(),
                            format!("{:.2}", s.ending_balance),
                        ])
                    })
                    .collect();

                let table = Table::new(
                    rows,
                    [
                        Constraint::Length(8),
                        Constraint::Length(14),
                        Constraint::Length(14),
                        Constraint::Length(14),
                        Constraint::Length(8),
                        Constraint::Length(14),
                    ],
                )
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Yearly Summary (↑↓ scroll)"),
                );

                frame.render_widget(table, chunks[1]);
            } else {
                let header = Row::new(vec![
                    "#",
                    "Date",
                    "Payment",
                    "Principal",
                    "Interest",
                    "Balance",
                ])
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

                let table = Table::new(
                    rows,
                    [
                        Constraint::Length(5),
                        Constraint::Length(12),
                        Constraint::Length(12),
                        Constraint::Length(12),
                        Constraint::Length(12),
                        Constraint::Length(12),
                    ],
                )
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Payments (↑↓ scroll)"),
                );

                frame.render_widget(table, chunks[1]);

                let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None);
                let mut scrollbar_state =
                    ScrollbarState::new(result.payments.len()).position(self.scroll_offset);
                frame.render_stateful_widget(
                    scrollbar,
                    chunks[1].inner(Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                    &mut scrollbar_state,
                );
            }
        }
    }

    fn draw_help(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let header = Paragraph::new("Mortgage Calculator — Help")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        let help_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "=== Form Navigation ===",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Tab / Down", Style::default().fg(Color::Cyan)),
                Span::raw("          Next field"),
            ]),
            Line::from(vec![
                Span::styled("Shift+Tab / Up", Style::default().fg(Color::Cyan)),
                Span::raw("    Previous field"),
            ]),
            Line::from(vec![
                Span::styled("Left / Right", Style::default().fg(Color::Cyan)),
                Span::raw("        Toggle enum values (currency, payment type, etc.)"),
            ]),
            Line::from(vec![
                Span::styled("0-9 . -", Style::default().fg(Color::Cyan)),
                Span::raw("           Type numeric values"),
            ]),
            Line::from(vec![
                Span::styled("Backspace", Style::default().fg(Color::Cyan)),
                Span::raw("           Delete character"),
            ]),
            Line::from(vec![
                Span::styled("Enter", Style::default().fg(Color::Cyan)),
                Span::raw("               Calculate / Add prepayment / Fetch Euribor"),
            ]),
            Line::from(vec![
                Span::styled("Delete", Style::default().fg(Color::Cyan)),
                Span::raw("              Remove last Euribor point or prepayment"),
            ]),
            Line::from(vec![
                Span::styled("q / Esc", Style::default().fg(Color::Cyan)),
                Span::raw("             Quit"),
            ]),
            Line::from(vec![
                Span::styled("h", Style::default().fg(Color::Cyan)),
                Span::raw("                 Show this help page"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "=== Results Screen ===",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("q / Esc", Style::default().fg(Color::Cyan)),
                Span::raw("             Return to form / close analysis"),
            ]),
            Line::from(vec![
                Span::styled("S", Style::default().fg(Color::Cyan)),
                Span::raw("                 Export CSV to /tmp/mortgage_tui_export.csv"),
            ]),
            Line::from(vec![
                Span::styled("Y", Style::default().fg(Color::Cyan)),
                Span::raw("                 Toggle yearly summary view"),
            ]),
            Line::from(vec![
                Span::styled("R", Style::default().fg(Color::Cyan)),
                Span::raw("                 Show rate sensitivity analysis"),
            ]),
            Line::from(vec![
                Span::styled("B", Style::default().fg(Color::Cyan)),
                Span::raw("                 Show break-even vs rent analysis"),
            ]),
            Line::from(vec![
                Span::styled("W", Style::default().fg(Color::Cyan)),
                Span::raw("                 Save session to /tmp/mortgage_session.json"),
            ]),
            Line::from(vec![
                Span::styled("L", Style::default().fg(Color::Cyan)),
                Span::raw("                 Load session from /tmp/mortgage_session.json"),
            ]),
            Line::from(vec![
                Span::styled("Up / Down", Style::default().fg(Color::Cyan)),
                Span::raw("           Scroll table"),
            ]),
            Line::from(vec![
                Span::styled("PgUp / PgDn", Style::default().fg(Color::Cyan)),
                Span::raw("         Page scroll"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "=== Help Screen ===",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("q / Esc / h", Style::default().fg(Color::Cyan)),
                Span::raw("        Return to previous screen"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key to return...",
                Style::default().fg(Color::Gray),
            )),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Keyboard Shortcuts"),
            )
            .alignment(Alignment::Left);
        frame.render_widget(help, chunks[1]);
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

    pub fn export_csv(&mut self, path: &str) {
        if let Some(ref result) = self.result {
            let csv = payments_to_csv(&result.payments);
            if let Err(e) = fs::write(path, csv) {
                self.popup_msg = Some(format!("CSV export failed: {}", e));
            } else {
                self.popup_msg = Some(format!("Saved CSV to {}", path));
            }
            self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
