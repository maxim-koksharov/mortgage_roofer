use crate::app::{App, Field, Screen, tenor_name};
use mortgage_core::payments_to_csv;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table},
    Frame,
};
use std::fs;

impl App {
    pub fn draw(&mut self, frame: &mut Frame) {
        match self.screen.clone() {
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
                Field::AddPrepayment => {
                    let count = self.prepayments.len();
                    ("Add prepayment", format!("[Enter to add, {} saved]", count))
                }
            };

            let hint = if is_sel {
                match field {
                    Field::Currency | Field::PaymentType | Field::RateMode | Field::EuriborTenor
                    | Field::MixedEuriborTenor | Field::SameSpread | Field::PrepaymentEffect => {
                        " [←→ toggle]"
                    }
                    Field::AddPrepayment => " [Enter=add, Del=remove last]",
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
        .constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)])
        .split(popup_layout[1])[1]
}
