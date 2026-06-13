# mortgage_cli

[English version](README.md)

Командная строка для ипотечного калькулятора.

## Зависимости

- `clap` — парсинг аргументов
- `mortgage_core` — расчёты, анализ, сессии
- `serde_json` — JSON-конфиги и сессии
- `chrono` — работа с датами

## Запуск

### Базовый расчёт

```bash
cargo run -p mortgage_cli -- -a 185000 -t 30 -r 3.6
```

Выводит сводку и таблицу первых 24 платежей.

### Все аргументы

```
Options:
  -a, --amount <AMOUNT>          Loan amount
  -t, --term <TERM>              Loan term in years
  -p, --payment-type <TYPE>      Payment type: annuity or diff
  -u, --currency <CURRENCY>      Currency: usd or eur
      --start-date <DATE>        Start date (YYYY-MM-DD)
      --rate-mode <MODE>         Rate mode: fix, euribor, or mixed
  -r, --rate <RATE>              Base rate (for fix) or fix rate (for mixed)
      --spread <SPREAD>          Bank spread
      --fix-years <YEARS>        Fixed period in years (for mixed mode)
      --euribor-tenor <TENOR>    Euribor tenor: 1m, 3m, 6m, 12m
      --euribor-spread <SPREAD>  Euribor spread
      --same-spread              Use same spread for fixed and euribor periods
      --upfront-cost <AMOUNT>    Upfront costs for break-even (fixed amount)
      --upfront-percent <PERCENT>  Upfront costs for break-even (percent of loan)
  -c, --config <FILE>            Path to JSON config file
  -o, --output <FILE>            Output file path (CSV)
      --format <FORMAT>          Output format: table or csv [default: table]
      --limit <N>                Number of payments to display [default: 24]
      --yearly                   Show yearly summary instead of monthly table
      --prepayment <PREP>        Prepayment "YYYY-MM-DD:amount:effect" (repeatable)
      --save <FILE>              Save session to JSON file
      --load <FILE>              Load session from JSON file
      --sensitivity <DELTAS>     Rate sensitivity deltas (e.g. "-2,-1,0,1,2")
      --break-even-rent <RENT>   Break-even analysis vs monthly rent
  -h, --help                     Print help
```

### Примеры

**Fix (фиксированная ставка):**
```bash
cargo run -p mortgage_cli -- -a 200000 -t 20 -r 4.5 --spread 0.5
```

**Euribor (плавающая):**
```bash
cargo run -p mortgage_cli -- -a 150000 -t 15 --rate-mode euribor --euribor-tenor 6m --spread 1.2
```

**Mixed (фикс → Euribor):**
```bash
cargo run -p mortgage_cli -- -a 250000 -t 25 --rate-mode mixed --rate 3.0 --spread 1.0 --fix-years 5 --euribor-tenor 6m --euribor-spread 1.5
```

**С датой начала:**
```bash
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --start-date 2025-01-01
```

**Годовая сводка:**
```bash
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --yearly
```

**С досрочным погашением:**
```bash
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 \
  --prepayment "2027-01-01:20000:ReduceTerm" \
  --prepayment "2028-06-01:10000:ReducePayment"
```

**Анализ чувствительности:**
```bash
cargo run -p mortgage_cli -- -a 200000 -t 20 -r 4.5 --sensitivity "-2,-1,-0.5,0,0.5,1,2"
```

**Break-even vs аренда:**
```bash
cargo run -p mortgage_cli -- -a 200000 -t 20 -r 4.5 --break-even-rent 1000 --upfront-percent 5
```

**Сохранение/загрузка сессии:**
```bash
# Сохранить
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --save session.json

# Загрузить
cargo run -p mortgage_cli -- --load session.json
```

### CSV-экспорт

```bash
# В файл
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 -o payments.csv

# В stdout
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --format csv
```

### JSON-конфиг

Создайте файл `config.json`:
```json
{
  "amount": 200000,
  "term_years": 20,
  "payment_type": "annuitet",
  "currency": "Eur",
  "start_date": "2025-01-01",
  "rate_mode": {
    "Mixed": {
      "fix_years": 2,
      "fix_rate": 2.5,
      "fix_spread": 1.0,
      "euribor_tenor": "6m",
      "euribor_spread": 1.5
    }
  },
  "same_spread": false,
  "euribor_curve": [
    { "date_from": "2027-01-01", "rate": 3.0 }
  ],
  "prepayments": [
    { "date": "2028-01-01", "amount": 50000, "effect": "ReduceTerm" },
    { "date": "2029-06-01", "amount": 20000, "effect": "ReducePayment" }
  ],
  "upfront_cost": null,
  "upfront_percent": null
}
```

Запустите:
```bash
cargo run -p mortgage_cli -- --config config.json
```

### Поля JSON-конфига

| Поле | Тип | Описание |
|------|-----|----------|
| `amount` | f64 | Сумма кредита |
| `term_years` | u32 | Срок в годах |
| `payment_type` | `"annuitet"` / `"diff"` | Тип платежа |
| `currency` | `"Usd"` / `"Eur"` | Валюта |
| `start_date` | "YYYY-MM-DD" | Дата начала |
| `rate_mode` | Fix / Euribor / Mixed | Режим ставки |
| `same_spread` | bool | Одинаковый спред на весь срок |
| `euribor_curve` | `Vec<EuriborPoint>` | Ручная кривая Euribor |
| `prepayments` | `Vec<Prepayment>` | Досрочные погашения (множественные) |
| `upfront_cost` | `Option<f64>` | Фиксированные upfront costs для break-even |
| `upfront_percent` | `Option<f64>` | Upfront costs как % от суммы кредита |

## Сборка

```bash
cargo build --release -p mortgage_cli
```

Бинарник: `target/release/mortgage_cli`

## Тесты

```bash
cargo test -p mortgage_cli
```

10 интеграционных тестов покрывают:
- Базовый расчёт
- Diff платежи
- CSV вывод
- CSV файл
- JSON конфиг
- Годовая сводка
- Mixed режим
- Limit опция
- Валюта USD
- Невалидный конфиг
