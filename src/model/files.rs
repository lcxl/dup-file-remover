use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for deleting a file.
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct DeleteFileRequest {
    /// The directory path of file to be deleted
    pub dir_path: String,
    /// The name of file to be deleted
    pub file_name: String,
    /// Whether to delete permanently or move to trash
    pub delete_permanently: Option<bool>,
    /// Force delete the file even if it is not duplicates. This option should be used with caution
    pub force_delete: Option<bool>,
}

/// Delete file path
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct DeleteFilePath {
/// The directory path of file to be deleted
pub dir_path: String,
/// The name of file to be deleted
pub file_name: String,
}

/// Request body for deleting multiple files.
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct DeleteFilesRequest {
    /// The directory path of file to be deleted
    pub files: Vec<DeleteFilePath>,
    /// Whether to delete permanently or move to trash
    pub delete_permanently: Option<bool>,
    /// Force delete the file even if it is not duplicates. This option should be used with caution
    pub force_delete: Option<bool>,
}
