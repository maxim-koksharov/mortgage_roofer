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
    Calendar { field: Field, state: CalendarState },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Calculator,
    ReverseCalculator,
}

impl Tab {
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Calculator => "Calculator",
            Tab::ReverseCalculator => "Max Loan",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalendarState {
    pub year: i32,
    pub month: u32,
    pub selected_day: Option<u32>,
}

impl CalendarState {
    pub fn new(today: NaiveDate) -> Self {
        use chrono::Datelike;
        Self {
            year: today.year(),
            month: today.month(),
            selected_day: Some(today.day()),
        }
    }

    pub fn prev_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
    }

    pub fn next_month(&mut self) {
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
    }
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
    DownPayment,
    ExtraMonthlyCost,
}

pub struct App {
    pub screen: Screen,
    pub active_tab: Tab,
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
    pub down_payment: String,
    pub extra_monthly_cost: String,

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

    pub reverse_fields: Vec<ReverseField>,
    pub reverse_selected: usize,
    pub reverse_target_payment: String,
    pub reverse_payment_type: usize,
    pub reverse_rate_mode: usize,
    pub reverse_fix_rate: String,
    pub reverse_fix_spread: String,
    pub reverse_euribor_tenor: usize,
    pub reverse_euribor_spread: String,
    pub reverse_extra_monthly: String,
    pub reverse_result: Option<Vec<ReverseRow>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisView {
    Sensitivity(Vec<mortgage_core::SensitivityPoint>),
    BreakEven(mortgage_core::BreakEvenResult),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReverseField {
    TargetPayment,
    PaymentType,
    RateMode,
    FixRate,
    FixSpread,
    EuriborTenor,
    EuriborSpread,
    EuriborFetchButton,
    ExtraMonthlyCost,
}

#[derive(Debug, Clone)]
pub struct ReverseRow {
    pub term_years: u32,
    pub max_amount: f64,
    pub monthly_payment: f64,
    pub extra_cost: f64,
    pub total_monthly: f64,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            screen: Screen::Form,
            active_tab: Tab::Calculator,
            fields: vec![],
            selected: 0,
            should_exit: false,
            amount: "185000".to_string(),
            term: "30".to_string(),
            start_date: chrono::Local::now()
                .date_naive()
                .format("%d-%m-%Y")
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
            prepayment_date: "01-01-2027".to_string(),
            prepayment_amount: "20000".to_string(),
            prepayment_effect: 0,
            prepayments: vec![],
            upfront_cost: "0".to_string(),
            upfront_percent: "5".to_string(),
            down_payment: "0".to_string(),
            extra_monthly_cost: "0".to_string(),
            euribor_cache: EuriborCache::default(),
            euribor_curve: vec![],
            euribor_date: chrono::Local::now()
                .date_naive()
                .format("%d-%m-%Y")
                .to_string(),
            euribor_rate: "3.0".to_string(),
            result: None,
            params: None,
            scroll_offset: 0,
            popup_msg: None,
            show_yearly: false,
            show_analysis: None,
            reverse_fields: vec![],
            reverse_selected: 0,
            reverse_target_payment: "1000".to_string(),
            reverse_payment_type: 0,
            reverse_rate_mode: 0,
            reverse_fix_rate: "3.6".to_string(),
            reverse_fix_spread: "0.0".to_string(),
            reverse_euribor_tenor: 2,
            reverse_euribor_spread: "1.0".to_string(),
            reverse_extra_monthly: "0".to_string(),
            reverse_result: None,
        };
        app.rebuild_fields();
        app.rebuild_reverse_fields();
        app
    }

    pub fn rebuild_fields(&mut self) {
        let mut f = vec![
            Field::Amount,
            Field::DownPayment,
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
            Field::ExtraMonthlyCost,
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
            Field::DownPayment => Some(&mut self.down_payment),
            Field::ExtraMonthlyCost => Some(&mut self.extra_monthly_cost),
            _ => None,
        }
    }

    fn is_date_field(field: Field) -> bool {
        matches!(
            field,
            Field::StartDate | Field::PrepaymentDate | Field::EuriborDate
        )
    }

