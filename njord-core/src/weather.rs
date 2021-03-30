use crate::geocoding::Coordinate;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{Date, Utc};

#[async_trait]
pub trait WeatherProvider {
    async fn daily_forecast(&self, location: Coordinate, date: Date<Utc>) -> Result<f32>;
    async fn weekly_forecast(&self, location: Coordinate) -> Result<Vec<(Date<Utc>, f32)>>;
}

#[derive(Debug, thiserror::Error)]
pub enum WeatherProviderError {
    #[error("InvalidFormat")]
    InvalidFormat,
    #[error("No weather data for given date")]
    NotFound,
}
mod openweather;
pub use openweather::OpenWeather;
mod met_no;
pub use met_no::MetNo;
