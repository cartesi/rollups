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

/// Http service serving graphql queries
use actix_web::{
    error::Result as HttpResult, middleware::Logger, web, web::Data, App,
    HttpResponse, HttpServer, Responder,
};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use juniper::http::playground::playground_source;
use juniper::http::GraphQLRequest;
use juniper::{EmptyMutation, EmptySubscription};
use std::sync::Arc;

struct Context {
    schema: Arc<crate::graphql::Schema>,
    db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

pub async fn start_service(
    host: &str,
    port: u16,
    pool: Pool<ConnectionManager<PgConnection>>,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        let schema = std::sync::Arc::new(crate::graphql::Schema::new(
            crate::graphql::Query,
            EmptyMutation::new(),
            EmptySubscription::new(),
        ));

        let context = Context {
            schema: schema.clone(),
            db_pool: Arc::new(pool.clone()),
        };

        App::new()
            .app_data(Data::new(context))
            .wrap(Logger::default())
            .service(graphql)
            .service(juniper_playground)
    })
    .bind((host, port))?
    .run()
    .await
}

#[actix_web::get("/graphql")]
fn juniper_playground() -> HttpResponse {
    let html = playground_source("", None);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[actix_web::post("/graphql")]
async fn graphql(
    query: web::Json<GraphQLRequest>,
    context: web::Data<Context>,
) -> HttpResult<impl Responder> {
    let ctx = crate::graphql::Context {
        db_pool: context.db_pool.clone(),
    };
    let res = query.execute(&context.schema, &ctx).await;
    let value = serde_json::to_string(&res)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(value))
}
