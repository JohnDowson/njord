mod api;
use std::sync::Arc;

use actix_web::{middleware::Logger, App, HttpServer};
use njord_core::weather::{MetNo, OpenWeather, WeatherProvider};

const OW_API_KEY: &str = "32b610b48c69c28535625ba98d4a58bb";
#[derive(Clone)]
pub struct Providers {
    inner: Vec<Arc<dyn WeatherProvider + Sync>>,
    client: reqwest::Client,
}
impl<'a, 'b> Providers {
    fn new() -> Self {
        Self {
            inner: vec![],
            client: reqwest::Client::builder()
                .user_agent(njord_core::USER_AGENT)
                .build()
                .expect("Failed to build client"),
        }
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
            .data(
                Providers::new()
                    .register(OpenWeather::new(OW_API_KEY))
                    .register(MetNo),
            )
            .wrap(Logger::default())
            .service(api::daily)
            .service(api::weekly)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    #[actix_rt::test]
    async fn get_daily() {
        let mut app = test::init_service(
            App::new()
                .data(
                    Providers::new()
                        .register(MetNo)
                        .register(OpenWeather::new(OW_API_KEY)),
                )
                .service(api::daily),
        )
        .await;
        let req = test::TestRequest::with_uri("/daily?location=Moscow").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success())
    }
    #[actix_rt::test]
    async fn get_weekly() {
        let mut app = test::init_service(
            App::new()
                .data(
                    Providers::new()
                        .register(MetNo)
                        .register(OpenWeather::new(OW_API_KEY)),
                )
                .service(api::daily),
        )
        .await;
        let req = test::TestRequest::with_uri("/weekly?location=Moscow").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success())
    }
    #[actix_rt::test]
    #[should_panic]
    async fn incorrect_params_weekly() {
        let mut app = test::init_service(
            App::new()
                .data(
                    Providers::new()
                        .register(MetNo)
                        .register(OpenWeather::new(OW_API_KEY)),
                )
                .service(api::daily),
        )
        .await;
        let req = test::TestRequest::with_uri("/weekly").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error())
    }
    #[actix_rt::test]
    async fn incorrect_params_daily() {
        let mut app = test::init_service(
            App::new()
                .data(
                    Providers::new()
                        .register(MetNo)
                        .register(OpenWeather::new(OW_API_KEY)),
                )
                .service(api::daily),
        )
        .await;
        let req = test::TestRequest::with_uri("/daily").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_client_error())
    }
}
