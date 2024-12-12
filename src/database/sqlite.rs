use std::sync::Arc;

use chrono::NaiveDateTime;
use rusqlite::Result;

use super::file_info::{FileInfo, FileInfoWithMd5Count, InodeInfo};
use r2d2_sqlite::SqliteConnectionManager;
pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub struct FileInfoDAO {
    pub inode_info_id: u64,
    pub file_path: String,
    pub file_name: String,
    pub file_extension: String,
    pub scan_time: NaiveDateTime,
}

pub struct InodeInfoDAO {
    pub inode_info: InodeInfo,
    pub id: u64,
}

pub struct DatabaseManager {
    pool: Pool,
}

pub struct PoolDatabaseManager(pub Arc<DatabaseManager>);

impl Clone for PoolDatabaseManager {
    fn clone(&self) -> PoolDatabaseManager {
        PoolDatabaseManager(self.0.clone())
    }
}

impl PoolDatabaseManager {
    pub fn new(path: &str) -> Result<Self> {
        let mgr = Arc::new(DatabaseManager::new(path).unwrap());
        Ok(PoolDatabaseManager(mgr))
    }
}

impl DatabaseManager {
    pub fn new(path: &str) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::new(manager).unwrap();

        Ok(Self { pool })
    }

    pub fn create_tables(&self) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        // begin transaction
        let tx = conn.transaction()?;
        let sql = "CREATE TABLE IF NOT EXISTS inode_info (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            inode INTEGER NOT NULL,
            dev_id INTEGER NOT NULL,
            permissions INTEGER NOT NULL,
            nlink INTEGER NOT NULL,
            uid INTEGER NOT NULL,
            gid INTEGER NOT NULL,
            created DATETIME DEFAULT CURRENT_TIMESTAMP,
            modified DATETIME DEFAULT CURRENT_TIMESTAMP,
            md5 TEXT NOT NULL,
            size INTEGER NOT NULL,
            UNIQUE(dev_id,inode)
        );";
        let result = tx.execute(sql, [])?;
        println!("create table result: {}", result);
        if result > 0 {
            tx.execute(
                "CREATE INDEX idx_inode_dev_id ON inode_info (inode,dev_id);",
                [],
            )?;
            tx.execute("CREATE INDEX idx_md5 ON inode_info (md5);", [])?;
            tx.execute("CREATE INDEX idx_size ON inode_info (size);", [])?;
            tx.execute("CREATE INDEX idx_created ON inode_info (created);", [])?;
            tx.execute("CREATE INDEX idx_modified ON inode_info (modified);", [])?;
        }

        let sql = "CREATE TABLE IF NOT EXISTS file_info (
            inode_info_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_extension TEXT NULL,
            scan_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(file_path)
        );";
        let result = tx.execute(sql, [])?;
        if result > 0 {
            tx.execute("CREATE INDEX idx_file_name ON file_info (file_name);", [])?;
            tx.execute(
                "CREATE INDEX idx_file_extension ON file_info (file_extension);",
                [],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn drop_tables(&self) -> Result<()> {
        let conn = self.pool.get().unwrap();

        let sql = "DROP TABLE IF EXISTS inode_info";
        conn.execute(sql, [])?;
        let sql = "DROP TABLE IF EXISTS file_info";
        conn.execute(sql, [])?;
        Ok(())
    }

    pub fn insert_file_info(&self, file_info: &FileInfo) -> Result<()> {
        let conn = self.pool.get().unwrap();
        let sql = "INSERT INTO inode_info (inode, dev_id, permissions, nlink, uid, gid, created, modified, md5, size) 
          VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";
        conn.execute(
            sql,
            (
                file_info.inode_info.inode,
                file_info.inode_info.dev_id,
                file_info.inode_info.permissions,
                file_info.inode_info.nlink,
                file_info.inode_info.uid,
                file_info.inode_info.gid,
                &file_info.inode_info.created,
                &file_info.inode_info.modified,
                &file_info.inode_info.md5,
                file_info.inode_info.size,
            ),
        )?;
        let last_insert_id: i64 = conn.last_insert_rowid();
        let sql = "INSERT or UPDATE INTO file_info (inode_info_id, file_path, file_name, file_extension, scan_time) 
          VALUES (?1, ?2, ?3, ?4, ?5)";
        let mut result = match conn.execute(
            sql,
            (
                last_insert_id,
                &file_info.file_path,
                &file_info.file_name,
                &file_info.file_extension,
                &file_info.scan_time,
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
        let conn = self.pool.get().unwrap();
        let sql = "SELECT inode_info_id, file_path, file_name, file_extension, scan_time
        FROM file_info 
        WHERE file_path = ?";
        let mut stmt = conn.prepare(sql)?;
        let file_iter = stmt.query_row([file_path], |row| {
            Ok(FileInfoDAO {
                inode_info_id: row.get(0)?,
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                file_extension: row.get(3)?,
                scan_time: row.get(4)?,
            })
        });
        let file_info_dao = file_iter?;

        let sql =
            "SELECT inode, dev_id, permissions, nlink, uid, gid, created, modified, md5, size 
        from inode_info
        WHERE id = ? ";
        let mut stmt = conn.prepare(sql)?;
        let inode_info_dao = stmt.query_row([file_info_dao.inode_info_id], |row| {
            Ok(InodeInfo {
                inode: row.get(0)?,
                dev_id: row.get(1)?,
                permissions: row.get(2)?,
                nlink: row.get(3)?,
                uid: row.get(4)?,
                gid: row.get(5)?,
                created: row.get(6)?,
                modified: row.get(7)?,
                md5: row.get(8)?,
                size: row.get(9)?,
            })
        })?;

        let file_info = FileInfo {
            inode_info: InodeInfo {
                inode: inode_info_dao.inode,
                dev_id: inode_info_dao.dev_id,
                permissions: inode_info_dao.permissions,
                nlink: inode_info_dao.nlink,
                uid: inode_info_dao.uid,
                gid: inode_info_dao.gid,
                created: inode_info_dao.created,
                modified: inode_info_dao.modified,
                md5: inode_info_dao.md5,
                size: inode_info_dao.size,
            },
            file_name: file_info_dao.file_name,
            file_path: file_info_dao.file_path,
            file_extension: file_info_dao.file_extension,
            scan_time: file_info_dao.scan_time,
        };
        return Ok(file_info);
    }

    pub fn get_file_list_by_md5(&self, md5: &str) -> Result<Vec<FileInfo>> {
        let conn = self.pool.get().unwrap();
        let sql = "SELECT 
        a1.inode, a1.dev_id, a1.permissions, a1.nlink, a1.uid, a1.gid, a1.created, a1.modified, a1.md5, a1.size,
        a2.file_path, a2.file_name, a2.file_extension, a2.scan_time
        FROM inode_info as a1
        JOIN file_info as a2 ON a1.id= a2.inode_info_id
        WHERE a1.md5 = ?";
        let mut stmt = conn.prepare(sql)?;
        let inode_iter = stmt.query_map([md5], |row| {
            let inode_info = InodeInfo {
                inode: row.get(0)?,
                dev_id: row.get(1)?,
                permissions: row.get(2)?,
                nlink: row.get(3)?,
                uid: row.get(4)?,
                gid: row.get(5)?,
                created: row.get(6)?,
                modified: row.get(7)?,
                md5: row.get(8)?,
                size: row.get(9)?,
            };

            Ok(FileInfo {
                inode_info,
                file_name: row.get(10)?,
                file_path: row.get(11)?,
                file_extension: row.get(12)?,
                scan_time: row.get(13)?,
            })
        });
        let mut files = Vec::new();
        for item in inode_iter? {
            files.push(item?);
        }
        Ok(files)
    }

    pub fn remove_file_by_path(&self, file_path: &str) -> Result<()> {
        let conn = self.pool.get().unwrap();
        let sql = "DELETE FROM inode_info as a1 WHERE a1.id in (select inode_info_id from file_info where file_path = ?)";
        let mut stmt = conn.prepare(sql)?;
        stmt.execute([file_path])?;

        let sql = "DELETE FROM file_info WHERE file_path = ?";
        let mut stmt = conn.prepare(sql)?;
        stmt.execute([file_path])?;
        Ok(())
    }

    pub fn list_files(
        &self,
        page_no: i64,
        page_count: i64,
        min_file_size: i64,
        max_file_size: i64,
    ) -> Result<Vec<FileInfoWithMd5Count>> {
        let conn = self.pool.get().unwrap();
        let sql = "SELECT a1.inode, a1.dev_id, a1.permissions, a1.nlink, a1.uid, a1.gid, a1.created, a1.modified, a1.md5, a1.size,
        a2.file_path, a2.file_name, a2.file_extension, a2.scan_time, a3.md5_count
from inode_info as a1, 
file_info as a2, 
(SELECT  md5, count(md5) as md5_count
            FROM inode_info
            WHERE
             ? <= size and ? >=size
            group by md5) as a3
            where a1.id = a2.inode_info_id and a1.md5 = a3.md5 
            order by a3.md5_count desc, a1.size desc
 LIMIT ? OFFSET ?;";
        let mut stmt = conn.prepare(sql)?;
        let file_iter = stmt.query_map(
            [
                min_file_size,
                max_file_size,
                page_count,
                (page_no - 1) * page_count,
            ],
            |row| {
                let inode_info = InodeInfo {
                    inode: row.get(0)?,
                    dev_id: row.get(1)?,
                    permissions: row.get(2)?,
                    nlink: row.get(3)?,
                    uid: row.get(4)?,
                    gid: row.get(5)?,
                    created: row.get(6)?,
                    modified: row.get(7)?,
                    md5: row.get(8)?,
                    size: row.get(9)?,
                };
                let file_info = FileInfo {
                    inode_info,
                    file_name: row.get(10)?,
                    file_path: row.get(11)?,
                    file_extension: row.get(12)?,
                    scan_time: row.get(13)?,
                };
                Ok(FileInfoWithMd5Count {
                    file_info,
                    md5_count: row.get(14)?,
                })
            },
        );
        let mut files = Vec::new();
        for item in file_iter? {
            files.push(item?);
        }
        Ok(files)
    }
}
