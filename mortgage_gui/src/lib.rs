pub mod chart;

use iced::{
    Alignment, Color, Element, Length, Pixels, Point, Size, Task, Theme,
    widget::{
        Column, Rule, button, checkbox, column, container, pick_list, row, scrollable, text,
        text_input,
    },
};
use mortgage_core::euribor::EuriborCache;
use mortgage_core::models::*;
use mortgage_core::{Calculator, payments_to_csv};
use std::fs;

use iced::alignment::{Horizontal, Vertical};
use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Program};

pub fn run() -> iced::Result {
    iced::application("Mortgage Calculator", update, view)
        .theme(|_| Theme::TokyoNightStorm)
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DateField {
    StartDate,
    PrepaymentDate,
}

#[derive(Debug, Clone)]
pub struct GuiCalendarState {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Default for GuiCalendarState {
    fn default() -> Self {
        let today = chrono::Local::now().date_naive();
        use chrono::Datelike;
        Self {
            year: today.year(),
            month: today.month(),
            day: today.day(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    AmountChanged(String),
    TermChanged(String),
    StartDateChanged(String),
    RateChanged(String),
    SpreadChanged(String),
    CurrencyChanged(String),
    PaymentTypeChanged(String),
    RateModeChanged(String),
    EuriborTenorChanged(String),
    EuriborSpreadChanged(String),
    MixedFixYearsChanged(String),
    MixedFixRateChanged(String),
    MixedFixSpreadChanged(String),
    MixedEuriborTenorChanged(String),
    MixedEuriborSpreadChanged(String),
    SameSpreadToggled(bool),
    PrepaymentDateChanged(String),
    PrepaymentAmountChanged(String),
    PrepaymentEffectChanged(String),
    UpfrontCostChanged(String),
    UpfrontPercentChanged(String),
    DownPaymentChanged(String),
    AddPrepayment,
    RemovePrepayment(usize),
    Calculate,
    ExportCsv,
    ExportPdf,
    ShowTable,
    ShowChart,
    ShowBalanceChart,
    ShowYearly,
    ShowSensitivity,
    ShowBreakEven,
    RentChanged(String),
    ReverseTargetChanged(String),
    ReversePaymentTypeChanged(String),
    ReverseRateModeChanged(String),
    ReverseFixRateChanged(String),
    ReverseFixSpreadChanged(String),
    ReverseEuriborTenorChanged(String),
    ReverseEuriborSpreadChanged(String),
    ReverseExtraChanged(String),
    ReverseCalculate,
    ShowReverseCalc,
    ShowCalculator,
    SaveSession,
    LoadSession,
    OpenCalendar(DateField),
    CloseCalendar,
    CalendarMonthPrev,
    CalendarMonthNext,
    CalendarDaySelect(u32),
    ToggleXAxis,
    StackedChartMouseMoved(usize),
    StackedChartMouseLeft,
    BalanceChartMouseMoved(usize),
    BalanceChartMouseLeft,
    AddEuriborManualPoint,
    RemoveEuriborManualPoint(usize),
    EuriborNewDateChanged(String),
    EuriborNewRateChanged(String),
    ToggleManualEuribor(bool),
    SaveEuriborPoints,
    LoadEuriborPoints,
    FetchEuribor,
    EuriborLoaded(Result<Vec<EuriborPoint>, String>),
}

#[derive(Debug, Clone)]
pub struct State {
    pub amount: String,
    pub term: String,
    pub start_date: String,
    pub rate: String,
    pub spread: String,
    pub currency: String,
    pub payment_type: String,
    pub rate_mode: String,
    pub euribor_tenor: String,
    pub euribor_spread: String,
    pub mixed_fix_years: String,
    pub mixed_fix_rate: String,
    pub mixed_fix_spread: String,
    pub mixed_euribor_tenor: String,
    pub mixed_euribor_spread: String,
    pub same_spread: bool,
    pub prepayment_date: String,
    pub prepayment_amount: String,
    pub prepayment_effect: String,
    pub prepayments: Vec<Prepayment>,
    pub upfront_cost: String,
    pub upfront_percent: String,
    pub down_payment: String,
    pub rent: String,
    pub reverse_target_payment: String,
    pub reverse_payment_type: String,
    pub reverse_rate_mode: String,
    pub reverse_fix_rate: String,
    pub reverse_fix_spread: String,
    pub reverse_euribor_tenor: String,
    pub reverse_euribor_spread: String,
    pub reverse_extra_monthly: String,
    pub reverse_result: Option<Vec<ReverseRow>>,
    pub params: Option<LoanParams>,
    pub result: Option<LoanResult>,
    pub active_tab: ViewTab,
    pub status: String,
    pub status_is_error: bool,
    euribor_cache: EuriborCache,
    pub calendar_target: Option<DateField>,
    pub calendar_state: GuiCalendarState,
    pub x_axis_mode: XAxisMode,
    pub hovered_payment: Option<usize>,
    pub euribor_manual_points: Vec<(String, String)>,
    pub euribor_new_date: String,
    pub euribor_new_rate: String,
    pub use_manual_euribor: bool,
    pub pending_calc: Option<LoanParams>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            amount: "185000".to_string(),
            term: "30".to_string(),
            start_date: chrono::Local::now()
                .date_naive()
                .format("%d-%m-%Y")
                .to_string(),
            rate: "3.6".to_string(),
            spread: "0.0".to_string(),
            currency: "EUR".to_string(),
            payment_type: "Annuity".to_string(),
            rate_mode: "Fix".to_string(),
            euribor_tenor: "6m".to_string(),
            euribor_spread: "1.0".to_string(),
            mixed_fix_years: "2".to_string(),
            mixed_fix_rate: "3.0".to_string(),
            mixed_fix_spread: "1.0".to_string(),
            mixed_euribor_tenor: "6m".to_string(),
            mixed_euribor_spread: "1.5".to_string(),
            same_spread: false,
            prepayment_date: "01-01-2027".to_string(),
            prepayment_amount: "20000".to_string(),
            prepayment_effect: "ReduceTerm".to_string(),
            prepayments: vec![],
            upfront_cost: "0".to_string(),
            upfront_percent: "5".to_string(),
            down_payment: "0".to_string(),
            rent: "900".to_string(),
            reverse_target_payment: "1000".to_string(),
            reverse_payment_type: "Annuity".to_string(),
            reverse_rate_mode: "Fix".to_string(),
            reverse_fix_rate: "3.6".to_string(),
            reverse_fix_spread: "0.0".to_string(),
            reverse_euribor_tenor: "6m".to_string(),
            reverse_euribor_spread: "1.0".to_string(),
            reverse_extra_monthly: "0".to_string(),
            reverse_result: None,
            params: None,
            result: None,
            active_tab: ViewTab::Table,
            status: String::new(),
            status_is_error: false,
            euribor_cache: EuriborCache::default(),
            calendar_target: None,
            calendar_state: GuiCalendarState::default(),
            x_axis_mode: XAxisMode::default(),
            hovered_payment: None,
            euribor_manual_points: vec![],
            euribor_new_date: String::new(),
            euribor_new_rate: String::new(),
            use_manual_euribor: false,
            pending_calc: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ViewTab {
    #[default]
    Table,
    Chart,
    BalanceChart,
    Yearly,
    Sensitivity,
    BreakEven,
    ReverseCalc,
}

#[derive(Debug, Clone)]
pub struct ReverseRow {
    pub term_years: u32,
    pub max_amount: f64,
    pub monthly_payment: f64,
    pub extra_cost: f64,
    pub total_monthly: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum XAxisMode {
    #[default]
    PaymentNumber,
    Date,
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::AmountChanged(v) => {
            state.amount = v;
            Task::none()
        }
        Message::TermChanged(v) => {
            state.term = v;
            Task::none()
        }
        Message::StartDateChanged(v) => {
            state.start_date = filter_date_input(&v);
            Task::none()
        }
        Message::RateChanged(v) => {
            state.rate = v;
            Task::none()
        }
        Message::SpreadChanged(v) => {
            state.spread = v;
            Task::none()
        }
        Message::CurrencyChanged(v) => {
            state.currency = v;
            Task::none()
        }
        Message::PaymentTypeChanged(v) => {
            state.payment_type = v;
            Task::none()
        }
        Message::RateModeChanged(v) => {
            state.rate_mode = v;
            Task::none()
        }
        Message::EuriborTenorChanged(v) => {
            state.euribor_tenor = v;
            Task::none()
        }
        Message::EuriborSpreadChanged(v) => {
            state.euribor_spread = v;
            Task::none()
        }
        Message::MixedFixYearsChanged(v) => {
            state.mixed_fix_years = v;
            Task::none()
        }
        Message::MixedFixRateChanged(v) => {
            state.mixed_fix_rate = v;
            Task::none()
        }
        Message::MixedFixSpreadChanged(v) => {
            state.mixed_fix_spread = v;
            Task::none()
        }
        Message::MixedEuriborTenorChanged(v) => {
            state.mixed_euribor_tenor = v;
            Task::none()
        }
        Message::MixedEuriborSpreadChanged(v) => {
            state.mixed_euribor_spread = v;
            Task::none()
        }
        Message::SameSpreadToggled(v) => {
            state.same_spread = v;
            Task::none()
        }
        Message::PrepaymentDateChanged(v) => {
            state.prepayment_date = filter_date_input(&v);
            Task::none()
        }
        Message::PrepaymentAmountChanged(v) => {
            state.prepayment_amount = v;
            Task::none()
        }
        Message::PrepaymentEffectChanged(v) => {
            state.prepayment_effect = v;
            Task::none()
        }
        Message::UpfrontCostChanged(v) => {
            state.upfront_cost = v;
            Task::none()
        }
        Message::UpfrontPercentChanged(v) => {
            state.upfront_percent = v;
            Task::none()
        }
        Message::DownPaymentChanged(v) => {
            state.down_payment = v;
            Task::none()
        }
        Message::AddPrepayment => {
            add_prepayment(state);
            Task::none()
        }
        Message::RemovePrepayment(idx) => {
            if idx < state.prepayments.len() {
                state.prepayments.remove(idx);
                state.status = format!("Removed prepayment. {} remaining", state.prepayments.len());
                state.status_is_error = false;
            }
            Task::none()
        }
        Message::Calculate => {
            let needs_euribor = state.rate_mode == "Euribor" || state.rate_mode == "Mixed";
            let has_manual = state.use_manual_euribor && !state.euribor_manual_points.is_empty();

            if needs_euribor && !has_manual {
                let amount = match state.amount.parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => {
                        state.status = "Invalid amount".to_string();
                        state.status_is_error = true;
                        return Task::none();
                    }
                };
                let term_years = match state.term.parse::<u32>() {
                    Ok(v) => v,
                    Err(_) => {
                        state.status = "Invalid term".to_string();
                        state.status_is_error = true;
                        return Task::none();
                    }
                };
                let start_date =
                    match chrono::NaiveDate::parse_from_str(&state.start_date, "%d-%m-%Y") {
                        Ok(d) => d,
                        Err(_) => {
                            state.status = "Invalid start date (DD-MM-YYYY)".to_string();
                            state.status_is_error = true;
                            return Task::none();
                        }
                    };
                let currency = if state.currency == "USD" {
                    Currency::Usd
                } else {
                    Currency::Eur
                };
                let payment_type = if state.payment_type == "Diff" {
                    PaymentType::Diff
                } else {
                    PaymentType::Annuity
                };
                let rate_mode = match state.rate_mode.as_str() {
                    "Fix" => RateMode::Fix {
                        rate: state.rate.parse::<f64>().unwrap_or(3.6),
                        spread: state.spread.parse::<f64>().unwrap_or(0.0),
                    },
                    "Euribor" => RateMode::Euribor {
                        tenor: parse_tenor(&state.euribor_tenor),
                        spread: state.euribor_spread.parse::<f64>().unwrap_or(1.0),
                    },
                    "Mixed" => RateMode::Mixed {
                        fix_years: state.mixed_fix_years.parse::<f64>().unwrap_or(2.0),
                        fix_rate: state.mixed_fix_rate.parse::<f64>().unwrap_or(3.0),
                        fix_spread: state.mixed_fix_spread.parse::<f64>().unwrap_or(1.0),
                        euribor_tenor: parse_tenor(&state.mixed_euribor_tenor),
                        euribor_spread: if state.same_spread {
                            state.mixed_fix_spread.parse::<f64>().unwrap_or(1.0)
                        } else {
                            state.mixed_euribor_spread.parse::<f64>().unwrap_or(1.5)
                        },
                    },
                    _ => RateMode::Fix {
                        rate: 3.6,
                        spread: 0.0,
                    },
                };

                let tenor = if state.rate_mode == "Mixed" {
                    parse_tenor(&state.mixed_euribor_tenor)
                } else {
                    parse_tenor(&state.euribor_tenor)
                };
                let curve_start = if state.rate_mode == "Mixed" {
                    let fix_years = state.mixed_fix_years.parse::<f64>().unwrap_or(2.0);
                    start_date
                        .checked_add_months(chrono::Months::new((fix_years * 12.0).round() as u32))
                        .unwrap_or(start_date)
                } else {
                    start_date
                };
                let today = chrono::Local::now().date_naive();
                let ecb_end = today;
                let ecb_start = curve_start.min(
                    today
                        .checked_sub_months(chrono::Months::new(3))
                        .unwrap_or(today),
                );

                let params = LoanParams {
                    amount,
                    term_years,
                    payment_type,
                    currency,
                    start_date,
                    rate_mode,
                    same_spread: state.same_spread,
                    euribor_curve: vec![],
                    prepayments: state.prepayments.clone(),
                    upfront_cost: state.upfront_cost.parse::<f64>().ok().filter(|&v| v != 0.0),
                    upfront_percent: state
                        .upfront_percent
                        .parse::<f64>()
                        .ok()
                        .filter(|&v| v != 0.0),
                    down_payment: state.down_payment.parse::<f64>().ok().filter(|&v| v != 0.0),
                };

                state.pending_calc = Some(params);
                state.status = format!("Loading Euribor {} historical data...", tenor);
                state.status_is_error = false;

                let mut cache = state.euribor_cache.clone();
                Task::perform(
                    async move {
                        cache
                            .fetch_historical(tenor, ecb_start, ecb_end)
                            .map_err(|e| e.to_string())
                    },
                    Message::EuriborLoaded,
                )
            } else {
                calculate(state);
                Task::none()
            }
        }
        Message::ExportCsv => {
            export_csv(state);
            Task::none()
        }
        Message::ExportPdf => {
            export_pdf(state);
            Task::none()
        }
        Message::ShowTable => {
            state.active_tab = ViewTab::Table;
            state.hovered_payment = None;
            Task::none()
        }
        Message::ShowChart => {
            state.active_tab = ViewTab::Chart;
            state.hovered_payment = None;
            Task::none()
        }
        Message::ShowBalanceChart => {
            state.active_tab = ViewTab::BalanceChart;
            state.hovered_payment = None;
            Task::none()
        }
        Message::ShowYearly => {
            state.active_tab = ViewTab::Yearly;
            state.hovered_payment = None;
            Task::none()
        }
        Message::ShowSensitivity => {
            state.active_tab = ViewTab::Sensitivity;
            state.hovered_payment = None;
            Task::none()
        }
        Message::ShowBreakEven => {
            state.active_tab = ViewTab::BreakEven;
            state.hovered_payment = None;
            Task::none()
        }
        Message::RentChanged(v) => {
            state.rent = v;
            Task::none()
        }
        Message::ReverseTargetChanged(v) => {
            state.reverse_target_payment = v;
            Task::none()
        }
        Message::ReversePaymentTypeChanged(v) => {
            state.reverse_payment_type = v;
            Task::none()
        }
        Message::ReverseRateModeChanged(v) => {
            state.reverse_rate_mode = v;
            Task::none()
        }
        Message::ReverseFixRateChanged(v) => {
            state.reverse_fix_rate = v;
            Task::none()
        }
        Message::ReverseFixSpreadChanged(v) => {
            state.reverse_fix_spread = v;
            Task::none()
        }
        Message::ReverseEuriborTenorChanged(v) => {
            state.reverse_euribor_tenor = v;
            Task::none()
        }
        Message::ReverseEuriborSpreadChanged(v) => {
            state.reverse_euribor_spread = v;
            Task::none()
        }
        Message::ReverseExtraChanged(v) => {
            state.reverse_extra_monthly = v;
            Task::none()
        }
        Message::ReverseCalculate => {
            reverse_calculate(state);
            Task::none()
        }
        Message::ShowReverseCalc => {
            state.active_tab = ViewTab::ReverseCalc;
            Task::none()
        }
        Message::ShowCalculator => {
            state.active_tab = ViewTab::Table;
            state.hovered_payment = None;
            Task::none()
        }
        Message::SaveSession => {
            save_session_gui(state);
            Task::none()
        }
        Message::LoadSession => {
            load_session_gui(state);
            Task::none()
        }
        Message::OpenCalendar(target) => {
            state.calendar_target = Some(target);
            let date_str = match target {
                DateField::StartDate => &state.start_date,
                DateField::PrepaymentDate => &state.prepayment_date,
            };
            if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date_str, "%d-%m-%Y") {
                use chrono::Datelike;
                state.calendar_state = GuiCalendarState {
                    year: parsed.year(),
                    month: parsed.month(),
                    day: parsed.day(),
                };
            }
            Task::none()
        }
        Message::CloseCalendar => {
            state.calendar_target = None;
            Task::none()
        }
        Message::CalendarMonthPrev => {
            if state.calendar_state.month == 1 {
                state.calendar_state.month = 12;
                state.calendar_state.year -= 1;
            } else {
                state.calendar_state.month -= 1;
            }
            Task::none()
        }
        Message::CalendarMonthNext => {
            if state.calendar_state.month == 12 {
                state.calendar_state.month = 1;
                state.calendar_state.year += 1;
            } else {
                state.calendar_state.month += 1;
            }
            Task::none()
        }
        Message::CalendarDaySelect(day) => {
            let s = &state.calendar_state;
            let date_str = format!("{:02}-{:02}-{}", day, s.month, s.year);
            if let Some(target) = state.calendar_target {
                match target {
                    DateField::StartDate => state.start_date = date_str,
                    DateField::PrepaymentDate => state.prepayment_date = date_str,
                }
            }
            state.calendar_target = None;
            Task::none()
        }
        Message::ToggleXAxis => {
            state.x_axis_mode = match state.x_axis_mode {
                XAxisMode::PaymentNumber => XAxisMode::Date,
                XAxisMode::Date => XAxisMode::PaymentNumber,
            };
            Task::none()
        }
        Message::StackedChartMouseMoved(idx) => {
            state.hovered_payment = Some(idx);
            Task::none()
        }
        Message::StackedChartMouseLeft => {
            state.hovered_payment = None;
            Task::none()
        }
        Message::BalanceChartMouseMoved(idx) => {
            state.hovered_payment = Some(idx);
            Task::none()
        }
        Message::BalanceChartMouseLeft => {
            state.hovered_payment = None;
            Task::none()
        }
        Message::AddEuriborManualPoint => {
            add_euribor_manual_point(state);
            Task::none()
        }
        Message::RemoveEuriborManualPoint(idx) => {
            if idx < state.euribor_manual_points.len() {
                state.euribor_manual_points.remove(idx);
                state.status = format!("Removed Euribor point #{}", idx + 1);
                state.status_is_error = false;
            }
            Task::none()
        }
        Message::EuriborNewDateChanged(v) => {
            state.euribor_new_date = v;
            Task::none()
        }
        Message::EuriborNewRateChanged(v) => {
            state.euribor_new_rate = v;
            Task::none()
        }
        Message::ToggleManualEuribor(v) => {
            state.use_manual_euribor = v;
            Task::none()
        }
        Message::SaveEuriborPoints => {
            save_euribor_points(state);
            Task::none()
        }
        Message::LoadEuriborPoints => {
            load_euribor_points(state);
            Task::none()
        }
        Message::FetchEuribor => {
            let start_date = match chrono::NaiveDate::parse_from_str(&state.start_date, "%d-%m-%Y")
            {
                Ok(d) => d,
                Err(_) => {
                    state.status = "Invalid start date (DD-MM-YYYY)".to_string();
                    state.status_is_error = true;
                    return Task::none();
                }
            };
            let tenor = if state.rate_mode == "Mixed" {
                parse_tenor(&state.mixed_euribor_tenor)
            } else {
                parse_tenor(&state.euribor_tenor)
            };
            let curve_start = if state.rate_mode == "Mixed" {
                let fix_years = state.mixed_fix_years.parse::<f64>().unwrap_or(2.0);
                start_date
                    .checked_add_months(chrono::Months::new((fix_years * 12.0).round() as u32))
                    .unwrap_or(start_date)
            } else {
                start_date
            };
            let today = chrono::Local::now().date_naive();
            let ecb_end = today;
            let ecb_start = curve_start.min(
                today
                    .checked_sub_months(chrono::Months::new(3))
                    .unwrap_or(today),
            );

            state.pending_calc = None;
            state.status = format!("Loading Euribor {} historical data...", tenor);
            state.status_is_error = false;

            let mut cache = state.euribor_cache.clone();
            Task::perform(
                async move {
                    cache
                        .fetch_historical(tenor, ecb_start, ecb_end)
                        .map_err(|e| e.to_string())
                },
                Message::EuriborLoaded,
            )
        }
        Message::EuriborLoaded(result) => {
            match result {
                Ok(points) => {
                    if let Some(mut params) = state.pending_calc.take() {
                        params.euribor_curve = points;
                        match Calculator::calculate(&params) {
                            Ok(calc_result) => {
                                state.params = Some(params);
                                state.result = Some(calc_result);
                                state.hovered_payment = None;
                                state.status = "Calculation complete".to_string();
                                state.status_is_error = false;
                            }
                            Err(e) => {
                                state.status = format!("Error: {}", e);
                                state.status_is_error = true;
                            }
                        }
                    } else {
                        state.euribor_manual_points = points
                            .iter()
                            .map(|p| {
                                (
                                    p.date_from.format("%d-%m-%Y").to_string(),
                                    format!("{:.3}", p.rate),
                                )
                            })
                            .collect();
                        state.use_manual_euribor = true;
                        state.status = format!(
                            "Loaded {} Euribor points",
                            state.euribor_manual_points.len()
                        );
                        state.status_is_error = false;
                    }
                }
                Err(e) => {
                    state.pending_calc = None;
                    state.status = format!("Euribor fetch failed: {}", e);
                    state.status_is_error = true;
                }
            }
            Task::none()
        }
    }
}

fn add_prepayment(state: &mut State) {
    if let Ok(date) = chrono::NaiveDate::parse_from_str(&state.prepayment_date, "%d-%m-%Y") {
        if let Ok(amount) = state.prepayment_amount.parse::<f64>() {
            if amount > 0.0 {
                let effect = if state.prepayment_effect == "ReducePayment" {
                    PrepaymentEffect::ReducePayment
                } else {
                    PrepaymentEffect::ReduceTerm
                };
                state.prepayments.push(Prepayment {
                    date,
                    amount,
                    effect,
                });
                state.status = format!("Added prepayment #{}", state.prepayments.len());
                state.status_is_error = false;
                return;
            }
            state.status = "Prepayment amount must be positive".to_string();
        } else {
            state.status = "Invalid prepayment amount".to_string();
        }
    } else {
        state.status = "Invalid date format (DD-MM-YYYY)".to_string();
    }
    state.status_is_error = true;
}

fn add_euribor_manual_point(state: &mut State) {
    let date_str = state.euribor_new_date.clone();
    let rate_str = state.euribor_new_rate.clone();

    if rate_str.parse::<f64>().is_err() {
        state.status = "Invalid rate".to_string();
        state.status_is_error = true;
        return;
    }
    let new_date = match chrono::NaiveDate::parse_from_str(&date_str, "%d-%m-%Y") {
        Ok(d) => d,
        Err(_) => {
            state.status = "Invalid date format (DD-MM-YYYY)".to_string();
            state.status_is_error = true;
            return;
        }
    };

    for (existing_date_str, _) in &state.euribor_manual_points {
        if let Ok(existing_date) = chrono::NaiveDate::parse_from_str(existing_date_str, "%d-%m-%Y")
            && new_date <= existing_date
        {
            state.status = "Dates must be in chronological order".to_string();
            state.status_is_error = true;
            return;
        }
    }

    state.euribor_manual_points.push((date_str, rate_str));
    state.euribor_new_date = String::new();
    state.euribor_new_rate = String::new();
    state.status = format!("Added Euribor point #{}", state.euribor_manual_points.len());
    state.status_is_error = false;
}

fn save_euribor_points(state: &mut State) {
    if state.euribor_manual_points.is_empty() {
        state.status = "No Euribor points to save".to_string();
        state.status_is_error = true;
        return;
    }
    let json = serde_json::to_string_pretty(&state.euribor_manual_points).unwrap();
    let path = "/tmp/euribor_points.json";
    match std::fs::write(path, json) {
        Ok(()) => {
            state.status = format!(
                "Saved {} Euribor points to {}",
                state.euribor_manual_points.len(),
                path
            );
            state.status_is_error = false;
        }
        Err(e) => {
            state.status = format!("Save failed: {}", e);
            state.status_is_error = true;
        }
    }
}

fn load_euribor_points(state: &mut State) {
    let path = "/tmp/euribor_points.json";
    match std::fs::read_to_string(path) {
        Ok(json) => match serde_json::from_str::<Vec<(String, String)>>(&json) {
            Ok(mut points) => {
                points.sort_by(|a, b| {
                    let da = chrono::NaiveDate::parse_from_str(&a.0, "%d-%m-%Y").ok();
                    let db = chrono::NaiveDate::parse_from_str(&b.0, "%d-%m-%Y").ok();
                    da.cmp(&db)
                });
                state.euribor_manual_points = points;
                state.status = format!(
                    "Loaded {} Euribor points",
                    state.euribor_manual_points.len()
                );
                state.status_is_error = false;
            }
            Err(e) => {
                state.status = format!("Parse failed: {}", e);
                state.status_is_error = true;
            }
        },
        Err(e) => {
            state.status = format!("Read failed: {}", e);
            state.status_is_error = true;
        }
    }
}

fn input_row<'a>(label: &'a str, content: Element<'a, Message>) -> Element<'a, Message> {
    row![text(label).size(16).width(Length::Fixed(110.0)), content]
        .spacing(3)
        .align_y(Alignment::Center)
        .into()
}

fn section_header(title: &str) -> Element<'_, Message> {
    container(text(title).size(16))
        .padding(0)
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(
                0.2, 0.3, 0.4,
            ))),
            ..Default::default()
        })
        .into()
}

