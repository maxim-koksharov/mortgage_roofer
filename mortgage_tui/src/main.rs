mod app;
mod ui;

use app::{App, Screen};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.screen {
                        Screen::Form => self.handle_form(key.code),
                        Screen::Results => self.handle_results(key.code),
                        Screen::Popup(_) => {
                            self.popup_msg = None;
                            self.screen = Screen::Form;
                        }
                    }
                }
            }

            if self.screen == Screen::Form && self.result.is_some() && self.popup_msg.is_none() {
                // Continue loop
            }
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
                if let Err(e) = self.calculate() {
                    self.popup_msg = Some(format!("Error: {}", e));
                    self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
                } else {
                    self.screen = Screen::Results;
                }
            }
            KeyCode::Esc => {
                if self.result.is_some() {
                    return;
                }
            }
            KeyCode::Char('q') => {
                if self.result.is_some() {
                    return;
                }
            }
            _ => {}
        }
    }

    fn handle_results(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::Form;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.export_csv("/tmp/mortgage_tui_export.csv");
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
}

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::new();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
