use std::{collections::HashMap, sync::Arc};

use crate::geocoding::Coordinate;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{Date, Utc};

#[async_trait]
pub trait WeatherProvider
where
    Self: Send,
{
    fn id(self: Arc<Self>) -> &'static str;
    async fn daily_forecast(
        &self,
        client: &Client,
        location: Coordinate,
        date: Date<Utc>,
    ) -> Result<f32>;
    async fn weekly_forecast(
        &self,
        client: &Client,
        location: Coordinate,
    ) -> Result<HashMap<Date<Utc>, f32>>;
}

#[derive(Debug, thiserror::Error)]
pub enum WeatherProviderError {
    #[error("InvalidFormat")]
    InvalidFormat,
    #[error("No weather data for given date {0}")]
    NotFound(Date<Utc>),
}
mod openweather;
pub use openweather::OpenWeather;
mod met_no;
pub use met_no::MetNo;
use reqwest::Client;
