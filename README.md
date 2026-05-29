# Mortgage Roofer

Кросс-платформенный калькулятор ипотеки на Rust. Поддерживает расчёт аннуитетных и дифференцированных платежей, досрочное погашение, изменение ставки в процессе кредита и экспорт результатов.

## Архитектура

Проект организован как Cargo workspace из 4 crate:

| Crate | Описание |
|-------|----------|
| `mortgage_core` | Бизнес-логика: модели, калькулятор, загрузка Euribor. Library crate. |
| `mortgage_cli` | Командная строка (`clap`). JSON-сценарии, CSV-экспорт. |
| `mortgage_tui` | Терминальный интерфейс (`ratatui`/`crossterm`). |
| `mortgage_gui` | Десктоп GUI (`iced`). Графики и экспорт PDF. |

## Возможности

- **Валюты**: USD, EUR (символ и форматирование)
- **Типы платежей**: аннуитетный (`Annuity`) и дифференцированный (`Diff`)
- **Режимы ставки**:
  - `Fix` — фиксированная ставка на весь срок
  - `Euribor` — плавающая ставка на основе Euribor + спред
  - `Mixed` — фиксированный период, затем переход на Euribor+спред
- **Досрочное погашение**:
  - `ReduceTerm` — уменьшение срока кредита
  - `ReducePayment` — уменьшение ежемесячного платежа
- **Euribor**:
  - Автозагрузка с ECB API (выбор tenor: 1m, 3m, 6m, 12m)
  - Ручная кривая: пользователь задаёт ставку на конкретный период
- **Экспорт**: CSV, PDF (включая графики)
- **Графики**: stacked bar (Principal vs Interest), линия остатка долга, маркер пересечения

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
# Весь workspace
cargo build --workspace --verbose

# Тесты
cargo test --workspace --verbose

# Только core
cargo check -p mortgage_core
cargo test -p mortgage_core
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
```

### TUI
```bash
cargo run -p mortgage_tui
```

### GUI
```bash
cargo run -p mortgage_gui
```

## Примеры JSON-конфигов

См. `test_config.json` в корне проекта.

## CI

`.github/workflows/rust.yml` — автоматическая сборка и тесты на push/PR в `main`.

## Лицензия

MIT
