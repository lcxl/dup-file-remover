use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ScanRequest {
    pub scan_path: String,
}
