use std::os::linux::fs::MetadataExt;

use chrono::NaiveDateTime;
use md5::{Digest, Md5};

#[derive(Debug)]
pub struct InodeInfo {
    pub inode: u64,  // inode number
    pub dev_id: u64, // New field to store the device ID
    pub permissions: u32,
    pub nlink: u64,
    pub uid: u32,
    pub gid: u32,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub md5: String,
    pub size: u64,
}

#[derive(Debug)]
pub struct FileInfo {
    pub inode_info: InodeInfo, // Renamed field to use the new struct
    pub file_path: String,
    pub file_name: String,
    pub file_extension: String,
    // scan_time is the time when the file was last scanned
    pub scan_time: NaiveDateTime,
}

#[derive(Debug)]
pub struct FileInfoWithMd5Count {
    pub file_info: FileInfo,
    pub md5_count: usize,
}

impl FileInfo {
    pub fn new(
        file_path: &str,
        scan_time: NaiveDateTime,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = std::fs::canonicalize(file_path)?;
        let metadata = std::fs::metadata(file_path.clone())?;
        let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
        let file_extension;
        if file_path.extension().is_none() {
            file_extension = "".to_string();
        } else {
            file_extension = file_path.extension().unwrap().to_string_lossy().to_string();
        }
        let created = metadata.created()?.elapsed()?;
        let modified = metadata.modified()?.elapsed()?;
        let inode_info = InodeInfo {
            inode: metadata.st_ino(),               // Get the inode number
            dev_id: metadata.st_dev(),              // New field to store the device ID
            permissions: metadata.st_mode() as u32, // Get the permissions
            nlink: metadata.st_nlink(),             // Get the number of links
            uid: metadata.st_uid(),                 // Get the user ID
            gid: metadata.st_gid(),                 // Get the group ID
            created: NaiveDateTime::from_timestamp_opt(created.as_secs() as i64, 0).unwrap(),
            modified: NaiveDateTime::from_timestamp_opt(modified.as_secs() as i64, 0).unwrap(),
            md5: format!(
                "{:x}",
                Md5::new()
                    .chain_update(std::fs::read(file_path.to_string_lossy().to_string())?)
                    .finalize()
            ),
            size: metadata.len(),
        };
        Ok(Self {
            inode_info,
            file_path: file_path.to_string_lossy().to_string(),
            file_name,
            file_extension,
            scan_time,
        })
    }
}