fn compact_input<'a>(
    placeholder: &str,
    value: &'a str,
    msg: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    text_input(placeholder, value)
        .on_input(msg)
        .padding(2)
        .size(16)
        .width(Length::Fixed(150.0))
        .into()
}

pub fn view(state: &State) -> Element<'_, Message> {
    let reverse_mode = state.active_tab == ViewTab::ReverseCalc;

    let mode_switcher: Element<Message> = row![
        button(text("Calculator").size(14))
            .padding(2)
            .style(if !reverse_mode {
                button::primary
            } else {
                button::secondary
            })
            .on_press(Message::ShowCalculator),
        button(text("Reverse Calc").size(14))
            .padding(2)
            .style(if reverse_mode {
                button::primary
            } else {
                button::secondary
            })
            .on_press(Message::ShowReverseCalc),
    ]
    .spacing(3)
    .into();

    let input_panel: Element<'_, Message> = if reverse_mode {
        scrollable(
            Column::from_vec(vec![
                mode_switcher,
                Rule::horizontal(1).into(),
                view_reverse_section(state),
            ])
            .width(Length::FillPortion(1))
            .height(Length::Shrink),
        )
        .into()
    } else {
        let mut children: Vec<Element<Message>> = vec![
            mode_switcher,
            Rule::horizontal(1).into(),
            view_loan_section(state),
            Rule::horizontal(1).into(),
            view_rate_section(state),
        ];
        if state.rate_mode == "Euribor" || state.rate_mode == "Mixed" {
            children.push(Rule::horizontal(1).into());
            children.push(view_euribor_manual_section(state));
        }
        children.push(Rule::horizontal(1).into());
        children.push(view_prepay_section(state));
        children.push(Rule::horizontal(1).into());
        children.push(view_actions_section(state));
        if state.calendar_target.is_some() {
            children.push(Rule::horizontal(1).into());
            children.push(calendar_widget(&state.calendar_state));
        }
        scrollable(
            Column::from_vec(children)
                .width(Length::FillPortion(1))
                .height(Length::Shrink),
        )
        .into()
    };

    let results_panel = container(view_results_panel(state)).width(Length::FillPortion(3));

    container(row![input_panel, results_panel].spacing(10).padding(2))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::Start)
        .into()
}

