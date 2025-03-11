use actix_web::{get, post, web, Error as AWError, HttpResponse};
use log::info;

use crate::{
    model::{common::RestResponse, settings::SettingsModel},
    SharedSettings,
};

#[utoipa::path(
    summary = "Query settings",
    responses(
        (status = 200, description = "Query settings successfully", body=RestResponse<SettingsModel>),
    ),
)]
#[get("/settings")]
pub async fn query_settings(settings: web::Data<SharedSettings>) -> Result<HttpResponse, AWError> {
    let settings = settings.lock().await;
    let settings_model: SettingsModel = settings.to_model();
    info!(
        "Query settings successfully, settings: {:?}",
        settings_model
    );
    Ok(HttpResponse::Ok().json(RestResponse::succeed_with_data(settings_model)))
}

#[utoipa::path(
    summary = "Update settings",
    request_body(content = SettingsModel),
    responses(
        (status = 200, description = "Update settings successfully"),
    ),
)]
#[post("/settings")]
pub async fn update_settings(
    requst_json: web::Json<SettingsModel>,
    settings: web::Data<SharedSettings>,
) -> Result<HttpResponse, AWError> {
    let params = requst_json.into_inner();
    let mut settings = settings.lock().await;
    settings.update(&params);
    // save new settings to file
    settings.save()?;
    info!("Update settings successfully");
    Ok(HttpResponse::Ok().finish())
}
