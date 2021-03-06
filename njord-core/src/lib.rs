#![allow(clippy::upper_case_acronyms)]
pub mod geocoding;
pub mod weather;
pub use reqwest::Client;
mod util;
pub static USER_AGENT: &str = "Njord/0.1 github.com/JohnDowson";
#[cfg(test)]
mod tests {
    use crate::*;
    use chrono::{TimeZone, Utc};
    use geocoding::*;
    use weather::OpenWeather;
    use weather::*;
    const KNOWN_LOCATION: Coordinate = Coordinate {
        lat: 55.750_446,
        lon: 37.617_493,
    };
    const EPSILON_F64: f64 = 0.00001;
    const EPSILON_F32: f32 = 0.001;
    #[tokio::test]
    async fn geocoding_moscow() {
        let moscow_location = match geocode("moscow").await {
            Ok(l) => l,
            Err(e) => {
                panic!("FOOBAR: {}", e);
            }
        };
        assert!({
            moscow_location.lat - KNOWN_LOCATION.lat < EPSILON_F64
                && moscow_location.lon - KNOWN_LOCATION.lon < EPSILON_F64
        })
    }
    #[test]
    fn parse_geocoding() {
        let json = r#"[{"lat": "55.7504461","lon": "37.6174943"}]"#;
        let location = get_location(json).expect("Couldn't parse location");
        assert!({
            location.lat - KNOWN_LOCATION.lat < EPSILON_F64
                && location.lon - KNOWN_LOCATION.lon < EPSILON_F64
        })
    }

    const OWM_JSON: &str = include_str!("../../onecall.json");

    #[test]
    fn parse_openweather_response() {
        let date = Utc.timestamp(1617094800, 0).date();
        assert!(
            OpenWeather::extract_daily_temp(OWM_JSON, date).expect("Failed to parse json") - 274.4
                < EPSILON_F32
        )
    }
    use std::collections::HashMap;
    #[test]
    fn parse_weekly_ow_response() {
        let date = Utc.timestamp(1617440400, 0).date();
        let other = vec![
            (Utc.timestamp(1617094800, 0).date(), 274.4f32),
            (Utc.timestamp(1617181200, 0).date(), 275.71),
            (Utc.timestamp(1617267600, 0).date(), 277.87),
            (Utc.timestamp(1617354000, 0).date(), 278.39),
            (Utc.timestamp(1617440400, 0).date(), 275.19),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
        assert!(OpenWeather::extract_period_temps(OWM_JSON, date)
            .expect("Failed to parse json")
            .into_iter()
            .all(|(d, ta)| {
                let tb = other.get(&d).unwrap();
                eprintln!("a: {:#?}\nb:{:#?}", ta, tb);
                ta - tb < EPSILON_F32
            }))
    }
    #[tokio::test]
    async fn get_data_from_openweather() {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build client");
        static API_KEY: &str = "32b610b48c69c28535625ba98d4a58bb";
        let provider = OpenWeather::new(API_KEY);
        let date = Utc::today();
        let daily_forecast = provider.daily_forecast(&client, KNOWN_LOCATION, date).await;
        assert!(daily_forecast.is_ok())
    }
    #[tokio::test]
    async fn get_weekly_data_from_openweather() {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build client");
        static API_KEY: &str = "32b610b48c69c28535625ba98d4a58bb";
        let provider = OpenWeather::new(API_KEY);
        let weekly_forecast = provider.weekly_forecast(&client, KNOWN_LOCATION).await;
        assert!(weekly_forecast.is_ok());
    }

    #[tokio::test]
    async fn get_daily_data_from_metno() {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to build client");
        let provider = MetNo;
        let daily_forecast = provider
            .daily_forecast(&client, KNOWN_LOCATION, Utc::today())
            .await;
        assert!(daily_forecast.is_ok())
    }
}
