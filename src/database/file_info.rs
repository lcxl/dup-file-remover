use std::os::linux::fs::MetadataExt;

use chrono::{DateTime, Local};
use md5::{Digest, Md5};

use super::sqlite::FileInfoDO;

#[derive(Debug)]
pub struct InodeInfo {
    pub inode: u64,  // inode number
    pub dev_id: u64, // New field to store the device ID
    pub permissions: u32,
    pub nlink: u64,
    pub uid: u32,
    pub gid: u32,
    pub created: DateTime<Local>,
    pub modified: DateTime<Local>,
    pub md5: Option<String>,
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
#[derive(Debug)]
pub struct FileInfo {
    pub inode_info: InodeInfo, // Renamed field to use the new struct
    pub dir_path: String,
    pub file_path: String,
    pub file_name: String,
    pub file_extension: Option<String>,
    // version is the version of the file, used to track changes
    pub version: u64,
    // scan_time is the time when the file was last scanned
    pub scan_time: DateTime<Local>,
}

#[derive(Debug)]
pub struct FileInfoWithMd5Count {
    pub file_info: FileInfo,
    pub md5_count: usize,
}

impl FileInfo {
    pub fn new(
        file_path: &str,
        version: u64,
        scan_time: DateTime<Local>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = std::fs::canonicalize(file_path)?;
        let metadata = std::fs::metadata(file_path.clone())?;
        let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
        let file_extension: Option<String>;
        if file_path.extension().is_none() {
            file_extension = Option::None;
        } else {
            file_extension =
                Option::Some(file_path.extension().unwrap().to_string_lossy().to_string());
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

    pub fn update_md5(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
