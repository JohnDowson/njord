use reqwest::Response;
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
    let client = reqwest::Client::default();
    client
        .get(format!("{}{}", NOMINATIM_URL, location_name))
        .header("User-Agent", "Njord/0.1.0 github.com/JohnDowson")
        .send()
        .await
        .map(get_location)?
        .await
}
async fn get_location(resp: Response) -> GeocodingResult {
    let resp_text: String = resp.text().await?;
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