    pub fn edit_date(&mut self, c: char) {
        let current = self.fields[self.selected];
        if !Self::is_date_field(current) {
            self.edit_text(c);
            return;
        }
        if !c.is_ascii_digit() {
            return;
        }
        if let Some(field) = self.field_mut(current) {
            let digits: String = field.chars().filter(|ch| ch.is_ascii_digit()).collect();
            if digits.len() >= 8 {
                return;
            }
            let next_digit_pos = digits.len();

            if next_digit_pos == 0 && (c == '0' || c == '1' || c == '2' || c == '3') {
                field.push(c);
            } else if next_digit_pos == 0 {
                field.push('0');
                field.push(c);
            } else if next_digit_pos == 1 {
                let first = digits.chars().next().unwrap();
                let day: u32 = format!("{}{}", first, c).parse().unwrap_or(0);
                if (1..=31).contains(&day) {
                    field.push(c);
                    field.push('-');
                }
            } else if next_digit_pos == 2 {
                if c == '0' || c == '1' {
                    field.push(c);
                } else {
                    field.push('0');
                    field.push(c);
                }
            } else if next_digit_pos == 3 {
                let third = digits.chars().nth(2).unwrap();
                let month: u32 = format!("{}{}", third, c).parse().unwrap_or(0);
                if (1..=12).contains(&month) {
                    field.push(c);
                    field.push('-');
                }
            } else if (4..8).contains(&next_digit_pos) {
                field.push(c);
            }
        }
    }

    pub fn backspace_date(&mut self) {
        let current = self.fields[self.selected];
        if !Self::is_date_field(current) {
            self.backspace();
            return;
        }
        if let Some(field) = self.field_mut(current) {
            if field.ends_with('-') {
                field.pop();
            }
            field.pop();
            while field.ends_with('-') {
                field.pop();
            }
            field.pop();
            let digits: String = field.chars().filter(|ch| ch.is_ascii_digit()).collect();
            field.clear();
            if digits.is_empty() {
                return;
            }
            for (i, ch) in digits.chars().enumerate() {
                if i == 0 {
                    if ch == '0' && digits.len() >= 2 {
                        continue;
                    }
                    field.push(ch);
                } else if i == 1 {
                    field.push(ch);
                    field.push('-');
                } else if i == 2 {
                    field.push(ch);
                } else if i == 3 {
                    field.push(ch);
                    field.push('-');
                } else {
                    field.push(ch);
                }
            }
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
        let start_date = NaiveDate::parse_from_str(&self.start_date, "%d-%m-%Y")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        if self.rate_mode == 2 {
            let fix_years = self.mixed_fix_years.parse::<f64>().unwrap_or(2.0);
            start_date
                .checked_add_months(chrono::Months::new((fix_years * 12.0).round() as u32))
                .unwrap_or(start_date)
        } else {
            start_date
        }
    }

    pub fn fetch_euribor(&mut self) -> Result<(), String> {
        let tenor = self.euribor_tenor();
        match self.euribor_cache.get_or_fetch(tenor) {
            Ok(rate) => {
                let date_from = self.euribor_start_date();
                if let Some(existing) = self
                    .euribor_curve
                    .iter_mut()
                    .find(|p| p.date_from == date_from)
                {
                    existing.rate = rate;
                } else {
                    self.euribor_curve.push(EuriborPoint { date_from, rate });
                }
                self.euribor_curve.sort_by_key(|p| p.date_from);
                self.popup_msg = Some(format!("Fetched Euribor {}: {:.3}%", tenor, rate));
                Ok(())
            }
            Err(e) => Err(format!("Euribor fetch failed: {}", e)),
        }
    }

    pub fn add_euribor_point(&mut self) -> Result<(), String> {
        let date = NaiveDate::parse_from_str(&self.euribor_date, "%d-%m-%Y")
            .map_err(|_| "Invalid date (DD-MM-YYYY)")?;
        let rate = self
            .euribor_rate
            .parse::<f64>()
            .map_err(|_| "Invalid rate")?;
        if !(0.0..=100.0).contains(&rate) {
            return Err("Rate must be 0-100%".to_string());
        }
        if let Some(existing) = self.euribor_curve.iter_mut().find(|p| p.date_from == date) {
            existing.rate = rate;
        } else {
            self.euribor_curve.push(EuriborPoint {
                date_from: date,
                rate,
            });
        }
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
        let start_date = chrono::NaiveDate::parse_from_str(&self.start_date, "%d-%m-%Y")
            .map_err(|_| "Invalid start date (DD-MM-YYYY)")?;

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
            down_payment: parse_optional(&self.down_payment)?,
        };

        let result = Calculator::calculate(&params).map_err(|e| e.to_string())?;
        self.params = Some(params);
        self.result = Some(result);
        self.scroll_offset = 0;
        Ok(())
    }

