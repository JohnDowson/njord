use crate::USER_AGENT;
use reqwest::Client;
use serde::{de, Deserialize, Deserializer};
use std::{fmt::Display, str::FromStr};
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
#[derive(Deserialize)]
pub struct Coordinate {
    #[serde(deserialize_with = "from_str")]
    pub lat: f64,
    #[serde(deserialize_with = "from_str")]
    pub lon: f64,
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
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
    serde_json::from_str::<Vec<Coordinate>>(&resp_text)?
        .into_iter()
        .next()
        .map_or_else(|| Err(GeocodingError::LocationNotFound.into()), Ok)
}
