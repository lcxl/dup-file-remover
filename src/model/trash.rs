use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request body for deleting a trash file.
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct DeleteTrashFileRequest {
    /// The directory path of trash file
    pub dir_path: String,
    /// The name of trash file
    pub file_name: String,
}

/// Delete trash file path
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct DeleteTrashFilePath {
    /// The directory path of trash file
    pub dir_path: String,
    /// The name of trash file
    pub file_name: String,
}

/// Request body for deleting multiple trash files.
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct DeleteTrashFilesRequest {
    /// The directory path of file to be deleted
    pub files: Vec<DeleteTrashFilePath>,
}

/// Request body for restore a trash file.
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct RestoreTrashFileRequest {
    /// The directory path of trash file
    pub dir_path: String,
    /// The name of trash file
    pub file_name: String,
}

/// Restore trash file path
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct RestoreTrashFilePath {
    /// The directory path of trash file
    pub dir_path: String,
    /// The name of trash file
    pub file_name: String,
}

/// Request body for restore multiple trash files.
#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct RestoreTrashFilesRequest {
    /// The directory path of file to be restore
    pub files: Vec<RestoreTrashFilePath>,
}