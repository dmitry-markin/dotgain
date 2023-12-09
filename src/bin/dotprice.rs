use anyhow::Result;
use clap::Parser;

use dotgain::{price::PriceClient, time::datetime_from_utc_string};

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
    let client = PriceClient::new();
    let datetime = datetime_from_utc_string(&args.date)?;
    let price = client.price(&args.convert, datetime)?;

    println!("{price:.}");

    Ok(())
}
