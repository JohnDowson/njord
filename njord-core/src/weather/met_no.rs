use super::{WeatherProvider, WeatherProviderError};
use crate::geocoding::Coordinate;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{Date, DateTime, Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

#[derive(Deserialize)]
struct MetNoResponceSchema {
    properties: MetNoProperties,
}
#[derive(Deserialize)]
struct MetNoProperties {
    timeseries: Vec<MetNoTimeseries>,
}
#[derive(Deserialize)]
struct MetNoTimeseries {
    #[serde(with = "crate::util::zulu_time_format")]
    time: DateTime<Utc>,
    data: MetNoData,
}
#[derive(Deserialize)]
struct MetNoData {
    instant: MetNoInstant,
}
#[derive(Deserialize)]
struct MetNoInstant {
    details: MetNoDetails,
}
#[derive(Deserialize)]
struct MetNoDetails {
    air_temperature: f32,
}

pub struct MetNo;
impl MetNo {
    const URL: &'static str =
        "https://api.met.no/weatherapi/locationforecast/2.0/complete?lat={lat}&lon={lon}";
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    fn extract_daily_temp(json: &str, date: Date<Utc>) -> Result<f32> {
        let r: MetNoResponceSchema = serde_json::from_str(json)?;
        r.properties
            .timeseries
            .iter()
            .find_map(|d| {
                if d.time.date() == date {
                    Some(d.data.instant.details.air_temperature)
                } else {
                    None
                }
            })
            .ok_or_else(|| WeatherProviderError::NotFound(date).into())
    }

    fn extract_period_temps(json: &str, end: Date<Utc>) -> Result<HashMap<Date<Utc>, f32>> {
        let r: MetNoResponceSchema = serde_json::from_str(json)?;
        let forecast: HashMap<_, _> = r
            .properties
            .timeseries
            .iter()
            .filter_map(|d| {
                let date = d.time.date();
                if date <= end {
                    Some((date, d.data.instant.details.air_temperature))
                } else {
                    None
                }
            })
            .collect();
        if forecast.is_empty() {
            Err(WeatherProviderError::NotFound(end).into())
        } else {
            Ok(forecast)
        }
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
        let resp = client
            .get(&Self::format_url(location.lat, location.lon))
            .send()
            .await?
            .text()
            .await?;
        Self::extract_daily_temp(&resp, date)
    }

    async fn weekly_forecast(
        &self,
        client: &Client,
        location: Coordinate,
    ) -> anyhow::Result<HashMap<Date<Utc>, f32>> {
        let resp = client
            .get(&Self::format_url(location.lat, location.lon))
            .send()
            .await?
            .text()
            .await?;
        let date = Utc::today() + Duration::days(5);
        Self::extract_period_temps(&resp, date)
    }

    fn id(self: Arc<Self>) -> &'static str {
        "met.no"
    }
}
