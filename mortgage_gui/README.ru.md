# mortgage_gui

[English version](README.md)

Десктоп GUI для ипотечного калькулятора на `iced` с темизированным интерфейсом, векторными графиками и экспортом PDF.

## Запуск

```bash
cargo run -p mortgage_gui
```

## Интерфейс

Окно разделено на две панели:
- **Левая** — форма ввода параметров (scrollable, секции с заголовками)
- **Правая** — результаты, графики и анализ

### Дизайн

- **Тема**: TokyoNightStorm (тёмная тема)
- **Секции**: Loan Parameters, Rate Configuration, Prepayments, Actions
- **Валидация**: красные рамки на невалидных полях (amount, term, date)
- **Status bar**: зелёный фон для успеха, красный для ошибок

### Форма ввода

| Поле | Описание |
|------|----------|
| Amount | Сумма кредита |
| Term (years) | Срок в годах |
| Start date | Дата начала (YYYY-MM-DD) |
| Currency | EUR / USD |
| Payment type | Annuity / Diff |
| Rate mode | Fix / Euribor / Mixed |

Поля динамически показываются/скрываются в зависимости от выбранного режима ставки:

**Fix:** Rate (%), Spread (%)

**Euribor:** Tenor (1m/3m/6m/12m), Spread (%)

**Mixed:** Fixed years, Fix rate (%), Fix spread (%), Euribor tenor, Euribor spread (%), Same spread checkbox

**Prepayments (множественные):**
- Date, Amount, Effect (ReduceTerm/ReducePayment)
- Кнопка "+ Add Prepayment" — добавляет в список
- Список prepayments с кнопками "X" для удаления

**Break-even upfront costs:**
- Upfront cost — фиксированная сумма (0 = не используется)
- Upfront percent — процент от суммы кредита (0 = не используется)

### Кнопки действий

- **Calculate** — расчёт графика платежей
- **Export CSV** — экспорт в `/tmp/mortgage_payments.csv`
- **Export PDF** — отчёт в `/tmp/mortgage_report.pdf`
- **Save Session** — сохранение в `/tmp/mortgage_session.json`
- **Load Session** — загрузка из `/tmp/mortgage_session.json`

### Результаты

После нажатия **Calculate** в правой панели появляется:
- Сводка: Monthly, Total Principal, Total Interest, Total Paid, Payments count
- Точка пересечения Principal > Interest
- Status bar с сообщением (зелёный — успех, красный — ошибка)

### Вкладки

Окно результатов имеет 7 вкладок:

| Вкладка | Описание |
|---------|----------|
| **Table** | Полная таблица платежей с прокруткой |
| **Stacked** | Stacked bar chart: Principal (зелёный) + Interest (красный) |
| **Balance** | Линейный график остатка долга |
| **Overlay** | Комбинированный: principal + interest + balance |
| **Yearly** | Годовая сводка (year, payment, principal, interest, months, balance) |
| **Sensitivity** | Анализ чувствительности (±2%, ±1%, ±0.5%, 0%) |
| **Break-Even** | Break-even vs аренда (с полем ввода rent) |

### Графики

Все графики рендерятся в SVG через `plotters` и отображаются в `iced::widget::svg`:

- **Stacked Bar**: Principal (зелёный) + Interest (красный) + маркер пересечения (синяя точка)
- **Balance Line**: линия остатка долга (синяя)
- **Overlay**: три линии — principal (зелёная), interest (красная), balance (синяя)

### Анализ чувствительности

Вкладка **Sensitivity** показывает таблицу:
- Delta — изменение ставки
- Rate % — эффективная ставка
- Monthly — ежемесячный платёж
- Interest — общие проценты
- Total Paid — общая выплата

### Break-Even vs Rent

Вкладка **Break-Even** показывает:
- Monthly rent (с полем ввода)
- Monthly mortgage
- Upfront costs
- Total interest
- Break-even (месяцы и годы)
- Explanation

## Экспорт

### CSV
Нажмите **Export CSV** → файл сохраняется в `/tmp/mortgage_payments.csv`

### PDF
Нажмите **Export PDF** → файл сохраняется в `/tmp/mortgage_report.pdf`

Содержит:
- **Страница 1**: сводка + таблица первых 60 платежей
- **Страница 2**: встроенный график (PNG → PDF)

## Сессии

- **Save Session** — сохраняет параметры и результаты в `/tmp/mortgage_session.json`
- **Load Session** — загружает сессию, восстанавливает все поля и результаты

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
- `mortgage_core` — расчёты, анализ, сессии

## Примечания

- График рендерится в SVG через `plotters` и отображается в `iced::widget::svg`
- Для PDF график сначала рендерится в PNG через `plotters` bitmap backend, затем встраивается через `printpdf::Image`
- При отсутствии графического адаптера `iced` автоматически переключается на программный рендерер
- Валидация полей показывает красные рамки при некорректном вводе
- Status bar меняет цвет в зависимости от статуса (зелёный — успех, красный — ошибка)
