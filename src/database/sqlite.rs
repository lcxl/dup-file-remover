use rusqlite::{Result};

use super::file_info::FileInfo;

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub struct DatabaseManager {
    conn: Connection,
}

impl DatabaseManager {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(Self { conn })
    }

    pub fn create_table(&self) -> Result<()> {
        let sql = "CREATE TABLE IF NOT EXISTS file_info (
            file_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_extension TEXT NULL,
            file_size INTEGER NOT NULL,
            create_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            update_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            scan_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            md5 TEXT NOT NULL,
            UNIQUE(file_path),
            INDEX(md5),
            INDEX(file_name),
            INDEX(file_size),
            INDEX(file_extension)
        )";
        self.conn.execute(sql, [])?;
        Ok(())
    }

    pub fn insert_file_info(&self, file_info: &FileInfo) -> Result<()> {
        let sql = "INSERT or UPDATE INTO files (file_path, file_name, file_extension, file_size, create_time, update_time, scan_time, md5) 
          VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
        let mut result = match self.conn.execute(
            sql,
            (
                &file_info.file_path,
                &file_info.file_name,
                &file_info.file_extension,
                file_info.file_size,
                &file_info.create_time,
                &file_info.update_time,
                &file_info.scan_time,
                &file_info.md5,
            ),
        ) {
            Ok(updated) => Ok(()),
            Err(_e) => {
                println!("Error inserting file info");
                Err(_e)
            }
        };
        return result;
    }

    pub fn get_file_by_path(&self, file_path: &str) -> Result<FileInfo> {
        let sql = "SELECT file_path, file_name, file_extension, file_size, create_time, update_time, scan_time, md5 
        FROM files 
        WHERE file_path = ?";
        let mut stmt = self.conn.prepare(sql)?;
        let file_iter = stmt.query_row([file_path], |row| {
            Ok(FileInfo {
                file_path: row.get(0)?,
                file_name: row.get(1)?,
                file_extension: row.get(2)?,
                file_size: row.get(3)?,
                create_time: row.get(4)?,
                update_time: row.get(5)?,
                scan_time: row.get(6)?,
                md5: row.get(7)?,
            })
        });
       
        let file_info = file_iter?;
        return Ok(file_info);
    }

    pub fn get_file_list_by_md5(&self, md5: &str) -> Result<Vec<FileInfo>> {
        let sql = "SELECT file_path, file_name, file_extension, file_size, create_time, update_time, scan_time, md5 
        FROM files 
        WHERE md5 = ?";
        let mut stmt = self.conn.prepare(sql)?;
        let file_iter = stmt.query_map([md5], |row| {
            Ok(FileInfo {
                file_path: row.get(0)?,
                file_name: row.get(1)?,
                file_extension: row.get(2)?,
                file_size: row.get(3)?,
                create_time: row.get(4)?,
                update_time: row.get(5)?,
                scan_time: row.get(6)?,
                md5: row.get(7)?,
            })
        });
        let mut files = Vec::new();
        for item in file_iter? {
            files.push(item?);
        }
        Ok(files)
    }

    pub fn remove_file_by_path(&self, file_path: &str) -> Result<()> {
        let sql = "DELETE FROM files 
        WHERE file_path = ?";
        let mut stmt = self.conn.prepare(sql)?;
        stmt.execute([file_path])?;
        Ok(())
    }

    pub fn list_files(&self, page_no: i64, page_count: i64, min_file_size: i64, max_file_size: i64) -> Result<Vec<FileInfo>> {
        let sql = "SELECT file_name, file_path, file_size, create_time, update_time, md5, count(md5) 
            FROM files 
            where ? <= file_size and ? >= file_size
            group by md5
            order by count(md5) desc, file_size desc
            LIMIT ? OFFSET ?";
        let mut stmt = self.conn.prepare(sql)?;
        let file_iter = stmt.query_map([min_file_size, max_file_size, page_count, (page_no - 1) * page_count], |row| {
            Ok(FileInfo {
                file_name: row.get(0)?,
                file_path: row.get(1)?,
                file_size: row.get(2)?,
                create_time: row.get(3)?,
                update_time: row.get(4)?,
                md5: row.get(5)?,
                file_extension: todo!(),
                scan_time: todo!(),
            })
        });
        let mut files = Vec::new();
        for item in file_iter? {
            files.push(item?);
        }
        Ok(files)
    }
}
