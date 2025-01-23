use serde::Deserialize;
use utoipa::IntoParams;

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
}
