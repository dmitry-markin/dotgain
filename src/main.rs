use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use csv::Reader;
use dotgain::{
    price::PriceClient,
    time::{IntoHuman, TryFromHuman},
};
use rust_decimal::Decimal;
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

const DATE_COLUMN: &str = "Date";
const VALUE_COLUMN: &str = "Value";
const FIAT_INCOME_COLUMN: &str = "Fiat income";
const TOTAL_ROW: &str = "TOTAL";
const AVG_PRICE_MIN_DECIMALS: u32 = 8;

/// Create Polkadot staking tax report assuming every reward income to be equal
/// the fiat value at the time of reward.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Symbol to use for conversion to fiat.
    #[arg(short, long, default_value_t = String::from("DOTEUR"))]
    convert: String,

    /// Start date & time.
    #[arg(short, long, value_parser = DateTime::<Utc>::try_from_human)]
    begin: Option<DateTime<Utc>>,

    /// End date & time. Not inclusive.
    #[arg(short, long, value_parser = DateTime::<Utc>::try_from_human)]
    end: Option<DateTime<Utc>>,

    /// Resulting report.
    #[arg(short, long)]
    output: PathBuf,

    /// Subscan reward report in CSV.
    input: PathBuf,
}

struct InputEntry {
    datetime: DateTime<Utc>,
    value: Decimal,
}

struct OutputEntry {
    datetime: DateTime<Utc>,
    value: Decimal,
    conversion: Decimal,
    fiat_income: Decimal,
}

struct TotalsEntry {
    total_value: Decimal,
    avg_conversion: Decimal,
    total_fiat_income: Decimal,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input = read_input(&args.input)?;
    let selected = filter_range(input, args.begin, args.end);
    let report = process(selected, &args.convert)?;
    let totals = calculate_totals(&report);

    write_output(report, totals, &args.convert, &args.output)?;

    println!("\nDone");

    Ok(())
}

fn read_input(path: &Path) -> Result<Vec<InputEntry>> {
    let mut reader = Reader::from_path(path)?;
    let headers = reader.headers()?;

    let date_column = headers
        .iter()
        .position(|column| column == DATE_COLUMN)
        .with_context(|| format!("no '{DATE_COLUMN}' column found"))?;
    let value_column = headers
        .iter()
        .position(|column| column == VALUE_COLUMN)
        .with_context(|| format!("no '{VALUE_COLUMN}' column found"))?;
    let min_columns = std::cmp::max(date_column, value_column) + 1;

    let mut entries = Vec::new();

    for record in reader.records() {
        let record = record?;
        if record.len() < min_columns {
            return Err(anyhow!("not enough columns in a raw"));
        }

        let datetime = DateTime::<Utc>::try_from_human(&record[date_column])?;
        let value_str = &record[value_column];
        let value = Decimal::from_str_exact(value_str)
            .with_context(|| format!("cannot convert {value_str} to number"))?;

        entries.push(InputEntry { datetime, value });
    }

    Ok(entries)
}

fn filter_range(
    input: Vec<InputEntry>,
    begin: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
) -> Vec<InputEntry> {
    input
        .into_iter()
        .filter(|entry| {
            if let Some(begin) = begin {
                if entry.datetime < begin {
                    return false;
                }
            }
            if let Some(end) = end {
                if entry.datetime >= end {
                    return false;
                }
            }
            true
        })
        .collect()
}

fn process(input: Vec<InputEntry>, symbol: &str) -> Result<Vec<OutputEntry>> {
    let mut client = PriceClient::default();
    let mut output = Vec::new();

    let total_lines = input.len();

    for (i, entry) in input.into_iter().enumerate() {
        print_progress(i + 1, total_lines);

        let conversion = client
            .price(symbol, entry.datetime)
            .with_context(|| format!("failed to fetch price for {}", entry.datetime))?;

        output.push(OutputEntry {
            datetime: entry.datetime,
            value: entry.value,
            conversion,
            fiat_income: entry.value * conversion,
        });
    }

    Ok(output)
}

fn print_progress(current: usize, total: usize) {
    print!("\rFetching prices: {current} / {total}  ");
    let _ = io::stdout().flush();
}

fn calculate_totals(report: &[OutputEntry]) -> TotalsEntry {
    let total_value = report
        .iter()
        .fold(Decimal::ZERO, |acc, entry| acc + entry.value);
    let total_fiat_income = report
        .iter()
        .fold(Decimal::ZERO, |acc, entry| acc + entry.fiat_income);
    let avg_conversion = if !total_value.is_zero() {
        total_fiat_income / total_value
    } else {
        Decimal::ZERO
    };

    TotalsEntry {
        total_value,
        avg_conversion,
        total_fiat_income,
    }
}

fn write_output(
    report: Vec<OutputEntry>,
    totals: TotalsEntry,
    symbol: &str,
    path: &Path,
) -> Result<()> {
    let mut w = File::create(path)?;

    // Write headers.
    writeln!(
        &mut w,
        "{DATE_COLUMN},{VALUE_COLUMN},{symbol},{FIAT_INCOME_COLUMN}"
    )?;

    // Write report.
    for entry in report {
        writeln!(
            &mut w,
            "{},{},{},{}",
            entry.datetime.naive_utc().into_human(),
            entry.value.normalize(),
            entry.conversion.normalize(),
            entry.fiat_income.normalize()
        )?;
    }

    // Write totals.
    let total_value = totals.total_value.normalize();
    let total_fiat_income = totals.total_fiat_income.normalize();
    // Decimal places estimation below is not quite correct, because we count only
    // fractional decimal places.
    // For this reason we always use at least `AVG_PRICE_MIN_DECIMALS` decimal places.
    let max_decimals = std::cmp::max(total_value.scale(), total_fiat_income.scale());
    let max_decimals = std::cmp::max(max_decimals, AVG_PRICE_MIN_DECIMALS);
    let avg_conversion = totals.avg_conversion.round_dp(max_decimals).normalize();
    writeln!(
        &mut w,
        "{TOTAL_ROW},{total_value},{avg_conversion},{total_fiat_income}"
    )?;

    Ok(())
}
