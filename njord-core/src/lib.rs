pub mod geocoding;
pub mod weather;
static USER_AGENT: &str = "Njord/0.1 github.com/JohnDowson";
#[cfg(test)]
mod tests {
    use crate::*;
    use chrono::{TimeZone, Utc};
    use geocoding::*;
    use weather::OpenWeather;
    const KNOWN_LOCATION: Coordinate = Coordinate {
        lat: 55.750_446,
        lon: 37.617_493,
    };
    const EPSILON: f64 = 0.00001;
    #[tokio::test]
    async fn geocoding_moscow() {
        let moscow_location = match geocode("moscow").await {
            Ok(l) => l,
            Err(e) => {
                panic!(format!("FOOBAR: {}", e));
            }
        };
        assert!({
            moscow_location.lat - KNOWN_LOCATION.lat < EPSILON
                && moscow_location.lon - KNOWN_LOCATION.lon < EPSILON
        })
    }
    #[test]
    fn parse_geocoding() {
        let json = r#"[{"lat": "55.7504461","lon": "37.6174943"}]"#;
        let location = get_location(json).expect("Couldn't parse location");
        assert!({
            location.lat - KNOWN_LOCATION.lat < EPSILON
                && location.lon - KNOWN_LOCATION.lon < EPSILON
        })
    }

    #[test]
    fn parse_openweather_response() {
        let json = r#"{
            "daily": [
                {
                    "dt": 1617094800,
                    "temp": {
                        "day": 274.4,
                        "min": 269.32,
                        "max": 275.84,
                        "night": 274.31,
                        "eve": 275.3,
                        "morn": 269.32
                    }
                },
                {
                    "dt": 1617181200,
                    "temp": {
                        "day": 275.71,
                        "min": 274.35,
                        "max": 277.5,
                        "night": 277.5,
                        "eve": 276.2,
                        "morn": 275.86
                    }
                }
            ]
        }"#;
        let date = Utc.timestamp(1617094800, 0).date();
        assert!(
            OpenWeather::extract_daily_temp(json, date).expect("Failed to parse json") - 274.4
                < EPSILON
        )
    }

    #[tokio::test]
    async fn get_data_from_openweather() {
        use weather::WeatherProvider;
        static API_KEY: &str = "32b610b48c69c28535625ba98d4a58bb";
        let provider = OpenWeather::new(API_KEY);
        let date = Utc::today();
        let daily_forecast = provider.daily_forecast(KNOWN_LOCATION, date).await;
        assert!(daily_forecast.is_ok())
    }
}
