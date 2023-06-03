use actix_web::{web, web::Data, App, HttpResponse, HttpServer, Responder};
use axum::{routing::get, Router};
use prometheus_client::{encoding::text::encode, registry::Registry};
use std::{
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex, MutexGuard},
};

// For Prometheus to poll from.
pub struct MetricsServer {
    pub host: IpAddr,
    pub port: u16,
}

pub fn get_metrics(registry: MutexGuard<'_, Registry>) -> String {
    let mut buffer = String::new();
    encode(&mut buffer, &registry).unwrap();
    buffer
}

impl MetricsServer {
    pub async fn run_with_axum(
        self,
        registry: Registry,
    ) -> Result<(), hyper::Error> {
        tracing::info!(
            "Starting metrics endpoint at http://{}:{}/metrics",
            self.host,
            self.port
        );

        let addr = SocketAddr::new(self.host, self.port);
        let registry = Arc::new(Mutex::new(registry));
        let router = Router::new().route(
            "/metrics",
            get(|| async move {
                let registry = registry.lock().unwrap();
                get_metrics(registry)
            }),
        );

        axum::Server::bind(&addr)
            .serve(router.into_make_service())
            .await
    }

    pub async fn run_with_actix(
        self,
        registry: Registry,
    ) -> std::io::Result<()> {
        tracing::info!(
            "Starting metrics endpoint at http://{}:{}/metrics",
            self.host,
            self.port
        );

        let registry = Data::new(Mutex::new(registry));
        HttpServer::new(move || {
            App::new()
                .app_data(registry.clone())
                .route("/metrics", web::get().to(actix_responder))
        })
        .bind((self.host.to_string(), self.port))?
        .run()
        .await
    }
}

async fn actix_responder(registry: Data<Mutex<Registry>>) -> impl Responder {
    let registry = registry.lock().unwrap();
    HttpResponse::Ok().body(get_metrics(registry))
}
