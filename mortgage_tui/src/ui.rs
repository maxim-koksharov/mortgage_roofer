use crate::app::{AnalysisView, App, CalendarState, Field, Screen, days_in_month, tenor_name};
use chrono::NaiveDate;
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

use crate::app::{ReverseField, Tab};

impl App {
    pub fn draw(&mut self, frame: &mut Frame) {
        match self.screen.clone() {
            Screen::Form => match self.active_tab {
                Tab::Calculator => self.draw_form(frame),
                Tab::ReverseCalculator => self.draw_reverse_form(frame),
            },
            Screen::Results => match self.active_tab {
                Tab::Calculator => self.draw_results(frame),
                Tab::ReverseCalculator => self.draw_reverse_results(frame),
            },
            Screen::Help => self.draw_help(frame),
            Screen::Calendar { state, .. } => {
                match self.active_tab {
                    Tab::Calculator => self.draw_form(frame),
                    Tab::ReverseCalculator => self.draw_reverse_form(frame),
                }
                self.draw_calendar(frame, &state);
            }
            Screen::Popup(_) => {
                match self.active_tab {
                    Tab::Calculator => self.draw_results(frame),
                    Tab::ReverseCalculator => self.draw_reverse_results(frame),
                }
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

        let tab_label = self.active_tab.label();
        let header = Paragraph::new(format!(
            "[{}] Mortgage Calculator — Ctrl+Tab:switch  Tab:next  Enter:calc  ←→:toggle  Backspace:delete",
            tab_label
        ))
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
                Field::DownPayment => ("Down payment", self.down_payment.clone()),
                Field::ExtraMonthlyCost => ("Extra monthly cost", self.extra_monthly_cost.clone()),
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
                    Field::EuriborDate => " [DD-MM-YYYY]",
                    Field::EuriborRate => " [type rate]",
                    Field::AddEuriborPoint => " [Enter=add, Del=remove last]",
                    Field::StartDate => " [DD-MM-YYYY]",
                    Field::UpfrontCost => " [fixed amount or 0]",
                    Field::UpfrontPercent => " [percent or 0]",
                    Field::DownPayment => " [amount or 0]",
                    Field::ExtraMonthlyCost => " [insurance etc.]",
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
                let extra = self.extra_monthly_cost.parse::<f64>().unwrap_or(0.0);
                let header = Row::new(vec![
                    "#",
                    "Date",
                    "Payment",
                    "Principal",
                    "Interest",
                    "Balance",
                    "TOTAL",
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
                            format!("{:.2}", p.payment + extra),
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
                Span::styled("Ctrl+Tab", Style::default().fg(Color::Cyan)),
                Span::raw("            Switch tab (Calculator / Max Loan)"),
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
                "=== Max Loan Tab ===",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Enter target monthly payment and rate;"),
            Line::from("press Enter to see max affordable amounts for 5-34 yr terms."),
            Line::from(""),
            Line::from(vec![
                Span::styled("Ctrl+Tab", Style::default().fg(Color::Cyan)),
                Span::raw("            Switch to main Calculator tab"),
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

    fn draw_calendar(&self, frame: &mut Frame, state: &CalendarState) {
        use chrono::Datelike;
        let area = frame.area();
        let popup_area = centered_rect(45, 55, area);
        frame.render_widget(Clear, popup_area);

        let month_names = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];
        let title = format!("{} {}", month_names[(state.month - 1) as usize], state.year);
        let hint =
            "Enter:select Esc:cancel \u{2190}\u{2191}\u{2193}\u{2192}:navigate PgUp/PgDn:month";

        let first_of_month = NaiveDate::from_ymd_opt(state.year, state.month, 1).unwrap();
        let weekday_idx = first_of_month.weekday().num_days_from_monday();
        let days_in_month = days_in_month(state.year, state.month);

        let mut lines: Vec<Line> = vec![
            Line::from(Span::styled(
                &title,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(hint, Style::default().fg(Color::Gray))),
            Line::from(""),
            Line::from(vec![
                Span::styled("Mo ", Style::default().fg(Color::Cyan)),
                Span::styled("Tu ", Style::default().fg(Color::Cyan)),
                Span::styled("We ", Style::default().fg(Color::Cyan)),
                Span::styled("Th ", Style::default().fg(Color::Cyan)),
                Span::styled("Fr ", Style::default().fg(Color::Cyan)),
                Span::styled("Sa ", Style::default().fg(Color::Cyan)),
                Span::styled("Su ", Style::default().fg(Color::Cyan)),
            ]),
        ];

        let mut week_spans: Vec<Span> = Vec::new();
        let mut day_count = 0;

        for _ in 0..weekday_idx {
            week_spans.push(Span::raw("   "));
            day_count += 1;
        }

        for day in 1..=days_in_month {
            let is_selected = state.selected_day == Some(day);
            let style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            week_spans.push(Span::styled(format!("{:>2} ", day), style));
            day_count += 1;

            if day_count % 7 == 0 {
                lines.push(Line::from(week_spans.clone()));
                week_spans.clear();
            }
        }

        if !week_spans.is_empty() {
            while day_count % 7 != 0 {
                week_spans.push(Span::raw("   "));
                day_count += 1;
            }
            lines.push(Line::from(week_spans));
        }

        let calendar = Paragraph::new(Text::from(lines))
            .block(Block::default().borders(Borders::ALL).title("Calendar"))
            .alignment(Alignment::Left);
        frame.render_widget(calendar, popup_area);
    }

    fn draw_reverse_form(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let tab_label = self.active_tab.label();
        let header = Paragraph::new(format!(
            "[{}] Max Loan Calculator — Ctrl+Tab:switch  Tab:next  Enter:calc  ←→:toggle  Backspace:delete",
            tab_label
        ))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        let mut lines = vec![];
        for (i, field) in self.reverse_fields.iter().enumerate() {
            let is_sel = i == self.reverse_selected;
            let style = if is_sel {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let (label, value) = match field {
                ReverseField::TargetPayment => {
                    ("Monthly payment", self.reverse_target_payment.clone())
                }
                ReverseField::PaymentType => (
                    "Payment type",
                    if self.reverse_payment_type == 0 {
                        "Annuity".to_string()
                    } else {
                        "Diff".to_string()
                    },
                ),
                ReverseField::RateMode => (
                    "Rate mode",
                    if self.reverse_rate_mode == 0 {
                        "Fix".to_string()
                    } else {
                        "Euribor".to_string()
                    },
                ),
                ReverseField::FixRate => ("Fix rate (%)", self.reverse_fix_rate.clone()),
                ReverseField::FixSpread => ("Fix spread (%)", self.reverse_fix_spread.clone()),
                ReverseField::EuriborTenor => (
                    "Euribor tenor",
                    tenor_name(self.reverse_euribor_tenor).to_string(),
                ),
                ReverseField::EuriborSpread => {
                    ("Euribor spread (%)", self.reverse_euribor_spread.clone())
                }
                ReverseField::EuriborFetchButton => ("Euribor fetch", String::new()),
                ReverseField::ExtraMonthlyCost => {
                    ("Extra monthly cost", self.reverse_extra_monthly.clone())
                }
            };

            let hint = if is_sel {
                match field {
                    ReverseField::PaymentType
                    | ReverseField::RateMode
                    | ReverseField::EuriborTenor => " [←→ toggle]",
                    ReverseField::EuriborFetchButton => " [Enter to fetch current rate]",
                    ReverseField::ExtraMonthlyCost => " [insurance etc.]",
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

        let form = Paragraph::new(Text::from(lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Max Loan Parameters"),
        );
        frame.render_widget(form, chunks[1]);
    }

    fn draw_reverse_results(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(area);

        let summary_text = if let Some(ref rows) = self.reverse_result {
            let sym = "€";
            let target = rows.first().map_or(0.0, |r| r.monthly_payment);
            let extra = rows.first().map_or(0.0, |r| r.extra_cost);
            Text::from(vec![
                Line::from(format!(
                    "Monthly payment: {}{:.2}  |  Extra costs: {}{:.2}  |  Rate mode: {}",
                    sym,
                    target,
                    sym,
                    extra,
                    if self.reverse_rate_mode == 0 {
                        "Fix"
                    } else {
                        "Euribor"
                    },
                )),
                Line::from(format!(
                    "Terms: 5–34 years  |  {} rows  |  Ctrl+Tab:switch  Esc:back",
                    rows.len()
                )),
            ])
        } else {
            Text::from("No results yet. Press Enter to calculate.")
        };

        let summary = Paragraph::new(summary_text).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Max Loan — Summary"),
        );
        frame.render_widget(summary, chunks[0]);

        if let Some(ref rows) = self.reverse_result {
            let header = Row::new(vec!["Term (yr)", "Max Amount", "Payment", "Extra", "TOTAL"])
                .style(Style::default().fg(Color::Yellow));
            let table_rows: Vec<Row> = rows
                .iter()
                .skip(self.scroll_offset)
                .take(chunks[1].height as usize - 2)
                .map(|r| {
                    Row::new(vec![
                        r.term_years.to_string(),
                        format!("{:.2}", r.max_amount),
                        format!("{:.2}", r.monthly_payment),
                        format!("{:.2}", r.extra_cost),
                        format!("{:.2}", r.total_monthly),
                    ])
                })
                .collect();

            let table = Table::new(
                table_rows,
                [
                    Constraint::Length(12),
                    Constraint::Length(14),
                    Constraint::Length(12),
                    Constraint::Length(11),
                    Constraint::Length(12),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Max Affordable Loan Amounts (↑↓ scroll)"),
            );

            frame.render_widget(table, chunks[1]);

            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            let mut scrollbar_state = ScrollbarState::new(rows.len()).position(self.scroll_offset);
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
