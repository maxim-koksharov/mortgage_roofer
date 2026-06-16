mod app;
mod ui;

use app::{App, Field, Screen, Tab, days_in_month};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                if key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.active_tab = match self.active_tab {
                        Tab::Calculator => Tab::ReverseCalculator,
                        Tab::ReverseCalculator => Tab::Calculator,
                    };
                    self.scroll_offset = 0;
                    continue;
                }

                match self.screen.clone() {
                    Screen::Form => match self.active_tab {
                        Tab::Calculator => self.handle_form(key.code),
                        Tab::ReverseCalculator => self.handle_reverse_form(key.code),
                    },
                    Screen::Results => match self.active_tab {
                        Tab::Calculator => self.handle_results(key.code),
                        Tab::ReverseCalculator => self.handle_reverse_results(key.code),
                    },
                    Screen::Help => self.handle_help(key.code),
                    Screen::Calendar { field, .. } => self.handle_calendar(key.code, field),
                    Screen::Popup(_) => {
                        self.popup_msg = None;
                        if self.result.is_some() || self.reverse_result.is_some() {
                            self.screen = Screen::Results;
                        } else {
                            self.screen = Screen::Form;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_form(&mut self, code: KeyCode) {
        match code {
            KeyCode::Tab | KeyCode::Down => {
                self.selected = (self.selected + 1) % self.fields.len();
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.selected = (self.selected + self.fields.len() - 1) % self.fields.len();
            }
            KeyCode::Char(c) if c.is_numeric() || c == '.' || c == '-' => {
                self.edit_date(c);
            }
            KeyCode::Backspace => {
                self.backspace_date();
            }
            KeyCode::Left => self.cycle_enum(-1),
            KeyCode::Right => self.cycle_enum(1),
            KeyCode::Enter => {
                if let Err(e) = self.handle_enter() {
                    self.popup_msg = Some(format!("Error: {}", e));
                    self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
                }
            }
            KeyCode::Delete => {
                if !self.euribor_curve.is_empty() {
                    self.euribor_curve.pop();
                } else if !self.prepayments.is_empty() {
                    self.prepayments.pop();
                }
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.screen = Screen::Help;
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_exit = true;
            }
            _ => {}
        }
    }

    fn handle_enter(&mut self) -> Result<(), String> {
        let current = self.fields[self.selected];
        match current {
            Field::EuriborFetchButton => self.fetch_euribor(),
            Field::AddEuriborPoint => self.add_euribor_point(),
            Field::AddPrepayment => self.add_prepayment(),
            Field::StartDate | Field::PrepaymentDate | Field::EuriborDate => {
                let date_str = match current {
                    Field::StartDate => &self.start_date,
                    Field::PrepaymentDate => &self.prepayment_date,
                    Field::EuriborDate => &self.euribor_date,
                    _ => unreachable!(),
                };
                let parsed = chrono::NaiveDate::parse_from_str(date_str, "%d-%m-%Y")
                    .unwrap_or_else(|_| chrono::Local::now().date_naive());
                let cal_state = app::CalendarState::new(parsed);
                self.screen = Screen::Calendar {
                    field: current,
                    state: cal_state,
                };
                Ok(())
            }
            _ => {
                self.calculate()?;
                self.screen = Screen::Results;
                Ok(())
            }
        }
    }

    fn handle_results(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
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
            KeyCode::Down | KeyCode::PageDown | KeyCode::Right => {
                if let Some(ref result) = self.result {
                    let max = result.payments.len().saturating_sub(1);
                    if self.scroll_offset < max {
                        self.scroll_offset += 1;
                    }
                }
            }
            KeyCode::Up | KeyCode::PageUp | KeyCode::Left if self.scroll_offset > 0 => {
                self.scroll_offset -= 1;
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.screen = Screen::Help;
            }
            _ => {}
        }
    }

    fn handle_help(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc
            | KeyCode::Char('q')
            | KeyCode::Char('Q')
            | KeyCode::Char('h')
            | KeyCode::Char('H') => {
                let has_results = match self.active_tab {
                    Tab::Calculator => self.result.is_some(),
                    Tab::ReverseCalculator => self.reverse_result.is_some(),
                };
                if has_results {
                    self.screen = Screen::Results;
                } else {
                    self.screen = Screen::Form;
                }
            }
            _ => {}
        }
    }

    fn handle_reverse_form(&mut self, code: KeyCode) {
        match code {
            KeyCode::Tab | KeyCode::Down => {
                self.reverse_selected = (self.reverse_selected + 1) % self.reverse_fields.len();
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.reverse_selected = (self.reverse_selected + self.reverse_fields.len() - 1)
                    % self.reverse_fields.len();
            }
            KeyCode::Char(c) if c.is_numeric() || c == '.' || c == '-' => {
                self.reverse_edit_text(c);
            }
            KeyCode::Backspace => {
                self.reverse_backspace();
            }
            KeyCode::Left => self.reverse_cycle_enum(-1),
            KeyCode::Right => self.reverse_cycle_enum(1),
            KeyCode::Enter => {
                if let Err(e) = self.handle_reverse_enter() {
                    self.popup_msg = Some(format!("Error: {}", e));
                    self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
                }
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_exit = true;
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.screen = Screen::Help;
            }
            _ => {}
        }
    }

    fn handle_reverse_enter(&mut self) -> Result<(), String> {
        let current = self.reverse_fields[self.reverse_selected];
        match current {
            crate::app::ReverseField::EuriborFetchButton => {
                let tenor = app::tenor_from_idx(self.reverse_euribor_tenor);
                match self.euribor_cache.get_or_fetch(tenor) {
                    Ok(rate) => {
                        self.popup_msg = Some(format!("Fetched Euribor {}: {:.3}%", tenor, rate));
                        self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
                        Ok(())
                    }
                    Err(e) => Err(format!("Euribor fetch failed: {}", e)),
                }
            }
            _ => {
                self.reverse_calculate()?;
                self.screen = Screen::Results;
                self.scroll_offset = 0;
                Ok(())
            }
        }
    }

    fn handle_reverse_results(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.screen = Screen::Form;
            }
            KeyCode::Down | KeyCode::PageDown | KeyCode::Right => {
                if let Some(ref rows) = self.reverse_result {
                    let max = rows.len().saturating_sub(1);
                    if self.scroll_offset < max {
                        self.scroll_offset += 1;
                    }
                }
            }
            KeyCode::Up | KeyCode::PageUp | KeyCode::Left if self.scroll_offset > 0 => {
                self.scroll_offset -= 1;
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.screen = Screen::Help;
            }
            _ => {}
        }
    }

    fn handle_calendar(&mut self, code: KeyCode, target_field: Field) {
        if let Screen::Calendar { field: _, state } = &mut self.screen {
            match code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.screen = Screen::Form;
                }
                KeyCode::Enter => {
                    if let Some(day) = state.selected_day {
                        let date_str = format!("{:02}-{:02}-{}", day, state.month, state.year);
                        match target_field {
                            Field::StartDate => self.start_date = date_str,
                            Field::PrepaymentDate => self.prepayment_date = date_str,
                            Field::EuriborDate => self.euribor_date = date_str,
                            _ => {}
                        }
                    }
                    self.screen = Screen::Form;
                }
                KeyCode::Up => {
                    if let Some(ref mut day) = state.selected_day
                        && *day > 7
                    {
                        *day -= 7;
                    }
                }
                KeyCode::Down => {
                    let days_in_month = days_in_month(state.year, state.month);
                    if let Some(ref mut day) = state.selected_day
                        && *day + 7 <= days_in_month
                    {
                        *day += 7;
                    }
                }
                KeyCode::Left => {
                    if let Some(ref mut day) = state.selected_day
                        && *day > 1
                    {
                        *day -= 1;
                    }
                }
                KeyCode::Right => {
                    let days_in_month = days_in_month(state.year, state.month);
                    if let Some(ref mut day) = state.selected_day
                        && *day < days_in_month
                    {
                        *day += 1;
                    }
                }
                KeyCode::PageUp => state.prev_month(),
                KeyCode::PageDown => state.next_month(),
                _ => {}
            }
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
