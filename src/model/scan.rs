use serde::Deserialize;
use utoipa::ToSchema;
///Scan request structure for initiating a file scan operation.
#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct ScanRequest {
    /// Scan path
    pub scan_path: String,
    /// Optional list of file extensions to include in the scan. If not provided, all files will be scanned.
    pub include_file_extensions: Option<Vec<String>>,
}
