# mortgage_core

[English version](README.md)

Библиотека бизнес-логики для ипотечного калькулятора.

## Модули

- `models` — доменные модели (LoanParams, Payment, LoanResult, RateMode, и т.д.)
- `calculator` — расчёт графика платежей
- `analysis` — анализ чувствительности и break-even анализ
- `euribor` — загрузка ставок Euribor с ECB API
- `export` — экспорт в CSV
- `session` — сохранение/загрузка сессий
- `error` — типизированные ошибки

## Модели

### `Currency`
```rust
pub enum Currency {
    Usd,
    Eur,
}
```

### `PaymentType`
```rust
pub enum PaymentType {
    Annuity,  // аннуитет
    Diff,     // дифференцированный
}
```

### `RateMode`
```rust
pub enum RateMode {
    Fix { rate: f64, spread: f64 },
    Euribor { tenor: EuriborTenor, spread: f64 },
    Mixed {
        fix_years: f64,
        fix_rate: f64,
        fix_spread: f64,
        euribor_tenor: EuriborTenor,
        euribor_spread: f64,
    },
}
```

### `Prepayment`
```rust
pub struct Prepayment {
    pub date: NaiveDate,
    pub amount: f64,
    pub effect: PrepaymentEffect, // ReduceTerm | ReducePayment
}
```

### `LoanParams`
```rust
pub struct LoanParams {
    pub amount: f64,
    pub term_years: u32,
    pub payment_type: PaymentType,
    pub currency: Currency,
    pub start_date: NaiveDate,
    pub rate_mode: RateMode,
    pub same_spread: bool,
    pub euribor_curve: Vec<EuriborPoint>,
    pub prepayments: Vec<Prepayment>,
    pub upfront_cost: Option<f64>,
    pub upfront_percent: Option<f64>,
}
```

### `YearlySummary`
```rust
pub struct YearlySummary {
    pub year: i32,
    pub total_payment: f64,
    pub total_principal: f64,
    pub total_interest: f64,
    pub payments_count: usize,
    pub ending_balance: f64,
}
```

## API Калькулятора

```rust
use mortgage_core::{Calculator, LoanParams};

let params = LoanParams { /* ... */ };
let result = Calculator::calculate(&params)?;

println!("Monthly: {:?}", result.monthly_payment);
println!("Total interest: {}", result.total_interest);

// Годовая сводка
for s in result.yearly_summaries() {
    println!("{}: payment={:.2} principal={:.2} interest={:.2} balance={:.2}",
        s.year, s.total_payment, s.total_principal, s.total_interest, s.ending_balance);
}

// Таблица платежей
for p in &result.payments {
    println!("{}: payment={:.2} principal={:.2} interest={:.2} balance={:.2}",
        p.date, p.payment, p.principal, p.interest, p.remaining_balance);
}
```

## Анализ

### Rate Sensitivity
```rust
use mortgage_core::sensitivity_analysis;

let deltas = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
let points = sensitivity_analysis(&params, &deltas);

for p in &points {
    println!("Delta: {:+.2}%, Rate: {:.2}%, Monthly: {:?}, Interest: {:.2}",
        p.rate_delta, p.effective_rate, p.monthly_payment, p.total_interest);
}
```

### Break-Even vs Rent
```rust
use mortgage_core::break_even_analysis;

let monthly_rent = 1000.0;
let be = break_even_analysis(&params, monthly_rent);

println!("Monthly mortgage: {:.2}", be.monthly_cost);
println!("Upfront costs: {:.2}", be.upfront_costs);
if let (Some(months), Some(years)) = (be.break_even_months, be.break_even_years) {
    println!("Break-even: {} months ({:.1} years)", months, years);
}
println!("{}", be.explanation);
```

## Сессии

```rust
use mortgage_core::{save_session, load_session};

// Сохранение
save_session("session.json", &params, &result)?;

// Загрузка
let session = load_session("session.json")?;
// session.params, session.result
```

## Euribor

### Ручная кривая
```rust
params.euribor_curve = vec![
    EuriborPoint { date_from: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(), rate: 3.0 },
    EuriborPoint { date_from: NaiveDate::from_ymd_opt(2028, 1, 1).unwrap(), rate: 4.0 },
];
```

### Автозагрузка с ECB
```rust
use mortgage_core::euribor;

let rate = euribor::fetch_euribor(EuriborTenor::SixMonths)?;
// или с кэшем
let mut cache = euribor::EuriborCache::default();
let rate = cache.get_or_fetch(EuriborTenor::SixMonths)?;
```

## Тесты

```bash
cargo test -p mortgage_core
```

70 тестов покрывают:
- Unit-тесты калькулятора (11)
- Edge cases (19)
- Serde round-trip (19)
- Property-based tests с proptest (8)
- Doc tests (3)

## Serde

Все модели реализуют `Serialize`/`Deserialize` для работы с JSON:
```rust
let json = serde_json::to_string(&params)?;
let params: LoanParams = serde_json::from_str(&json)?;
```

## Валидация

```rust
if let Err(errors) = params.validate() {
    for e in &errors {
        eprintln!("Validation error: {}", e);
    }
}
```

Проверяет:
- Amount > 0 и <= 100,000,000
- Term > 0 и <= 50 лет
- Rate >= 0 и <= 100%
- Spread >= 0
- Prepayment date >= start_date
- Prepayment amount > 0
- Upfront cost/percent non-negative (and not both specified)
