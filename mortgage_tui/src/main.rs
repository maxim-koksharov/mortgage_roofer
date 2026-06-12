mod app;
mod ui;

use app::{App, Field, Screen};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                match self.screen {
                    Screen::Form => self.handle_form(key.code),
                    Screen::Results => self.handle_results(key.code),
                    Screen::Popup(_) => {
                        self.popup_msg = None;
                        self.screen = Screen::Form;
                    }
                }
            }

            if self.screen == Screen::Form && self.result.is_some() && self.popup_msg.is_none() {}
        }
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
                if self.fields[self.selected] == Field::AddPrepayment {
                    if let Err(e) = self.add_prepayment() {
                        self.popup_msg = Some(format!("Error: {}", e));
                        self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
                    }
                } else if let Err(e) = self.calculate() {
                    self.popup_msg = Some(format!("Error: {}", e));
                    self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
                } else {
                    self.screen = Screen::Results;
                }
            }
            KeyCode::Delete => {
                if !self.prepayments.is_empty() {
                    self.prepayments.pop();
                }
            }
            KeyCode::Esc | KeyCode::Char('q') if self.result.is_some() => {}
            _ => {}
        }
    }

    fn handle_results(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                if self.show_analysis.is_some() {
                    self.show_analysis = None;
                } else {
                    self.screen = Screen::Form;
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.export_csv("/tmp/mortgage_tui_export.csv");
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.show_yearly = !self.show_yearly;
                self.show_analysis = None;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                if let Some(ref params) = self.params {
                    let deltas = vec![-2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0];
                    let points = mortgage_core::sensitivity_analysis(params, &deltas);
                    self.show_analysis = Some(app::AnalysisView::Sensitivity(points));
                    self.show_yearly = false;
                }
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                if let Some(ref params) = self.params {
                    let rent = params.amount * 0.005;
                    let be = mortgage_core::break_even_analysis(params, rent);
                    self.show_analysis = Some(app::AnalysisView::BreakEven(be));
                    self.show_yearly = false;
                }
            }
            KeyCode::Char('w') | KeyCode::Char('W') => {
                self.save_session("/tmp/mortgage_session.json");
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.load_session("/tmp/mortgage_session.json");
            }
            KeyCode::Down | KeyCode::PageDown => {
                if let Some(ref result) = self.result {
                    let max = result.payments.len().saturating_sub(1);
                    if self.scroll_offset < max {
                        self.scroll_offset += 1;
                    }
                }
            }
            KeyCode::Up | KeyCode::PageUp if self.scroll_offset > 0 => {
                self.scroll_offset -= 1;
            }
            _ => {}
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
