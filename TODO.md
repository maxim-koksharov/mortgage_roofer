# Mortgage Roofer — TODO

## Этап 0. Workspace (скелет)
- [x] Создать `mortgage_core/`, `mortgage_cli/`, `mortgage_tui/`, `mortgage_gui/` crates.
- [x] Настроить корневой `Cargo.toml` с `[workspace]`.
- [x] Перенести текущий `mortgage_logic.rs` в `mortgage_core/src/lib.rs`.
- [x] Обновить зависимости (workspace deps) для всех crates.
- [x] Проверить сборку всего workspace: `cargo check --workspace` проходит.
- [x] Обновить Dockerfile с `fontconfig-devel` и `freetype-devel`.

## Этап 1. `mortgage_core` — ядро

### 1.1 Модели
- [x] `Currency` (`Usd`, `Eur`) — символ + форматирование.
- [x] `EuriborTenor` (`1m`, `3m`, `6m`, `12m`), по умолчанию `6m`.
- [x] `PaymentType` enum — `Annuity`, `Diff`.
- [x] `RateMode`:
  - `Fix { rate: f64, spread: f64 }`
  - `Euribor { tenor: EuriborTenor, spread: f64 }`
  - `Mixed { fix_years: f64, fix_rate: f64, fix_spread: f64, euribor_tenor: EuriborTenor, euribor_spread: f64 }`
- [x] `PrepaymentEffect` — `ReduceTerm`, `ReducePayment`.
- [x] `Prepayment` — `date`, `amount`, `effect`. `Vec<Prepayment>` для комбинирования.
- [x] `EuriborPoint` — ручная кривая: `(date_from, rate)`.
- [x] `LoanParams` — полный набор входных данных.
- [x] `Payment` — расширен полем `applied_rate: f64`.
- [x] `LoanResult` — полный график + сводка.
- [x] Все модели с `serde::{Serialize, Deserialize}`.

### 1.2 Калькулятор
- [x] Расчёт аннуитета и дифференцированного платежа с `f64`.
- [x] Поддержка смены ставки по датам (фикс → Euribor+спред, ручная кривая Euribor).
- [x] Пересчёт графика при каждом `Prepayment`:
  - `ReduceTerm` — уменьшается количество месяцев.
  - `ReducePayment` — уменьшается ежемесячный платёж, срок не меняется.
- [x] Флаг `same_spread: bool`.

### 1.3 Euribor
- [x] Модуль `euribor::fetch`: HTTP-запрос к API ECB, парсинг.
- [x] Модуль `euribor::curve`: ручные точки `EuriborPoint`, поиск актуальной ставки для даты.

### 1.4 Тесты
- [x] Аннуитет и дифф — проверка сумм.
- [x] Досрочка `ReduceTerm` — срок уменьшился.
- [x] Досрочка `ReducePayment` — платёж уменьшился, срок сохранился.
- [x] Смена ставки (Mixed) и ручная кривая Euribor.
- [x] Граничные: 0% ставка, досрочка на последний месяц, округление.

## Этап 2. `mortgage_cli`
- [x] `clap` — CLI аргументы + `--config file.json`.
- [x] Чтение/валидация JSON-сценария (`serde`).
- [x] Вывод сводки и таблицы платежей.
- [x] CSV-экспорт (`--format csv --output`).

## Этап 3. `mortgage_tui`
- [x] `ratatui` + `crossterm` — формы ввода параметров.
- [x] Таблица платежей с прокруткой.
- [x] Экран сводки.
- [x] Навигация `Tab`/`Esc`/`Enter`.

## Этап 4. `mortgage_gui` — Iced
- [x] Ввод параметров (amount, term, rate, spread).
- [x] Таблица платежей с прокруткой.
- [x] Графики (`plotters` SVG + `iced::widget::svg`): линия остатка + principal + interest.
- [x] Экспорт CSV.
- [x] Экспорт PDF: `printpdf` с таблицами и сводкой.

## Этап 5. Интеграция и CI
- [x] Сборка workspace: `cargo build --workspace`.
- [x] Обновить `.github/workflows/rust.yml` для workspace.
- [x] Обновить `AGENTS.md`.
