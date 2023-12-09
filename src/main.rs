use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use csv::Reader;
use dotgain::{price::PriceClient, time::datetime_from_utc_string};
use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

const DATE_COLUMN: &str = "Date";
const VALUE_COLUMN: &str = "Value";
const FIAT_GAIN_COLUMN: &str = "Fiat gain";
const TOTAL_ROW: &str = "TOTAL";

/// Create Polkadot staking tax report assuming every reward capital gain to be equal
/// fiat value at the time of reward.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Symbol to use for conversion to fiat.
    #[arg(short, long, default_value_t = String::from("DOTEUR"))]
    convert: String,

    /// Start date & time.
    #[arg(short, long, value_parser = datetime_from_utc_string)]
    begin: Option<DateTime<Utc>>,

    /// End date & time. Not inclusive.
    #[arg(short, long, value_parser = datetime_from_utc_string)]
    end: Option<DateTime<Utc>>,

    /// Resulting report.
    #[arg(short, long)]
    output: PathBuf,

    /// Subscan reward report in CSV.
    input: PathBuf,
}

struct InputEntry {
    datetime: String,
    value: f64,
}

struct OutputEntry {
    datetime: String,
    value: f64,
    conversion: f64,
    fiat_gain: f64,
}

struct TotalsEntry {
    total_value: f64,
    avg_conversion: f64,
    total_fiat_gain: f64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input = read_input(&args.input)?;
    let data = filter_range(input, args.begin, args.end)?;
    let report = process(data, &args.convert)?;
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

        let datetime = record[date_column].to_string();
        let value_str = &record[value_column];
        let value = value_str
            .parse::<f64>()
            .with_context(|| format!("cannot convert {value_str} to number"))?;

        entries.push(InputEntry { datetime, value });
    }

    Ok(entries)
}

fn filter_range(
    input: Vec<InputEntry>,
    begin: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
) -> Result<Vec<InputEntry>> {
    let mut output = Vec::new();

    for entry in input {
        let datetime =
            datetime_from_utc_string(&entry.datetime).context("invalid date in input data")?;
        if let Some(begin) = begin {
            if datetime < begin {
                continue;
            }
        }
        if let Some(end) = end {
            if datetime >= end {
                continue;
            }
        }
        output.push(entry);
    }

    Ok(output)
}

fn process(input: Vec<InputEntry>, symbol: &str) -> Result<Vec<OutputEntry>> {
    let mut client = PriceClient::new();
    let mut output = Vec::new();

    let total_lines = input.len();

    for (i, entry) in input.into_iter().enumerate() {
        print_progress(i + 1, total_lines);

        let datetime =
            datetime_from_utc_string(&entry.datetime).context("invalid date in input data")?;
        let conversion = client
            .price(symbol, datetime)
            .with_context(|| format!("failed to fetch price for {}", entry.datetime))?;

        output.push(OutputEntry {
            datetime: entry.datetime.clone(),
            value: entry.value,
            conversion,
            fiat_gain: entry.value * conversion,
        });
    }

    Ok(output)
}

fn print_progress(current: usize, total: usize) {
    print!("\rFetching prices: {current} / {total}  ");
    let _ = io::stdout().flush();
}

fn calculate_totals(report: &Vec<OutputEntry>) -> TotalsEntry {
    let total_value = report.iter().fold(0f64, |acc, entry| acc + entry.value);
    let total_fiat_gain = report.iter().fold(0f64, |acc, entry| acc + entry.fiat_gain);
    let avg_conversion = total_fiat_gain / total_value;

    TotalsEntry {
        total_value,
        avg_conversion,
        total_fiat_gain,
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
        "{DATE_COLUMN},{VALUE_COLUMN},{symbol},{FIAT_GAIN_COLUMN}"
    )?;

    // Write report.
    for OutputEntry {
        datetime,
        value,
        conversion,
        fiat_gain,
    } in report
    {
        writeln!(&mut w, "{datetime},{value:.},{conversion:.},{fiat_gain:.}")?;
    }

    // Write totals.
    let TotalsEntry {
        total_value,
        avg_conversion,
        total_fiat_gain,
    } = totals;
    writeln!(
        &mut w,
        "{TOTAL_ROW},{total_value:.},{avg_conversion:.},{total_fiat_gain:.}"
    )?;

    Ok(())
}
