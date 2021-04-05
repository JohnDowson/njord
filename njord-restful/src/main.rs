mod api;
use std::sync::Arc;

use actix_web::{middleware::Logger, App, HttpServer};
use njord_core::weather::{MetNo, OpenWeather, WeatherProvider};
#[doc(hidden)]
const OW_API_KEY: &str = "32b610b48c69c28535625ba98d4a58bb";
#[derive(Clone)]
/// Provides access to weather providers
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
    /**
    Registers a new weather provider. Calls can be chained together. i.e:i
    ```
    Providers::new()
        .register(MetNo)
        .register(OpenWeather::new("apike"))
    ```
    */

    fn register<T: 'static + WeatherProvider + Sync>(mut self, provider: T) -> Self {
        self.inner.push(Arc::new(provider));
        self
    }
}

#[actix_web::main]
#[doc(hidden)]
async fn main() -> std::io::Result<()> {
    use std::env;
    if let Err(env::VarError::NotPresent) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "actix_web=info")
    }
    env_logger::init();

    let addr: std::net::IpAddr = env::var("NJORD_ADDR")
        .unwrap_or_else(|_| "127.0.0.1".to_owned())
        .parse()
        .expect("Invalid value for IP address");
    let port = env::var("NJORD_PORT")
        .unwrap_or_else(|_| "8080".to_owned())
        .parse()
        .expect("Invalid value for port");

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
    .bind((addr, port))?
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
                .service(api::weekly),
        )
        .await;
        let req = test::TestRequest::with_uri("/weekly?location=Moscow").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success())
    }
    #[actix_rt::test]
    async fn incorrect_params_weekly() {
        let mut app = test::init_service(
            App::new()
                .data(
                    Providers::new()
                        .register(MetNo)
                        .register(OpenWeather::new(OW_API_KEY)),
                )
                .service(api::weekly),
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