fn view_loan_section(state: &State) -> Element<'_, Message> {
    let currencies = vec!["EUR".to_string(), "USD".to_string()];
    let payment_types = vec!["Annuity".to_string(), "Diff".to_string()];

    column![
        section_header("Loan"),
        input_row(
            "Amount:",
            validated_input(
                "185000",
                &state.amount,
                Message::AmountChanged,
                state.amount.parse::<f64>().is_ok()
            )
        ),
        input_row(
            "Down:",
            compact_input("0", &state.down_payment, Message::DownPaymentChanged),
        ),
        input_row(
            "Term:",
            validated_input(
                "30",
                &state.term,
                Message::TermChanged,
                state.term.parse::<u32>().is_ok()
            )
        ),
        {
            let date_input: Element<'_, Message> = validated_input(
                "01-01-2025",
                &state.start_date,
                Message::StartDateChanged,
                chrono::NaiveDate::parse_from_str(&state.start_date, "%d-%m-%Y").is_ok(),
            );
            let cal_btn = button(text("...").size(12))
                .padding(1)
                .on_press(Message::OpenCalendar(DateField::StartDate));
            let r = row![
                text("Start:").size(16).width(Length::Fixed(110.0)),
                date_input,
                cal_btn,
            ]
            .spacing(3)
            .align_y(Alignment::Center);
            Element::from(r)
        },
        input_row(
            "Curr:",
            pick_list(
                currencies,
                Some(state.currency.clone()),
                Message::CurrencyChanged
            )
            .text_size(16)
            .into()
        ),
        input_row(
            "Type:",
            pick_list(
                payment_types,
                Some(state.payment_type.clone()),
                Message::PaymentTypeChanged
            )
            .text_size(16)
            .into()
        ),
    ]
    .spacing(0)
    .padding(0)
    .into()
}

fn view_rate_section(state: &State) -> Element<'_, Message> {
    let rate_modes = vec![
        "Fix".to_string(),
        "Euribor".to_string(),
        "Mixed".to_string(),
    ];
    let tenors = vec![
        "1m".to_string(),
        "3m".to_string(),
        "6m".to_string(),
        "12m".to_string(),
    ];

    let mut fields: Vec<Element<'_, Message>> = vec![section_header("Rate")];
    fields.push(input_row(
        "Mode:",
        pick_list(
            rate_modes,
            Some(state.rate_mode.clone()),
            Message::RateModeChanged,
        )
        .text_size(16)
        .into(),
    ));

    match state.rate_mode.as_str() {
        "Fix" => {
            fields.push(input_row(
                "Rate:",
                compact_input("3.6", &state.rate, Message::RateChanged),
            ));
            fields.push(input_row(
                "Spread:",
                compact_input("0.0", &state.spread, Message::SpreadChanged),
            ));
        }
        "Euribor" => {
            fields.push(input_row(
                "Tenor:",
                pick_list(
                    tenors.clone(),
                    Some(state.euribor_tenor.clone()),
                    Message::EuriborTenorChanged,
                )
                .text_size(16)
                .into(),
            ));
            fields.push(input_row(
                "Spread:",
                compact_input("1.0", &state.euribor_spread, Message::EuriborSpreadChanged),
            ));
        }
        "Mixed" => {
            fields.push(input_row(
                "Fix yrs:",
                compact_input("2", &state.mixed_fix_years, Message::MixedFixYearsChanged),
            ));
            fields.push(input_row(
                "Fix rate:",
                compact_input("3.0", &state.mixed_fix_rate, Message::MixedFixRateChanged),
            ));
            fields.push(input_row(
                "Fix spr:",
                compact_input(
                    "1.0",
                    &state.mixed_fix_spread,
                    Message::MixedFixSpreadChanged,
                ),
            ));
            fields.push(input_row(
                "Euri tnr:",
                pick_list(
                    tenors.clone(),
                    Some(state.mixed_euribor_tenor.clone()),
                    Message::MixedEuriborTenorChanged,
                )
                .text_size(16)
                .into(),
            ));
            if !state.same_spread {
                fields.push(input_row(
                    "Euri spr:",
                    compact_input(
                        "1.5",
                        &state.mixed_euribor_spread,
                        Message::MixedEuriborSpreadChanged,
                    ),
                ));
            }
            fields.push(input_row(
                "Same spr:",
                checkbox("", state.same_spread)
                    .on_toggle(Message::SameSpreadToggled)
                    .into(),
            ));
        }
        _ => {}
    }
    Column::from_vec(fields).spacing(0).padding(0).into()
}

fn view_prepay_section(state: &State) -> Element<'_, Message> {
    let effects = vec!["ReduceTerm".to_string(), "ReducePayment".to_string()];

    let mut fields: Vec<Element<'_, Message>> = vec![section_header("Prepay")];
    fields.push({
        let date_input = compact_input(
            "01-01-2027",
            &state.prepayment_date,
            Message::PrepaymentDateChanged,
        );
        let cal_btn = button(text("...").size(12))
            .padding(1)
            .on_press(Message::OpenCalendar(DateField::PrepaymentDate));
        let r = row![
            text("Date:").size(16).width(Length::Fixed(110.0)),
            date_input,
            cal_btn,
        ]
        .spacing(3)
        .align_y(Alignment::Center);
        Element::from(r)
    });
    fields.push(input_row(
        "Amt:",
        compact_input(
            "20000",
            &state.prepayment_amount,
            Message::PrepaymentAmountChanged,
        ),
    ));
    fields.push(input_row(
        "Effect:",
        pick_list(
            effects,
            Some(state.prepayment_effect.clone()),
            Message::PrepaymentEffectChanged,
        )
        .text_size(16)
        .into(),
    ));
    fields.push(
        button(" +Add ")
            .padding(2)
            .on_press(Message::AddPrepayment)
            .into(),
    );

    for (i, prep) in state.prepayments.iter().enumerate() {
        fields.push(
            row![
                text(format!(
                    "  #{}: {} {:.0} {}",
                    i + 1,
                    prep.date,
                    prep.amount,
                    prep.effect
                ))
                .size(16)
                .width(Length::Fill),
                button(" X")
                    .padding(0)
                    .on_press(Message::RemovePrepayment(i)),
            ]
            .spacing(5)
            .align_y(Alignment::Center)
            .into(),
        );
    }
    Column::from_vec(fields).spacing(0).padding(0).into()
}

fn view_actions_section(state: &State) -> Element<'_, Message> {
    let _ = state;
    column![
        section_header("Actions"),
        row![
            button(" Calc ").padding(0).on_press(Message::Calculate),
            button(" CSV ").padding(0).on_press(Message::ExportCsv),
            button(" PDF ").padding(0).on_press(Message::ExportPdf),
        ]
        .spacing(3),
        row![
            button(" Save ").padding(0).on_press(Message::SaveSession),
            button(" Load ").padding(0).on_press(Message::LoadSession),
        ]
        .spacing(3),
    ]
    .spacing(0)
    .padding(0)
    .into()
}

fn view_euribor_manual_section(state: &State) -> Element<'_, Message> {
    let mut fields: Vec<Element<'_, Message>> = vec![section_header("Euribor History")];

    fields.push(input_row(
        "Use manual:",
        checkbox("", state.use_manual_euribor)
            .on_toggle(Message::ToggleManualEuribor)
            .into(),
    ));

    for (i, (date, rate)) in state.euribor_manual_points.iter().enumerate() {
        fields.push(
            row![
                text(format!("  #{}: {} {}%", i + 1, date, rate))
                    .size(16)
                    .width(Length::Fill),
                button(" X")
                    .padding(0)
                    .on_press(Message::RemoveEuriborManualPoint(i)),
            ]
            .spacing(5)
            .align_y(Alignment::Center)
            .into(),
        );
    }

    fields.push(input_row(
        "Date:",
        compact_input(
            "DD-MM-YYYY",
            &state.euribor_new_date,
            Message::EuriborNewDateChanged,
        ),
    ));
    fields.push(input_row(
        "Rate:",
        compact_input(
            "2.5",
            &state.euribor_new_rate,
            Message::EuriborNewRateChanged,
        ),
    ));
    fields.push(
        button(" +Add ")
            .padding(0)
            .on_press(Message::AddEuriborManualPoint)
            .into(),
    );
    fields.push(
        row![
            button(" Fetch ").padding(0).on_press(Message::FetchEuribor),
            button(" Save ")
                .padding(0)
                .on_press(Message::SaveEuriborPoints),
            button(" Load ")
                .padding(0)
                .on_press(Message::LoadEuriborPoints),
        ]
        .spacing(3)
        .into(),
    );

    Column::from_vec(fields).spacing(0).padding(0).into()
}

