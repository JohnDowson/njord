use std::collections::HashMap;

use crate::Providers;
use actix_web::{
    get,
    web::{self, Json, Query},
    Responder,
};
use chrono::{Date, NaiveDate, Utc};
use njord_core::geocoding;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(crate) enum WeeklyForecastResponce {
    Ok(HashMap<NaiveDate, f32>),
    Error { errors: Vec<String> },
}
#[derive(Deserialize, Serialize)]
pub(crate) enum DailyForecastResponce {
    Ok(f32),
    Error { errors: Vec<String> },
}
#[doc(hidden)]
#[derive(Deserialize)]
pub struct WeeklyQuery {
    location: String,
}
#[doc(hidden)]
#[derive(Deserialize)]
pub struct DailyQuery {
    location: String,
    date: Option<NaiveDate>,
}

#[get("/daily")]
/**
Usage:
`GET /daily?location={Location name}[&date={yyyy-mm-dd}]`
*/
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
    let date = if let Some(d) = query.date {
        Date::<Utc>::from_utc(d, Utc)
    } else {
        Utc::today()
    };
    let mut errors = Vec::with_capacity(providers.inner.len());
    let mut temps = Vec::with_capacity(providers.inner.len());
    for p in providers.inner.iter() {
        match p.daily_forecast(&providers.client, location, date).await {
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
/**
Usage:
`GET /weekly?location={Location name}`
*/
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
        match p.weekly_forecast(&providers.client, location).await {
            Err(e) => errors.push(format!("{} : {}", p.clone().id(), e.to_string())),
            Ok(t) => t.into_iter().for_each(|(d, t)| {
                temps.entry(d).or_insert_with(|| vec![t]);
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