    pub fn add_prepayment(&mut self) -> Result<(), String> {
        let date = NaiveDate::parse_from_str(&self.prepayment_date, "%d-%m-%Y")
            .map_err(|_| "Invalid prepayment date (DD-MM-YYYY)")?;
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
                self.start_date = session.params.start_date.format("%d-%m-%Y").to_string();
                self.currency = match session.params.currency {
                    Currency::Eur => 0,
                    Currency::Usd => 1,
                };
                self.payment_type = match session.params.payment_type {
                    PaymentType::Annuity => 0,
                    PaymentType::Diff => 1,
                };
                self.same_spread = session.params.same_spread;
                match &session.params.rate_mode {
                    RateMode::Fix { rate, spread } => {
                        self.rate_mode = 0;
                        self.fix_rate = format!("{}", rate);
                        self.fix_spread = format!("{}", spread);
                    }
                    RateMode::Euribor { tenor, spread } => {
                        self.rate_mode = 1;
                        self.euribor_tenor = match tenor {
                            EuriborTenor::OneMonth => 0,
                            EuriborTenor::ThreeMonths => 1,
                            EuriborTenor::SixMonths => 2,
                            EuriborTenor::TwelveMonths => 3,
                        };
                        self.euribor_spread = format!("{}", spread);
                    }
                    RateMode::Mixed {
                        fix_years,
                        fix_rate,
                        fix_spread,
                        euribor_tenor,
                        euribor_spread,
                    } => {
                        self.rate_mode = 2;
                        self.mixed_fix_years = format!("{}", fix_years);
                        self.mixed_fix_rate = format!("{}", fix_rate);
                        self.mixed_fix_spread = format!("{}", fix_spread);
                        self.mixed_euribor_tenor = match euribor_tenor {
                            EuriborTenor::OneMonth => 0,
                            EuriborTenor::ThreeMonths => 1,
                            EuriborTenor::SixMonths => 2,
                            EuriborTenor::TwelveMonths => 3,
                        };
                        self.mixed_euribor_spread = format!("{}", euribor_spread);
                    }
                }
                self.euribor_curve = session.params.euribor_curve.clone();
                self.prepayments = session.params.prepayments.clone();
                self.down_payment = session
                    .params
                    .down_payment
                    .map(|v| format!("{}", v))
                    .unwrap_or_else(|| "0".to_string());
                self.upfront_cost = session
                    .params
                    .upfront_cost
                    .map(|v| format!("{}", v))
                    .unwrap_or_else(|| "0".to_string());
                self.upfront_percent = session
                    .params
                    .upfront_percent
                    .map(|v| format!("{}", v))
                    .unwrap_or_else(|| "5".to_string());
                self.rebuild_fields();
                self.result = Some(session.result);
                self.params = Some(session.params);
                self.popup_msg = Some("Session loaded".to_string());
                self.screen = Screen::Results;
            }
            Err(e) => {
                self.popup_msg = Some(format!("Load failed: {}", e));
                self.screen = Screen::Popup(self.popup_msg.clone().unwrap());
            }
        }
    }

    pub fn rebuild_reverse_fields(&mut self) {
        let mut f = vec![
            ReverseField::TargetPayment,
            ReverseField::PaymentType,
            ReverseField::RateMode,
        ];
        match self.reverse_rate_mode {
            0 => {
                f.push(ReverseField::FixRate);
                f.push(ReverseField::FixSpread);
            }
            1 => {
                f.push(ReverseField::EuriborTenor);
                f.push(ReverseField::EuriborSpread);
                f.push(ReverseField::EuriborFetchButton);
            }
            _ => {}
        }
        f.push(ReverseField::ExtraMonthlyCost);
        self.reverse_fields = f;
        if self.reverse_selected >= self.reverse_fields.len() {
            self.reverse_selected = self.reverse_fields.len().saturating_sub(1);
        }
    }

    fn reverse_field_mut(&mut self, field: ReverseField) -> Option<&mut String> {
        match field {
            ReverseField::TargetPayment => Some(&mut self.reverse_target_payment),
            ReverseField::FixRate => Some(&mut self.reverse_fix_rate),
            ReverseField::FixSpread => Some(&mut self.reverse_fix_spread),
            ReverseField::EuriborSpread => Some(&mut self.reverse_euribor_spread),
            ReverseField::ExtraMonthlyCost => Some(&mut self.reverse_extra_monthly),
            _ => None,
        }
    }

    pub fn reverse_edit_text(&mut self, c: char) {
        if let Some(field) = self.reverse_field_mut(self.reverse_fields[self.reverse_selected]) {
            field.push(c);
        }
    }

    pub fn reverse_backspace(&mut self) {
        if let Some(field) = self.reverse_field_mut(self.reverse_fields[self.reverse_selected]) {
            field.pop();
        }
    }

    pub fn reverse_cycle_enum(&mut self, delta: i8) {
        let current = self.reverse_fields[self.reverse_selected];
        let len = match current {
            ReverseField::PaymentType => 2,
            ReverseField::RateMode => 2,
            ReverseField::EuriborTenor => 4,
            _ => return,
        };

        let value = match current {
            ReverseField::PaymentType => &mut self.reverse_payment_type,
            ReverseField::RateMode => &mut self.reverse_rate_mode,
            ReverseField::EuriborTenor => &mut self.reverse_euribor_tenor,
            _ => return,
        };

        let old = *value;
        *value = ((*value as i8 + delta).rem_euclid(len)) as usize;
        if current == ReverseField::RateMode && old != *value {
            self.rebuild_reverse_fields();
        }
    }

    pub fn reverse_calculate(&mut self) -> Result<(), String> {
        let target = self
            .reverse_target_payment
            .parse::<f64>()
            .map_err(|_| "Invalid target payment")?;
        if target <= 0.0 {
            return Err("Target payment must be positive".to_string());
        }

        let extra = self
            .reverse_extra_monthly
            .parse::<f64>()
            .map_err(|_| "Invalid extra monthly cost")?;
        if extra < 0.0 {
            return Err("Extra monthly cost cannot be negative".to_string());
        }

        let payment_type = if self.reverse_payment_type == 0 {
            PaymentType::Annuity
        } else {
            PaymentType::Diff
        };

        let annual_rate = match self.reverse_rate_mode {
            0 => {
                let rate = self
                    .reverse_fix_rate
                    .parse::<f64>()
                    .map_err(|_| "Invalid fix rate")?;
                let spread = self
                    .reverse_fix_spread
                    .parse::<f64>()
                    .map_err(|_| "Invalid fix spread")?;
                if rate + spread < 0.0 {
                    return Err("Total rate cannot be negative".to_string());
                }
                rate + spread
            }
            1 => {
                let spread = self
                    .reverse_euribor_spread
                    .parse::<f64>()
                    .map_err(|_| "Invalid euribor spread")?;
                if spread < 0.0 {
                    return Err("Spread cannot be negative".to_string());
                }
                let tenor = tenor_from_idx(self.reverse_euribor_tenor);
                match self.euribor_cache.get_or_fetch(tenor) {
                    Ok(rate) => {
                        self.popup_msg = Some(format!("Fetched Euribor {}: {:.3}%", tenor, rate));
                        rate + spread
                    }
                    Err(e) => return Err(format!("Euribor fetch failed: {}", e)),
                }
            }
            _ => return Err("Unknown rate mode".to_string()),
        };

        let mut rows = Vec::new();
        for term in 5..=34 {
            let amount = Calculator::reverse_calculate(target, annual_rate, term, payment_type);
            rows.push(ReverseRow {
                term_years: term,
                max_amount: (amount * 100.0).round() / 100.0,
                monthly_payment: target,
                extra_cost: extra,
                total_monthly: target + extra,
            });
        }

        self.reverse_result = Some(rows);
        Ok(())
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

pub fn days_in_month(year: i32, month: u32) -> u32 {
    chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|d| d.checked_add_months(chrono::Months::new(1)))
        .map(|next| {
            let prev = next - chrono::Duration::days(1);
            use chrono::Datelike;
            prev.day()
        })
        .unwrap_or(30)
}
