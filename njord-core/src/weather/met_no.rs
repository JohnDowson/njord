use std::{collections::HashMap, sync::Arc};

use super::WeatherProvider;
use crate::{geocoding::Coordinate, USER_AGENT};
use async_trait::async_trait;
use chrono::{Date, Utc};
use reqwest::Client;
pub struct MetNo {}
impl MetNo {
    const URL: &'static str =
        "https://api.met.no/weatherapi/locationforecast/2.0/complete?lat={lat}&lon={lon}";
    fn new(client: Client) -> Self {
        Self {}
    }

    fn format_url(lat: f64, lon: f64) -> String {
        Self::URL
            .replace("{lat}", &lat.to_string())
            .replace("{lon}", &lon.to_string())
    }
}

#[async_trait]
impl WeatherProvider for MetNo {
    async fn daily_forecast(
        &self,
        client: &Client,
        location: Coordinate,
        date: Date<Utc>,
    ) -> anyhow::Result<f32> {
        client
            .get(&Self::format_url(location.lat, location.lon))
            .send()
            .await?
            .text()
            .await?;
        todo!()
    }

    async fn weekly_forecast(
        &self,
        client: &Client,
        location: Coordinate,
    ) -> anyhow::Result<HashMap<Date<Utc>, f32>> {
        todo!()
    }

    fn id(self: Arc<Self>) -> &'static str {
        "met.no"
    }
}