fn view_reverse_section(state: &State) -> Element<'_, Message> {
    let payment_types = vec!["Annuity".to_string(), "Diff".to_string()];
    let rate_modes = vec!["Fix".to_string(), "Euribor".to_string()];
    let tenors = vec![
        "1m".to_string(),
        "3m".to_string(),
        "6m".to_string(),
        "12m".to_string(),
    ];

    let mut fields: Vec<Element<'_, Message>> = vec![section_header("Reverse Calc")];

    fields.push(input_row(
        "Target:",
        compact_input(
            "1000",
            &state.reverse_target_payment,
            Message::ReverseTargetChanged,
        ),
    ));

    fields.push(input_row(
        "Type:",
        pick_list(
            payment_types,
            Some(state.reverse_payment_type.clone()),
            Message::ReversePaymentTypeChanged,
        )
        .text_size(16)
        .into(),
    ));

    fields.push(input_row(
        "Mode:",
        pick_list(
            rate_modes,
            Some(state.reverse_rate_mode.clone()),
            Message::ReverseRateModeChanged,
        )
        .text_size(16)
        .into(),
    ));

    match state.reverse_rate_mode.as_str() {
        "Fix" => {
            fields.push(input_row(
                "Rate:",
                compact_input(
                    "3.6",
                    &state.reverse_fix_rate,
                    Message::ReverseFixRateChanged,
                ),
            ));
            fields.push(input_row(
                "Spread:",
                compact_input(
                    "0.0",
                    &state.reverse_fix_spread,
                    Message::ReverseFixSpreadChanged,
                ),
            ));
        }
        "Euribor" => {
            fields.push(input_row(
                "Tenor:",
                pick_list(
                    tenors.clone(),
                    Some(state.reverse_euribor_tenor.clone()),
                    Message::ReverseEuriborTenorChanged,
                )
                .text_size(16)
                .into(),
            ));
            fields.push(input_row(
                "Spread:",
                compact_input(
                    "1.0",
                    &state.reverse_euribor_spread,
                    Message::ReverseEuriborSpreadChanged,
                ),
            ));
        }
        _ => {}
    }

    fields.push(input_row(
        "Extra/mo:",
        compact_input(
            "0",
            &state.reverse_extra_monthly,
            Message::ReverseExtraChanged,
        ),
    ));

    fields.push(
        button(" Calculate Max Loan ")
            .padding(2)
            .on_press(Message::ReverseCalculate)
            .into(),
    );

    Column::from_vec(fields).spacing(0).padding(0).into()
}

fn days_in_month_gui(year: i32, month: u32) -> u32 {
    chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|d| d.checked_add_months(chrono::Months::new(1)))
        .map(|next| {
            let prev = next - chrono::Duration::days(1);
            use chrono::Datelike;
            prev.day()
        })
        .unwrap_or(30)
}

fn calendar_widget(state: &GuiCalendarState) -> Element<'_, Message> {
    use chrono::Datelike;
    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let title = format!("{} {}", month_names[(state.month - 1) as usize], state.year);
    let days_count = days_in_month_gui(state.year, state.month);
    let first_wday = chrono::NaiveDate::from_ymd_opt(state.year, state.month, 1)
        .unwrap()
        .weekday()
        .num_days_from_monday();

    let mut grid = column![
        row![
            button("<").padding(1).on_press(Message::CalendarMonthPrev),
            text(title)
                .size(16)
                .width(Length::Fill)
                .align_x(iced::Center),
            button(">").padding(1).on_press(Message::CalendarMonthNext),
        ]
        .spacing(2),
        button("X").padding(1).on_press(Message::CloseCalendar),
    ]
    .spacing(2);

    let cell_width = Length::Fixed(40.0);

    let day_headers = row![
        container(text("Mo").size(16))
            .width(cell_width)
            .center_x(Length::Fill),
        container(text("Tu").size(16))
            .width(cell_width)
            .center_x(Length::Fill),
        container(text("We").size(16))
            .width(cell_width)
            .center_x(Length::Fill),
        container(text("Th").size(16))
            .width(cell_width)
            .center_x(Length::Fill),
        container(text("Fr").size(16))
            .width(cell_width)
            .center_x(Length::Fill),
        container(text("Sa").size(16))
            .width(cell_width)
            .center_x(Length::Fill),
        container(text("Su").size(16))
            .width(cell_width)
            .center_x(Length::Fill),
    ]
    .spacing(0);
    grid = grid.push(day_headers);

    let mut week: Vec<Element<Message>> = Vec::new();
    let mut col = 0u32;
    for _ in 0..first_wday {
        week.push(container(text("")).width(cell_width).into());
        col += 1;
    }
    for day in 1..=days_count {
        let is_sel = state.day == day;
        let day_text = if is_sel {
            button(text(format!("{:>2}", day)).size(16))
                .padding(0)
                .style(button::primary)
                .on_press(Message::CalendarDaySelect(day))
        } else {
            button(text(format!("{:>2}", day)).size(16))
                .padding(0)
                .on_press(Message::CalendarDaySelect(day))
        };
        week.push(
            container(day_text)
                .width(cell_width)
                .center_x(Length::Fill)
                .into(),
        );
        col += 1;
        if col == 7 {
            grid = grid.push(row![].extend(week.drain(..)).spacing(0));
            col = 0;
        }
    }
    if !week.is_empty() {
        grid = grid.push(row![].extend(week.drain(..)).spacing(0));
    }

    container(grid)
        .padding(4)
        .style(|_theme: &Theme| container::Style {
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.5, 0.7),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn view_results_panel(state: &State) -> Element<'_, Message> {
    if state.active_tab == ViewTab::ReverseCalc {
        let content = view_reverse_tab(state);
        let status_bar = view_status_bar(state);
        return column![content, status_bar].spacing(4).padding(4).into();
    }

    let Some(ref result) = state.result else {
        return container(text("Enter parameters and press Calculate"))
            .padding(10)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into();
    };

    let sym = if state.currency == "USD" { "$" } else { "€" };
    let summary = container(
        column![
            text("Results").size(16),
            text(format!(
                "Monthly: {}{:.2}",
                sym,
                result.monthly_payment.unwrap_or(0.0)
            )),
            text(format!(
                "Total Principal: {}{:.2}",
                sym, result.total_principal
            )),
            text(format!(
                "Total Interest: {}{:.2}",
                sym, result.total_interest
            )),
            text(format!("Total Paid: {}{:.2}", sym, result.total_paid)),
            text(format!("Payments: {}", result.payments.len())),
            if let Some(idx) = result.principal_exceeds_interest_at {
                text(format!(
                    "Principal > Interest at #{} ({})",
                    idx + 1,
                    result.payments[idx].date
                ))
            } else {
                text("")
            },
        ]
        .spacing(1),
    )
    .padding(4)
    .width(Length::Fill);

    let tabs = view_tabs(state);
    let content = view_tab_content(state, result);
    let status_bar = view_status_bar(state);

    column![summary, tabs, content, status_bar]
        .spacing(4)
        .padding(4)
        .into()
}

fn view_tabs(state: &State) -> Element<'_, Message> {
    let tab = |label, tab, active_tab| {
        button(label)
            .padding(2)
            .style(if active_tab == tab {
                button::primary
            } else {
                button::secondary
            })
            .on_press(match tab {
                ViewTab::Table => Message::ShowTable,
                ViewTab::Chart => Message::ShowChart,
                ViewTab::BalanceChart => Message::ShowBalanceChart,
                ViewTab::Yearly => Message::ShowYearly,
                ViewTab::Sensitivity => Message::ShowSensitivity,
                ViewTab::BreakEven => Message::ShowBreakEven,
                ViewTab::ReverseCalc => Message::ShowReverseCalc,
            })
    };

    row![
        tab("Table", ViewTab::Table, state.active_tab),
        tab("Stacked", ViewTab::Chart, state.active_tab),
        tab("Balance", ViewTab::BalanceChart, state.active_tab),
        tab("Yearly", ViewTab::Yearly, state.active_tab),
        tab("Sensitivity", ViewTab::Sensitivity, state.active_tab),
        tab("Break-Even", ViewTab::BreakEven, state.active_tab),
    ]
    .spacing(3)
    .into()
}

fn view_tab_content<'a>(state: &'a State, result: &'a LoanResult) -> Element<'a, Message> {
    match state.active_tab {
        ViewTab::Table => view_table_tab(result),
        ViewTab::Chart => {
            let toggle = row![
                button("X: Payment # / Date")
                    .padding(2)
                    .on_press(Message::ToggleXAxis),
            ]
            .spacing(3)
            .align_y(Alignment::Center);
            let chart = Canvas::new(StackedChart {
                payments: &result.payments,
                x_axis_mode: state.x_axis_mode,
                hovered: state.hovered_payment,
                cross_idx: result.principal_exceeds_interest_at,
            })
            .width(Length::Fill)
            .height(Length::Fill);
            column![toggle, chart].spacing(2).into()
        }
        ViewTab::BalanceChart => {
            let toggle = row![
                button("X: Payment # / Date")
                    .padding(2)
                    .on_press(Message::ToggleXAxis),
            ]
            .spacing(3)
            .align_y(Alignment::Center);
            let chart = Canvas::new(BalanceChart {
                payments: &result.payments,
                x_axis_mode: state.x_axis_mode,
                hovered: state.hovered_payment,
                cross_idx: result.principal_exceeds_interest_at,
            })
            .width(Length::Fill)
            .height(Length::Fill);
            column![toggle, chart].spacing(2).into()
        }
        ViewTab::Yearly => view_yearly_tab(result),
        ViewTab::Sensitivity => view_sensitivity_tab(state),
        ViewTab::BreakEven => view_break_even_tab(state),
        ViewTab::ReverseCalc => view_reverse_tab(state),
    }
}

fn view_table_tab(result: &LoanResult) -> Element<'_, Message> {
    let table_header = row![
        text("#").width(Length::Fixed(40.0)),
        text("Date").width(Length::Fixed(100.0)),
        text("Payment").width(Length::Fixed(100.0)),
        text("Principal").width(Length::Fixed(100.0)),
        text("Interest").width(Length::Fixed(100.0)),
        text("Balance").width(Length::Fixed(100.0)),
    ]
    .spacing(5);

    let mut table_rows: Vec<Element<Message>> = vec![table_header.into()];
    for (i, p) in result.payments.iter().enumerate() {
        table_rows.push(
            row![
                text(format!("{}", i + 1)).width(Length::Fixed(40.0)),
                text(p.date.to_string()).width(Length::Fixed(100.0)),
                text(format!("{:.2}", p.payment)).width(Length::Fixed(100.0)),
                text(format!("{:.2}", p.principal)).width(Length::Fixed(100.0)),
                text(format!("{:.2}", p.interest)).width(Length::Fixed(100.0)),
                text(format!("{:.2}", p.remaining_balance)).width(Length::Fixed(100.0)),
            ]
            .spacing(5)
            .into(),
        );
    }

    scrollable(container(Column::from_vec(table_rows).spacing(2)).padding(4)).into()
}

fn view_yearly_tab(result: &LoanResult) -> Element<'_, Message> {
    let summaries = result.yearly_summaries();
    let header = row![
        text("Year").width(Length::Fixed(60.0)),
        text("Payment").width(Length::Fixed(110.0)),
        text("Principal").width(Length::Fixed(110.0)),
        text("Interest").width(Length::Fixed(110.0)),
        text("Months").width(Length::Fixed(60.0)),
        text("Balance").width(Length::Fixed(110.0)),
    ]
    .spacing(5);

    let mut rows: Vec<Element<Message>> = vec![header.into()];
    for s in &summaries {
        rows.push(
            row![
                text(format!("{}", s.year)).width(Length::Fixed(60.0)),
                text(format!("{:.2}", s.total_payment)).width(Length::Fixed(110.0)),
                text(format!("{:.2}", s.total_principal)).width(Length::Fixed(110.0)),
                text(format!("{:.2}", s.total_interest)).width(Length::Fixed(110.0)),
                text(format!("{}", s.payments_count)).width(Length::Fixed(60.0)),
                text(format!("{:.2}", s.ending_balance)).width(Length::Fixed(110.0)),
            ]
            .spacing(5)
            .into(),
        );
    }

    scrollable(container(Column::from_vec(rows).spacing(2)).padding(4)).into()
}

fn view_sensitivity_tab(state: &State) -> Element<'_, Message> {
    let Some(ref params) = state.params else {
        return text("Calculate first to see sensitivity analysis").into();
    };

    let deltas: Vec<f64> = (-15..=15).map(|i| i as f64 / 10.0).collect();
    let points = mortgage_core::sensitivity_analysis(params, &deltas);
    let header = row![
        text("Delta").width(Length::Fixed(60.0)),
        text("Rate %").width(Length::Fixed(80.0)),
        text("Monthly").width(Length::Fixed(110.0)),
        text("Interest").width(Length::Fixed(110.0)),
        text("Total Paid").width(Length::Fixed(110.0)),
    ]
    .spacing(5);

    let mut rows: Vec<Element<Message>> = vec![header.into()];
    for p in &points {
        let monthly = p
            .monthly_payment
            .map(|m| format!("{:.2}", m))
            .unwrap_or_else(|| "N/A".to_string());
        rows.push(
            row![
                text(format!("{:+.2}", p.rate_delta)).width(Length::Fixed(60.0)),
                text(format!("{:.2}", p.effective_rate)).width(Length::Fixed(80.0)),
                text(monthly).width(Length::Fixed(110.0)),
                text(format!("{:.2}", p.total_interest)).width(Length::Fixed(110.0)),
                text(format!("{:.2}", p.total_paid)).width(Length::Fixed(110.0)),
            ]
            .spacing(5)
            .into(),
        );
    }

    scrollable(container(Column::from_vec(rows).spacing(2)).padding(4)).into()
}

