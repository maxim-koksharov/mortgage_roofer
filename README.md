# Mortgage Roofer

[Русская версия](README.ru.md)

Cross-platform mortgage calculator written in Rust. Supports annuity and declining balance payments, multiple prepayments, variable rate modes, sensitivity analysis, break-even analysis, and result export.

## Architecture

The project is organized as a Cargo workspace with 4 crates:

| Crate | Description |
|-------|-------------|
| `mortgage_core` | Business logic: models, calculator, analysis, Euribor, sessions. Library crate. |
| `mortgage_cli` | Command-line interface (`clap`). JSON configs, CSV export, sensitivity/break-even. |
| `mortgage_tui` | Terminal UI (`ratatui`/`crossterm`). All features + keyboard shortcuts. |
| `mortgage_gui` | Desktop GUI (`iced`). Themed UI, 7 tabs, charts, PDF export. |

## Features

### Core
- **Currencies**: USD, EUR (symbol and formatting)
- **Payment types**: annuity (`Annuity`) and declining balance (`Diff`)
- **Start date**: configurable loan start date
- **Rate modes**:
  - `Fix` — fixed rate for the entire term
  - `Euribor` — floating rate based on Euribor + spread
  - `Mixed` — fixed period, then switches to Euribor + spread
- **Prepayments** (multiple):
  - `ReduceTerm` — reduce the loan term
  - `ReducePayment` — reduce the monthly payment
- **Euribor**:
  - Auto-fetch from ECB API (tenor selection: 1m, 3m, 6m, 12m)
  - Manual curve: user-defined rate for specific periods

### Analysis
- **Yearly Summary** — yearly aggregation of payments (payment, principal, interest, balance)
- **Rate Sensitivity** — payment changes at ±0.5%, ±1%, ±2% rate adjustments
- **Break-Even vs Rent** — calculates when buying beats renting (with monthly rent and upfront costs)

### Charts (GUI)
- **Stacked Bar** — Principal (green) + Interest (red) + crossover marker
- **Balance Line** — remaining balance over time
- **Overlay** — combined: principal + interest + balance on one chart

### Export & Sessions
- **CSV** — export payment schedule
- **PDF** — report with summary, table, and chart
- **Session Save/Load** — save/load parameters and results to JSON

## System Dependencies

Already installed in Docker:
```dockerfile
fontconfig-devel
freetype-devel
```

When building outside Docker:
```bash
# AlmaLinux/RHEL/Fedora
sudo dnf install -y fontconfig-devel freetype-devel

# Debian/Ubuntu
sudo apt-get install -y libfontconfig1-dev libfreetype6-dev
```

## Build

```bash
# Full workspace (debug)
cargo build --workspace

# Full workspace (release)
cargo build --workspace --release

# Tests
cargo test --workspace

# Checks
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
```

## Running

### CLI
```bash
# Basic calculation
cargo run -p mortgage_cli -- -a 185000 -t 30 -r 3.6

# With JSON config
cargo run -p mortgage_cli -- --config test_config.json

# CSV export
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --format csv --output payments.csv

# Yearly summary
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --yearly

# With prepayment
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --prepayment "2027-01-01:20000:ReduceTerm"

# Sensitivity analysis
cargo run -p mortgage_cli -- -a 200000 -t 20 -r 4.5 --sensitivity "-2,-1,0,1,2"

# Break-even vs rent (with upfront costs)
cargo run -p mortgage_cli -- -a 200000 -t 20 -r 4.5 --break-even-rent 1000 --upfront-percent 5
cargo run -p mortgage_cli -- -a 200000 -t 20 -r 4.5 --break-even-rent 1000 --upfront-cost 10000

# Save session
cargo run -p mortgage_cli -- -a 100000 -t 10 -r 5 --save session.json

# Load session
cargo run -p mortgage_cli -- --load session.json
```

### TUI
```bash
cargo run -p mortgage_tui
```

**Hotkeys in results:**
- `Y` — yearly summary
- `R` — sensitivity analysis
- `B` — break-even vs rent
- `S` — export CSV
- `W` — save session
- `L` — load session

### GUI
```bash
cargo run -p mortgage_gui
```

**Tabs:** Table, Stacked, Balance, Overlay, Yearly, Sensitivity, Break-Even

## JSON Config Examples

See `test_config.json` in the project root.

## CI

`.github/workflows/rust.yml` — automatic build, tests, fmt, and clippy on push/PR to `main`.

## Tests

73 tests covering:
- Calculator unit tests (11)
- Edge cases (19)
- Serde round-trip (19)
- Property-based tests with proptest (8)
- CLI integration (10)
- Break-even (3)
- Doc tests (3)

## Documentation

- [`AGENTS.md`](AGENTS.md) — instructions for AI agents and developers
- Per-crate README: [`mortgage_core/`](mortgage_core/README.md), [`mortgage_cli/`](mortgage_cli/README.md), [`mortgage_tui/`](mortgage_tui/README.md), [`mortgage_gui/`](mortgage_gui/README.md)

## License

MIT
