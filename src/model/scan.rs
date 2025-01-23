use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ScanRequest {
    /// Scan path
    pub scan_path: String,
}
