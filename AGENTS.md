# AGENTS.md

## Project

- Small Rust binary crate `mortgage_roofer`.
- Entrypoint: `src/main.rs`. Business logic in `src/mortgage_logic.rs`.
- No tests, benchmarks, or examples exist; CI still runs `cargo test --verbose`.

## Code Conventions & Quirks

- Payment types: `"annuitet"` (fixed annuity) and `"diff"` (declining balance). Any other string panics.
- `rust_decimal` is imported only for its `ToPrimitive` trait. Redundant `.to_f64().unwrap()` calls on `f64` values. `rust_decimal_macros` and `Decimal` are unused — do not introduce `Decimal` unless explicitly requested.
- `Payment::new` and `LoanResult::new` are private and never called; code uses direct struct literals.
- `chrono::NaiveDate` is used for payment date tracking; dates advance by one month via `checked_add_months`. Passing `None` as `start_date` will panic at runtime via `expect("REASON")`.
- `LoanResult::monthly_payment` is `None` for `"diff"` and `Some(...)` for `"annuitet"`.

## Toolchain

- `edition = "2024"` in Cargo.toml. Requires Rust 1.85+; ensure the local toolchain is sufficiently recent.
- No `Cargo.lock` is committed; fresh dependency resolution on each build.

## Build & Run

- `cargo build --verbose`
- `cargo test --verbose`
- `cargo run`
- No custom `rustfmt.toml` or `clippy.toml`; use defaults.

## OpenCode Config

- `opencode.json` references this file and defines `cargo` commands (build, test, run, check, clippy, fmt, fmt-check) with permissive `cargo *` bash rules.

## CI

- `.github/workflows/rust.yml`: `cargo build --verbose` + `cargo test --verbose` on push/PR to `main`.
