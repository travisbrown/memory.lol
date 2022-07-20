use super::error::Error;
use rocket::serde::json::Json;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
struct SnowflakeInfo {
    #[serde(rename = "epoch-second")]
    epoch_second: i64,
    #[serde(rename = "utc-rfc2822")]
    utc_rfc2822: String,
}

#[get("/tw/util/snowflake/<id>")]
pub fn info(id: i64) -> Result<Json<Value>, Error> {
    let timestamp = crate::util::snowflake_to_date_time(id).ok_or(Error::InvalidSnowflake(id))?;

    Ok(Json(serde_json::to_value(SnowflakeInfo {
        epoch_second: timestamp.timestamp(),
        utc_rfc2822: timestamp.to_rfc2822(),
    })?))
}
