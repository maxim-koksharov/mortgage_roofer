use chrono::NaiveDate;
use mortgage_core::Calculator;
use mortgage_core::euribor::EuriborCache;
use mortgage_core::models::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Form,
    Results,
    Help,
    Popup(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    Amount,
    Term,
    StartDate,
    Currency,
    PaymentType,
    RateMode,
    FixRate,
    FixSpread,
    EuriborTenor,
    EuriborSpread,
    EuriborFetchButton,
    EuriborDate,
    EuriborRate,
    AddEuriborPoint,
    MixedFixYears,
    MixedFixRate,
    MixedFixSpread,
    MixedEuriborTenor,
    MixedEuriborSpread,
    SameSpread,
    PrepaymentDate,
    PrepaymentAmount,
    PrepaymentEffect,
    AddPrepayment,
    UpfrontCost,
    UpfrontPercent,
}

pub struct App {
    pub screen: Screen,
    pub fields: Vec<Field>,
    pub selected: usize,
    pub should_exit: bool,

    pub amount: String,
    pub term: String,
    pub start_date: String,
    pub currency: usize,
    pub payment_type: usize,
    pub rate_mode: usize,
    pub fix_rate: String,
    pub fix_spread: String,
    pub euribor_tenor: usize,
    pub euribor_spread: String,
    pub mixed_fix_years: String,
    pub mixed_fix_rate: String,
    pub mixed_fix_spread: String,
    pub mixed_euribor_tenor: usize,
    pub mixed_euribor_spread: String,
    pub same_spread: bool,
    pub prepayment_date: String,
    pub prepayment_amount: String,
    pub prepayment_effect: usize,
    pub prepayments: Vec<Prepayment>,
    pub upfront_cost: String,
    pub upfront_percent: String,

    pub euribor_cache: EuriborCache,
    pub euribor_curve: Vec<EuriborPoint>,
    pub euribor_date: String,
    pub euribor_rate: String,

