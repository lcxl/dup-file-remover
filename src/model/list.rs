use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
pub struct QueryListParams {
    pub page_no: i64,
    pub page_count: i64,
    pub min_file_size: Option<i64>,
    pub max_file_size: Option<i64>,
}
