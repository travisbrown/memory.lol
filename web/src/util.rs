use chrono::{DateTime, TimeZone, Utc};

const FIRST_SNOWFLAKE: i64 = 250000000000000;

fn is_snowflake(value: i64) -> bool {
    value >= FIRST_SNOWFLAKE
}

pub(crate) fn snowflake_to_date_time(value: i64) -> Option<DateTime<Utc>> {
    if is_snowflake(value) {
        let timestamp_millis = (value >> 22) + 1288834974657;

        Utc.timestamp_millis_opt(timestamp_millis).single()
    } else {
        None
    }
}
