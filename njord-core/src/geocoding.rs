use crate::USER_AGENT;
use reqwest::Client;
use serde_json::Value;
use thiserror::Error;
pub type GeocodingResult = anyhow::Result<Coordinate>;
#[derive(Debug, Error)]
pub enum GeocodingError {
    #[error("{0:?}")]
    RequestError(reqwest::Error),
    #[error("Location not found")]
    LocationNotFound,
    #[error("Unexpected geocoding responce")]
    UnexpectedResponceFormat,
}
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
}
static NOMINATIM_URL: &str = "https://nominatim.openstreetmap.org/search.php?format=json&city=";
pub async fn geocode(location_name: &str) -> GeocodingResult {
    let client = Client::builder().user_agent(USER_AGENT).build()?;
    client
        .get(format!("{}{}", NOMINATIM_URL, location_name))
        .send()
        .await
        .map(|r| async { get_location(&r.text().await?) })?
        .await
}
pub(crate) fn get_location(resp_text: &str) -> GeocodingResult {
    let json_value: Value = serde_json::from_str(&resp_text)?;
    if let Value::Array(v) = json_value {
        let location = match v.first() {
            Some(Value::Object(location)) => Ok(location),
            None => Err(GeocodingError::LocationNotFound),
            _ => Err(GeocodingError::UnexpectedResponceFormat),
        }?;

        let (lat, lon) = match (location.get("lat"), location.get("lon")) {
            (Some(Value::String(lat)), Some(Value::String(lon))) => {
                Ok((lat.parse()?, lon.parse()?))
            }
            _ => Err(GeocodingError::UnexpectedResponceFormat),
        }?;
        Ok(Coordinate { lat, lon })
    } else {
        Err(GeocodingError::UnexpectedResponceFormat.into())
    }
}
