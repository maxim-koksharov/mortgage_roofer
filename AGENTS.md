# AGENTS.md

> See also: [`README.md`](README.md) (user-facing overview), plus per-crate READMEs in `mortgage_core/`, `mortgage_cli/`, `mortgage_tui/`, `mortgage_gui/`.

## Project

- Cargo workspace `mortgage_roofer` with 4 member crates:
  - `mortgage_core` — business logic (models, calculator, analysis, Euribor fetcher, sessions). Library crate.
  - `mortgage_cli` — command-line interface (`clap`). Binary crate.
  - `mortgage_tui` — terminal UI (`ratatui`/`crossterm`). Binary crate.
  - `mortgage_gui` — desktop GUI (`iced`). Binary crate.
- Old single-binary layout (`src/main.rs`, `src/mortgage_logic.rs`) removed.

## Code Conventions & Quirks

- Payment types: `"annuitet"` (fixed annuity) and `"diff"` (declining balance). Any other string panics (legacy; new code uses enums).
- `chrono::NaiveDate` is used for payment date tracking; dates advance by one month via `checked_add_months`. Passing `None` as `start_date` will panic at runtime via `expect("REASON")`.
- `LoanResult::monthly_payment` is `None` for `"diff"` and `Some(...)` for `"annuitet"`.
- All models use `serde` derive for JSON serialization/deserialization.
- `f64` is used for monetary calculations (consistent with legacy code).
- `EuriborTenor` uses `#[derive(Default)]` with `#[default]` attribute on `SixMonths`.
- Analysis structs (`SensitivityPoint`, `BreakEvenResult`) implement `PartialEq` for TUI state comparison.

## Features

### Core (mortgage_core)
- **Models**: LoanParams, Payment, LoanResult, RateMode, Currency, PaymentType, Prepayment, YearlySummary
- **Calculator**: annuity and diff payments, multiple prepayments, mixed rate mode, validation
- **Analysis**: sensitivity_analysis(), break_even_analysis()
- **Export**: payments_to_csv()
- **Session**: save_session(), load_session()
- **Euribor**: fetch_euribor(), EuriborCache

### CLI (mortgage_cli)
- All core features via command-line arguments
- `--yearly` flag for yearly summary
- `--prepayment "YYYY-MM-DD:amount:effect"` (repeatable)
- `--save FILE` / `--load FILE` for sessions
- `--sensitivity "-2,-1,0,1,2"` for rate sensitivity
- `--break-even-rent 1000` for break-even analysis
- `--config FILE` for JSON config
- `--format csv` / `--output FILE` for CSV export

### TUI (mortgage_tui)
- Form with all parameters including start_date
- Multiple prepayments (Enter to add, Delete to remove last)
- Results screen with hotkeys:
  - `Y` — toggle yearly summary
  - `R` — rate sensitivity analysis
  - `B` — break-even vs rent
  - `S` — export CSV
  - `W` — save session
  - `L` — load session
- Popup for errors and messages

### GUI (mortgage_gui)
- Theme: TokyoNightStorm
- Sections: Loan Parameters, Rate Configuration, Prepayments, Actions
- Input validation with red borders
- Color-coded status bar (green=success, red=error)
- Multiple prepayments with add/remove buttons
- 7 tabs: Table, Stacked, Balance, Overlay, Yearly, Sensitivity, Break-Even
- Session save/load buttons
- PDF export with chart

## System Dependencies

- `fontconfig-devel` and `freetype-devel` (via Dockerfile) required for `plotters` (used in GUI crate).
- If building outside Docker, ensure these system packages are installed.

## Toolchain

- `edition = "2024"` in all Cargo.toml files. Requires Rust 1.85+.
- No `Cargo.lock` is committed; fresh dependency resolution on each build.
- Workspace dependencies: chrono, serde, serde_json, thiserror, ureq, clap, ratatui, crossterm, iced, printpdf.

## Build & Run

- `cargo build --workspace` (debug)
- `cargo build --workspace --release` (release)
- `cargo test --workspace` (70 tests)
- `cargo fmt --all -- --check` (formatting)
- `cargo clippy --workspace -- -D warnings` (linting)
- `cargo run -p mortgage_cli`
- `cargo run -p mortgage_tui`
- `cargo run -p mortgage_gui`
- No custom `rustfmt.toml` or `clippy.toml`; use defaults.

## Testing

70 tests total:
- **Unit tests** (11): calculator basic operations
- **Edge cases** (19): zero rate, large amounts, prepayments
- **Serde round-trip** (19): all models serialization
- **Property-based** (8): proptest invariants
- **CLI integration** (10): argument parsing, output formats
- **Doc tests** (3): public API examples

## OpenCode Config

- `opencode.json` references this file and defines `cargo` commands (build, test, run, check, clippy, fmt, fmt-check) with permissive `cargo *` bash rules.

## CI

- `.github/workflows/rust.yml`:
  - `cargo build --workspace --verbose`
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo test --workspace --verbose`
  - Runs on push/PR to `main`.
