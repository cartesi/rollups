// Copyright 2021 Cartesi Pte. Ltd.
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

use actix_web::{
    error, error::Result as HttpResult, middleware::Logger, web::Data, web::Json, App,
    HttpResponse, HttpServer, Responder,
};

use crate::config::Config;
use crate::controller::{Controller, ControllerError};
use crate::model::{FinishStatus, Notice, Report, RollupException, Voucher};

use super::model::{
    HttpFinishRequest, HttpIndexResponse, HttpNotice, HttpReport, HttpRollupException,
    HttpRollupRequest, HttpVoucher,
};

pub async fn start_service(config: &Config, controller: Controller) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(controller.clone()))
            .wrap(Logger::default())
            .service(voucher)
            .service(notice)
            .service(report)
            .service(exception)
            .service(finish)
    })
    .bind((
        config.http_rollup_server_address.as_str(),
        config.http_rollup_server_port,
    ))?
    .run()
    .await
}

#[actix_web::post("/voucher")]
async fn voucher(
    voucher: Json<HttpVoucher>,
    controller: Data<Controller>,
) -> HttpResult<impl Responder> {
    let voucher: Voucher = voucher.into_inner().try_into()?;
    let rx = controller.insert_voucher(voucher).await;
    let index = rx.await.map_err(|_| {
        log::error!("sender dropped the channel");
        error::ErrorInternalServerError("failed to insert voucher")
    })??;
    let response = HttpIndexResponse {
        index: index as u64,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::post("/notice")]
async fn notice(
    notice: Json<HttpNotice>,
    controller: Data<Controller>,
) -> HttpResult<impl Responder> {
    let notice: Notice = notice.into_inner().try_into()?;
    let rx = controller.insert_notice(notice).await;
    let index = rx.await.map_err(|_| {
        log::error!("sender dropped the channel");
        error::ErrorInternalServerError("failed to insert notice")
    })??;
    let response = HttpIndexResponse {
        index: index as u64,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[actix_web::post("/report")]
async fn report(
    report: Json<HttpReport>,
    controller: Data<Controller>,
) -> HttpResult<impl Responder> {
    let report: Report = report.into_inner().try_into()?;
    let rx = controller.insert_report(report).await;
    rx.await.map_err(|_| {
        log::error!("sender dropped the channel");
        error::ErrorInternalServerError("failed to insert report")
    })??;
    Ok(HttpResponse::Ok())
}

#[actix_web::post("/exception")]
async fn exception(
    exception: Json<HttpRollupException>,
    controller: Data<Controller>,
) -> HttpResult<impl Responder> {
    let exception: RollupException = exception.into_inner().try_into()?;
    let rx = controller.notify_exception(exception).await;
    rx.await.map_err(|_| {
        log::error!("sender dropped the channel");
        error::ErrorInternalServerError("failed to notify exception")
    })??;
    Ok(HttpResponse::Ok())
}

#[actix_web::post("/finish")]
async fn finish(
    body: Json<HttpFinishRequest>,
    controller: Data<Controller>,
) -> HttpResult<impl Responder> {
    let status: FinishStatus = body.into_inner().try_into()?;
    let rx = controller.finish(status).await;
    let result = rx.await.map_err(|_| {
        log::error!("sender dropped the channel");
        error::ErrorInternalServerError("failed to finish")
    })?;
    let response = match result {
        Ok(rollup_request) => HttpResponse::Ok().json(HttpRollupRequest::from(rollup_request)),
        Err(e) => match e {
            ControllerError::FetchRequestTimeout => HttpResponse::Accepted().body(e.to_string()),
            ControllerError::InvalidRequest { .. } => {
                HttpResponse::BadRequest().body(e.to_string())
            }
        },
    };
    Ok(response)
}
