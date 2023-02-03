// Copyright 2023 Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use crate::schema::{Context, Query, RollupsGraphQLScalarValue, Schema};
use actix_cors::Cors;
use actix_web::{
    middleware::Logger, web, web::Data, App, HttpResponse, HttpServer,
    Responder,
};
use juniper::http::playground::playground_source;
use juniper::http::GraphQLRequest;
use juniper::{EmptyMutation, EmptySubscription};
use std::sync::Arc;

struct HttpContext {
    schema: Arc<Schema>,
    context: Context,
}

pub async fn start_service(
    host: &str,
    port: u16,
    context: Context,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        let schema = std::sync::Arc::new(Schema::new_with_scalar_value(
            Query,
            EmptyMutation::new(),
            EmptySubscription::new(),
        ));

        let http_context = HttpContext {
            schema: schema.clone(),
            context: context.clone(),
        };

        let cors = Cors::permissive();

        App::new()
            .app_data(Data::new(http_context))
            .wrap(Logger::default())
            .wrap(cors)
            .service(graphql)
            .service(juniper_playground)
            .service(health)
            .service(healthz)
    })
    .bind((host, port))?
    .run()
    .await
}

#[actix_web::get("/")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body("")
}

#[actix_web::get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body("")
}

#[actix_web::get("/graphql")]
async fn juniper_playground() -> impl Responder {
    let html = playground_source("", None);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[actix_web::post("/graphql")]
async fn graphql(
    query: web::Json<GraphQLRequest<RollupsGraphQLScalarValue>>,
    http_context: web::Data<HttpContext>,
) -> HttpResponse {
    // Execute resolvers in blocking thread as there are lot of blocking diesel db operations
    let query = Arc::new(query);
    let return_value: HttpResponse = match tokio::task::spawn_blocking(
        move || {
            let res =
                query.execute_sync(&http_context.schema, &http_context.context);
            serde_json::to_string(&res)
        },
    )
    .await
    {
        Ok(value) => match value {
            Ok(value) => HttpResponse::Ok()
                .content_type("application/json")
                .body(value),
            Err(err) => {
                let error_message = format!(
                            "unable to execute query, internal server error, details: {}", err
                        );
                return HttpResponse::BadRequest().body(error_message);
            }
        },
        Err(err) => {
            let error_message = format!(
                "unable to execute query, internal server error, details: {}",
                err
            );
            return HttpResponse::BadRequest().body(error_message);
        }
    };
    return_value
}
