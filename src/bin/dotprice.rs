use anyhow::{anyhow, Result};
use chrono::{NaiveDate, NaiveDateTime};
use clap::Parser;

use dotgain::price::price;

/// Lookup historic coin price using Binance Public API.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Symbol to lookup.
    #[arg(short, long, default_value_t = String::from("DOTEUR"))]
    convert: String,

    /// Date & time in UTC. Example: '2023-02-21 17:53:28'.
    /// Selected minute close price will be returned.
    /// If only date is provided, time 00:00 is assumed.
    date: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let time_ms = unix_time_from_utc_string(&args.date)? * 1000;
    let price = price(&args.convert, time_ms)?;

    println!("{price:.}");

    Ok(())
}

/// Convert UTC date time string into Unix time.
fn unix_time_from_utc_string(datetime: &str) -> Result<i64> {
    if let Ok(datetime) = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S") {
        return Ok(datetime.timestamp());
    }
    if let Ok(datetime) = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M") {
        return Ok(datetime.timestamp());
    }
    if let Ok(date) = NaiveDate::parse_from_str(datetime, "%Y-%m-%d") {
        let datetime = date.and_hms_opt(0, 0, 0).expect("zero H, M, S are valid");
        return Ok(datetime.timestamp());
    }
    Err(anyhow!("invalid date supplied: {datetime}"))
}
