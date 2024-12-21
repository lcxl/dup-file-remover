use std::path::Path;

use actix_web::{web, HttpResponse, Error as AWError};
use chrono::Utc;
use log::info;
use crate::database::file_info::FileInfo;
use crate::database::sqlite::{Pool, PoolDatabaseManager};
use crate::model::scan::ScanRequest;
use crate::model::common::RestResponse;


pub async fn scan_files(requst_json: web::Json<ScanRequest>, db: web::Data<PoolDatabaseManager>) -> Result<HttpResponse, AWError>  {
    
    let scan_path = &requst_json.scan_path;
    let path = Path::new(scan_path);
    if !path.exists() {
        let failed_response: RestResponse<()> = RestResponse::failed(11, "Scan path does not exist".to_owned());
        return Ok(HttpResponse::NotFound().json(failed_response));
    }
    scan_all_files(path, db.get_ref()).await?;

    let test_none_response: RestResponse<()> = RestResponse::succeed();
    HttpResponse::Ok().json(test_none_response);
    
    Ok(HttpResponse::Ok().json(RestResponse::succeed_with_data(None::<()>)))
}

pub async fn scan_all_files(path: &Path, db: &PoolDatabaseManager) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_dir() {
        let mut entries = tokio::fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            info!("{:?}", entry.path()); // For demonstration purposes, print the path of each file/directory.
            if path.is_dir() {
                let sub_path = entry.path();
                let list_task = Box::pin(scan_all_files(&sub_path, db));
                list_task.await?;
            } else {
                let file_info = FileInfo::new(entry.path().to_string_lossy().to_string().as_str(), Utc::now().naive_utc())?;
                db.0.insert_file_info(&file_info)?;
            }
        }
    } else {
        let file_info = FileInfo::new(path.to_string_lossy().to_string().as_str(), Utc::now().naive_utc())?;
        db.0.insert_file_info(&file_info)?;
    }
    
    Ok(())
}