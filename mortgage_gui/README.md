# mortgage_gui

[Русская версия](README.ru.md)

Desktop GUI for the mortgage calculator built with `iced`, featuring a themed interface, vector charts, and PDF export.

## Running

```bash
cargo run -p mortgage_gui
```

## Interface

The window is split into two panels:
- **Left** — parameter input form (scrollable, sections with headers)
- **Right** — results, charts, and analysis

### Design

- **Theme**: TokyoNightStorm (dark theme)
- **Sections**: Loan Parameters, Rate Configuration, Prepayments, Actions
- **Validation**: red borders on invalid fields (amount, term, date)
- **Status bar**: green background for success, red for errors

### Input Form

| Field | Description |
|-------|-------------|
| Amount | Loan amount |
| Term (years) | Term in years |
| Start date | Start date (YYYY-MM-DD) |
| Currency | EUR / USD |
| Payment type | Annuity / Diff |
| Rate mode | Fix / Euribor / Mixed |

Fields are dynamically shown/hidden depending on the selected rate mode:

**Fix:** Rate (%), Spread (%)

**Euribor:** Tenor (1m/3m/6m/12m), Spread (%)

**Mixed:** Fixed years, Fix rate (%), Fix spread (%), Euribor tenor, Euribor spread (%), Same spread checkbox

**Prepayments (multiple):**
- Date, Amount, Effect (ReduceTerm/ReducePayment)
- "+ Add Prepayment" button — adds to the list
- List of prepayments with "X" buttons for removal

**Break-even upfront costs:**
- Upfront cost — fixed amount (0 = not used)
- Upfront percent — percentage of loan amount (0 = not used)

### Action Buttons

- **Calculate** — calculate payment schedule
- **Export CSV** — export to `/tmp/mortgage_payments.csv`
- **Export PDF** — report to `/tmp/mortgage_report.pdf`
- **Save Session** — save to `/tmp/mortgage_session.json`
- **Load Session** — load from `/tmp/mortgage_session.json`

### Results

After pressing **Calculate**, the right panel shows:
- Summary: Monthly, Total Principal, Total Interest, Total Paid, Payments count
- Principal > Interest crossover point
- Status bar with message (green — success, red — error)

### Tabs

The results window has 7 tabs:

| Tab | Description |
|-----|-------------|
| **Table** | Full payment schedule with scrolling |
| **Stacked** | Stacked bar chart: Principal (green) + Interest (red) |
| **Balance** | Line chart of remaining balance |
| **Overlay** | Combined: principal + interest + balance |
| **Yearly** | Yearly summary (year, payment, principal, interest, months, balance) |
| **Sensitivity** | Sensitivity analysis (±2%, ±1%, ±0.5%, 0%) |
| **Break-Even** | Break-even vs rent (with rent input field) |

### Charts

All charts are rendered as SVG via `plotters` and displayed in `iced::widget::svg`:

- **Stacked Bar**: Principal (green) + Interest (red) + crossover marker (blue dot)
- **Balance Line**: remaining balance line (blue)
- **Overlay**: three lines — principal (green), interest (red), balance (blue)

### Sensitivity Analysis

The **Sensitivity** tab shows a table:
- Delta — rate change
- Rate % — effective rate
- Monthly — monthly payment
- Interest — total interest
- Total Paid — total amount paid

### Break-Even vs Rent

The **Break-Even** tab shows:
- Monthly rent (with input field)
- Monthly mortgage
- Upfront costs
- Total interest
- Break-even (months and years)
- Explanation

## Export

### CSV
Click **Export CSV** → file is saved to `/tmp/mortgage_payments.csv`

### PDF
Click **Export PDF** → file is saved to `/tmp/mortgage_report.pdf`

Contains:
- **Page 1**: summary + table of first 60 payments
- **Page 2**: embedded chart (PNG → PDF)

## Sessions

- **Save Session** — saves parameters and results to `/tmp/mortgage_session.json`
- **Load Session** — loads session, restores all fields and results

## System Dependencies

The GUI uses `iced` (wgpu/tiny-skia) and `plotters` (SVG backend). Make sure the following are installed:
```bash
# AlmaLinux/RHEL
sudo dnf install -y fontconfig-devel freetype-devel

# Debian/Ubuntu
sudo apt-get install -y libfontconfig1-dev libfreetype6-dev
```

## Build

```bash
# Debug
cargo build -p mortgage_gui

# Release
cargo build --release -p mortgage_gui
```

Binary: `target/release/mortgage_gui`

## Dependencies

- `iced` — GUI framework (with `svg` feature)
- `plotters` — vector charts (SVG + bitmap backends)
- `printpdf` — PDF generation (with `embedded_images` feature)
- `image` — PNG encoding for embedding in PDF
- `mortgage_core` — calculations, analysis, sessions

## Notes

- Charts are rendered as SVG via `plotters` and displayed in `iced::widget::svg`
- For PDF, the chart is first rendered as PNG via `plotters` bitmap backend, then embedded via `printpdf::Image`
- If no GPU is available, `iced` automatically falls back to software rendering
- Field validation shows red borders on invalid input
- Status bar changes color depending on status (green — success, red — error)
