use crate::geocoding::Coordinate;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{serde::ts_seconds, Date, DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use super::USER_AGENT;
#[async_trait]
pub trait WeatherProvider {
    async fn daily_forecast(&self, location: Coordinate, date: Date<Utc>) -> Result<f32>;
    async fn weekly_forecast(&self, location: Coordinate) -> Result<f32>;
}

#[derive(Debug, thiserror::Error)]
pub enum WeatherProviderError {
    #[error("InvalidFormat")]
    InvalidFormat,
    #[error("No weather data for given date")]
    NotFound,
}

#[derive(Deserialize)]
struct OWForecast {
    daily: Vec<OWDailyForecast>,
}
#[derive(Deserialize)]
struct OWDailyForecast {
    #[serde(with = "ts_seconds")]
    dt: DateTime<Utc>,
    temp: OWDailyTemp,
}

#[derive(Deserialize)]
struct OWDailyTemp {
    day: f32,
}
pub struct OpenWeather {
    url: String,
    client: Client,
}
impl OpenWeather {
    const URL: &'static str = "https://api.openweathermap.org/data/2.5/onecall?lat={lat}&lon={lon}&exclude=current,minutely,hourly,alerts&appid={API_key}";
    pub fn new(api_key: &str) -> Self {
        Self {
            url: Self::URL.replace("{API_key}", api_key),
            // Unwrap: non-recoverable
            client: Client::builder().user_agent(USER_AGENT).build().unwrap(),
        }
    }
    fn format_url(&self, lon: f64, lat: f64) -> String {
        self.url
            .replace("{lon}", &lon.to_string())
            .replace("{lat}", &lat.to_string())
    }
    pub(crate) fn extract_daily_temp(json: &str, date: Date<Utc>) -> Result<f32> {
        serde_json::from_str::<OWForecast>(json)?
            .daily
            .iter()
            .find_map(|d| {
                if d.dt.date() == date {
                    Some(d.temp.day)
                } else {
                    None
                }
            })
            .map_or_else(|| Err(WeatherProviderError::NotFound.into()), Ok)
    }
}
#[async_trait]
impl WeatherProvider for OpenWeather {
    async fn daily_forecast(&self, location: Coordinate, date: Date<Utc>) -> Result<f32> {
        let resp = self
            .client
            .get(self.format_url(location.lon, location.lat))
            .send()
            .await?
            .text()
            .await?;
        Self::extract_daily_temp(&resp, date)
    }

    async fn weekly_forecast(&self, location: Coordinate) -> Result<f32> {
        unimplemented!()
    }
}
