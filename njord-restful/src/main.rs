mod api;
use std::sync::Arc;

use actix_web::{middleware::Logger, App, HttpServer};
use njord_core::weather::{OpenWeather, WeatherProvider};

#[derive(Clone)]
pub struct Providers {
    inner: Vec<Arc<dyn WeatherProvider + Sync>>,
}
impl<'a, 'b> Providers {
    fn new() -> Self {
        Self { inner: vec![] }
    }
    fn register<T: 'static + WeatherProvider + Sync>(mut self, provider: T) -> Self {
        self.inner.push(Arc::new(provider));
        self
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    HttpServer::new(|| {
        App::new()
            .data(Providers::new().register(OpenWeather::new("32b610b48c69c28535625ba98d4a58bb")))
            .wrap(Logger::default())
            .service(api::daily)
            .service(api::weekly)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
