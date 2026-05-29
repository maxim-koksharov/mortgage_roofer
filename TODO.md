# Mortgage Roofer — TODO

## Этап 0. Workspace (скелет)
- [x] Создать `mortgage_core/`, `mortgage_cli/`, `mortgage_tui/`, `mortgage_gui/` crates.
- [x] Настроить корневой `Cargo.toml` с `[workspace]`.
- [x] Перенести текущий `mortgage_logic.rs` в `mortgage_core/src/lib.rs`.
- [x] Обновить зависимости (workspace deps) для всех crates.
- [x] Проверить сборку всего workspace: `cargo check --workspace` проходит.

## Этап 1. `mortgage_core` — ядро
- [ ] Определить модели: `Currency`, `EuriborTenor`, `RateMode`, `PrepaymentEffect`, `Prepayment`, `EuriborPoint`, `LoanParams`, `Payment`, `LoanResult`/`LoanSchedule`.
- [ ] Реализовать калькулятор аннуитета и дифференцированных платежей.
- [ ] Поддержка смены ставки по датам (Mixed, ручная кривая Euribor).
- [ ] Реализовать досрочное погашение: ReduceTerm и ReducePayment, комбинирование.
- [ ] Реализовать флаг `same_spread`.
- [ ] Euribor: авто-загрузка с ECB API (выбор tenor, по умолчанию 6м), fallback на ручной ввод.
- [ ] Euribor: ручная кривая `EuriborPoint` с интерполяцией.
- [ ] Юнит-тесты: аннуитет, дифф, досрочка ReduceTerm, досрочка ReducePayment, Mixed, ручная кривая, 0% ставка, округление.

## Этап 2. `mortgage_cli`
- [ ] `clap` — CLI аргументы + `--config file.json`.
- [ ] Чтение/валидация JSON-сценария (`serde`).
- [ ] Вывод сводки и таблицы платежей.
- [ ] CSV-экспорт (`--format csv --output`).
- [ ] Интеграция авто-загрузки Euribor.

## Этап 3. `mortgage_tui`
- [ ] `ratatui` + `crossterm` — формы ввода всех параметров.
- [ ] Таблица платежей с прокруткой.
- [ ] Экран сводки.
- [ ] Навигация `Tab`/`Esc`/`Enter`.

## Этап 4. `mortgage_gui` — Iced
- [ ] Полный ввод параметров (виджеты iced).
- [ ] Таблица платежей с прокруткой.
- [ ] Графики (векторные, `plotters`):
  - [ ] Линия остатка долга по месяцам.
  - [ ] Stacked bar «Тело кредита vs Проценты».
  - [ ] Точка пересечения: principal > interest (аннотация/маркер).
- [ ] Экспорт CSV.
- [ ] Экспорт PDF: `plotters` (SVG backend) + `printpdf`/`svg2pdf` для векторного PDF с таблицами и графиками.

## Этап 5. Интеграция и CI
- [ ] Сборка workspace: `cargo build --workspace`.
- [ ] Обновить `.github/workflows/rust.yml` для workspace.
- [ ] Обновить `AGENTS.md`.
