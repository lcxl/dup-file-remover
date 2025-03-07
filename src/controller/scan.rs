use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::file_info::FileInfo;
use crate::database::sqlite::PoolDatabaseManager;
use crate::model::common::{ErrorCode, RestResponse};
use crate::model::scan::{ScanRequest, ScanStatus, SharedScanStatus};
use crate::utils::error::DfrError;
use crate::Settings;
use actix_web::{get, post, web, Error as AWError, HttpResponse};
use chrono::{DateTime, Local};
use log::{debug, error, info, warn};

static STOP_SCAN_FLAG: AtomicBool = AtomicBool::new(false);
static SCAN_FLAG: AtomicBool = AtomicBool::new(false);

#[utoipa::path(
    summary = "Get scan status",
    responses(
        (status = 200, description = "Scan status", body = RestResponse<ScanStatus>),
    ),
)]
#[get("/scan/status")]
pub async fn query_scan_status(
    scan_status: web::Data<SharedScanStatus>,
) -> Result<HttpResponse, AWError> {
    // Implementation of scan_status function
    let response = scan_status.lock().await.clone();
    Ok(HttpResponse::Ok().json(RestResponse::succeed_with_data(response)))
}

/// Start a new scan. If a scan is already in progress, return a conflict error.
#[utoipa::path(
    summary = "Start a new file scan",
    request_body(content = ScanRequest),
    responses(
        (status = 200, description = "Scan started successfully", body = RestResponse<i64>),
        (status  = 409, description = "Scan already in progress"),
    ),
)]
#[post("/scan/start")]
pub async fn start_scan(
    requst_json: web::Json<ScanRequest>,
    db: web::Data<PoolDatabaseManager>,
    scan_status: web::Data<SharedScanStatus>,
    settings: web::Data<Settings>,
) -> Result<HttpResponse, AWError> {
    let is_scan_started = SCAN_FLAG.load(Ordering::Acquire);
    if is_scan_started {
        warn!("Scan already in progress. Please wait for it to complete or stop it first.");
        return Ok(HttpResponse::Conflict().body("Scan already in progress"));
    }
    let scan_request = requst_json.into_inner();
    let path;
    if scan_request.scan_path.len() == 0 || scan_request.scan_path.trim().len() == 0 {
        // Use default scan path if no path is provided
        path = Path::new(settings.default_scan_path.as_str());
    } else {
        path = Path::new(&scan_request.scan_path);
    }
    
    if !path.exists() {
        return Ok(HttpResponse::Ok().json(RestResponse::failed(
            ErrorCode::FILE_PATH_NOT_FOUND,
            format!("Scan path '{}' does not exist", &scan_request.scan_path),
        )));
    }
    STOP_SCAN_FLAG.store(false, Ordering::Relaxed);
    tokio::spawn(async move {
        let result = SCAN_FLAG.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
        if result.is_err() {
            error!("Failed to acquire lock for scan, giving up");
            return;
        }

        let result = scan_all_files(&scan_request, db.get_ref(), scan_status.get_ref()).await;
        // reset the flag after scan completion or failure
        SCAN_FLAG.store(false, Ordering::Relaxed);
        if result.is_err() {
            error!("Failed to scan files: {:?}", result.err());
        } else {
            info!("Scan completed successfully");
        }
    });

    Ok(HttpResponse::Ok().json(RestResponse::succeed()))
}

/// Stop the current file scan.
#[utoipa::path(summary = "Stop the current file scan")]
#[post("/scan/stop")]
pub async fn stop_scan(scan_status: web::Data<SharedScanStatus>) -> Result<HttpResponse, AWError> {
    info!("Stopping scan");
    STOP_SCAN_FLAG.store(true, Ordering::Relaxed);
    let mut status = scan_status.lock().await;
    status.started = false;
    Ok(HttpResponse::Ok().json(RestResponse::succeed()))
}

