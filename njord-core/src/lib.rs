pub mod geocoding;
pub mod weather;
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn geocoding_moscow() {
        use crate::geocoding::*;
        let moscow_location = match geocode("moscow").await {
            Ok(l) => l,
            Err(e) => {
                panic!(format!("FOOBAR: {}", e));
            }
        };
        let known_location = Coordinate {
            lat: 55.750_446,
            lon: 37.617_493,
        };
        const EPSILON: f64 = 0.00001;
        assert!({
            moscow_location.lat - known_location.lat < EPSILON
                && moscow_location.lon - known_location.lon < EPSILON
        })
    }
}
