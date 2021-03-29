use chrono::NaiveDate;

pub trait WeatherProvider {
    fn daily_forecast(location: (f32, f32), date: NaiveDate);
}
