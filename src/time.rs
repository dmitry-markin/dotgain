use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};

/// `TryFrom` alternative for conversions from human formats.
pub trait TryFromHuman
where
    Self: Sized,
{
    type Error;

    fn try_from_human(string: &str) -> Result<Self, Self::Error>;
}

impl TryFromHuman for DateTime<Utc> {
    type Error = anyhow::Error;

    fn try_from_human(string: &str) -> Result<DateTime<Utc>> {
        if let Ok(naive_datetime) = NaiveDateTime::parse_from_str(string, "%Y-%m-%d %H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&naive_datetime));
        }
        if let Ok(naive_datetime) = NaiveDateTime::parse_from_str(string, "%Y-%m-%d %H:%M") {
            return Ok(Utc.from_utc_datetime(&naive_datetime));
        }
        if let Ok(naive_date) = NaiveDate::parse_from_str(string, "%Y-%m-%d") {
            let naive_datetime = naive_date
                .and_hms_opt(0, 0, 0)
                .expect("zero H, M, S are valid");
            return Ok(Utc.from_utc_datetime(&naive_datetime));
        }
        Err(anyhow!("invalid date: {string}"))
    }
}

impl TryFromHuman for NaiveDate {
    type Error = anyhow::Error;

    fn try_from_human(string: &str) -> Result<NaiveDate> {
        NaiveDate::parse_from_str(string, "%Y-%m-%d").context("invalid date")
    }
}

/// `Into` alternative for conversion into human readable format.
pub trait IntoHuman {
    fn into_human(self) -> String;
}

impl IntoHuman for NaiveDateTime {
    fn into_human(self: NaiveDateTime) -> String {
        self.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
