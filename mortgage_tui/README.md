# mortgage_tui

[Русская версия](README.ru.md)

Terminal UI for the mortgage calculator built with `ratatui` + `crossterm`.

## Running

```bash
cargo run -p mortgage_tui
```

## Navigation

### Input Form

| Key | Action |
|-----|--------|
| `Tab` / `↓` | Next field |
| `Shift+Tab` / `↑` | Previous field |
| `←` / `→` | Toggle enum values (currency, payment type, rate mode, etc.) |
| Digits / `.` / `-` | Enter numeric values and dates |
| `Backspace` | Delete character |
| `Enter` | Calculate / Add prepayment (on AddPrepayment field) |
| `Delete` | Remove last prepayment |
| `Esc` / `q` | Quit |

### Form Fields

1. **Amount** — loan amount
2. **Term (yrs)** — term in years
3. **Start date** — start date (YYYY-MM-DD)
4. **Currency** — EUR / USD (toggle ←→)
5. **Payment type** — Annuity / Diff (toggle ←→)
6. **Rate mode** — Fix / Euribor / Mixed (toggle ←→)

Depending on the selected rate mode, the corresponding fields appear dynamically:

**Fix:**
- Fix rate (%)
- Fix spread (%)

**Euribor:**
- Euribor tenor — 1m / 3m / 6m / 12m
- Euribor spread (%)

**Mixed:**
- Fixed years — duration of the fixed period
- Mixed fix rate (%)
- Mixed fix spread (%)
- Mixed euribor tenor
- Mixed euribor spread (%) — hidden when same_spread is enabled
- Same spread — Yes / No (toggle ←→)

**Prepayments (multiple):**
- Prepayment date — YYYY-MM-DD
- Prepayment amount
- Prepayment effect — ReduceTerm / ReducePayment
- Add prepayment — Enter to add, Delete to remove last

**Break-even upfront costs:**
- Upfront cost — fixed amount (0 = not used)
- Upfront percent — percentage of loan amount (0 = not used)

### Results

After pressing `Enter`, the results screen is displayed:

**Top panel — summary:**
- Amount, term, payment type, rate mode
- Total principal repaid
- Total interest paid
- Total amount paid
- Monthly payment (for annuity)
- Crossover point: when Principal > Interest

**Bottom panel — payment schedule:**
- Columns: #, Date, Payment, Principal, Interest, Balance
- Scrolling: `↑` / `↓` or `PgUp` / `PgDown`

### Hotkeys in Results

| Key | Action |
|-----|--------|
| `Esc` / `q` | Return to form / close analysis (quit from form) |
| `S` | Export table to CSV (`/tmp/mortgage_tui_export.csv`) |
| `Y` | Toggle yearly summary |
| `R` | Sensitivity analysis (±2%, ±1%, ±0.5%, 0%) |
| `B` | Break-even vs rent (automatically 0.5% of amount) |
| `W` | Save session (`/tmp/mortgage_session.json`) |
| `L` | Load session |
| `↑` / `↓` | Scroll table |

### Yearly Summary

When pressing `Y`, yearly aggregation is displayed:
- Year — calendar year
- Payment — total payments for the year
- Principal — principal repaid
- Interest — interest paid
- Months — number of payments
- Balance — ending balance

### Sensitivity Analysis

When pressing `R`, a table is displayed:
- Delta — rate change
- Rate % — effective rate
- Monthly — monthly payment
- Interest — total interest
- Total Paid — total amount paid

### Break-Even vs Rent

When pressing `B`, the following is displayed:
- Monthly rent — monthly rent (0.5% of amount)
- Monthly mortgage — monthly payment
- Upfront costs — initial costs
- Total interest — total interest
- Break-even — how many months until buying pays off

### Sessions

- `W` — saves current parameters and results to `/tmp/mortgage_session.json`
- `L` — loads session from `/tmp/mortgage_session.json`

### Validation

On input errors (invalid date, negative number, etc.) a popup with the error description appears. Press any key to close.

## Build

```bash
cargo build --release -p mortgage_tui
```

Binary: `target/release/mortgage_tui`