fn view_break_even_tab(state: &State) -> Element<'_, Message> {
    let Some(ref params) = state.params else {
        return text("Calculate first to see break-even analysis").into();
    };

    let rent = state.rent.parse::<f64>().unwrap_or(0.0);
    let content = if rent > 0.0 {
        let mut params = params.clone();
        params.upfront_cost = state.upfront_cost.parse::<f64>().ok().filter(|&v| v != 0.0);
        params.upfront_percent = state
            .upfront_percent
            .parse::<f64>()
            .ok()
            .filter(|&v| v != 0.0);
        let be = mortgage_core::break_even_analysis(&params, rent);
        column![
            text("Break-Even vs Rent").size(16),
            text(format!("Monthly rent:      {:.2}", be.monthly_rent)),
            text(format!("Monthly mortgage:  {:.2}", be.monthly_cost)),
            text(format!("Upfront costs:     {:.2}", be.upfront_costs)),
            text(format!("Total interest:    {:.2}", be.total_interest)),
            text(""),
            if let (Some(months), Some(years)) = (be.break_even_months, be.break_even_years) {
                text(format!(
                    "Break-even:        {} months ({:.1} years)",
                    months, years
                ))
            } else {
                text("Break-even:        N/A")
            },
            text(""),
            text(be.explanation.clone()),
            text(""),
            input_row(
                "Monthly rent:",
                text_input("900", &state.rent)
                    .on_input(Message::RentChanged)
                    .width(Length::Fixed(150.0))
                    .into()
            ),
            input_row(
                "Upfront cost:",
                text_input("0", &state.upfront_cost)
                    .on_input(Message::UpfrontCostChanged)
                    .width(Length::Fixed(150.0))
                    .into()
            ),
            input_row(
                "Upfront %:",
                text_input("5", &state.upfront_percent)
                    .on_input(Message::UpfrontPercentChanged)
                    .width(Length::Fixed(150.0))
                    .into()
            ),
        ]
        .spacing(2)
    } else {
        column![
            text("Enter monthly rent for break-even analysis"),
            input_row(
                "Rent:",
                text_input("900", &state.rent)
                    .on_input(Message::RentChanged)
                    .width(Length::Fill)
                    .into(),
            ),
        ]
        .spacing(2)
    };

    container(content).padding(4).into()
}

fn view_reverse_tab(state: &State) -> Element<'_, Message> {
    let Some(ref rows) = state.reverse_result else {
        return text("Set parameters and press Calculate Max Loan").into();
    };

    let header = row![
        text("Term").width(Length::Fixed(60.0)),
        text("Max Amount").width(Length::Fixed(130.0)),
        text("Payment").width(Length::Fixed(110.0)),
        text("Extra").width(Length::Fixed(80.0)),
        text("TOTAL").width(Length::Fixed(110.0)),
    ]
    .spacing(5);

    let mut table_rows: Vec<Element<Message>> = vec![header.into()];
    for r in rows {
        table_rows.push(
            row![
                text(format!("{} yr", r.term_years)).width(Length::Fixed(60.0)),
                text(format!("{:.2}", r.max_amount)).width(Length::Fixed(130.0)),
                text(format!("{:.2}", r.monthly_payment)).width(Length::Fixed(110.0)),
                text(format!("{:.2}", r.extra_cost)).width(Length::Fixed(80.0)),
                text(format!("{:.2}", r.total_monthly)).width(Length::Fixed(110.0)),
            ]
            .spacing(5)
            .into(),
        );
    }

    scrollable(container(Column::from_vec(table_rows).spacing(2)).padding(4)).into()
}

fn view_status_bar(state: &State) -> Element<'_, Message> {
    container(text(&state.status))
        .padding(4)
        .width(Length::Fill)
        .style(|_theme: &Theme| {
            if state.status_is_error {
                container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.5, 0.1, 0.1,
                    ))),
                    ..Default::default()
                }
            } else if !state.status.is_empty() {
                container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(
                        0.1, 0.4, 0.1,
                    ))),
                    ..Default::default()
                }
            } else {
                container::Style::default()
            }
        })
        .into()
}

