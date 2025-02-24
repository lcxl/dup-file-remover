use std::ops::Deref;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utoipa::ToSchema;

use crate::database::file_info::FileInfo;
///Scan request structure for initiating a file scan operation.
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct ScanRequest {
    /// Scan path
    pub scan_path: String,
    /// Optional list of file extensions to include in the scan. If not provided, all files will be scanned.
    pub include_file_extensions: Option<Vec<String>>,
    /// Minimum file size in bytes to include in the scan. If not provided, there is no minimum size limit.
    pub min_file_size: Option<u64>,
    /// Maximum file size in bytes to include in the scan. If not provided, there is no maximum size limit.
    pub max_file_size: Option<u64>,
}
/// Scan status structure to keep track of the progress and state of a file scan operation.
#[derive(Serialize, ToSchema, Debug, Clone)]
pub struct ScanStatus {
    /// Scan request details
    pub scan_request: Option<ScanRequest>,
    /// Indicates whether the scan has started.
    pub started: bool,
    /// Start time of the scan.
    pub start_time: Option<DateTime<Local>>,
    /// Number of files scanned so far.
    pub scanned_file_count: usize,
    /// Information about the current file being processed during the scan.
    pub current_file_info: Option<FileInfo>,
}

impl ScanStatus {
    pub fn new() -> Self {
        ScanStatus {
            scan_request: None,
            started: false,
            start_time: None,
            scanned_file_count: 0,
            current_file_info: None,
        }
    }
}

pub struct SharedScanStatus(pub Mutex<ScanStatus>);

impl SharedScanStatus {
    pub fn new() -> Self {
        SharedScanStatus(Mutex::new(ScanStatus::new()))
    }
}

impl Deref for SharedScanStatus {
    type Target = Mutex<ScanStatus>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
