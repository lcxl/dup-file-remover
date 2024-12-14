use serde::Deserialize;

#[derive(Deserialize)]
pub struct ScanRequest {
    pub scan_path: String,
}