    pub result: Option<LoanResult>,
    pub params: Option<LoanParams>,
    pub scroll_offset: usize,
    pub popup_msg: Option<String>,
    pub show_yearly: bool,
    pub show_analysis: Option<AnalysisView>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisView {
    Sensitivity(Vec<mortgage_core::SensitivityPoint>),
    BreakEven(mortgage_core::BreakEvenResult),
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            screen: Screen::Form,
            fields: vec![],
            selected: 0,
            should_exit: false,
            amount: "185000".to_string(),
            term: "30".to_string(),
            start_date: chrono::Local::now()
                .date_naive()
                .format("%Y-%m-%d")
                .to_string(),
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
            prepayments: vec![],
            upfront_cost: "0".to_string(),
            upfront_percent: "5".to_string(),
            euribor_cache: EuriborCache::default(),
            euribor_curve: vec![],
            euribor_date: chrono::Local::now()
                .date_naive()
                .format("%Y-%m-%d")
                .to_string(),
            euribor_rate: "3.0".to_string(),
            result: None,
            params: None,
            scroll_offset: 0,
            popup_msg: None,
            show_yearly: false,
            show_analysis: None,
        };
        app.rebuild_fields();
        app
    }

    pub fn rebuild_fields(&mut self) {
        let mut f = vec![
            Field::Amount,
            Field::Term,
            Field::StartDate,
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
                f.push(Field::EuriborFetchButton);
                f.push(Field::EuriborDate);
                f.push(Field::EuriborRate);
                f.push(Field::AddEuriborPoint);
            }
            2 => {
                f.push(Field::MixedFixYears);
                f.push(Field::MixedFixRate);
                f.push(Field::MixedFixSpread);
                f.push(Field::MixedEuriborTenor);
                f.push(Field::MixedEuriborSpread);
                f.push(Field::SameSpread);
                f.push(Field::EuriborFetchButton);
                f.push(Field::EuriborDate);
                f.push(Field::EuriborRate);
                f.push(Field::AddEuriborPoint);
            }
            _ => {}
        }
        f.extend_from_slice(&[
            Field::PrepaymentDate,
            Field::PrepaymentAmount,
            Field::PrepaymentEffect,
            Field::AddPrepayment,
            Field::UpfrontCost,
            Field::UpfrontPercent,
        ]);
        self.fields = f;
        if self.selected >= self.fields.len() {
            self.selected = self.fields.len().saturating_sub(1);
        }
    }

    fn field_mut(&mut self, field: Field) -> Option<&mut String> {
        match field {
            Field::Amount => Some(&mut self.amount),
            Field::Term => Some(&mut self.term),
            Field::StartDate => Some(&mut self.start_date),
            Field::FixRate => Some(&mut self.fix_rate),
            Field::FixSpread => Some(&mut self.fix_spread),
            Field::EuriborSpread => Some(&mut self.euribor_spread),
            Field::EuriborDate => Some(&mut self.euribor_date),
            Field::EuriborRate => Some(&mut self.euribor_rate),
            Field::MixedFixYears => Some(&mut self.mixed_fix_years),
            Field::MixedFixRate => Some(&mut self.mixed_fix_rate),
            Field::MixedFixSpread => Some(&mut self.mixed_fix_spread),
            Field::MixedEuriborSpread => Some(&mut self.mixed_euribor_spread),
            Field::PrepaymentDate => Some(&mut self.prepayment_date),
            Field::PrepaymentAmount => Some(&mut self.prepayment_amount),
            Field::UpfrontCost => Some(&mut self.upfront_cost),
            Field::UpfrontPercent => Some(&mut self.upfront_percent),
            _ => None,
        }
    }

    pub fn edit_text(&mut self, c: char) {
        if let Some(field) = self.field_mut(self.fields[self.selected]) {
            field.push(c);
        }
    }

    pub fn backspace(&mut self) {
        if let Some(field) = self.field_mut(self.fields[self.selected]) {
            field.pop();
        }
    }

    pub fn cycle_enum(&mut self, delta: i8) {
        let current = self.fields[self.selected];
        let len = match current {
            Field::Currency | Field::PaymentType => 2,
            Field::RateMode => 3,
            Field::EuriborTenor | Field::MixedEuriborTenor => 4,
            Field::PrepaymentEffect => 2,
            Field::SameSpread => {
                self.same_spread = !self.same_spread;
                return;
            }
            _ => return,
        };

        let value = match current {
            Field::Currency => &mut self.currency,
            Field::PaymentType => &mut self.payment_type,
            Field::RateMode => &mut self.rate_mode,
            Field::EuriborTenor => &mut self.euribor_tenor,
            Field::MixedEuriborTenor => &mut self.mixed_euribor_tenor,
            Field::PrepaymentEffect => &mut self.prepayment_effect,
            _ => return,
        };

        let old = *value;
        *value = ((*value as i8 + delta).rem_euclid(len)) as usize;
        if current == Field::RateMode && old != *value {
            self.rebuild_fields();
        }
    }

    fn euribor_tenor(&self) -> EuriborTenor {
        tenor_from_idx(if self.rate_mode == 2 {
            self.mixed_euribor_tenor
        } else {
            self.euribor_tenor
        })
    }

    fn euribor_start_date(&self) -> NaiveDate {
        let start_date = NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        if self.rate_mode == 2 {
            let fix_years = self.mixed_fix_years.parse::<f64>().unwrap_or(2.0);
            start_date + chrono::Duration::days((fix_years * 365.25) as i64)
        } else {
            start_date
        }
    }

    pub fn fetch_euribor(&mut self) -> Result<(), String> {
        let tenor = self.euribor_tenor();
        match self.euribor_cache.get_or_fetch(tenor) {
            Ok(rate) => {
                self.euribor_curve.push(EuriborPoint {
                    date_from: self.euribor_start_date(),
                    rate,
                });
                self.euribor_curve.sort_by_key(|p| p.date_from);
                self.popup_msg = Some(format!("Fetched Euribor {}: {:.3}%", tenor, rate));
                Ok(())
            }
            Err(e) => Err(format!("Euribor fetch failed: {}", e)),
        }
    }

    pub fn add_euribor_point(&mut self) -> Result<(), String> {
        let date = NaiveDate::parse_from_str(&self.euribor_date, "%Y-%m-%d")
            .map_err(|_| "Invalid date (YYYY-MM-DD)")?;
        let rate = self
            .euribor_rate
            .parse::<f64>()
            .map_err(|_| "Invalid rate")?;
        if !(0.0..=100.0).contains(&rate) {
            return Err("Rate must be 0-100%".to_string());
        }
        self.euribor_curve.push(EuriborPoint {
            date_from: date,
            rate,
        });
        self.euribor_curve.sort_by_key(|p| p.date_from);
        self.euribor_rate = "3.0".to_string();
        Ok(())
    }

    pub fn calculate(&mut self) -> Result<(), String> {
        let amount = self.amount.parse::<f64>().map_err(|_| "Invalid amount")?;
        let term_years = self.term.parse::<u32>().map_err(|_| "Invalid term")?;
        let currency = if self.currency == 0 {
            Currency::Eur
        } else {
            Currency::Usd
        };
        let payment_type = if self.payment_type == 0 {
            PaymentType::Annuity
        } else {
            PaymentType::Diff
        };
        let start_date = chrono::NaiveDate::parse_from_str(&self.start_date, "%Y-%m-%d")
            .map_err(|_| "Invalid start date (YYYY-MM-DD)")?;

        let rate_mode = match self.rate_mode {
            0 => RateMode::Fix {
                rate: self
                    .fix_rate
                    .parse::<f64>()
                    .map_err(|_| "Invalid fix rate")?,
                spread: self
                    .fix_spread
                    .parse::<f64>()
                    .map_err(|_| "Invalid fix spread")?,
            },
            1 => RateMode::Euribor {
                tenor: tenor_from_idx(self.euribor_tenor),
                spread: self
                    .euribor_spread
                    .parse::<f64>()
                    .map_err(|_| "Invalid euribor spread")?,
            },
            2 => RateMode::Mixed {
                fix_years: self
                    .mixed_fix_years
                    .parse::<f64>()
                    .map_err(|_| "Invalid fix years")?,
                fix_rate: self
                    .mixed_fix_rate
                    .parse::<f64>()
                    .map_err(|_| "Invalid mixed fix rate")?,
                fix_spread: self
                    .mixed_fix_spread
                    .parse::<f64>()
                    .map_err(|_| "Invalid mixed fix spread")?,
                euribor_tenor: tenor_from_idx(self.mixed_euribor_tenor),
                euribor_spread: if self.same_spread {
                    self.mixed_fix_spread
                        .parse::<f64>()
                        .map_err(|_| "Invalid spread")?
                } else {
                    self.mixed_euribor_spread
                        .parse::<f64>()
                        .map_err(|_| "Invalid euribor spread")?
                },
            },
            _ => return Err("Unknown rate mode".to_string()),
        };

        let euribor_curve = if (self.rate_mode == 1 || self.rate_mode == 2)
            && self.euribor_curve.is_empty()
        {
            let tenor = self.euribor_tenor();
            match self.euribor_cache.get_or_fetch(tenor) {
                Ok(rate) => {
                    self.popup_msg = Some(format!("Auto-fetched Euribor {}: {:.3}%", tenor, rate));
                    vec![EuriborPoint {
                        date_from: self.euribor_start_date(),
                        rate,
                    }]
                }
                Err(_) => {
                    self.popup_msg = Some("Euribor fetch failed. Using empty curve.".to_string());
                    vec![]
                }
            }
        } else {
            self.euribor_curve.clone()
        };

        let upfront_cost = parse_optional(&self.upfront_cost)?;
        let upfront_percent = parse_optional(&self.upfront_percent)?;
        if upfront_cost.is_some() && upfront_percent.is_some() {
            return Err("Specify upfront cost or percent, not both".to_string());
        }

        let params = LoanParams {
            amount,
            term_years,
            payment_type,
            currency,
            start_date,
            rate_mode,
            same_spread: self.same_spread,
            euribor_curve,
            prepayments: self.prepayments.clone(),
            upfront_cost,
            upfront_percent,
        };

        let result = Calculator::calculate(&params).map_err(|e| e.to_string())?;
        self.params = Some(params);
        self.result = Some(result);
        self.scroll_offset = 0;
        Ok(())
    }

    pub fn add_prepayment(&mut self) -> Result<(), String> {
        let date = NaiveDate::parse_from_str(&self.prepayment_date, "%Y-%m-%d")
            .map_err(|_| "Invalid prepayment date (YYYY-MM-DD)")?;
        let amount = self
            .prepayment_amount
            .parse::<f64>()
            .map_err(|_| "Invalid prepayment amount")?;
        if amount <= 0.0 {
            return Err("Prepayment amount must be positive".to_string());
        }
        let effect = if self.prepayment_effect == 0 {
            PrepaymentEffect::ReduceTerm
        } else {
            PrepaymentEffect::ReducePayment
        };
        self.prepayments.push(Prepayment {
            date,
            amount,
            effect,
        });
        self.prepayment_amount = "0".to_string();
        Ok(())
    }

    pub fn save_session(&mut self, path: &str) {
        if let (Some(params), Some(result)) = (&self.params, &self.result) {
            match mortgage_core::save_session(path, params, result) {
                Ok(()) => self.popup_msg = Some(format!("Session saved to {}", path)),
                Err(e) => self.popup_msg = Some(format!("Save failed: {}", e)),
            }
            self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
        }
    }

    pub fn load_session(&mut self, path: &str) {
        match mortgage_core::load_session(path) {
            Ok(session) => {
                self.amount = format!("{}", session.params.amount);
                self.term = format!("{}", session.params.term_years);
                self.start_date = session.params.start_date.format("%Y-%m-%d").to_string();
                self.currency = match session.params.currency {
                    Currency::Eur => 0,
                    Currency::Usd => 1,
                };
                self.payment_type = match session.params.payment_type {
                    PaymentType::Annuity => 0,
                    PaymentType::Diff => 1,
                };
                self.prepayments = session.params.prepayments;
                self.result = Some(session.result);
                self.params = None;
                self.popup_msg = Some("Session loaded".to_string());
                self.screen = Screen::Results;
            }
            Err(e) => {
                self.popup_msg = Some(format!("Load failed: {}", e));
                self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
            }
        }
    }
}

fn parse_optional(s: &str) -> Result<Option<f64>, String> {
    if s.is_empty() || s == "0" {
        Ok(None)
    } else {
        s.parse::<f64>()
            .map(Some)
            .map_err(|_| "Invalid number".to_string())
    }
}

pub fn tenor_name(idx: usize) -> &'static str {
    match idx {
        0 => "1m",
        1 => "3m",
        2 => "6m",
        _ => "12m",
    }
}

pub fn tenor_from_idx(idx: usize) -> EuriborTenor {
    match idx {
        0 => EuriborTenor::OneMonth,
        1 => EuriborTenor::ThreeMonths,
        2 => EuriborTenor::SixMonths,
        _ => EuriborTenor::TwelveMonths,
    }
}
