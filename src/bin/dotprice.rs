use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use dotgain::{price::PriceClient, time::TryFromHuman};

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
    let mut client = PriceClient::default();
    let datetime = DateTime::<Utc>::try_from_human(&args.date)?;
    let price = client.price(&args.convert, datetime)?;

    println!("{price}");

    Ok(())
}