fn validated_input(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'static,
    valid: bool,
) -> Element<'static, Message> {
    let input = text_input(placeholder, value)
        .on_input(on_input)
        .padding(0)
        .size(16)
        .width(Length::Fill);
    if valid {
        input.into()
    } else {
        container(input)
            .style(|_theme: &Theme| container::Style {
                border: iced::Border {
                    color: iced::Color::from_rgb(0.8, 0.2, 0.2),
                    width: 2.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}

fn calculate(state: &mut State) {
    let amount = match state.amount.parse::<f64>() {
        Ok(v) => v,
        Err(_) => {
            state.status = "Invalid amount".to_string();
            state.status_is_error = true;
            return;
        }
    };
    let term_years = match state.term.parse::<u32>() {
        Ok(v) => v,
        Err(_) => {
            state.status = "Invalid term".to_string();
            state.status_is_error = true;
            return;
        }
    };
    let start_date = match chrono::NaiveDate::parse_from_str(&state.start_date, "%d-%m-%Y") {
        Ok(d) => d,
        Err(_) => {
            state.status = "Invalid start date (DD-MM-YYYY)".to_string();
            state.status_is_error = true;
            return;
        }
    };
    let currency = if state.currency == "USD" {
        Currency::Usd
    } else {
        Currency::Eur
    };
    let payment_type = if state.payment_type == "Diff" {
        PaymentType::Diff
    } else {
        PaymentType::Annuity
    };
    let rate_mode = match state.rate_mode.as_str() {
        "Fix" => RateMode::Fix {
            rate: state.rate.parse::<f64>().unwrap_or(3.6),
            spread: state.spread.parse::<f64>().unwrap_or(0.0),
        },
        "Euribor" => RateMode::Euribor {
            tenor: parse_tenor(&state.euribor_tenor),
            spread: state.euribor_spread.parse::<f64>().unwrap_or(1.0),
        },
        "Mixed" => RateMode::Mixed {
            fix_years: state.mixed_fix_years.parse::<f64>().unwrap_or(2.0),
            fix_rate: state.mixed_fix_rate.parse::<f64>().unwrap_or(3.0),
            fix_spread: state.mixed_fix_spread.parse::<f64>().unwrap_or(1.0),
            euribor_tenor: parse_tenor(&state.mixed_euribor_tenor),
            euribor_spread: if state.same_spread {
                state.mixed_fix_spread.parse::<f64>().unwrap_or(1.0)
            } else {
                state.mixed_euribor_spread.parse::<f64>().unwrap_or(1.5)
            },
        },
        _ => RateMode::Fix {
            rate: 3.6,
            spread: 0.0,
        },
    };
    let euribor_curve = if state.use_manual_euribor && !state.euribor_manual_points.is_empty() {
        state
            .euribor_manual_points
            .iter()
            .filter_map(|(d, r)| {
                let date = chrono::NaiveDate::parse_from_str(d, "%d-%m-%Y").ok()?;
                let rate = r.parse::<f64>().ok()?;
                Some(EuriborPoint {
                    date_from: date,
                    rate,
                })
            })
            .collect()
    } else {
        vec![]
    };
    let params = LoanParams {
        amount,
        term_years,
        payment_type,
        currency,
        start_date,
        rate_mode,
        same_spread: state.same_spread,
        euribor_curve,
        prepayments: state.prepayments.clone(),
        upfront_cost: state.upfront_cost.parse::<f64>().ok().filter(|&v| v != 0.0),
        upfront_percent: state
            .upfront_percent
            .parse::<f64>()
            .ok()
            .filter(|&v| v != 0.0),
        down_payment: state.down_payment.parse::<f64>().ok().filter(|&v| v != 0.0),
    };
    match Calculator::calculate(&params) {
        Ok(result) => {
            state.params = Some(params);
            state.result = Some(result);
            state.hovered_payment = None;
            state.status = "Calculation complete".to_string();
            state.status_is_error = false;
        }
        Err(e) => {
            state.status = format!("Error: {}", e);
            state.status_is_error = true;
        }
    }
}

fn reverse_calculate(state: &mut State) {
    let target = match state.reverse_target_payment.parse::<f64>() {
        Ok(v) if v > 0.0 => v,
        _ => {
            state.status = "Invalid target payment".to_string();
            state.status_is_error = true;
            return;
        }
    };

    let extra = state.reverse_extra_monthly.parse::<f64>().unwrap_or(0.0);
    if extra < 0.0 {
        state.status = "Extra cost cannot be negative".to_string();
        state.status_is_error = true;
        return;
    }

    let payment_type = if state.reverse_payment_type == "Diff" {
        PaymentType::Diff
    } else {
        PaymentType::Annuity
    };

    let annual_rate = match state.reverse_rate_mode.as_str() {
        "Fix" => {
            let rate = state.reverse_fix_rate.parse::<f64>().unwrap_or(3.6);
            let spread = state.reverse_fix_spread.parse::<f64>().unwrap_or(0.0);
            rate + spread
        }
        "Euribor" => {
            let tenor: EuriborTenor = state
                .reverse_euribor_tenor
                .parse()
                .unwrap_or(EuriborTenor::SixMonths);
            let spread = state.reverse_euribor_spread.parse::<f64>().unwrap_or(1.0);
            match state.euribor_cache.get_or_fetch(tenor) {
                Ok(rate) => {
                    state.status = format!("Euribor {}: {:.3}%", tenor, rate);
                    state.status_is_error = false;
                    rate + spread
                }
                Err(e) => {
                    state.status = format!("Euribor fetch failed: {}", e);
                    state.status_is_error = true;
                    return;
                }
            }
        }
        _ => {
            state.status = "Unknown rate mode".to_string();
            state.status_is_error = true;
            return;
        }
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

    state.reverse_result = Some(rows);
    state.status = "Reverse calculation complete".to_string();
    state.status_is_error = false;
}

fn save_session_gui(state: &mut State) {
    if let (Some(params), Some(result)) = (&state.params, &state.result) {
        match mortgage_core::save_session("/tmp/mortgage_session.json", params, result) {
            Ok(()) => {
                state.status = "Session saved to /tmp/mortgage_session.json".to_string();
                state.status_is_error = false;
            }
            Err(e) => {
                state.status = format!("Save failed: {}", e);
                state.status_is_error = true;
            }
        }
    } else {
        state.status = "No results to save. Calculate first.".to_string();
        state.status_is_error = true;
    }
}

fn load_session_gui(state: &mut State) {
    match mortgage_core::load_session("/tmp/mortgage_session.json") {
        Ok(session) => {
            state.amount = format!("{}", session.params.amount);
            state.term = format!("{}", session.params.term_years);
            state.start_date = session.params.start_date.format("%d-%m-%Y").to_string();
            state.currency = match session.params.currency {
                Currency::Usd => "USD".to_string(),
                Currency::Eur => "EUR".to_string(),
            };
            state.payment_type = match session.params.payment_type {
                PaymentType::Annuity => "Annuity".to_string(),
                PaymentType::Diff => "Diff".to_string(),
            };
            state.same_spread = session.params.same_spread;
            match &session.params.rate_mode {
                RateMode::Fix { rate, spread } => {
                    state.rate_mode = "Fix".to_string();
                    state.rate = format!("{}", rate);
                    state.spread = format!("{}", spread);
                }
                RateMode::Euribor { tenor, spread } => {
                    state.rate_mode = "Euribor".to_string();
                    state.euribor_tenor = tenor.as_str().to_string();
                    state.euribor_spread = format!("{}", spread);
                }
                RateMode::Mixed {
                    fix_years,
                    fix_rate,
                    fix_spread,
                    euribor_tenor,
                    euribor_spread,
                } => {
                    state.rate_mode = "Mixed".to_string();
                    state.mixed_fix_years = format!("{}", fix_years);
                    state.mixed_fix_rate = format!("{}", fix_rate);
                    state.mixed_fix_spread = format!("{}", fix_spread);
                    state.mixed_euribor_tenor = euribor_tenor.as_str().to_string();
                    state.mixed_euribor_spread = format!("{}", euribor_spread);
                }
            }
            state.prepayments = session.params.prepayments.clone();
            state.down_payment = session
                .params
                .down_payment
                .map(|v| format!("{}", v))
                .unwrap_or_else(|| "0".to_string());
            state.upfront_cost = session
                .params
                .upfront_cost
                .map(|v| format!("{}", v))
                .unwrap_or_else(|| "0".to_string());
            state.upfront_percent = session
                .params
                .upfront_percent
                .map(|v| format!("{}", v))
                .unwrap_or_else(|| "5".to_string());
            state.params = Some(session.params);
            state.result = Some(session.result);
            state.hovered_payment = None;
            state.status = "Session loaded successfully".to_string();
            state.status_is_error = false;
        }
        Err(e) => {
            state.status = format!("Load failed: {}", e);
            state.status_is_error = true;
        }
    }
}

fn export_csv(state: &mut State) {
    if let Some(ref result) = state.result {
        let csv = payments_to_csv(&result.payments);
        if let Err(e) = fs::write("/tmp/mortgage_payments.csv", csv) {
            state.status = format!("Export failed: {}", e);
            state.status_is_error = true;
        } else {
            state.status = "Saved to /tmp/mortgage_payments.csv".to_string();
            state.status_is_error = false;
        }
    } else {
        state.status = "No results to export. Calculate first.".to_string();
        state.status_is_error = true;
    }
}

fn export_pdf(state: &mut State) {
    use ::image::open as image_open;
    use printpdf::*;
    use std::io::BufWriter;

    if let Some(ref result) = state.result {
        let (doc, page1, layer1) =
            PdfDocument::new("Mortgage Report", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        let font = match doc.add_builtin_font(BuiltinFont::Helvetica) {
            Ok(f) => f,
            Err(e) => {
                state.status = format!("Font error: {}", e);
                state.status_is_error = true;
                return;
            }
        };

        let mut y = Mm(280.0);
        let line_height = Mm(6.0);

        let write_line = |layer: &PdfLayerReference, text: &str, y: Mm| {
            layer.use_text(text, 10.0, Mm(20.0), y, &font);
        };

        write_line(&current_layer, "Mortgage Loan Report", y);
        y -= line_height * 2.0;

        let sym = if state.currency == "USD" { "$" } else { "€" };
        write_line(
            &current_layer,
            &format!(
                "Monthly Payment: {}{:.2}",
                sym,
                result.monthly_payment.unwrap_or(0.0)
            ),
            y,
        );
        y -= line_height;
        write_line(
            &current_layer,
            &format!("Total Principal: {}{:.2}", sym, result.total_principal),
            y,
        );
        y -= line_height;
        write_line(
            &current_layer,
            &format!("Total Interest: {}{:.2}", sym, result.total_interest),
            y,
        );
        y -= line_height;
        write_line(
            &current_layer,
            &format!("Total Paid: {}{:.2}", sym, result.total_paid),
            y,
        );
        y -= line_height;
        write_line(
            &current_layer,
            &format!("Payments Count: {}", result.payments.len()),
            y,
        );
        if let Some(idx) = result.principal_exceeds_interest_at {
            y -= line_height;
            write_line(
                &current_layer,
                &format!(
                    "Principal > Interest at payment #{} ({})",
                    idx + 1,
                    result.payments[idx].date
                ),
                y,
            );
        }
        y -= line_height * 2.0;

        write_line(&current_layer, "Payment Schedule (first 60):", y);
        y -= line_height;

        for (i, p) in result.payments.iter().take(60).enumerate() {
            let line = format!(
                "{:>3} | {} | {:>10.2} | {:>10.2} | {:>10.2} | {:>12.2}",
                i + 1,
                p.date,
                p.payment,
                p.principal,
                p.interest,
                p.remaining_balance
            );
            write_line(&current_layer, &line, y);
            y -= line_height;
        }

        let png_bytes = match chart::generate_stacked_bar_chart_png(result) {
            Ok(bytes) => bytes,
            Err(e) => {
                state.status = format!("PNG generation error: {}", e);
                state.status_is_error = true;
                return;
            }
        };
        let png_path = "/tmp/mortgage_chart.png";
        if let Err(e) = fs::write(png_path, &png_bytes) {
            state.status = format!("PNG write error: {}", e);
            state.status_is_error = true;
            return;
        }

        let dynamic_image = match image_open(png_path) {
            Ok(img) => img,
            Err(e) => {
                state.status = format!("PNG open error: {}", e);
                state.status_is_error = true;
                return;
            }
        };
        let chart_image = Image::from_dynamic_image(&dynamic_image);
        let (page2, layer2) = doc.add_page(Mm(210.0), Mm(297.0), "Chart Layer");
        let chart_layer = doc.get_page(page2).get_layer(layer2);
        chart_image.add_to_layer(
            chart_layer,
            ImageTransform {
                translate_x: Some(Mm(20.0)),
                translate_y: Some(Mm(120.0)),
                dpi: Some(150.0),
                ..Default::default()
            },
        );

        let path = "/tmp/mortgage_report.pdf";
        if let Ok(file) = fs::File::create(path) {
            let mut writer = BufWriter::new(file);
            if doc.save(&mut writer).is_ok() {
                state.status = format!("Saved PDF to {}", path);
                state.status_is_error = false;
            } else {
                state.status = "PDF save failed".to_string();
                state.status_is_error = true;
            }
        } else {
            state.status = "PDF file creation failed".to_string();
            state.status_is_error = true;
        }
    } else {
        state.status = "No results to export. Calculate first.".to_string();
        state.status_is_error = true;
    }
}

fn filter_date_input(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return String::new();
    }
    let mut result = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i == 0 {
            if matches!(ch, '0'..='3') {
                result.push(ch);
            } else {
                result.push('0');
                result.push(ch);
            }
        } else if i == 1 {
            let first = digits.chars().next().unwrap_or('0');
            let day: u32 = format!("{}{}", first, ch).parse().unwrap_or(0);
            if (1..=31).contains(&day) {
                result.push(ch);
                result.push('-');
            } else {
                result.clear();
                result.push('0');
                result.push(ch);
                result.push('-');
            }
        } else if i == 2 {
            if matches!(ch, '0'..='1') {
                result.push(ch);
            } else {
                result.push('0');
                result.push(ch);
            }
        } else if i == 3 {
            result.push(ch);
            result.push('-');
        } else {
            result.push(ch);
        }
        if result.len() >= 10 {
            break;
        }
    }
    result
}

// ── Canvas chart programs ──────────────────────────────────────────

struct StackedChart<'a> {
    payments: &'a [Payment],
    x_axis_mode: XAxisMode,
    hovered: Option<usize>,
    cross_idx: Option<usize>,
}

impl StackedChart<'_> {
    const ML: f32 = 65.0;
    const MR: f32 = 20.0;
    const MT: f32 = 10.0;
    const MB: f32 = 45.0;

    fn chart_area(&self, bounds: iced::Rectangle) -> iced::Rectangle {
        iced::Rectangle {
            x: Self::ML,
            y: Self::MT,
            width: (bounds.width - Self::ML - Self::MR).max(1.0),
            height: (bounds.height - Self::MT - Self::MB).max(1.0),
        }
    }
}

impl Program<Message> for StackedChart<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let chart = self.chart_area(bounds);
        let n = self.payments.len();
        if n == 0 {
            return vec![frame.into_geometry()];
        }

        let max_payment = self
            .payments
            .iter()
            .map(|p| p.principal + p.interest)
            .fold(0.0, f64::max)
            * 1.1;
        if max_payment <= 0.0 {
            return vec![frame.into_geometry()];
        }
        let max_y = max_payment as f32;

        // Grid & axis lines
        let grid_color = Color::from_rgb(0.85, 0.85, 0.85);
        let axis_color = Color::from_rgb(0.4, 0.4, 0.4);
        let text_color = Color::from_rgb(0.2, 0.2, 0.2);
        let steps = 5;
        for i in 0..=steps {
            let y = chart.y + chart.height * (1.0 - i as f32 / steps as f32);
            let path = canvas::Path::new(|b| {
                b.move_to(Point::new(chart.x, y));
                b.line_to(Point::new(chart.x + chart.width, y));
            });
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_color(grid_color)
                    .with_width(1.0),
            );
            // Y-axis label
            let value = max_y * i as f32 / steps as f32;
            frame.fill_text(canvas::Text {
                content: if value >= 1000.0 {
                    format!("{:.1}K", value / 1000.0)
                } else {
                    format!("{:.0}", value)
                },
                position: Point::new(chart.x - 6.0, y),
                color: text_color,
                size: Pixels(11.0),
                horizontal_alignment: Horizontal::Right,
                vertical_alignment: Vertical::Center,
                ..Default::default()
            });
        }

        // X-axis labels
        let x_step = (n / 20).max(1);
        for i in (0..n).step_by(x_step) {
            let x = chart.x + (i as f32 / (n - 1).max(1) as f32) * chart.width;
            let label = match self.x_axis_mode {
                XAxisMode::PaymentNumber => format!("{}", i + 1),
                XAxisMode::Date => self.payments[i].date.format("%b %Y").to_string(),
            };
            frame.fill_text(canvas::Text {
                content: label,
                position: Point::new(x, chart.y + chart.height + 5.0),
                color: text_color,
                size: Pixels(10.0),
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Top,
                ..Default::default()
            });
        }

        // Axis outline
        let axis_path = canvas::Path::new(|b| {
            b.move_to(Point::new(chart.x, chart.y));
            b.line_to(Point::new(chart.x, chart.y + chart.height));
            b.line_to(Point::new(chart.x + chart.width, chart.y + chart.height));
        });
        frame.stroke(
            &axis_path,
            canvas::Stroke::default()
                .with_color(axis_color)
                .with_width(1.0),
        );

        // Axis title
        frame.fill_text(canvas::Text {
            content: match self.x_axis_mode {
                XAxisMode::PaymentNumber => "Payment #".into(),
                XAxisMode::Date => "Date".into(),
            },
            position: Point::new(chart.x + chart.width / 2.0, chart.y + chart.height + 28.0),
            color: text_color,
            size: Pixels(12.0),
            horizontal_alignment: Horizontal::Center,
            vertical_alignment: Vertical::Top,
            ..Default::default()
        });

        // Bars
        let step = chart.width / (n - 1).max(1) as f32;
        let bar_w = step * 0.8;
        for (i, p) in self.payments.iter().enumerate() {
            let bx = chart.x + i as f32 * step - bar_w / 2.0;
            let ph = (p.principal as f32 / max_y) * chart.height;
            let ih = (p.interest as f32 / max_y) * chart.height;
            if ph > 0.0 {
                frame.fill_rectangle(
                    Point::new(bx, chart.y + chart.height - ph),
                    Size::new(bar_w, ph),
                    Color::from_rgb(0.2, 0.7, 0.2),
                );
            }
            if ih > 0.0 {
                frame.fill_rectangle(
                    Point::new(bx, chart.y + chart.height - ph - ih),
                    Size::new(bar_w, ih),
                    Color::from_rgb(0.8, 0.2, 0.2),
                );
            }
        }

        // Pink dot at principal-exceeds-interest crossover
        if let Some(cross_idx) = self.cross_idx
            && cross_idx < n
        {
            let p = &self.payments[cross_idx];
            let cx = chart.x + cross_idx as f32 * step;
            let ph = (p.principal as f32 / max_y) * chart.height;
            let ih = (p.interest as f32 / max_y) * chart.height;
            let cy = chart.y + chart.height - ph - ih;
            let circle = canvas::Path::new(|b| {
                b.circle(Point::new(cx, cy), 5.0);
            });
            frame.fill(&circle, Color::from_rgb(1.0, 0.4, 0.7));
        }

        // Hover highlight
        if let Some(idx) = self.hovered
            && idx < n
        {
            let bx = chart.x + idx as f32 * step - bar_w / 2.0;
            frame.fill_rectangle(
                Point::new(bx, chart.y),
                Size::new(bar_w, chart.height),
                Color::from_rgba(0.0, 0.0, 1.0, 0.08),
            );
        }

        // Legend
        let leg_x = chart.x + chart.width - 100.0;
        let leg_y = chart.y + 5.0;
        frame.fill_rectangle(
            Point::new(leg_x, leg_y),
            Size::new(95.0, 38.0),
            Color::from_rgba(1.0, 1.0, 1.0, 0.85),
        );
        let leg_border = canvas::Path::new(|b| {
            b.move_to(Point::new(leg_x, leg_y));
            b.line_to(Point::new(leg_x + 95.0, leg_y));
            b.line_to(Point::new(leg_x + 95.0, leg_y + 38.0));
            b.line_to(Point::new(leg_x, leg_y + 38.0));
            b.close();
        });
        frame.stroke(
            &leg_border,
            canvas::Stroke::default()
                .with_color(grid_color)
                .with_width(0.5),
        );
        frame.fill_rectangle(
            Point::new(leg_x + 5.0, leg_y + 4.0),
            Size::new(12.0, 12.0),
            Color::from_rgb(0.2, 0.7, 0.2),
        );
        frame.fill_text(canvas::Text {
            content: "Principal".into(),
            position: Point::new(leg_x + 21.0, leg_y + 3.0),
            color: text_color,
            size: Pixels(11.0),
            ..Default::default()
        });
        frame.fill_rectangle(
            Point::new(leg_x + 5.0, leg_y + 21.0),
            Size::new(12.0, 12.0),
            Color::from_rgb(0.8, 0.2, 0.2),
        );
        frame.fill_text(canvas::Text {
            content: "Interest".into(),
            position: Point::new(leg_x + 21.0, leg_y + 20.0),
            color: text_color,
            size: Pixels(11.0),
            ..Default::default()
        });

        // Tooltip
        if let Some(idx) = self.hovered
            && idx < n
        {
            let p = &self.payments[idx];
            let bx = chart.x + idx as f32 * step - bar_w / 2.0;
            let lines = [
                format!("#{} ({})", idx + 1, p.date.format("%b %Y")),
                format!("Principal: {:.2}", p.principal),
                format!("Interest: {:.2}", p.interest),
                format!("Total: {:.2}", p.payment),
            ];
            let char_w = 6.6;
            let pad_x = 5.0;
            let pad_y = 2.0;
            let line_h = 16.0;
            let bottom_pad = 6.0;
            let max_w = lines
                .iter()
                .map(|l| l.len() as f32 * char_w)
                .fold(0.0, f32::max);
            let tw = max_w + 2.0 * pad_x;
            let th = lines.len() as f32 * line_h + pad_y + bottom_pad;
            let tx = (bx + bar_w / 2.0 - tw / 2.0)
                .max(chart.x + 5.0)
                .min((chart.x + chart.width - tw).max(chart.x + 5.0));
            let ty = chart.y + 2.0;
            frame.fill_rectangle(
                Point::new(tx, ty),
                Size::new(tw, th),
                Color::from_rgba(1.0, 1.0, 0.95, 1.0),
            );
            let tt_border = canvas::Path::new(|b| {
                b.move_to(Point::new(tx, ty));
                b.line_to(Point::new(tx + tw, ty));
                b.line_to(Point::new(tx + tw, ty + th));
                b.line_to(Point::new(tx, ty + th));
                b.close();
            });
            frame.stroke(
                &tt_border,
                canvas::Stroke::default()
                    .with_color(axis_color)
                    .with_width(1.0),
            );
            let colors = [
                Color::from_rgb(0.0, 0.0, 0.0),
                Color::from_rgb(0.2, 0.6, 0.2),
                Color::from_rgb(0.8, 0.2, 0.2),
                text_color,
            ];
            for (i, content) in lines.iter().enumerate() {
                frame.fill_text(canvas::Text {
                    content: content.clone(),
                    position: Point::new(tx + pad_x, ty + pad_y + i as f32 * line_h),
                    color: colors[i],
                    size: Pixels(11.0),
                    ..Default::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position() {
                    let chart = self.chart_area(bounds);
                    let cx = bounds.x + chart.x;
                    let cy = bounds.y + chart.y;
                    if pos.x >= cx
                        && pos.x <= cx + chart.width
                        && pos.y >= cy
                        && pos.y <= cy + chart.height
                    {
                        let n = self.payments.len();
                        if n > 0 {
                            let idx = ((pos.x - cx) / chart.width * n as f32) as usize;
                            let idx = idx.min(n - 1);
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::StackedChartMouseMoved(idx)),
                            );
                        }
                    }
                }
                (
                    canvas::event::Status::Captured,
                    Some(Message::StackedChartMouseLeft),
                )
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) => (
                canvas::event::Status::Captured,
                Some(Message::StackedChartMouseLeft),
            ),
            _ => (canvas::event::Status::Ignored, None),
        }
    }
}

