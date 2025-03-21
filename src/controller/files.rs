use actix_web::{delete, get, web, Error as AWError, HttpResponse};
use log::info;

use crate::{
    database::{file_info::FileInfoList, sqlite::PoolDatabaseManager},
    model::{
        common::{ErrorCode, RestResponse},
        files::DeleteFileRequest, settings::ListSettings,
    }, SharedSettings,
};

#[utoipa::path(
    summary = "Query list file settings",
    responses(
        (status = 200, description = "List settings", body = RestResponse<ListSettings>),
    ),
)]
#[get("/list/settings")]
pub async fn query_list_settings( settings: web::Data<SharedSettings>,) -> Result<HttpResponse, AWError> {
    let response = settings.lock().await.list.clone();
    Ok(HttpResponse::Ok().json(RestResponse::succeed_with_data(response)))
}

#[utoipa::path(
    summary = "List files",
    params(ListSettings),
    responses(
        (status = 200, description = "The list of file info with md5 count", body=FileInfoList)
    ),
)]
#[get("/list")]
pub async fn list_files(
    query_list: web::Query<ListSettings>,
    db: web::Data<PoolDatabaseManager>,
    settings: web::Data<SharedSettings>,
) -> Result<HttpResponse, AWError> {
    let conn = db.get_ref();
    let result = conn.0.list_files(&query_list);
    match result {
        Ok(file_info_list) => {
            {
                let mut settings = settings.lock().await;
                settings.list = query_list.into_inner().clone();
                settings.save()?;
            }
            Ok(HttpResponse::Ok().json(file_info_list))
        },
        Err(e) => Ok(HttpResponse::Ok().json(RestResponse::failed(
            ErrorCode::UNKNOWN_ERROR,
            e.to_string(),
        ))),
    }
}

#[utoipa::path(
    summary = "Delete a file",
    responses(
        (status = 200, description = "Delete file successfully"),
        (status = 400, description = "Bad request"),
        (status = 501, description = "Not implemented"),
    ),
)]
#[delete("file")]
pub async fn delete_file(
    requst_json: web::Json<DeleteFileRequest>,
) -> Result<HttpResponse, AWError> {
    let delete_file_request = requst_json.into_inner();
    info!("Delete file {} in {} successfully", delete_file_request.file_name, delete_file_request.dir_path);
    Ok(HttpResponse::Ok().finish())
}
