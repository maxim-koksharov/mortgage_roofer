use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use mortgage_core::models::*;
use mortgage_core::Calculator;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table},
    DefaultTerminal, Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Screen {
    Form,
    Results,
}

#[derive(Debug)]
struct App {
    screen: Screen,
    // Form fields
    amount: String,
    term: String,
    rate: String,
    spread: String,
    selected_field: usize,
    // Results
    result: Option<LoanResult>,
    scroll_offset: usize,
    params: Option<LoanParams>,
}

impl App {
    fn new() -> Self {
        Self {
            screen: Screen::Form,
            amount: "185000".to_string(),
            term: "30".to_string(),
            rate: "3.6".to_string(),
            spread: "0.0".to_string(),
            selected_field: 0,
            result: None,
            scroll_offset: 0,
            params: None,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while self.screen != Screen::Results || self.result.is_some() {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.screen {
                        Screen::Form => self.handle_form_input(key.code),
                        Screen::Results => self.handle_results_input(key.code),
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_form_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Tab | KeyCode::Down => {
                self.selected_field = (self.selected_field + 1) % 4;
            }
            KeyCode::Up => {
                self.selected_field = (self.selected_field + 3) % 4;
            }
            KeyCode::Char(c) if c.is_numeric() || c == '.' => {
                self.edit_field(c);
            }
            KeyCode::Backspace => {
                self.backspace_field();
            }
            KeyCode::Enter => {
                self.calculate();
                self.screen = Screen::Results;
            }
            KeyCode::Esc => {
                if self.result.is_some() {
                    self.screen = Screen::Results;
                }
            }
            _ => {}
        }
    }

    fn handle_results_input(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::Form;
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

    fn edit_field(&mut self, c: char) {
        let target = match self.selected_field {
            0 => &mut self.amount,
            1 => &mut self.term,
            2 => &mut self.rate,
            3 => &mut self.spread,
            _ => return,
        };
        target.push(c);
    }

    fn backspace_field(&mut self) {
        let target = match self.selected_field {
            0 => &mut self.amount,
            1 => &mut self.term,
            2 => &mut self.rate,
            3 => &mut self.spread,
            _ => return,
        };
        target.pop();
    }

    fn calculate(&mut self) {
        let amount = self.amount.parse::<f64>().unwrap_or(100_000.0);
        let term_years = self.term.parse::<u32>().unwrap_or(10);
        let rate = self.rate.parse::<f64>().unwrap_or(5.0);
        let spread = self.spread.parse::<f64>().unwrap_or(0.0);

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

        let result = Calculator::calculate(&params);
        self.params = Some(params);
        self.result = Some(result);
        self.scroll_offset = 0;
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.screen {
            Screen::Form => self.draw_form(frame),
            Screen::Results => self.draw_results(frame),
        }
    }

    fn draw_form(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let header = Paragraph::new("Mortgage Calculator - Press Enter to calculate, Esc to quit")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(header, layout[0]);

        let fields = ["Amount:", "Term (years):", "Rate (%):", "Spread (%):"];
        let values = [&self.amount, &self.term, &self.rate, &self.spread];

        let mut text = Text::default();
        for (i, (label, value)) in fields.iter().zip(values.iter()).enumerate() {
            let style = if i == self.selected_field {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            let line = Line::from(vec![
                Span::styled(format!("{} ", label), style),
                Span::styled(value.to_string(), style.add_modifier(ratatui::style::Modifier::BOLD)),
            ]);
            text.push_line(line);
            text.push_line("");
        }

        let form = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Parameters"));
        frame.render_widget(form, layout[1]);
    }

    fn draw_results(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(0)])
            .split(area);

        // Summary block
        let summary_text = if let (Some(params), Some(result)) = (&self.params, &self.result) {
            let sym = params.currency.symbol();
            let mut lines = vec![
                Line::from(format!("Amount: {}{:.2} | Term: {} years | Rate mode: {:?}", sym, params.amount, params.term_years, params.rate_mode)),
                Line::from(format!("Total principal: {}{:.2} | Total interest: {}{:.2} | Total paid: {}{:.2}", sym, result.total_principal, sym, result.total_interest, sym, result.total_paid)),
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
            Text::from("No results yet. Press Enter in the form to calculate.")
        };

        let summary = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title("Summary"));
        frame.render_widget(summary, chunks[0]);

        // Table
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
            .block(Block::default().borders(Borders::ALL).title("Payments (Esc to go back, ↑↓ to scroll)"));

            frame.render_widget(table, chunks[1]);

            // Scrollbar
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
}

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
