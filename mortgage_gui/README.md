# mortgage_gui

Десктоп GUI для ипотечного калькулятора на `iced` с векторными графиками и экспортом PDF.

## Запуск

```bash
cargo run -p mortgage_gui
```

## Интерфейс

Окно разделено на две панели:
- **Левая** — форма ввода параметров
- **Правая** — результаты, графики и таблица

### Форма ввода

| Поле | Описание |
|------|----------|
| Amount | Сумма кредита |
| Term (yrs) | Срок в годах |
| Currency | EUR / USD |
| Payment | Annuity / Diff |
| Rate mode | Fix / Euribor / Mixed |

Поля динамически показываются/скрываются в зависимости от выбранного режима ставки:

**Fix:** Rate (%), Spread (%)

**Euribor:** Tenor (1m/3m/6m/12m), Spread (%)

**Mixed:** Fixed years, Fix rate (%), Fix spread (%), Euribor tenor, Euribor spread (%), Same spread checkbox

**Prepayment:** Date, Amount, Effect (ReduceTerm/ReducePayment)

### Результаты

После нажатия **Calculate** в правой панели появляется:
- Сводка: Monthly, Total Principal, Total Interest, Total Paid, Payments count
- Точка пересечения Principal > Interest

### Таблица платежей

Вкладка **Table** — полная таблица платежей с прокруткой:
- #, Date, Payment, Principal, Interest, Balance

### Графики

Вкладка **Chart** — векторный SVG-график:
- **Stacked bar**: Principal (зелёный) + Interest (красный)
- **Маркер пересечения**: синяя точка и аннотация, где Principal впервые превышает Interest

## Экспорт

### CSV
Нажмите **Export CSV** → файл сохраняется в `/tmp/mortgage_payments.csv`

### PDF
Нажмите **Export PDF** → файл сохраняется в `/tmp/mortgage_report.pdf`

Содержит:
- **Страница 1**: сводка + таблица первых 60 платежей
- **Страница 2**: встроенный график (PNG → PDF)

## Системные зависимости

GUI использует `iced` (wgpu/tiny-skia) и `plotters` (SVG backend). Убедитесь, что установлены:
```bash
# AlmaLinux/RHEL
sudo dnf install -y fontconfig-devel freetype-devel

# Debian/Ubuntu
sudo apt-get install -y libfontconfig1-dev libfreetype6-dev
```

## Сборка

```bash
# Debug
cargo build -p mortgage_gui

# Release
cargo build --release -p mortgage_gui
```

Бинарник: `target/release/mortgage_gui`

## Зависимости

- `iced` — GUI framework (with `svg` feature)
- `plotters` — векторные графики (SVG + bitmap backends)
- `printpdf` — PDF генерация (with `embedded_images` feature)
- `image` — PNG кодирование для встраивания в PDF
- `mortgage_core` — расчёты

## Примечания

- График рендерится в SVG через `plotters` и отображается в `iced::widget::svg`
- Для PDF график сначала рендерится в PNG через `plotters` bitmap backend, затем встраивается через `printpdf::Image`
- При отсутствии графического адаптера `iced` автоматически переключается на программный рендерер
