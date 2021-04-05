use std::collections::HashMap;

use crate::Providers;
use actix_web::{
    get,
    web::{self, Json, Query},
    Responder,
};
use chrono::{Date, DateTime, NaiveDate, Utc};
use njord_core::geocoding::{self, Coordinate};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
enum WeeklyForecastResponce {
    Ok(HashMap<NaiveDate, f32>),
    Error { errors: Vec<String> },
}
#[derive(Deserialize, Serialize)]
enum DailyForecastResponce {
    Ok(f32),
    Error { errors: Vec<String> },
}
#[derive(Deserialize)]
pub struct WeeklyQuery {
    location: String,
}
#[derive(Deserialize)]
pub struct DailyQuery {
    location: String,
    date: NaiveDate,
}

#[get("/daily")]
pub async fn daily(
    Query(query): Query<DailyQuery>,
    providers: web::Data<Providers>,
) -> impl Responder {
    let location = match geocoding::geocode(&query.location).await {
        Ok(l) => l,
        Err(e) => {
            return Json(DailyForecastResponce::Error {
                errors: vec![format!("Geocoding: {}", e.to_string())],
            })
        }
    };
    let date = Date::<Utc>::from_utc(query.date, Utc);
    let mut errors = Vec::with_capacity(providers.inner.len());
    let mut temps = Vec::with_capacity(providers.inner.len());
    for p in providers.inner.iter() {
        match p.daily_forecast(location, date).await {
            Err(e) => errors.push(format!("{} : {}", p.clone().id(), e.to_string())),
            Ok(t) => temps.push(t),
        };
    }
    if temps.is_empty() {
        Json(DailyForecastResponce::Error { errors })
    } else {
        Json(DailyForecastResponce::Ok(
            temps.iter().fold(0.0, std::ops::Add::add) / (temps.len() as f32),
        ))
    }
}

#[get("/weekly")]
pub async fn weekly(
    Query(query): Query<WeeklyQuery>,
    providers: web::Data<Providers>,
) -> impl Responder {
    let location = match geocoding::geocode(&query.location).await {
        Ok(l) => l,
        Err(e) => {
            return Json(WeeklyForecastResponce::Error {
                errors: vec![format!("Geocoding: {}", e.to_string())],
            })
        }
    };
    let mut errors = Vec::with_capacity(providers.inner.len());
    let mut temps = HashMap::with_capacity(providers.inner.len());
    for p in providers.inner.iter() {
        match p.weekly_forecast(location).await {
            Err(e) => errors.push(format!("{} : {}", p.clone().id(), e.to_string())),
            Ok(t) => t.into_iter().for_each(|(d, t)| {
                temps.entry(d).or_insert(vec![t]);
            }),
        };
    }
    if temps.is_empty() {
        Json(WeeklyForecastResponce::Error { errors })
    } else {
        Json(WeeklyForecastResponce::Ok(
            temps
                .into_iter()
                .map(|(d, t)| {
                    let len = t.len();
                    (
                        d.naive_utc(),
                        t.into_iter().fold(0.0, std::ops::Add::add) / (len as f32),
                    )
                })
                .collect::<HashMap<_, _>>(),
        ))
    }
}
