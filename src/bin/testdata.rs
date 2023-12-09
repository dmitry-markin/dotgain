use anyhow::{anyhow, Result};
use chrono::{Days, NaiveDate, NaiveTime};
use clap::Parser;
use dotgain::time::date_from_string;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
};

/// Create mock staking report for smoke testing.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start date
    #[arg(short, long, value_parser = date_from_string)]
    begin: NaiveDate,

    /// End date (not inclusive)
    #[arg(short, long, value_parser = date_from_string)]
    end: NaiveDate,
}

struct RandomTime {
    rng: ThreadRng,
    day_seconds: Uniform<u32>,
}

impl RandomTime {
    fn new() -> Self {
        Self {
            rng: ThreadRng::default(),
            day_seconds: Uniform::from(0..(24 * 60 * 60)),
        }
    }

    fn sample(&mut self) -> NaiveTime {
        let seconds = self.day_seconds.sample(&mut self.rng);
        NaiveTime::from_num_seconds_from_midnight_opt(seconds, 0)
            .expect("Uniform distribution above provides seconds within 24h range.")
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.begin >= args.end {
        return Err(anyhow!("end date must be greater than begin date"));
    }

    // Print header.
    println!("Date,Value");

    let mut random_time = RandomTime::new();

    let mut date = args.begin;
    while date < args.end {
        let datetime = date
            .and_time(random_time.sample())
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        // Print row.
        println!("{datetime},1");

        date = date
            .checked_add_days(Days::new(1))
            .expect("date is <= end date, and end date is valid; qed");
    }

    Ok(())
}
