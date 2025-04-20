use std::{
    fs::{FileTimes, Permissions},
    os::unix::fs::{chown, PermissionsExt},
    path::PathBuf,
};

use actix_web::{delete, get, post, web, Error as AWError, HttpResponse};
use chrono::Local;
use log::{error, info, warn};
use tokio::{
    fs::{self, File},
    time::{self, Duration, Instant, MissedTickBehavior},
};

use crate::{
    database::{
        file_info::{TrashFileInfo, TrashFileInfoList},
        sqlite::PoolDatabaseManager,
    },
    model::{
        common::{ErrorCode, RestResponse},
        settings::TrashListSettings,
        trash::{
            DeleteTrashFileRequest, DeleteTrashFilesRequest, RestoreTrashFileRequest,
            RestoreTrashFilesRequest,
        },
    },
    utils::error::DfrError,
    SharedSettings,
};

pub async fn remove_trash_file_timer(
    settings: web::Data<SharedSettings>,
    db: PoolDatabaseManager,
) -> Result<(), DfrError> {
    let db = db.clone();
    // Implement the logic to remove trash files based on the timer
    tokio::spawn(async move {
        // Logic to remove old trash files
        info!("Start to setup removing trash file timer");
        // Set up a timer to run every 60 secondss
        let mut intv = time::interval_at(
            Instant::now() + Duration::from_secs(5),
            Duration::from_secs(60),
        );
        intv.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            intv.tick().await;
            let clear_trash_interval_s = {
                let settings = settings.lock().await;
                settings.system.clear_trash_interval_s
            };

            let mut query_list_params = TrashListSettings::default();
            let need_remove_time =
                Local::now() - Duration::from_secs(clear_trash_interval_s as u64);
            query_list_params.end_removed_time = Some(need_remove_time.clone());
            info!("Start to clear old trash files before {} ", need_remove_time);
            loop {
                let list_result = db.list_trash_files(&query_list_params);
                if list_result.is_err() {
                    error!("Failed to list trash files: {:?}", list_result.err());
                    break;
                }
                let trash_files = list_result.unwrap();
                for file in trash_files.trash_file_info_list.iter() {
                    if let Err(e) = clear_trash_file(&settings, &db, file).await {
                        error!("Failed to delete trash file {:?}: {:?}", file, e);
                    } else {
                        info!("Deleted trash file {:?}", file);
                    }
                }
                if trash_files.trash_file_info_list.len() < query_list_params.page_count as usize {
                    break;
                }
            }
        }
    });
    Ok(())
}

async fn clear_trash_file(
    settings: &SharedSettings,
    db: &PoolDatabaseManager,
    trash_file_info: &TrashFileInfo,
) -> Result<(), DfrError> {
    let trash_path = {
        let settings = settings.lock().await;
        settings.system.trash_path.clone()
    };
    let mut file = PathBuf::from(trash_path);
    file.push(trash_file_info.md5.as_str());
    if !file.exists() {
        // remove trash file from db
        db.remove_trash_file_by_md5(&trash_file_info.md5)?;
        warn!(
            "Trash file {:?} is not exist, remove trash file from db by md5",
            file
        );

        return Ok(());
    }
    let mut query_list_params = TrashListSettings::default();
    query_list_params.md5 = Some(trash_file_info.md5.clone());

    let trash_file_list = db.list_trash_files(&query_list_params)?;
    if trash_file_list.trash_file_info_list.len() == 1 {
        fs::remove_file(file.as_path()).await?;
    }
    db.remove_trash_file_by_path(&trash_file_info.dir_path, &trash_file_info.file_name)?;

    info!("Delete trash file '{:?}' successfully", file.as_path());
    Ok(())
}

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

    clear_trash_file(&settings, db.get_ref(), &db_file_info).await?;
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

    info!(
        "Delete trash files {:?} successfully",
        delete_file_request.files
    );
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
#[post("/trash/file/restore")]
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
    let std_file = restore_file.into_std().await;
    // set modified time
    // FIXME we can not set create time by using std library
    let times = FileTimes::new().set_modified(trash_file_info.modified.into());
    std_file.set_times(times)?;
    // set owner
    chown(
        origin_file_path.as_path(),
        Some(trash_file_info.uid),
        Some(trash_file_info.gid),
    )?;

    // check if trash file is unique
    let mut query_list_params = TrashListSettings::default();
    query_list_params.md5 = Some(trash_file_info.md5.clone());

    let trash_file_list = db.list_trash_files(&query_list_params)?;
    if trash_file_list.trash_file_info_list.len() == 1 {
        info!("Remove trash file {:?}", file);
        fs::remove_file(file.as_path()).await?;
    }
    db.restore_trash_file_by_path(&trash_file_info)?;

    info!(
        "Restore trash file '{}' successfully",
        trash_file_info.get_file_path()
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
#[post("/trash/files/restore")]
pub async fn restore_trash_files(
    requst_json: web::Json<RestoreTrashFilesRequest>,
) -> Result<HttpResponse, AWError> {
    let restore_file_request = requst_json.into_inner();

    info!(
        "Restore files {:?} successfully",
        restore_file_request.files
    );
    Ok(HttpResponse::Ok().finish())
}
