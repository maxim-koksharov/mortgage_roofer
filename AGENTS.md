# AGENTS.md

> See also: [`README.md`](README.md) (user-facing overview), plus per-crate READMEs in `mortgage_core/`, `mortgage_cli/`, `mortgage_tui/`, `mortgage_gui/`.

## Project

- Cargo workspace `mortgage_roofer` with 4 member crates:
  - `mortgage_core` — business logic (models, calculator, Euribor fetcher). Library crate.
  - `mortgage_cli` — command-line interface (`clap`). Binary crate.
  - `mortgage_tui` — terminal UI (`ratatui`/`crossterm`). Binary crate.
  - `mortgage_gui` — desktop GUI (`iced`). Binary crate.
- Old single-binary layout (`src/main.rs`, `src/mortgage_logic.rs`) removed.

## Code Conventions & Quirks

- Payment types: `"annuitet"` (fixed annuity) and `"diff"` (declining balance). Any other string panics (legacy; new code uses enums).
- `rust_decimal` is imported only for its `ToPrimitive` trait. Redundant `.to_f64().unwrap()` calls on `f64` values. `rust_decimal_macros` and `Decimal` are unused — do not introduce `Decimal` unless explicitly requested.
- `chrono::NaiveDate` is used for payment date tracking; dates advance by one month via `checked_add_months`. Passing `None` as `start_date` will panic at runtime via `expect("REASON")`.
- `LoanResult::monthly_payment` is `None` for `"diff"` and `Some(...)` for `"annuitet"`.
- New models use `serde` derive for JSON serialization/deserialization.
- `f64` is used for monetary calculations (consistent with legacy code).

## System Dependencies

- `fontconfig-devel` and `freetype-devel` (via Dockerfile) required for `plotters` (used in GUI crate).
- If building outside Docker, ensure these system packages are installed.

## Toolchain

- `edition = "2024"` in all Cargo.toml files. Requires Rust 1.85+.
- No `Cargo.lock` is committed; fresh dependency resolution on each build.

## Build & Run

- `cargo build --workspace --verbose`
- `cargo test --workspace --verbose`
- `cargo run -p mortgage_cli`
- `cargo run -p mortgage_tui`
- `cargo run -p mortgage_gui`
- No custom `rustfmt.toml` or `clippy.toml`; use defaults.

## OpenCode Config

- `opencode.json` references this file and defines `cargo` commands (build, test, run, check, clippy, fmt, fmt-check) with permissive `cargo *` bash rules.

## CI

- `.github/workflows/rust.yml`: `cargo build --workspace --verbose` + `cargo test --workspace --verbose` on push/PR to `main`.
