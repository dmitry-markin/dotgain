# dotgain

Polkadot staking tax report generator. Generates a staking capital income report from a [Subscan](https://www.subscan.io/) CSV using the [Binance Public API](https://binance-docs.github.io/apidocs/spot/en/#kline-candlestick-data).

## Usage

Create a report in EUR for the complete time range in the input CSV:
```
dotgain -o output.csv subscan_report.csv
```

Create a report in USDT for 2022:
```
dotgain --convert DOTUSDT --begin 2022-01-01 --end 2023-01-01 -o output.csv subscan_report.csv
```

See `dotgain -h` for the usage description.

Note that all date & time data is expected to be in UTC.

## Example input (not required columns omitted)

| Date                | Value   |
|---------------------|---------|
| 2022-01-01 01:57:03 | 0.532   |
| 2022-01-02 01:56:56 | 0.5214  |
| 2022-01-03 12:25:16 | 0.5256  |
| 2022-01-04 00:31:02 | 0.5334  |
| 2022-01-05 23:31:01 | 0.5264  |
| 2022-01-06 19:11:22 | 0.54465 |
| 2022-01-07 07:03:00 | 0.543   |
| 2022-01-08 00:29:31 | 0.52675 |
| 2022-01-09 15:40:49 | 0.5398  |
| 2022-01-10 22:09:37 | 0.5234  |
| ...                 |         |
| 2022-12-31 07:17:22 | 0.54234 |

## Example output

| Date                | Value   | DOTEUR      | Fiat gain    |
|---------------------|---------|-------------|--------------|
| 2022-01-01 01:57:03 | 0.532   | 23.93       | 12.73076     |
| 2022-01-02 01:56:56 | 0.5214  | 24.79       | 12.925506    |
| 2022-01-03 12:25:16 | 0.5256  | 26.7        | 14.03352     |
| 2022-01-04 00:31:02 | 0.5334  | 26.78       | 14.284452    |
| 2022-01-05 23:31:01 | 0.5264  | 23.84       | 12.549376    |
| 2022-01-06 19:11:22 | 0.54465 | 24.11       | 13.1315115   |
| 2022-01-07 07:03:00 | 0.543   | 21.85       | 11.86455     |
| 2022-01-08 00:29:31 | 0.52675 | 22.14       | 11.662245    |
| 2022-01-09 15:40:49 | 0.5398  | 21.52       | 11.616496    |
| 2022-01-10 22:09:37 | 0.5234  | 20.88       | 10.928592    |
| ...                 |         |             |              |
| 2022-12-31 07:17:22 | 0.54234 | 4.038       | 2.18996892   |
| TOTAL               | 5.85874 | 21.83353032 | 127.91697742 |
