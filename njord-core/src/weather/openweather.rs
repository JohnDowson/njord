use super::{Coordinate, WeatherProvider, WeatherProviderError};
use crate::USER_AGENT;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{Date, Duration, TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct OWForecast {
    pub timezone_offset: i64,
    pub daily: Vec<OWDailyForecast>,
}
#[derive(Deserialize)]
pub struct OWDailyForecast {
    pub dt: i64,
    pub temp: OWDailyTemp,
}

#[derive(Deserialize)]
pub struct OWDailyTemp {
    pub day: f32,
}
pub struct OpenWeather {
    url: String,
    client: Client,
}
impl OpenWeather {
    const URL: &'static str = "https://api.openweathermap.org/data/2.5/onecall?lat={lat}&lon={lon}&exclude=current,minutely,hourly,alerts&units=metric&appid={API_key}";
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
        let OWForecast {
            timezone_offset,
            daily,
        } = serde_json::from_str(json)?;
        daily
            .iter()
            .find_map(|d| {
                if Utc.timestamp(d.dt + timezone_offset, 0).date() == date {
                    Some(d.temp.day)
                } else {
                    None
                }
            })
            .map_or_else(|| Err(WeatherProviderError::NotFound.into()), Ok)
    }

    pub(crate) fn extract_period_temps(
        json: &str,
        end: Date<Utc>,
    ) -> Result<Vec<(Date<Utc>, f32)>> {
        let OWForecast {
            timezone_offset,
            daily,
        } = serde_json::from_str(json)?;
        let forecast = daily
            .iter()
            .filter_map(|d| {
                let date = Utc.timestamp(d.dt + timezone_offset, 0).date();
                if date <= end {
                    Some((date, d.temp.day))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if forecast.is_empty() {
            Err(WeatherProviderError::NotFound.into())
        } else {
            Ok(forecast)
        }
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

    async fn weekly_forecast(&self, location: Coordinate) -> Result<Vec<(Date<Utc>, f32)>> {
        let end_of_week = Utc::today() + Duration::days(5);
        let resp = self
            .client
            .get(self.format_url(location.lon, location.lat))
            .send()
            .await?
            .text()
            .await?;
        Self::extract_period_temps(&resp, end_of_week)
    }
}
