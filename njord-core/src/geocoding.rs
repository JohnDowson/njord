use crate::USER_AGENT;
use reqwest::Client;
use serde::Deserialize;
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
    serde_json::from_str::<Vec<Coordinate>>(&resp_text)?
        .into_iter()
        .next()
        .map_or_else(|| Err(GeocodingError::LocationNotFound.into()), Ok)
}
