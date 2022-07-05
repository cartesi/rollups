/* Copyright 2022 Cartesi Pte. Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not
 * use this file except in compliance with the License. You may obtain a copy of
 * the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations under
 * the License.
 */

use actix_web::{
    middleware::Logger, web, web::Data, App, HttpResponse, HttpServer,
    Responder,
};
use async_mutex::Mutex;
use std::sync::Arc;
use tracing::info;

pub struct HealthStatus {
    pub state_server: Result<(), String>,
    pub server_manager: Result<(), String>,
    pub postgres: Result<(), String>,
    pub indexer_status: Result<(), String>,
}

struct HttpContext {
    health_status: Arc<Mutex<HealthStatus>>,
}

pub async fn start_http_service(
    host: &str,
    port: u16,
    health_status: Arc<Mutex<HealthStatus>>,
) -> std::io::Result<()> {
    info!(
        "Starting indexer health endpoint at address: {}:{}",
        host, port
    );
    HttpServer::new(move || {
        let http_context = HttpContext {
            health_status: health_status.clone(),
        };

        App::new()
            .app_data(Data::new(http_context))
            .wrap(Logger::default())
            .service(healthz)
    })
    .bind((host, port))?
    .run()
    .await
}

#[actix_web::get("/healthz")]
async fn healthz(http_context: web::Data<HttpContext>) -> impl Responder {
    let status = http_context.health_status.lock().await;
    if let Err(e) = &status.state_server {
        HttpResponse::BadRequest()
            .content_type("text/html; charset=utf-8")
            .body(format!(
                "Faulty indexer due to state server problems: {}",
                e
            ))
    } else if let Err(e) = &status.server_manager {
        HttpResponse::BadRequest()
            .content_type("text/html; charset=utf-8")
            .body(format!(
                "Faulty indexer due to server manager problems: {}",
                e
            ))
    } else if let Err(e) = &status.postgres {
        HttpResponse::BadRequest()
            .content_type("text/html; charset=utf-8")
            .body(format!("Faulty indexer due to database problems: {}", e))
    } else {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body("")
    }
}
