use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::database::file_info::FileInfo;
use crate::database::sqlite::{Pool, PoolDatabaseManager};
use crate::model::common::RestResponse;
use crate::model::scan::ScanRequest;
use actix_web::{web, Error as AWError, HttpResponse};
use chrono::{DateTime, Local, Utc};
use log::{debug, error, info, warn};

static STOP_SCAN_FLAG: AtomicBool = AtomicBool::new(false);
static SCAN_FLAG: AtomicBool = AtomicBool::new(false);

/// Start a new scan. If a scan is already in progress, return a conflict error.
pub async fn start_scan(
    requst_json: web::Json<ScanRequest>,
    db: web::Data<PoolDatabaseManager>,
) -> Result<HttpResponse, AWError> {
    let is_scan_started = SCAN_FLAG.load(Ordering::Acquire);
    if is_scan_started {
        warn!("Scan already in progress. Please wait for it to complete or stop it first.");
        return Ok(HttpResponse::Conflict().body("Scan already in progress"));
    }

    let scan_path = requst_json.scan_path.clone();
    let path = Path::new(&scan_path);
    if !path.exists() {
        let failed_response: RestResponse<()> =
            RestResponse::failed(11, format!("Scan path '{}' does not exist", &scan_path));
        return Ok(HttpResponse::NotFound().json(failed_response));
    }
    STOP_SCAN_FLAG.store(false, Ordering::Relaxed);
    tokio::spawn(async move {
        let result = SCAN_FLAG.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
        if result.is_err() {
            error!("Failed to acquire lock for scan, giving up");
            return;
        }
        let result: Result<(), Box<dyn std::error::Error>> =
            scan_all_files(&scan_path, db.get_ref()).await;
        // reset the flag after scan completion or failure
        SCAN_FLAG.store(false, Ordering::Relaxed);
        if result.is_err() {
            error!("Failed to scan files: {:?}", result.err());
        } else {
            info!("Scan completed successfully");
        }
    });

    let test_none_response: RestResponse<()> = RestResponse::succeed();
    HttpResponse::Ok().json(test_none_response);

    Ok(HttpResponse::Ok().json(RestResponse::succeed_with_data(None::<()>)))
}

/// Stop the current file scan.
pub async fn stop_scan() -> Result<HttpResponse, AWError> {
    info!("Stopping scan");
    STOP_SCAN_FLAG.store(true, Ordering::Relaxed);
    Ok(HttpResponse::Ok().json(RestResponse::succeed_with_data(None::<()>)))
}

/// Scan all files in a directory and its subdirectories.
pub async fn scan_all_files(
    scan_path: &String,
    db: &PoolDatabaseManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(scan_path);
    _scan_all_files(path, db).await
}

/// Scan all files in a directory and its subdirectories.
async fn _scan_all_files(
    path: &Path,
    db: &PoolDatabaseManager,
) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_dir() {
        let mut entries = tokio::fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if STOP_SCAN_FLAG.load(Ordering::Acquire) {
                info!("Received stop scan flag, stop scanning");
                return Ok(());
            }
            debug!("{:?}", entry.path()); // For demonstration purposes, print the path of each file/directory.
            if path.is_dir() {
                let sub_path = entry.path();
                let list_task = Box::pin(_scan_all_files(&sub_path, db));
                list_task.await?;
            } else {
                scan_file(&entry.path(), db).await?;
            }
        }
    } else {
        scan_file(path, db).await?;
    }

    Ok(())
}

async fn scan_file(
    file_path: &Path,
    db: &PoolDatabaseManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file_info = FileInfo::new(
        file_path.to_string_lossy().to_string().as_str(),
        Local::now(),
    )?;
    let manager = &db.0;
    let get_file_result = manager.get_file_by_path(&file_info.file_path);
    if get_file_result.is_ok() {
        // check file update time and update if necessary
        let db_file_info = get_file_result.unwrap();
        if db_file_info.inode_info == file_info.inode_info {
            debug!(
                "File '{}' already exists and is same in database, skipping",
                file_info.file_path
            );
            return Ok(());
        } else {
            info!(
                "File '{}' is changed, need to update",
                file_info.file_path
            );
        }
    } else {
        info!("Add new file '{}' to db", file_info.file_path);
    }
    // update file md5 and insert into db
    file_info.update_md5()?;
    manager.insert_file_info(&file_info)?;
    Ok(())
}
