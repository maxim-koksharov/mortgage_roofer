# mortgage_core

Библиотека бизнес-логики для ипотечного калькулятора.

## Модули

- `models` — доменные модели (LoanParams, Payment, LoanResult, RateMode, и т.д.)
- `calculator` — расчёт графика платежей
- `euribor` — загрузка ставок Euribor с ECB API

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
}
```

## API Калькулятора

```rust
use mortgage_core::{Calculator, LoanParams};

let params = LoanParams { /* ... */ };
let result = Calculator::calculate(&params);

println!("Monthly: {:?}", result.monthly_payment);
println!("Total interest: {}", result.total_interest);
for p in &result.payments {
    println!("{}: payment={:.2} principal={:.2} interest={:.2} balance={:.2}",
        p.date, p.payment, p.principal, p.interest, p.remaining_balance);
}
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
let mut cache = euribor::EuriborCache::new();
let rate = cache.get_or_fetch(EuriborTenor::SixMonths)?;
```

## Тесты

```bash
cargo test -p mortgage_core
```

Покрывают:
- Аннуитет и дифференцированный платёж
- Досрочное погашение (ReduceTerm / ReducePayment)
- Смешанный режим ставки (Mixed)
- Ручную кривую Euribor
- Флаг same_spread
- Граничные случаи (0% ставка)
- Точка пересечения principal > interest

## Serde

Все модели реализуют `Serialize`/`Deserialize` для работы с JSON:
```rust
let json = serde_json::to_string(&params)?;
let params: LoanParams = serde_json::from_str(&json)?;
```
