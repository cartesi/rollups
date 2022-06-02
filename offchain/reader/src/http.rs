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
    middleware::Logger, web, web::Data, App, HttpResponse, HttpServer,
    Responder,
};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use juniper::http::playground::playground_source;
use juniper::http::GraphQLRequest;
use juniper::{EmptyMutation, EmptySubscription};
use rollups_data::graphql::types::RollupsGraphQLScalarValue;
use std::sync::Arc;

struct HttpContext {
    schema: Arc<rollups_data::graphql::types::Schema>,
    db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

pub async fn start_service(
    host: &str,
    port: u16,
    pool: Pool<ConnectionManager<PgConnection>>,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        let schema = std::sync::Arc::new(
            rollups_data::graphql::resolvers::Schema::new_with_scalar_value(
                rollups_data::graphql::resolvers::Query,
                EmptyMutation::<rollups_data::graphql::resolvers::Context>::new(
                ),
                EmptySubscription::new(),
            ),
        );

        let http_context = HttpContext {
            schema: schema.clone(),
            db_pool: Arc::new(pool.clone()),
        };

        App::new()
            .app_data(Data::new(http_context))
            .wrap(Logger::default())
            .service(graphql)
            .service(juniper_playground)
    })
    .bind((host, port))?
    .run()
    .await
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
    let ctx = rollups_data::graphql::resolvers::Context {
        db_pool: http_context.db_pool.clone(),
    };
    // Execute resolvers in blocking thread as there are lot of blocking diesel db operations
    let query = Arc::new(query);
    let ctx = Arc::new(ctx);
    let return_value: HttpResponse = match tokio::task::spawn_blocking(
        move || {
            let res = query.execute_sync(&http_context.schema, &ctx);
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
                            "unable to execute query, internal server error, details: {}", err.to_string()
                        );
                return HttpResponse::BadRequest().body(error_message);
            }
        },
        Err(err) => {
            let error_message = format!(
                "unable to execute query, internal server error, details: {}",
                err.to_string()
            );
            return HttpResponse::BadRequest().body(error_message);
        }
    };
    return_value
}
