use std::{fs::Permissions, os::unix::fs::PermissionsExt, path::PathBuf};

use actix_web::{delete, get, web, Error as AWError, HttpResponse};
use log::{error, info, warn};
use tokio::fs::{self, File};

use crate::{
    database::{file_info::TrashFileInfoList, sqlite::PoolDatabaseManager},
    model::{
        common::{ErrorCode, RestResponse},
        settings::TrashListSettings,
        trash::{DeleteTrashFileRequest, DeleteTrashFilesRequest, RestoreTrashFileRequest, RestoreTrashFilesRequest},
    },
    utils::error::DfrError,
    SharedSettings,
};

#[utoipa::path(
    summary = "Query list trash file settings",
    responses(
        (status = 200, description = "Trash List settings", body = RestResponse<TrashListSettings>),
    ),
)]
#[get("/trash/list/settings")]
pub async fn query_trash_list_settings(
    settings: web::Data<SharedSettings>,
) -> Result<HttpResponse, AWError> {
    let response = settings.lock().await.trash_list.clone();
    Ok(HttpResponse::Ok().json(RestResponse::succeed_with_data(response)))
}

#[utoipa::path(
    summary = "List trash files",
    params(TrashListSettings),
    responses(
        (status = 200, description = "The list of trash file", body=TrashFileInfoList)
    ),
)]
#[get("/trash/list")]
pub async fn list_trash_files(
    query_list: web::Query<TrashListSettings>,
    db: web::Data<PoolDatabaseManager>,
    settings: web::Data<SharedSettings>,
) -> Result<HttpResponse, AWError> {
    let conn = db.get_ref();
    let file_info_list = conn.0.list_trash_files(&query_list)?;
    {
        let mut settings = settings.lock().await;
        settings.trash_list = query_list.into_inner().clone();
        settings.save()?;
    }
    Ok(HttpResponse::Ok().json(file_info_list))
}

#[utoipa::path(
    summary = "Delete a trash file",
    request_body(content = DeleteTrashFileRequest),
    responses(
        (status = 200, description = "Delete trash file successfully"),
        (status = 400, description = "Bad request"),
        (status = 501, description = "Not implemented"),
    ),
)]
#[delete("/trash/file")]
pub async fn delete_trash_file(
    requst_json: web::Json<DeleteTrashFileRequest>,
    db: web::Data<PoolDatabaseManager>,
    settings: web::Data<SharedSettings>,
) -> Result<HttpResponse, DfrError> {
    let delete_trash_file_request = requst_json.into_inner();

    let db_file_info = db.get_trash_file_by_path(
        delete_trash_file_request.dir_path.as_str(),
        delete_trash_file_request.file_name.as_str(),
    )?;

    let trash_path = {
        let settings = settings.lock().await;
        settings.system.trash_path.clone()
    };
    let mut file = PathBuf::from(trash_path);
    file.push(db_file_info.md5.as_str());
    if !file.exists() {
        // remove trash file from db
        db.remove_trash_file_by_md5(&db_file_info.md5)?;
        warn!(
            "Trash file {:?} is not exist, remove trash file from db by md5",
            file
        );

        return Ok(
            HttpResponse::Ok().json(RestResponse::succeed_with_message(format!(
                "Trash file {} is not exist",
                file.display()
            ))),
        );
    }
    let mut query_list_params = TrashListSettings::default();
    query_list_params.md5 = Some(db_file_info.md5);

    let trash_file_list = db.list_trash_files(&query_list_params)?;
    if trash_file_list.trash_file_info_list.len() == 1 {
        fs::remove_file(file.as_path()).await?;
    }
    db.remove_trash_file_by_path(&db_file_info.dir_path, &db_file_info.file_name)?;

    info!(
        "Delete trash file '{}/{}' successfully",
        delete_trash_file_request.file_name, delete_trash_file_request.dir_path
    );
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    summary = "Delete trash files",
    request_body(content = DeleteTrashFilesRequest),
    responses(
        (status = 200, description = "Delete trash files successfully"),
        (status = 400, description = "Bad request"),
        (status = 501, description = "Not implemented"),
    ),
)]
#[delete("/trash/files")]
pub async fn delete_trash_files(
    requst_json: web::Json<DeleteTrashFilesRequest>,
) -> Result<HttpResponse, AWError> {
    let delete_file_request = requst_json.into_inner();

    info!("Delete files {:?} successfully", delete_file_request.files);
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    summary = "Restore a trash file",
    request_body(content = DeleteTrashFileRequest),
    responses(
        (status = 200, description = "Delete trash file successfully"),
        (status = 400, description = "Bad request"),
        (status = 501, description = "Not implemented"),
    ),
)]
#[delete("/trash/file/restore")]
pub async fn restore_trash_file(
    requst_json: web::Json<RestoreTrashFileRequest>,
    db: web::Data<PoolDatabaseManager>,
    settings: web::Data<SharedSettings>,
) -> Result<HttpResponse, DfrError> {
    let delete_trash_file_request = requst_json.into_inner();

    let trash_file_info = db.get_trash_file_by_path(
        delete_trash_file_request.dir_path.as_str(),
        delete_trash_file_request.file_name.as_str(),
    )?;

    let trash_path = {
        let settings = settings.lock().await;
        settings.system.trash_path.clone()
    };
    let mut file = PathBuf::from(trash_path);
    file.push(trash_file_info.md5.as_str());
    if !file.exists() {
        error!("Trash file {:?} is not exist", file);
        let message = format!("Trash file {:?} is not exist", file);
        return DfrError::custom_error(ErrorCode::SYSTEM_ERROR, message);
    }
    // copy trash file to origin path

    let mut origin_file_path = PathBuf::from(trash_file_info.dir_path.as_str());
    if !origin_file_path.is_dir() {
        // create directory
        fs::create_dir_all(origin_file_path.as_path()).await?;
    }
    origin_file_path.push(trash_file_info.file_name.as_str());

    fs::copy(file.as_path(), origin_file_path.as_path()).await?;
    let restore_file = File::open(origin_file_path.as_path()).await?;
    // set permissions
    restore_file
        .set_permissions(Permissions::from_mode(trash_file_info.permissions))
        .await?;

    std::os::unix::fs::chown(origin_file_path.as_path(), Some(0), Some(0))?;

    // check if trash file is unique
    let mut query_list_params = TrashListSettings::default();
    query_list_params.md5 = Some(trash_file_info.md5);

    let trash_file_list = db.list_trash_files(&query_list_params)?;
    if trash_file_list.trash_file_info_list.len() == 1 {
        info!("Remove trash file {:?}", file);
        fs::remove_file(file.as_path()).await?;
    }
    db.remove_trash_file_by_path(&trash_file_info.dir_path, &trash_file_info.file_name)?;

    info!(
        "Restore trash file '{}/{}' successfully",
        delete_trash_file_request.file_name, delete_trash_file_request.dir_path
    );
    Ok(HttpResponse::Ok().finish())
}


#[utoipa::path(
    summary = "Restore trash files",
    request_body(content = RestoreTrashFilesRequest),
    responses(
        (status = 200, description = "Restore trash files successfully"),
        (status = 400, description = "Bad request"),
        (status = 501, description = "Not implemented"),
    ),
)]
#[delete("/trash/files/restore")]
pub async fn restore_trash_files(
    requst_json: web::Json<RestoreTrashFilesRequest>,
) -> Result<HttpResponse, AWError> {
    let restore_file_request = requst_json.into_inner();

    info!("Restore files {:?} successfully", restore_file_request.files);
    Ok(HttpResponse::Ok().finish())
}