use std::os::linux::fs::MetadataExt;

use chrono::{DateTime, Local};
use md5::{Digest, Md5};
use serde::Serialize;
use utoipa::ToSchema;

use crate::utils::error::DfrError;

use super::sqlite::FileInfoDO;
/// Inode info
#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct InodeInfo {
    /// Inode number
    pub inode: u64,
    /// Device ID
    pub dev_id: u64,
    pub permissions: u32,
    pub nlink: u64,
    pub uid: u32,
    pub gid: u32,
    pub created: DateTime<Local>,
    pub modified: DateTime<Local>,
    /// File md5
    pub md5: Option<String>,
    /// File size
    pub size: u64,
}

/// Implement PartialEq for InodeInfo to compare two instances based on their fields.
/// This comparison ignores the md5 field.
impl PartialEq<InodeInfo> for InodeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.inode == other.inode
            && self.size == other.size
            && self.dev_id == other.dev_id
            && self.permissions == other.permissions
            && self.nlink == other.nlink
            && self.uid == other.uid
            && self.gid == other.gid
            && self.created == other.created
            && self.modified == other.modified
            && self.size == other.size
    }
}
/// File info
#[derive(Debug, Serialize, ToSchema, Clone)]
pub struct FileInfo {
    /// Inode info
    pub inode_info: InodeInfo,
    /// Dir path of the directory containing the file
    pub dir_path: String,
    /// File path
    pub file_path: String,
    /// File name
    pub file_name: String,
    /// File extension
    pub file_extension: Option<String>,
    /// version is the version of the file, used to track changes
    pub version: u64,
    /// scan_time is the time when the file was last scanned
    pub scan_time: DateTime<Local>,
}

impl FileInfo {
    pub fn new(
        file_path: &str,
        version: u64,
        scan_time: DateTime<Local>,
    ) -> Result<Self, DfrError> {
        let file_path = std::fs::canonicalize(file_path)?;
        let metadata = std::fs::metadata(file_path.clone())?;
        let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
        let file_extension: Option<String>;
        if let Some(file_ext) = file_path.extension() {
            // convert to lowercase
            file_extension = Some(file_ext.to_string_lossy().to_string().to_lowercase());
        } else {
            // no extension
            file_extension = None;
        }

        let created = DateTime::<Local>::from(metadata.created()?);
        let modified = DateTime::<Local>::from(metadata.modified()?);
        let inode_info = InodeInfo {
            inode: metadata.st_ino(),               // Get the inode number
            dev_id: metadata.st_dev(),              // New field to store the device ID
            permissions: metadata.st_mode() as u32, // Get the permissions
            nlink: metadata.st_nlink(),             // Get the number of links
            uid: metadata.st_uid(),                 // Get the user ID
            gid: metadata.st_gid(),                 // Get the group ID
            created,
            modified,
            md5: None, // Initialize MD5 as None
            size: metadata.len(),
        };
        Ok(Self {
            inode_info,
            dir_path: file_path.parent().unwrap().to_string_lossy().to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            file_name,
            file_extension,
            scan_time,
            version,
        })
    }

    pub fn update_md5(&mut self) -> Result<(), DfrError> {
        let file_path = format!("{}/{}", self.dir_path, self.file_name);
        self.inode_info.md5 = Some(format!(
            "{:x}",
            Md5::new()
                .chain_update(std::fs::read(file_path)?)
                .finalize()
        ));
        Ok(())
    }

    pub fn from_do(inode_info: InodeInfo, file_info_do: FileInfoDO) -> Self {
        let file_path = format!("{}/{}", file_info_do.dir_path, file_info_do.file_name);
        Self {
            inode_info,
            file_name: file_info_do.file_name,
            dir_path: file_info_do.dir_path,
            file_path,
            file_extension: file_info_do.file_extension,
            scan_time: file_info_do.scan_time,
            version: file_info_do.version,
        }
    }
}

/// File info with md5 count
#[derive(Debug, Serialize, ToSchema)]
pub struct FileInfoWithMd5Count {
    /// File info
    pub file_info: FileInfo,
    /// Md5 count
    pub md5_count: usize,
    /// Optional filter md5 count
    pub filter_md5_count: Option<usize>,
}

/// File info list with total count
#[derive(Debug, Serialize, ToSchema)]
pub struct FileInfoList {
    /// File info list
    pub file_info_list: Vec<FileInfoWithMd5Count>,
    /// Total file count
    pub total_count: u64,
}
