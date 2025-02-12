use chrono::{DateTime, Local};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
/// Query time params for list files
#[derive(Deserialize, ToSchema, Clone)]
pub struct QueryTimeParams {
    /// Start time of the query range, use local time zone.
    pub start_time: DateTime<Local>,
    /// End time of the query range, use local time zone.
    pub end_time: DateTime<Local>,
}

/// Query parameters for listing files.
#[derive(Deserialize, IntoParams)]
pub struct QueryListParams {
    /// Page number, start from 1
    pub page_no: i64,
    /// Page count, must be greater than 0
    pub page_count: i64,
    /// Minimum file size
    pub min_file_size: Option<i64>,
    /// Max file size
    pub max_file_size: Option<i64>,
    /// Dir path of the directory containing the file
    pub dir_path: Option<String>,
    /// File name filtering
    pub file_name: Option<String>,
    /// New field for file extension filtering
    pub file_extension: Option<String>,
    /// MD5 hash of the file content, used for filtering files by their content.
    pub md5: Option<String>,
    /// Optional time range filter for file creation.
    pub created: Option<QueryTimeParams>,
    /// Optional time range filter for file modification.
    pub modified: Option<QueryTimeParams>,

}
