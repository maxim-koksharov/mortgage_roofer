# mortgage_tui

Терминальный интерфейс для ипотечного калькулятора на `ratatui` + `crossterm`.

## Запуск

```bash
cargo run -p mortgage_tui
```

## Навигация

### Форма ввода

| Клавиша | Действие |
|---------|----------|
| `Tab` / `↓` | Следующее поле |
| `Shift+Tab` / `↑` | Предыдущее поле |
| `←` / `→` | Переключение enum (валюта, тип платежа, режим ставки и т.д.) |
| Цифры / `.` | Ввод числовых значений |
| `Backspace` | Удаление символа |
| `Enter` | Расчёт |
| `Esc` | Выход |

### Поля формы

1. **Amount** — сумма кредита
2. **Term (yrs)** — срок в годах
3. **Currency** — EUR / USD (переключение ←→)
4. **Payment type** — Annuity / Diff (переключение ←→)
5. **Rate mode** — Fix / Euribor / Mixed (переключение ←→)

При выборе режима ставки динамически появляются соответствующие поля:

**Fix:**
- Fix rate (%)
- Fix spread (%)

**Euribor:**
- Euribor tenor — 1m / 3m / 6m / 12m
- Euribor spread (%)

**Mixed:**
- Fixed years — срок фиксированного периода
- Mixed fix rate (%)
- Mixed fix spread (%)
- Mixed euribor tenor
- Mixed euribor spread (%) — скрывается при same_spread
- Same spread — Yes / No (переключение ←→)

**Prepayment:**
- Prepayment date — YYYY-MM-DD
- Prepayment amount
- Prepayment effect — ReduceTerm / ReducePayment

### Результаты

После нажатия `Enter` отображается экран результатов:

**Верхняя панель — сводка:**
- Сумма, срок, тип платежа, режим ставки
- Общая сумма погашения основного долга
- Общая сумма процентов
- Итоговая переплата
- Ежемесячный платёж (для аннуитета)
- Точка пересечения: когда Principal > Interest

**Нижняя панель — таблица платежей:**
- Столбцы: #, Date, Payment, Principal, Interest, Balance
- Прокрутка: `↑` / `↓` или `PgUp` / `PgDown`

### Горячие клавиши в результатах

| Клавиша | Действие |
|---------|----------|
| `Esc` / `q` | Вернуться к форме |
| `S` | Экспортировать таблицу в CSV (`/tmp/mortgage_tui_export.csv`) |
| `↑` / `↓` | Прокрутка таблицы |

### Валидация

При ошибках ввода (некорректная дата, отрицательное число и т.д.) появляется popup с описанием ошибки. Нажмите любую клавишу для закрытия.

## Сборка

```bash
cargo build --release -p mortgage_tui
```

Бинарник: `target/release/mortgage_tui`
