use super::error::Error;
use axum::{Json, extract::Path};
use chrono::{DateTime, TimeZone, Utc};
use serde::Serialize;

/// The lowest value treated as a Snowflake ID rather than a legacy sequential ID.
const FIRST_SNOWFLAKE: i64 = 250000000000000;

/// The Twitter epoch (2010-11-04T01:42:54.657Z) in Unix milliseconds.
const TWITTER_EPOCH_MILLIS: i64 = 1288834974657;

/// The timestamp encoded in a Twitter Snowflake ID.
#[derive(Debug, Serialize)]
pub struct SnowflakeInfo {
    /// Seconds since the Unix epoch.
    #[serde(rename = "epoch-second")]
    epoch_second: i64,
    /// The timestamp in RFC 2822 format.
    #[serde(rename = "utc-rfc2822")]
    utc_rfc2822: String,
}

/// Extract the timestamp from a Snowflake ID, or `None` for pre-Snowflake IDs.
fn snowflake_to_date_time(value: i64) -> Option<DateTime<Utc>> {
    if value >= FIRST_SNOWFLAKE {
        // The top 41 bits (after a sign bit) encode milliseconds since the Twitter epoch.
        let timestamp_millis = (value >> 22) + TWITTER_EPOCH_MILLIS;

        Utc.timestamp_millis_opt(timestamp_millis).single()
    } else {
        None
    }
}

/// Handle `GET /tw/util/snowflake/{id}` by decoding the ID's embedded timestamp.
///
/// # Errors
///
/// Returns a 404 via [`Error::InvalidSnowflake`] if the value is not a Snowflake ID.
pub async fn info(Path(id): Path<i64>) -> Result<Json<SnowflakeInfo>, Error> {
    let timestamp = snowflake_to_date_time(id).ok_or(Error::InvalidSnowflake(id))?;

    Ok(Json(SnowflakeInfo {
        epoch_second: timestamp.timestamp(),
        utc_rfc2822: timestamp.to_rfc2822(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snowflake_timestamps_decode() {
        let timestamp = snowflake_to_date_time(1213770116885774336).expect("valid Snowflake");
        assert_eq!(timestamp.timestamp(), 1578220321);
    }

    #[test]
    fn pre_snowflake_ids_are_rejected() {
        assert_eq!(snowflake_to_date_time(20), None);
    }
}
