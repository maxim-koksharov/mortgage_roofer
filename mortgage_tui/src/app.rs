use chrono::NaiveDate;
use mortgage_core::models::*;
use mortgage_core::Calculator;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Form,
    Results,
    Popup(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Field {
    Amount,
    Term,
    Currency,
    PaymentType,
    RateMode,
    FixRate,
    FixSpread,
    EuriborTenor,
    EuriborSpread,
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
}

pub struct App {
    pub screen: Screen,
    pub fields: Vec<Field>,
    pub selected: usize,

    pub amount: String,
    pub term: String,
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

    pub result: Option<LoanResult>,
    pub params: Option<LoanParams>,
    pub scroll_offset: usize,
    pub popup_msg: Option<String>,
}

impl App {
    pub fn new() -> Self {
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
            prepayments: vec![],
            result: None,
            params: None,
            scroll_offset: 0,
            popup_msg: None,
        };
        app.rebuild_fields();
        app
    }

    pub fn rebuild_fields(&mut self) {
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
        f.push(Field::AddPrepayment);
        self.fields = f;
        if self.selected >= self.fields.len() {
            self.selected = self.fields.len().saturating_sub(1);
        }
    }

    pub fn edit_text(&mut self, c: char) {
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

    pub fn backspace(&mut self) {
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

    pub fn cycle_enum(&mut self, delta: i8) {
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

    pub fn calculate(&mut self) -> Result<(), String> {
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

        let params = LoanParams {
            amount,
            term_years,
            payment_type,
            currency,
            start_date,
            rate_mode,
            same_spread: self.same_spread,
            euribor_curve: vec![],
            prepayments: self.prepayments.clone(),
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
        let amount = self.prepayment_amount.parse::<f64>()
            .map_err(|_| "Invalid prepayment amount")?;
        if amount <= 0.0 {
            return Err("Prepayment amount must be positive".to_string());
        }
        let effect = if self.prepayment_effect == 0 {
            PrepaymentEffect::ReduceTerm
        } else {
            PrepaymentEffect::ReducePayment
        };
        self.prepayments.push(Prepayment { date, amount, effect });
        self.prepayment_amount = "0".to_string();
        Ok(())
    }

    pub fn remove_prepayment(&mut self, idx: usize) {
        if idx < self.prepayments.len() {
            self.prepayments.remove(idx);
        }
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
