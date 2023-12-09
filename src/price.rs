use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue},
    StatusCode,
};

const BASE_URL: &str = "https://api.binance.com";
const KLINE_FIELDS_NUM: usize = 12;

/// Binance Public API price client.
pub struct PriceClient {
    client: Client,
}

impl PriceClient {
    /// Create new instance.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Request symbol price.
    pub fn price(&mut self, symbol: &str, datetime: DateTime<Utc>) -> Result<f64> {
        // Get UNIX timestamp in milliseconds.
        let time_ms = datetime.timestamp() * 1000;

        // Time needs to be rounded down to the nearest minute,
        // otherwise we'll get the value for the next minute.
        let start_time_ms = time_ms / 60000 * 60000;

        let url = format!(
            "{BASE_URL}/api/v3/klines?symbol={symbol}&interval=1m&startTime={start_time_ms}&limit=1"
        );

        let res = self.client.get(&url).send()?;

        let status = res.status();
        let headers = res.headers().clone();
        let body = match res.text() {
            Ok(body) => body,
            Err(e) => return Err(e).context(request_context_no_body(&url, status, &headers)),
        };

        if status.is_success() {
            Ok(extract_price_from_body(&body, start_time_ms)
                .with_context(|| request_context(&url, status, &headers, &body))?)
        } else {
            Err(anyhow!(request_context(&url, status, &headers, &body)))
        }
    }
}

/// Parse kline response body and extract close price.
fn extract_price_from_body(body: &str, start_time_ms: i64) -> Result<f64> {
    Ok(extract_price_from_payload(
        serde_json::from_str(body)?,
        start_time_ms,
    )?)
}

/// Extract close price from kline response containing at least one kline entry.
// Example respone:
// [
//   [
//     1499040000000,      // Kline open time
//     "0.01634790",       // Open price
//     "0.80000000",       // High price
//     "0.01575800",       // Low price
//     "0.01577100",       // Close price
//     "148976.11427815",  // Volume
//     1499644799999,      // Kline Close time
//     "2434.19055334",    // Quote asset volume
//     308,                // Number of trades
//     "1756.87402397",    // Taker buy base asset volume
//     "28.46694368",      // Taker buy quote asset volume
//     "0"                 // Unused field, ignore.
//   ]
// ]
//
// See https://binance-docs.github.io/apidocs/spot/en/#kline-candlestick-data
fn extract_price_from_payload(
    payload: Vec<Vec<serde_json::Value>>,
    start_time_ms: i64,
) -> Result<f64> {
    if payload.is_empty() {
        return Err(anyhow!(
            "response must contain at least one price (kline) entry"
        ));
    }

    if payload[0].len() != KLINE_FIELDS_NUM {
        return Err(anyhow!(
            "price (kline) entry contains {} fields instead of {KLINE_FIELDS_NUM}",
            payload[0].len(),
        ));
    }

    let returned_time_ms = payload[0][0]
        .as_i64()
        .ok_or(anyhow!("timestamp entry is not a number"))?;
    if returned_time_ms != start_time_ms {
        let returned = NaiveDateTime::from_timestamp_millis(returned_time_ms)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string());
        let requested = NaiveDateTime::from_timestamp_millis(start_time_ms)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string());

        return match (returned, requested) {
            (Some(returned), Some(requested)) => Err(anyhow!(
                "returned timestamp {} ({}) doesn't match requested timestamp {} ({})",
                returned_time_ms,
                returned,
                start_time_ms,
                requested,
            )),
            (_, _) => Err(anyhow!(
                "returned timestamp {} doesn't match requested timestamp {}",
                returned_time_ms,
                start_time_ms,
            )),
        };
    }

    let price_str = payload[0][4]
        .as_str()
        .ok_or(anyhow!("price entry is not a string"))?;
    let price = price_str
        .parse::<f64>()
        .with_context(|| format!("cannot convert price entry \"{price_str}\" to number"))?;
    Ok(price)
}

/// Format detailed information about response for error reporting.
fn request_context(
    url: &str,
    status: StatusCode,
    headers: &HeaderMap<HeaderValue>,
    body: &str,
) -> String {
    format!(
        "request to '{url}' failed.\n\
         Status: {}\n\
         Headers:\n{:#?}\n\
         Body:\n{}",
        status, headers, body,
    )
}

/// Format detailed information about response for error reporting in case no body is available.
fn request_context_no_body(
    url: &str,
    status: StatusCode,
    headers: &HeaderMap<HeaderValue>,
) -> String {
    format!(
        "request to '{url}' failed:\n\
         Status: {}\n\
         Headers:\n{:#?}",
        status, headers,
    )
}
