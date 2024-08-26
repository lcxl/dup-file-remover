use chrono::NaiveDateTime;
use md5::{Digest, Md5};

pub struct FileInfo {
    pub file_path: String,
    pub file_name: String,
    pub file_extension: String,
    pub file_size: u64,
    pub create_time: NaiveDateTime,
    pub update_time: NaiveDateTime,
    // scan_time is the time when the file was last scanned
    pub scan_time: NaiveDateTime,
    pub md5: String,
}

impl FileInfo {
    pub fn new(file_path: &str, scan_time: NaiveDateTime) -> Result<Self,  Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(file_path)?;
        let file_name = std::path::Path::new(file_path).file_name().unwrap().to_string_lossy().to_string();
        let file_extension = std::path::Path::new(file_path).extension().unwrap().to_string_lossy().to_string();
        let create_time = metadata.created()?.elapsed()?;
        let update_time = metadata.modified()?.elapsed()?;
        Ok(Self {
            file_path: file_path.to_string(),
            file_name,
            file_extension,
            file_size: metadata.len(),
            create_time: NaiveDateTime::from_timestamp_opt(create_time.as_secs() as i64, 0).unwrap(),
            update_time: NaiveDateTime::from_timestamp_opt(update_time.as_secs() as i64, 0).unwrap(),
            scan_time,
            md5: format!("{:x}", Md5::new().chain_update(std::fs::read(file_path)?).finalize()),
        })
    }
}   