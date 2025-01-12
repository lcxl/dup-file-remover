use std::sync::Arc;

use chrono::{DateTime, Local};
use log::{error, info};
use rusqlite::{Connection, Params, Result};

use super::file_info::{FileInfo, FileInfoWithMd5Count, InodeInfo};
use r2d2_sqlite::SqliteConnectionManager;
pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

pub struct FileInfoDO {
    pub inode_info_id: u64,
    pub dir_path: String,
    pub file_name: String,
    pub file_extension: Option<String>,
    pub scan_time: DateTime<Local>,
    pub version: u64,
}

pub struct InodeInfoDO {
    pub inode_info: InodeInfo,
    pub id: i64,
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
        info!("create table result: {}", result);
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
            dir_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_extension TEXT NULL,
            version INTEGER NOT NULL,
            scan_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(dir_path, file_name)
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

    pub fn update_version(&self, file_info: &FileInfo) -> Result<usize> {
        let conn = self.pool.get().unwrap();
        let mut statement = conn.prepare("UPDATE file_info SET version = ? WHERE dir_path = ? and file_name = ?")?;
        statement.execute((file_info.version, &file_info.dir_path, &file_info.file_name))
    }

    pub fn insert_file_info(&self, file_info: &FileInfo) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        let tx = conn.transaction()?;
        let node_info_do_result = self.query_node_info_do_by_inode(
            &tx,
            file_info.inode_info.dev_id,
            file_info.inode_info.inode,
        );
        let last_insert_id = if node_info_do_result.is_err() {
            let query_error = node_info_do_result.err().unwrap();
            match query_error {
                rusqlite::Error::QueryReturnedNoRows => info!(
                    "Inode {} (dev_id: {}) not found in database, try to insert, error: {:?}",
                    file_info.inode_info.inode, file_info.inode_info.dev_id, query_error
                ),
                _ => error!(
                    "Query inode {} (dev_id: {}) error, try to insert, error: {:?}",
                    file_info.inode_info.inode, file_info.inode_info.dev_id, query_error
                ),
            }

            let sql = "INSERT or replace INTO inode_info (inode, dev_id, permissions, nlink, uid, gid, created, modified, md5, size) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";
            tx.execute(
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
            tx.last_insert_rowid()
        } else {
            node_info_do_result?.id
        };

        let sql = "insert or replace into file_info (inode_info_id, dir_path, file_name, file_extension, scan_time, version) 
          VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
        let result = match tx.execute(
            sql,
            (
                last_insert_id,
                &file_info.dir_path,
                &file_info.file_name,
                &file_info.file_extension,
                &file_info.scan_time,
                &file_info.version,
            ),
        ) {
            Ok(_) => Ok(()),
            Err(_e) => {
                error!("Error inserting file info: {}", _e);
                Err(_e)
            }
        };
        tx.commit()?;
        return result;
    }

    pub fn query_node_info_do_by_inode(
        &self,
        conn: &Connection,
        dev_id: u64,
        node_id: u64,
    ) -> Result<InodeInfoDO> {
        let sql =
            "SELECT inode, dev_id, permissions, nlink, uid, gid, created, modified, md5, size, id
        from inode_info
        WHERE dev_id = ? and inode = ?";
        self.query_node_info_do(&conn, sql, [dev_id, node_id])
    }

    pub fn get_node_info_do_by_id(&self, conn: &Connection, id: u64) -> Result<InodeInfoDO> {
        let sql =
            "SELECT inode, dev_id, permissions, nlink, uid, gid, created, modified, md5, size, id
        from inode_info
        WHERE id = ?";
        self.query_node_info_do(&conn, sql, [id])
    }

    fn query_node_info_do(
        &self,
        conn: &Connection,
        sql: &str,
        params: impl Params,
    ) -> Result<InodeInfoDO> {
        let mut stmt = conn.prepare(sql)?;
        stmt.query_row(params, |row| {
            Ok(InodeInfoDO {
                inode_info: InodeInfo {
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
                },
                id: row.get(10)?,
            })
        })
    }

