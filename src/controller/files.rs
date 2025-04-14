use std::path::PathBuf;

use actix_web::{delete, get, web, Error as AWError, HttpResponse};
use chrono::Local;
use log::{info, warn};
use tokio::fs;

use crate::{
    database::{
        file_info::{FileInfo, FileInfoList},
        sqlite::PoolDatabaseManager,
    },
    model::{
        common::{ErrorCode, RestResponse},
        files::{DeleteFileRequest, DeleteFilesRequest},
        settings::ListSettings,
    },
    utils::error::DfrError,
    SharedSettings,
};

#[utoipa::path(
    summary = "Query list file settings",
    responses(
        (status = 200, description = "List settings", body = RestResponse<ListSettings>),
    ),
)]
#[get("/list/settings")]
pub async fn query_list_settings(
    settings: web::Data<SharedSettings>,
) -> Result<HttpResponse, AWError> {
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
    let file_info_list = conn.0.list_files(&query_list)?;
    {
        let mut settings = settings.lock().await;
        settings.list = query_list.into_inner().clone();
        settings.save()?;
    }
    Ok(HttpResponse::Ok().json(file_info_list))
}

#[utoipa::path(
    summary = "Delete a file",
    request_body(content = DeleteFileRequest),
    responses(
        (status = 200, description = "Delete file successfully"),
        (status = 400, description = "Bad request"),
        (status = 501, description = "Not implemented"),
    ),
)]
#[delete("/file")]
pub async fn delete_file(
    requst_json: web::Json<DeleteFileRequest>,
    db: web::Data<PoolDatabaseManager>,
    settings: web::Data<SharedSettings>,
) -> Result<HttpResponse, DfrError> {
    let delete_file_request = requst_json.into_inner();

    let db_file_info = db.get_file_by_path(
        delete_file_request.dir_path.as_str(),
        delete_file_request.file_name.as_str(),
    )?;

    let file = PathBuf::from(db_file_info.file_path.as_str());
    if !file.exists() {
        // remove file from db
        db.remove_file_by_path(
            &delete_file_request.dir_path,
            &delete_file_request.file_name,
        )?;
        info!("Remove file {:?} from db", file);

        return Ok(
            HttpResponse::Ok().json(RestResponse::succeed_with_message(format!(
                "File {} is not exist",
                file.display()
            ))),
        );
    }
    let file_info = FileInfo::new(db_file_info.file_path.as_str(), 0, Local::now())?;
    if file_info.inode_info != db_file_info.inode_info {
        // file is changed, need to rescan
        return DfrError::custom_error(
            ErrorCode::FILE_CHANGED,
            format!("File {} changed, need to rescan", file_info.file_path),
        );
    }

    if let Some(md5) = db_file_info.inode_info.md5.clone() {
        let db_file_info_list = db.get_file_list_by_md5(md5.as_str())?;

        let mut dup_len = db_file_info_list.len();
        for item in db_file_info_list.iter() {
            if item.file_path == file_info.file_path {
                dup_len -= 1;
                continue;
            }

            // check if file is exist
            let file = PathBuf::from(db_file_info.file_path.as_str());
            if !file.exists() {
                dup_len -= 1;
                continue;
            }

            // check if file is changed
            let file_info = FileInfo::new(item.file_path.as_str(), 0, Local::now())?;
            if file_info.inode_info != item.inode_info {
                // file is changed, need to rescan
                dup_len -= 1;
                continue;
            }
        }

        if dup_len < 1 {
            let need_delete = delete_file_request.force_delete.unwrap_or(false);

            if !need_delete {
                return DfrError::custom_error(
                    ErrorCode::NOT_ALLOW_DELETE_FILE,
                    format!(
                        "Not allow to delete file {}, the file is not duplicated",
                        file_info.file_path
                    ),
                );
            }
        }
    } else {
        return DfrError::custom_error(
            ErrorCode::SYSTEM_ERROR,
            format!(
                "Not allow to delete file {}, the file hash is none",
                file_info.file_path
            ),
        );
    }

    // remove file
    let delete_permanently = delete_file_request.delete_permanently.unwrap_or(false);
    if delete_permanently {
        warn!("Delete file {} permanently", db_file_info.file_path);
        fs::remove_file(db_file_info.file_path.as_str()).await?;
        db.remove_file_by_path(&file_info.dir_path, &file_info.file_name)?;
    } else {
        // move to trash dir
        info!("Move file {} to trash dir", db_file_info.file_path);
        let trash_file_path = {
            let settings = settings.lock().await;
            let mut trash_path = PathBuf::from(settings.system.trash_path.as_str());
            trash_path.push(db_file_info.inode_info.md5.clone().unwrap());
            trash_path
        };
        // try to rename file
        if !trash_file_path.exists() {
            let result =
                fs::rename(db_file_info.file_path.as_str(), trash_file_path.as_path()).await;
            if let Err(error) = result {
                warn!("Failed to rename file: {:?}, try to copy and delete", error);
                // if rename failed, try to copy and delete
                fs::copy(db_file_info.file_path.as_str(), trash_file_path.as_path()).await?;
                fs::remove_file(db_file_info.file_path.as_str()).await?;
            }
        } else {
            info!(
                "Found file in trash dir: {:?}, remove file directly",
                trash_file_path
            );
            fs::remove_file(db_file_info.file_path.as_str()).await?;
        }

        db.move_file_to_trash(&db_file_info)?;
    }

    info!(
        "Delete file {} in {} successfully",
        delete_file_request.file_name, delete_file_request.dir_path
    );
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    summary = "Delete files",
    request_body(content = DeleteFilesRequest),
    responses(
        (status = 200, description = "Delete files successfully"),
        (status = 400, description = "Bad request"),
        (status = 501, description = "Not implemented"),
    ),
)]
#[delete("/files")]
pub async fn delete_files(
    requst_json: web::Json<DeleteFilesRequest>,
) -> Result<HttpResponse, AWError> {
    let delete_file_request = requst_json.into_inner();

    info!("Delete files {:?} successfully", delete_file_request.files);
    Ok(HttpResponse::Ok().finish())
}