/// Scan all files in a directory and its subdirectories.
pub async fn scan_all_files(
    scan_request: &ScanRequest,
    db: &PoolDatabaseManager,
    scan_status: &SharedScanStatus,
) -> Result<(), DfrError> {
    let current_path = Path::new(scan_request.scan_path.as_str());
    let start = SystemTime::now();
    let scan_version = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    {
        let mut status = scan_status.lock().await;
        // convert system time to chrono local datetime
        status.start_time = Some(DateTime::<Local>::from(start));
        status.scanned_file_count = 0;
        status.scan_request = Some(scan_request.clone());
        status.started = true;
    }
    let result = _scan_all_files(current_path, scan_request, scan_version, db, scan_status).await;

    {
        let mut status = scan_status.lock().await;
        status.started = false;
    }

    return result;
}

/// Scan all files in a directory and its subdirectories.
async fn _scan_all_files(
    current_path: &Path,
    scan_request: &ScanRequest,
    scan_version: u64,
    db: &PoolDatabaseManager,
    scan_status: &SharedScanStatus,
) -> Result<(), DfrError> {
    if current_path.is_dir() {
        let mut entries = tokio::fs::read_dir(current_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if STOP_SCAN_FLAG.load(Ordering::Acquire) {
                info!("Received stop scan flag, stop scanning");
                db.remove_deleted_inode()?;
                return Ok(());
            }
            debug!("{:?}", entry.path()); // For demonstration purposes, print the path of each file/directory.
            if entry.path().is_dir() {
                let sub_path = entry.path();
                let list_task = Box::pin(_scan_all_files(
                    &sub_path,
                    scan_request,
                    scan_version,
                    db,
                    scan_status,
                ));
                list_task.await?;
            } else {
                scan_file(scan_request, &entry.path(), scan_version, db, scan_status).await?;
            }
        }
        //remove deleted files from db if path is directory
        db.remove_deleted_files(
            current_path.to_string_lossy().to_string().as_str(),
            scan_version,
        )?;
    } else {
        scan_file(scan_request, current_path, scan_version, db, scan_status).await?;
    }
    Ok(())
}

async fn scan_file(
    scan_request: &ScanRequest,
    file_path: &Path,
    scan_version: u64,
    db: &PoolDatabaseManager,
    scan_status: &SharedScanStatus,
) -> Result<(), DfrError> {
    if let Some(ref extensions) = scan_request.include_file_extensions {
        if let Some(ext) = file_path.extension().and_then(|ext| ext.to_str()) {
            if !extensions.contains(&String::from(ext)) {
                debug!("Skipping file '{:?}' with extension {:?}", file_path, ext);
                return Ok(());
            }
        } else {
            // Handle the case where there is no valid extension
            debug!("Skipping file '{:?}' due to invalid extension", file_path);
            return Ok(());
        }
    }

    let mut file_info = FileInfo::new(
        file_path.to_string_lossy().to_string().as_str(),
        scan_version,
        Local::now(),
    )?;
    if let Some(min_file_size) = scan_request.min_file_size {
        if file_info.inode_info.size < min_file_size {
            debug!("Skipping file '{:?}' due to size below minimum", file_path);
            return Ok(());
        }
    }

    if let Some(max_file_size) = scan_request.max_file_size {
        if file_info.inode_info.size > max_file_size {
            debug!("Skipping file '{:?}' due to size above maximum", file_path);
            return Ok(());
        }
    }

    {
        let mut status = scan_status.lock().await;
        //inc scanned file count and set current file info
        status.scanned_file_count += 1;
        status.current_file_info = Some(file_info.clone());
    }
    let get_file_result = db.get_file_by_path(&file_info.dir_path, &file_info.file_name);
    match get_file_result {
        Ok(db_file_info) => {
            if db_file_info.inode_info == file_info.inode_info {
                debug!(
                "File '{}' already exists and is same in database, update version from {} to {}",
                file_info.file_path, db_file_info.version, file_info.version
            );
                db.update_version(&file_info)?;
                return Ok(());
            } else {
                info!("File '{}' is changed, need to update", file_info.file_path);
            }
        }
        Err(error) => {
            info!(
                "File '{}' not found in database, or error: {:?}",
                file_info.file_path, error
            );
            info!("Add new file '{}' to db", file_info.file_path);
        }
    }

    // update file md5 and insert into db
    file_info.update_md5()?;
    db.insert_file_info(&file_info)?;
    Ok(())
}