    pub fn get_file_by_path(&self, dir_path: &str, file_name: &str) -> Result<FileInfo> {
        let conn = self.pool.get().unwrap();
        let sql = "SELECT inode_info_id, dir_path, file_name, file_extension, scan_time, version
        FROM file_info 
        WHERE dir_path = ? and file_name = ?";
        let mut stmt = conn.prepare(sql)?;
        let file_iter = stmt.query_row([dir_path, file_name], |row| {
            Ok(FileInfoDO {
                inode_info_id: row.get(0)?,
                dir_path: row.get(1)?,
                file_name: row.get(2)?,
                file_extension: row.get(3)?,
                scan_time: row.get(4)?,
                version: row.get(5)?,
            })
        });
        if file_iter.is_err() {
            return Err(file_iter.err().unwrap());
        }
        let file_info_do = file_iter?;

        let inode_info_result = self.get_node_info_do_by_id(&conn, file_info_do.inode_info_id);

        if inode_info_result.is_err() {
            return Err(inode_info_result.err().unwrap());
        }

        let inode_info_do = inode_info_result?;
        let file_info = FileInfo::from_do(inode_info_do.inode_info, file_info_do);
        return Ok(file_info);
    }

    pub fn get_file_list_by_md5(&self, md5: &str) -> Result<Vec<FileInfo>> {
        let conn = self.pool.get().unwrap();
        let sql = "SELECT 
        a1.inode, a1.dev_id, a1.permissions, a1.nlink, a1.uid, a1.gid, a1.created, a1.modified, a1.md5, a1.size,
        a2.dir_path, a2.file_name, a2.file_extension, a2.scan_time, a2.version
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

            Ok(FileInfo::from_do(inode_info, FileInfoDO {
                inode_info_id:0,
                dir_path: row.get(10)?,
                file_name: row.get(11)?,
                file_extension: row.get(12)?,
                scan_time: row.get(13)?,
                version: row.get(14)?,
            }))
        });
        let mut files = Vec::new();
        for item in inode_iter? {
            files.push(item?);
        }
        Ok(files)
    }

    pub fn remove_file_by_path(&self, dir_path: &str, file_name: &str) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        let tx = conn.transaction()?;
      
        let sql = "DELETE FROM file_info WHERE dir_path = ? and file_name = ?";
        tx.execute(sql, (dir_path, file_name))?;
        tx.commit()?;
        Ok(())
    }

    /// remove not existed files from database based on dir_path and version
    pub fn remove_deleted_files(&self, dir_path: &str, version: u64) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        let tx = conn.transaction()?;
        let sql = "DELETE FROM file_info WHERE dir_path = ? and version != ?";
        let update_rows = tx.execute(sql, (dir_path, version))?;
        tx.commit()?;
        if update_rows > 0 {
            info!("deleted {} rows in file_info by dir_path '{}' and version '{}'", update_rows, dir_path, version);
        }
        Ok(())
    }

    /// remove not existed inodes from database
    pub fn remove_deleted_inode(&self)->Result<()> {
        let mut conn = self.pool.get().unwrap();
        let tx = conn.transaction()?;

        let sql = "DELETE FROM inode_info WHERE id not in (select DISTINCT inode_info_id from file_info)";
        let update_rows = tx.execute(sql, ())?;
        tx.commit()?;
        if update_rows > 0 {
            info!("deleted {} rows in inode_info", update_rows);
        }
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
        a2.dir_path, a2.file_name, a2.file_extension, a2.scan_time, a2.version, a3.md5_count
from inode_info as a1, 
file_info as a2, 
(SELECT  md5, count(md5) as md5_count
            FROM inode_info
            WHERE size >= ? and size < ?
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
                let file_info = FileInfo::from_do(inode_info, FileInfoDO{
                    inode_info_id:0,
                    dir_path: row.get(10)?,
                    file_name: row.get(11)?,
                    file_extension: row.get(12)?,
                    scan_time: row.get(13)?,
                    version: row.get(14)?,
                });
                Ok(FileInfoWithMd5Count {
                    file_info,
                    md5_count: row.get(15)?,
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
