# Mortgage Roofer — TODO

## Этап 0. Workspace (скелет) ✅
- [x] Создать `mortgage_core/`, `mortgage_cli/`, `mortgage_tui/`, `mortgage_gui/` crates.
- [x] Настроить корневой `Cargo.toml` с `[workspace]`.
- [x] Перенести текущий `mortgage_logic.rs` в `mortgage_core/src/lib.rs`.
- [x] Обновить зависимости (workspace deps) для всех crates.
- [x] Проверить сборку всего workspace: `cargo check --workspace` проходит.
- [x] Обновить Dockerfile с `fontconfig-devel` и `freetype-devel`.

## Этап 1. `mortgage_core` — ядро ✅
- [x] Модели, калькулятор, Euribor, тесты.

## Этап 2. `mortgage_cli` ✅
- [x] CLI аргументы, JSON-конфиг, таблица, CSV-экспорт.

## Этап 3. `mortgage_tui` — TUI (ratatui) ✅

### 3.1 Форма ввода
- [x] Выбор валюты (USD/EUR) — enum picker.
- [x] Выбор типа платежа (Annuity/Diff) — enum picker.
- [x] Выбор режима ставки (Fix/Euribor/Mixed) — enum picker.
- [x] Поля для Fix: rate, spread.
- [x] Поля для Euribor: tenor (1m/3m/6m/12m), spread.
- [x] Поля для Mixed: fix_years, fix_rate, fix_spread, euribor_tenor, euribor_spread, same_spread.
- [x] Ввод prepayment (дата, сумма, эффект).
- [x] Валидация ввода и обработка ошибок (popup).

### 3.2 Результаты
- [x] Таблица платежей с прокруткой (↑↓/PgUp/PgDown).
- [x] Сводка (totals, monthly payment, principal>interest point).
- [x] Экспорт CSV в файл (горячая клавиша 'S').

### 3.3 Навигация
- [x] Tab/Shift+Tab — переключение полей.
- [x] Enter — расчёт.
- [x] Esc — возврат к форме из результатов.
- [x] Улучшенный UI (цвета, выравнивание, подсказки).

## Этап 4. `mortgage_gui` — GUI (Iced) ✅

### 4.1 Форма ввода
- [x] Выбор валюты (USD/EUR) — pick_list.
- [x] Выбор типа платежа (Annuity/Diff) — pick_list.
- [x] Выбор режима ставки (Fix/Euribor/Mixed) — pick_list + динамические поля.
- [x] Поля для Fix, Euribor, Mixed (динамическое показ/скрытие полей).
- [x] same_spread checkbox.
- [x] Ввод prepayment (дата, сумма, эффект).

### 4.2 Графики
- [x] Stacked bar chart: Principal (зелёный) + Interest (красный) + маркер пересечения principal > interest.
- [x] Линейный график остатка долга (SVG).
- [x] Аннотация точки пересечения.

### 4.3 Экспорт
- [x] CSV экспорт.
- [x] PDF экспорт (сводка + таблица первых 60 платежей + график PNG на второй странице).

### 4.4 UI/UX
- [x] Улучшенный layout (responsive split pane).
- [x] Подсказки через status bar.

## Этап 5. Интеграция и CI ✅
- [x] Сборка workspace.
- [x] CI `.github/workflows/rust.yml`.
- [x] Обновить `AGENTS.md`.