struct BalanceChart<'a> {
    payments: &'a [Payment],
    x_axis_mode: XAxisMode,
    hovered: Option<usize>,
    cross_idx: Option<usize>,
}

impl BalanceChart<'_> {
    const ML: f32 = 65.0;
    const MR: f32 = 20.0;
    const MT: f32 = 10.0;
    const MB: f32 = 45.0;

    fn chart_area(&self, bounds: iced::Rectangle) -> iced::Rectangle {
        iced::Rectangle {
            x: Self::ML,
            y: Self::MT,
            width: (bounds.width - Self::ML - Self::MR).max(1.0),
            height: (bounds.height - Self::MT - Self::MB).max(1.0),
        }
    }
}

impl Program<Message> for BalanceChart<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let chart = self.chart_area(bounds);
        let n = self.payments.len();
        if n == 0 {
            return vec![frame.into_geometry()];
        }

        let max_balance = self
            .payments
            .iter()
            .map(|p| p.remaining_balance)
            .fold(0.0, f64::max)
            * 1.1;
        if max_balance <= 0.0 {
            return vec![frame.into_geometry()];
        }
        let max_y = max_balance as f32;

        // Grid & axis labels
        let grid_color = Color::from_rgb(0.85, 0.85, 0.85);
        let axis_color = Color::from_rgb(0.4, 0.4, 0.4);
        let text_color = Color::from_rgb(0.2, 0.2, 0.2);
        let steps = 5;
        for i in 0..=steps {
            let y = chart.y + chart.height * (1.0 - i as f32 / steps as f32);
            let path = canvas::Path::new(|b| {
                b.move_to(Point::new(chart.x, y));
                b.line_to(Point::new(chart.x + chart.width, y));
            });
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_color(grid_color)
                    .with_width(1.0),
            );
            let value = max_y * i as f32 / steps as f32;
            frame.fill_text(canvas::Text {
                content: if value >= 1000.0 {
                    format!("{:.1}K", value / 1000.0)
                } else {
                    format!("{:.0}", value)
                },
                position: Point::new(chart.x - 6.0, y),
                color: text_color,
                size: Pixels(11.0),
                horizontal_alignment: Horizontal::Right,
                vertical_alignment: Vertical::Center,
                ..Default::default()
            });
        }

        // X-axis labels
        let x_step = (n / 20).max(1);
        for i in (0..n).step_by(x_step) {
            let x = chart.x + (i as f32 / (n - 1).max(1) as f32) * chart.width;
            let label = match self.x_axis_mode {
                XAxisMode::PaymentNumber => format!("{}", i + 1),
                XAxisMode::Date => self.payments[i].date.format("%b %Y").to_string(),
            };
            frame.fill_text(canvas::Text {
                content: label,
                position: Point::new(x, chart.y + chart.height + 5.0),
                color: text_color,
                size: Pixels(10.0),
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Top,
                ..Default::default()
            });
        }

        // Axis outline
        let axis_path = canvas::Path::new(|b| {
            b.move_to(Point::new(chart.x, chart.y));
            b.line_to(Point::new(chart.x, chart.y + chart.height));
            b.line_to(Point::new(chart.x + chart.width, chart.y + chart.height));
        });
        frame.stroke(
            &axis_path,
            canvas::Stroke::default()
                .with_color(axis_color)
                .with_width(1.0),
        );

        // Axis title
        frame.fill_text(canvas::Text {
            content: match self.x_axis_mode {
                XAxisMode::PaymentNumber => "Payment #".into(),
                XAxisMode::Date => "Date".into(),
            },
            position: Point::new(chart.x + chart.width / 2.0, chart.y + chart.height + 28.0),
            color: text_color,
            size: Pixels(12.0),
            horizontal_alignment: Horizontal::Center,
            vertical_alignment: Vertical::Top,
            ..Default::default()
        });

        // Line chart
        let balance_color = Color::from_rgb(0.2, 0.4, 0.8);
        for i in 0..n.saturating_sub(1) {
            let x1 = chart.x + (i as f32 / (n - 1).max(1) as f32) * chart.width;
            let y1 = chart.y + chart.height
                - (self.payments[i].remaining_balance as f32 / max_y) * chart.height;
            let x2 = chart.x + ((i + 1) as f32 / (n - 1).max(1) as f32) * chart.width;
            let y2 = chart.y + chart.height
                - (self.payments[i + 1].remaining_balance as f32 / max_y) * chart.height;
            let path = canvas::Path::new(|b| {
                b.move_to(Point::new(x1, y1));
                b.line_to(Point::new(x2, y2));
            });
            frame.stroke(
                &path,
                canvas::Stroke::default()
                    .with_color(balance_color)
                    .with_width(2.0),
            );
        }

        // Hovered point marker
        if let Some(idx) = self.hovered
            && idx < n
        {
            let x = chart.x + (idx as f32 / (n - 1).max(1) as f32) * chart.width;
            let y = chart.y + chart.height
                - (self.payments[idx].remaining_balance as f32 / max_y) * chart.height;
            frame.fill_rectangle(
                Point::new(x - 4.0, y - 4.0),
                Size::new(8.0, 8.0),
                balance_color,
            );
        }

        // Pink dot at principal-exceeds-interest crossover
        if let Some(cross_idx) = self.cross_idx
            && cross_idx < n
        {
            let cx = chart.x + (cross_idx as f32 / (n - 1).max(1) as f32) * chart.width;
            let cy = chart.y + chart.height
                - (self.payments[cross_idx].remaining_balance as f32 / max_y) * chart.height;
            let circle = canvas::Path::new(|b| {
                b.circle(Point::new(cx, cy), 5.0);
            });
            frame.fill(&circle, Color::from_rgb(1.0, 0.4, 0.7));
        }

        // Legend
        let leg_x = chart.x + chart.width - 80.0;
        let leg_y = chart.y + 5.0;
        frame.fill_rectangle(
            Point::new(leg_x, leg_y),
            Size::new(75.0, 22.0),
            Color::from_rgba(1.0, 1.0, 1.0, 0.85),
        );
        let leg_border = canvas::Path::new(|b| {
            b.move_to(Point::new(leg_x, leg_y));
            b.line_to(Point::new(leg_x + 75.0, leg_y));
            b.line_to(Point::new(leg_x + 75.0, leg_y + 22.0));
            b.line_to(Point::new(leg_x, leg_y + 22.0));
            b.close();
        });
        frame.stroke(
            &leg_border,
            canvas::Stroke::default()
                .with_color(grid_color)
                .with_width(0.5),
        );
        let leg_line = canvas::Path::new(|b| {
            b.move_to(Point::new(leg_x + 5.0, leg_y + 11.0));
            b.line_to(Point::new(leg_x + 17.0, leg_y + 11.0));
        });
        frame.stroke(
            &leg_line,
            canvas::Stroke::default()
                .with_color(balance_color)
                .with_width(2.0),
        );
        frame.fill_text(canvas::Text {
            content: "Balance".into(),
            position: Point::new(leg_x + 21.0, leg_y + 3.0),
            color: text_color,
            size: Pixels(11.0),
            ..Default::default()
        });

        // Tooltip
        if let Some(idx) = self.hovered
            && idx < n
        {
            let p = &self.payments[idx];
            let lines = [
                format!("#{} ({})", idx + 1, p.date.format("%b %Y")),
                format!("Balance: {:.2}", p.remaining_balance),
                format!("Paid: {:.2}", p.payment),
            ];
            let char_w = 6.6;
            let pad_x = 5.0;
            let pad_y = 2.0;
            let line_h = 16.0;
            let bottom_pad = 6.0;
            let max_w = lines
                .iter()
                .map(|l| l.len() as f32 * char_w)
                .fold(0.0, f32::max);
            let tw = max_w + 2.0 * pad_x;
            let th = lines.len() as f32 * line_h + pad_y + bottom_pad;
            let x = chart.x + (idx as f32 / (n - 1).max(1) as f32) * chart.width;
            let tx = (x - tw / 2.0)
                .max(chart.x + 5.0)
                .min((chart.x + chart.width - tw).max(chart.x + 5.0));
            let ty = chart.y + 2.0;
            frame.fill_rectangle(
                Point::new(tx, ty),
                Size::new(tw, th),
                Color::from_rgba(1.0, 1.0, 0.95, 1.0),
            );
            let tt_border = canvas::Path::new(|b| {
                b.move_to(Point::new(tx, ty));
                b.line_to(Point::new(tx + tw, ty));
                b.line_to(Point::new(tx + tw, ty + th));
                b.line_to(Point::new(tx, ty + th));
                b.close();
            });
            frame.stroke(
                &tt_border,
                canvas::Stroke::default()
                    .with_color(axis_color)
                    .with_width(1.0),
            );
            let colors = [
                Color::from_rgb(0.0, 0.0, 0.0),
                Color::from_rgb(0.2, 0.4, 0.8),
                text_color,
            ];
            for (i, content) in lines.iter().enumerate() {
                frame.fill_text(canvas::Text {
                    content: content.clone(),
                    position: Point::new(tx + pad_x, ty + pad_y + i as f32 * line_h),
                    color: colors[i],
                    size: Pixels(11.0),
                    ..Default::default()
                });
            }
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pos) = cursor.position() {
                    let chart = self.chart_area(bounds);
                    let cx = bounds.x + chart.x;
                    let cy = bounds.y + chart.y;
                    if pos.x >= cx
                        && pos.x <= cx + chart.width
                        && pos.y >= cy
                        && pos.y <= cy + chart.height
                    {
                        let n = self.payments.len();
                        if n > 0 {
                            let idx = ((pos.x - cx) / chart.width * n as f32) as usize;
                            let idx = idx.min(n - 1);
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::BalanceChartMouseMoved(idx)),
                            );
                        }
                    }
                }
                (
                    canvas::event::Status::Captured,
                    Some(Message::BalanceChartMouseLeft),
                )
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) => (
                canvas::event::Status::Captured,
                Some(Message::BalanceChartMouseLeft),
            ),
            _ => (canvas::event::Status::Ignored, None),
        }
    }
}

