use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};

/// Convert UTC date time string into `DateTime`.
pub fn datetime_from_utc_string(datetime: &str) -> Result<DateTime<Utc>> {
    if let Ok(naive_datetime) = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S") {
        return Ok(Utc.from_utc_datetime(&naive_datetime));
    }
    if let Ok(naive_datetime) = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M") {
        return Ok(Utc.from_utc_datetime(&naive_datetime));
    }
    if let Ok(naive_date) = NaiveDate::parse_from_str(datetime, "%Y-%m-%d") {
        let naive_datetime = naive_date
            .and_hms_opt(0, 0, 0)
            .expect("zero H, M, S are valid");
        return Ok(Utc.from_utc_datetime(&naive_datetime));
    }
    Err(anyhow!("invalid date supplied: {datetime}"))
}
