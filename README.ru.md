# Mortgage Roofer

[English version](README.md)

Кросс-платформенный калькулятор ипотеки на Rust. Поддерживает расчёт аннуитетных и дифференцированных платежей, множественные досрочные погашения, изменение ставки в процессе кредита, анализ чувствительности, break-even анализ и экспорт результатов.

## Архитектура

Проект организован как Cargo workspace из 4 crate:

| Crate | Описание |
|-------|----------|
| `mortgage_core` | Бизнес-логика: модели, калькулятор, анализ, Euribor, сессии. Library crate. |
| `mortgage_cli` | Командная строка (`clap`). JSON-сценарии, CSV-экспорт, sensitivity/break-even. |
| `mortgage_tui` | Терминальный интерфейс (`ratatui`/`crossterm`). Все фичи + горячие клавиши. |
| `mortgage_gui` | Десктоп GUI (`iced`). Темизированный UI, 7 вкладок, графики, PDF-экспорт. |

## Возможности

### Основные
- **Валюты**: USD, EUR (символ и форматирование)
- **Типы платежей**: аннуитетный (`Annuity`) и дифференцированный (`Diff`)
- **Дата начала**: настраиваемая дата старта кредита
- **Режимы ставки**:
  - `Fix` — фиксированная ставка на весь срок
  - `Euribor` — плавающая ставка на основе Euribor + спред
  - `Mixed` — фиксированный период, затем переход на Euribor+спред
- **Досрочное погашение** (множественное):
  - `ReduceTerm` — уменьшение срока кредита
  - `ReducePayment` — уменьшение ежемесячного платежа
- **Euribor**:
  - Автозагрузка с ECB API (выбор tenor: 1m, 3m, 6m, 12m)
  - Ручная кривая: пользователь задаёт ставку на конкретный период

### Анализ
- **Yearly Summary** — годовая агрегация платежей (платёж, основной долг, проценты, остаток)
- **Rate Sensitivity** — таблица изменения платежей при ±0.5%, ±1%, ±2% ставки
- **Break-Even vs Rent** — расчёт окупаемости покупки vs аренда (с учётом ежемесячной аренды и upfront costs)

### Графики (GUI)
- **Stacked Bar** — Principal (зелёный) + Interest (красный) + маркер пересечения
- **Balance Line** — линия остатка долга
- **Overlay** — комбинированный: principal + interest + balance на одном графике

### Экспорт и сессии
- **CSV** — экспорт таблицы платежей
- **PDF** — отчёт с сводкой, таблицей и графиком
- **Session Save/Load** — сохранение/загрузка параметров и результатов в JSON

## Системные зависимости

В Docker уже установлены:
```dockerfile
fontconfig-devel
freetype-devel
```

При сборке вне Docker:
```bash
# AlmaLinux/RHEL/Fedora
sudo dnf install -y fontconfig-devel freetype-devel

# Debian/Ubuntu
sudo apt-get install -y libfontconfig1-dev libfreetype6-dev
```

## Сборка

```bash
# Весь workspace (debug)
cargo build --workspace

# Весь workspace (release)
cargo build --workspace --release

# Тесты
cargo test --workspace

# Проверки
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

## Запуск

### CLI
```bash
# Базовый расчёт
cargo run -p mortgage_cli -- -a 185000 -t 30 -r 3.6

# С JSON-конфигом
cargo run -p mortgage_cli -- --config test_config.json

# CSV-экспорт
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --format csv --output payments.csv

# Годовая сводка
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --yearly

# С досрочным погашением
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --prepayment "2027-01-01:20000:ReduceTerm"

# Анализ чувствительности
cargo run -p mortgage_cli -- -a 200000 -t 20 -r 4.5 --sensitivity "-2,-1,0,1,2"

# Break-even vs аренда (с upfront costs)
cargo run -p mortgage_cli -- -a 200000 -t 20 -r 4.5 --break-even-rent 1000 --upfront-percent 5
cargo run -p mortgage_cli -- -a 200000 -t 20 -r 4.5 --break-even-rent 1000 --upfront-cost 10000

# Сохранение сессии
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --save session.json

# Загрузка сессии
cargo run -p mortgage_cli -- --load session.json
```

### TUI
```bash
cargo run -p mortgage_tui
```

**Горячие клавиши в результатах:**
- `Y` — годовая сводка
- `R` — анализ чувствительности
- `B` — break-even vs аренда
- `S` — экспорт CSV
- `W` — сохранить сессию
- `L` — загрузить сессию

### GUI
```bash
cargo run -p mortgage_gui
```

**Вкладки:** Table, Stacked, Balance, Overlay, Yearly, Sensitivity, Break-Even

## Примеры JSON-конфигов

См. `test_config.json` в корне проекта.

## CI

`.github/workflows/rust.yml` — автоматическая сборка, тесты, fmt и clippy на push/PR в `main`.

## Тесты

73 теста покрывают:
- Unit-тесты калькулятора (11)
- Edge cases (19)
- Serde round-trip (19)
- Property-based tests с proptest (8)
- CLI integration (10)
- Break-even (3)
- Doc tests (3)

## Документация

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — детальное описание архитектуры проекта
- [`AGENTS.md`](AGENTS.md) — инструкции для AI-агентов и разработчиков
- Per-crate README: [`mortgage_core/`](mortgage_core/README.md), [`mortgage_cli/`](mortgage_cli/README.md), [`mortgage_tui/`](mortgage_tui/README.md), [`mortgage_gui/`](mortgage_gui/README.md)

## Лицензия

MIT
