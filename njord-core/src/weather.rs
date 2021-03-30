use crate::geocoding::Coordinate;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{Date, TimeZone, Utc};
use reqwest::Client;
use serde_json::Value;

use super::USER_AGENT;
#[async_trait]
pub trait WeatherProvider {
    async fn daily_forecast(&self, location: Coordinate, date: Date<Utc>) -> Result<f64>;
    async fn weekly_forecast(&self, location: Coordinate) -> Result<f64>;
}

#[derive(Debug, thiserror::Error)]
pub enum WeatherProviderError {
    #[error("InvalidFormat")]
    InvalidFormat,
    #[error("No weather data for given date")]
    NotFound,
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
    pub(crate) fn extract_daily_temp(json: &str, date: Date<Utc>) -> Result<f64> {
        macro_rules! err {
            () => {
                return Some(Err(crate::weather::WeatherProviderError::InvalidFormat))
            };
        }
        let value: Value = serde_json::from_str(json)?;
        let daily = if let Value::Object(o) = value {
            if let Some(Value::Array(ds)) = o.get("daily") {
                ds.iter()
                    .find_map(|d| -> Option<Result<f64, WeatherProviderError>> {
                        if let Value::Object(d) = d {
                            let dt = if let Some(Value::Number(dt)) = d.get("dt") {
                                if let Some(dt) = dt.as_i64() {
                                    dt
                                } else {
                                    err!();
                                }
                            } else {
                                err!();
                            };
                            if Utc.timestamp(dt, 0).date() != date {
                                None
                            } else if let Some(Value::Object(ts)) = d.get("temp") {
                                if let Some(Value::Number(t)) = ts.get("day") {
                                    if let Some(temp) = t.as_f64() {
                                        Some(Ok(temp))
                                    } else {
                                        err!();
                                    }
                                } else {
                                    err!();
                                }
                            } else {
                                err!();
                            }
                        } else {
                            err!();
                        }
                    })
            } else {
                None
            }
        } else {
            None
        };
        if let Some(daily) = daily {
            daily.map_err(|e| e.into())
        } else {
            Err(WeatherProviderError::NotFound.into())
        }
    }
}
#[async_trait]
impl WeatherProvider for OpenWeather {
    async fn daily_forecast(&self, location: Coordinate, date: Date<Utc>) -> Result<f64> {
        let resp = self
            .client
            .get(self.format_url(location.lon, location.lat))
            .send()
            .await?
            .text()
            .await?;
        Self::extract_daily_temp(&resp, date)
    }

    async fn weekly_forecast(&self, location: Coordinate) -> Result<f64> {
        unimplemented!()
    }
}