fn parse_tenor(s: &str) -> EuriborTenor {
    s.parse().unwrap_or(EuriborTenor::SixMonths)
}

#[cfg(test)]
mod chart_tests {
    use super::*;
    use iced::Rectangle;
    use iced::mouse;
    use mortgage_core::models::Payment;

    fn dummy_payments(n: usize) -> Vec<Payment> {
        (0..n)
            .map(|i| Payment {
                payment: 1000.0,
                date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
                    .unwrap()
                    .checked_add_months(chrono::Months::new(i as u32))
                    .unwrap(),
                principal: 500.0,
                interest: 500.0,
                remaining_balance: 100000.0 - i as f64 * 500.0,
                applied_rate: 5.0,
            })
            .collect()
    }

    fn stacked_chart(payments: &[Payment]) -> StackedChart<'_> {
        StackedChart {
            payments,
            x_axis_mode: XAxisMode::PaymentNumber,
            hovered: None,
            cross_idx: None,
        }
    }

    fn balance_chart(payments: &[Payment]) -> BalanceChart<'_> {
        BalanceChart {
            payments,
            x_axis_mode: XAxisMode::PaymentNumber,
            hovered: None,
            cross_idx: None,
        }
    }

    // ── chart_area tests ───────────────────────────────────────

    #[test]
    fn chart_area_origin_at_zero() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 400.0,
        };
        let chart = stacked_chart(&[]).chart_area(bounds);
        assert_eq!(chart.x, StackedChart::ML);
        assert_eq!(chart.y, StackedChart::MT);
        assert_eq!(chart.width, 800.0 - StackedChart::ML - StackedChart::MR);
        assert_eq!(chart.height, 400.0 - StackedChart::MT - StackedChart::MB);
    }

    #[test]
    fn chart_area_independent_of_bounds_origin() {
        let bounds = Rectangle {
            x: 300.0,
            y: 100.0,
            width: 800.0,
            height: 400.0,
        };
        let chart = stacked_chart(&[]).chart_area(bounds);
        assert_eq!(chart.x, StackedChart::ML);
        assert_eq!(chart.y, StackedChart::MT);
    }

    #[test]
    fn chart_area_clamps_minimum_width() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: StackedChart::ML + StackedChart::MR - 1.0,
            height: 400.0,
        };
        let chart = stacked_chart(&[]).chart_area(bounds);
        assert_eq!(chart.width, 1.0);
    }

    #[test]
    fn chart_area_clamps_minimum_height() {
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: StackedChart::MT + StackedChart::MB - 1.0,
        };
        let chart = stacked_chart(&[]).chart_area(bounds);
        assert_eq!(chart.height, 1.0);
    }

    #[test]
    fn chart_area_balance_chart() {
        let bounds = Rectangle {
            x: 150.0,
            y: 50.0,
            width: 600.0,
            height: 300.0,
        };
        let chart = balance_chart(&[]).chart_area(bounds);
        assert_eq!(chart.x, BalanceChart::ML);
        assert_eq!(chart.y, BalanceChart::MT);
        assert_eq!(chart.width, 600.0 - BalanceChart::ML - BalanceChart::MR);
        assert_eq!(chart.height, 300.0 - BalanceChart::MT - BalanceChart::MB);
    }

    // ── StackedChart::update hit-test ───────────────────────────

    #[test]
    fn stacked_mouse_inside_returns_moved() {
        let payments = dummy_payments(10);
        let chart = stacked_chart(&payments);
        let mut state = ();
        let bounds = Rectangle {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 400.0,
        };
        let local = chart.chart_area(bounds);
        let cx = bounds.x + local.x + local.width / 2.0;
        let cy = bounds.y + local.y + local.height / 2.0;
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(cx, cy),
        });
        let cursor = mouse::Cursor::Available(Point::new(cx, cy));
        let (status, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(status, canvas::event::Status::Captured);
        assert!(matches!(msg, Some(Message::StackedChartMouseMoved(_))));
    }

    #[test]
    fn stacked_mouse_left_of_chart_returns_left() {
        let payments = dummy_payments(10);
        let chart = stacked_chart(&payments);
        let mut state = ();
        let bounds = Rectangle {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 400.0,
        };
        let local = chart.chart_area(bounds);
        let cx = bounds.x + local.x - 10.0;
        let cy = bounds.y + local.y + local.height / 2.0;
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(cx, cy),
        });
        let cursor = mouse::Cursor::Available(Point::new(cx, cy));
        let (status, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(status, canvas::event::Status::Captured);
        assert!(matches!(msg, Some(Message::StackedChartMouseLeft)));
    }

    #[test]
    fn stacked_mouse_above_chart_returns_left() {
        let payments = dummy_payments(10);
        let chart = stacked_chart(&payments);
        let mut state = ();
        let bounds = Rectangle {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 400.0,
        };
        let local = chart.chart_area(bounds);
        let cx = bounds.x + local.x + local.width / 2.0;
        let cy = bounds.y + local.y - 10.0;
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(cx, cy),
        });
        let cursor = mouse::Cursor::Available(Point::new(cx, cy));
        let (status, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(status, canvas::event::Status::Captured);
        assert!(matches!(msg, Some(Message::StackedChartMouseLeft)));
    }

    #[test]
    fn stacked_mouse_cursor_left_event() {
        let payments = dummy_payments(10);
        let chart = stacked_chart(&payments);
        let mut state = ();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 400.0));
        let event = canvas::Event::Mouse(mouse::Event::CursorLeft);
        let cursor = mouse::Cursor::Unavailable;
        let (status, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(status, canvas::event::Status::Captured);
        assert!(matches!(msg, Some(Message::StackedChartMouseLeft)));
    }

    #[test]
    fn stacked_mouse_index_at_left_edge() {
        let payments = dummy_payments(10);
        let chart = stacked_chart(&payments);
        let mut state = ();
        let bounds = Rectangle {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 400.0,
        };
        let local = chart.chart_area(bounds);
        let cx = bounds.x + local.x + 1.0;
        let cy = bounds.y + local.y + 1.0;
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(cx, cy),
        });
        let cursor = mouse::Cursor::Available(Point::new(cx, cy));
        let (_, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(msg, Some(Message::StackedChartMouseMoved(0)));
    }

    #[test]
    fn stacked_mouse_index_at_right_edge() {
        let payments = dummy_payments(10);
        let chart = stacked_chart(&payments);
        let mut state = ();
        let bounds = Rectangle {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 400.0,
        };
        let local = chart.chart_area(bounds);
        let cx = bounds.x + local.x + local.width - 1.0;
        let cy = bounds.y + local.y + 1.0;
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(cx, cy),
        });
        let cursor = mouse::Cursor::Available(Point::new(cx, cy));
        let (_, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(msg, Some(Message::StackedChartMouseMoved(9)));
    }

    // ── BalanceChart::update hit-test ───────────────────────────

    #[test]
    fn balance_mouse_inside_returns_moved() {
        let payments = dummy_payments(10);
        let chart = balance_chart(&payments);
        let mut state = ();
        let bounds = Rectangle {
            x: 120.0,
            y: 60.0,
            width: 700.0,
            height: 350.0,
        };
        let local = chart.chart_area(bounds);
        let cx = bounds.x + local.x + local.width / 2.0;
        let cy = bounds.y + local.y + local.height / 2.0;
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(cx, cy),
        });
        let cursor = mouse::Cursor::Available(Point::new(cx, cy));
        let (status, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(status, canvas::event::Status::Captured);
        assert!(matches!(msg, Some(Message::BalanceChartMouseMoved(_))));
    }

    #[test]
    fn balance_mouse_left_of_chart_returns_left() {
        let payments = dummy_payments(10);
        let chart = balance_chart(&payments);
        let mut state = ();
        let bounds = Rectangle {
            x: 120.0,
            y: 60.0,
            width: 700.0,
            height: 350.0,
        };
        let local = chart.chart_area(bounds);
        let cx = bounds.x + local.x - 10.0;
        let cy = bounds.y + local.y + 10.0;
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(cx, cy),
        });
        let cursor = mouse::Cursor::Available(Point::new(cx, cy));
        let (status, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(status, canvas::event::Status::Captured);
        assert!(matches!(msg, Some(Message::BalanceChartMouseLeft)));
    }

    #[test]
    fn balance_mouse_cursor_left_event() {
        let payments = dummy_payments(10);
        let chart = balance_chart(&payments);
        let mut state = ();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 400.0));
        let event = canvas::Event::Mouse(mouse::Event::CursorLeft);
        let cursor = mouse::Cursor::Unavailable;
        let (status, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(status, canvas::event::Status::Captured);
        assert!(matches!(msg, Some(Message::BalanceChartMouseLeft)));
    }

    #[test]
    fn balance_mouse_index_edges() {
        let payments = dummy_payments(10);
        let chart = balance_chart(&payments);
        let mut state = ();
        let bounds = Rectangle {
            x: 100.0,
            y: 50.0,
            width: 800.0,
            height: 400.0,
        };
        let local = chart.chart_area(bounds);

        let cx_left = bounds.x + local.x + 1.0;
        let cy = bounds.y + local.y + 1.0;
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(cx_left, cy),
        });
        let cursor = mouse::Cursor::Available(Point::new(cx_left, cy));
        let (_, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(msg, Some(Message::BalanceChartMouseMoved(0)));

        let cx_right = bounds.x + local.x + local.width - 1.0;
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(cx_right, cy),
        });
        let cursor = mouse::Cursor::Available(Point::new(cx_right, cy));
        let (_, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(msg, Some(Message::BalanceChartMouseMoved(9)));
    }

    // ── Edge cases ──────────────────────────────────────────────

    #[test]
    fn stacked_empty_payments_no_panic() {
        let payments: Vec<Payment> = vec![];
        let chart = stacked_chart(&payments);
        let mut state = ();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 400.0));
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(100.0, 100.0),
        });
        let cursor = mouse::Cursor::Available(Point::new(100.0, 100.0));
        let (status, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(status, canvas::event::Status::Captured);
        assert!(matches!(msg, Some(Message::StackedChartMouseLeft)));
    }

    #[test]
    fn balance_empty_payments_no_panic() {
        let payments: Vec<Payment> = vec![];
        let chart = balance_chart(&payments);
        let mut state = ();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 400.0));
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(100.0, 100.0),
        });
        let cursor = mouse::Cursor::Available(Point::new(100.0, 100.0));
        let (status, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(status, canvas::event::Status::Captured);
        assert!(matches!(msg, Some(Message::BalanceChartMouseLeft)));
    }

    #[test]
    fn chart_area_both_charts_have_same_margins() {
        assert_eq!(StackedChart::ML, BalanceChart::ML);
        assert_eq!(StackedChart::MR, BalanceChart::MR);
        assert_eq!(StackedChart::MT, BalanceChart::MT);
        assert_eq!(StackedChart::MB, BalanceChart::MB);
    }

    #[test]
    fn single_payment_hit_test() {
        let payments = dummy_payments(1);
        let chart = stacked_chart(&payments);
        let mut state = ();
        let bounds = Rectangle {
            x: 50.0,
            y: 50.0,
            width: 600.0,
            height: 300.0,
        };
        let local = chart.chart_area(bounds);
        let cx = bounds.x + local.x + local.width / 2.0;
        let cy = bounds.y + local.y + local.height / 2.0;
        let event = canvas::Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(cx, cy),
        });
        let cursor = mouse::Cursor::Available(Point::new(cx, cy));
        let (_, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(msg, Some(Message::StackedChartMouseMoved(0)));
    }

    // ── Ignored events ──────────────────────────────────────────

    #[test]
    fn non_mouse_event_is_ignored() {
        let payments = dummy_payments(5);
        let chart = stacked_chart(&payments);
        let mut state = ();
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 400.0));
        let event = canvas::Event::Touch(iced::touch::Event::FingerMoved {
            id: iced::touch::Finger(0),
            position: Point::new(100.0, 100.0),
        });
        let cursor = mouse::Cursor::Unavailable;
        let (status, msg) = chart.update(&mut state, event, bounds, cursor);
        assert_eq!(status, canvas::event::Status::Ignored);
        assert_eq!(msg, None);
    }
}
