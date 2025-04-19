use std::{fs, ops::Deref, path::PathBuf, sync::Arc};

use chrono::{DateTime, Local};
use log::{debug, error, info};
use rusqlite::{params_from_iter, Connection, Params, Result, ToSql};

use crate::{
    database::file_info::TrashFileInfo,
    model::settings::{ListSettings, TrashListSettings},
    utils::{self, error::DfrError},
};

use super::file_info::{
    FileInfo, FileInfoList, FileInfoWithMd5Count, InodeInfo, TrashFileInfoList,
};
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
#[derive(Debug)]
pub struct InodeInfoDO {
    pub inode_info: InodeInfo,
    pub id: i64,
}

pub struct DatabaseManager {
    pool: Pool,
}

pub struct PoolDatabaseManager(pub Arc<DatabaseManager>);

impl PoolDatabaseManager {
    pub fn new(path: &str) -> Result<Self, DfrError> {
        let mgr = Arc::new(DatabaseManager::new(path)?);
        Ok(PoolDatabaseManager(mgr))
    }
}

impl Clone for PoolDatabaseManager {
    fn clone(&self) -> PoolDatabaseManager {
        PoolDatabaseManager(self.0.clone())
    }
}

impl Deref for PoolDatabaseManager {
    type Target = Arc<DatabaseManager>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DatabaseManager {
    pub fn new(path: &str) -> Result<Self, DfrError> {
        let path_buf = PathBuf::from(path);
        if let Some(parent_path) = path_buf.parent() {
            if !parent_path.exists() {
                info!("Creating database directory: {:?}", parent_path);
                fs::create_dir_all(parent_path)?;
            }
        }
        let manager = SqliteConnectionManager::file(path).with_init(|c| {
            // enable WAL mode for better performance and concurrency support
            c.pragma_update(None, "journal_mode", "WAL")?;
            // set busy timeout to 5 seconds to avoid locking issues during concurrent access
            c.pragma_update(None, "busy_timeout", "5000")?;
            // set synchronous mode to NORMAL for better performance
            c.pragma_update(None, "synchronous", "NORMAL")?;
            Ok(())
        });
        let pool = Pool::new(manager)?;

        Ok(Self { pool })
    }

    pub fn create_tables(&self) -> Result<(), DfrError> {
        let mut conn = self.pool.get()?;
        // begin transaction
        let tx = conn.transaction()?;
        // create inode_info table if it doesn't exist
        let sql = "
        CREATE TABLE IF NOT EXISTS inode_info (
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
        );
        CREATE INDEX IF NOT EXISTS idx_inode_dev_id ON inode_info (inode,dev_id);
        CREATE INDEX IF NOT EXISTS idx_md5 ON inode_info (md5);
        CREATE INDEX IF NOT EXISTS idx_size ON inode_info (size);
        CREATE INDEX IF NOT EXISTS idx_created ON inode_info (created);
        CREATE INDEX IF NOT EXISTS idx_modified ON inode_info (modified);

        CREATE TABLE IF NOT EXISTS file_info (
            inode_info_id INTEGER NOT NULL,
            dir_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_extension TEXT NULL,
            version INTEGER NOT NULL,
            scan_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(dir_path, file_name)
        );
        CREATE INDEX IF NOT EXISTS idx_file_name ON file_info (file_name);
        CREATE INDEX IF NOT EXISTS idx_file_extension ON file_info (file_extension);
        CREATE INDEX IF NOT EXISTS idx_version_dir_path ON file_info (version, dir_path);

        CREATE TABLE IF NOT EXISTS trash_info (
            dir_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            file_extension TEXT NULL,
            remove_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            permissions INTEGER NOT NULL,
            uid INTEGER NOT NULL,
            gid INTEGER NOT NULL,
            created DATETIME DEFAULT CURRENT_TIMESTAMP,
            modified DATETIME DEFAULT CURRENT_TIMESTAMP,
            md5 TEXT NOT NULL,
            size INTEGER NOT NULL,
            UNIQUE(dir_path, file_name)
        );
        CREATE INDEX IF NOT EXISTS idx_trash_info_file_name ON trash_info (file_name);
        CREATE INDEX IF NOT EXISTS idx_trash_info_md5 ON trash_info (md5);
        ";
        tx.execute_batch(sql)?;
        tx.commit()?;
        Ok(())
    }

    pub fn drop_tables(&self) -> Result<(), DfrError> {
        let conn = self.pool.get()?;
        let sql = "
        DROP TABLE IF EXISTS trash_info;
        DROP TABLE IF EXISTS inode_info;
        DROP TABLE IF EXISTS file_info;
        ";
        conn.execute_batch(sql)?;
        Ok(())
    }

    pub fn update_version(&self, file_info: &FileInfo) -> Result<usize, DfrError> {
        let conn = self.pool.get()?;
        let mut statement =
            conn.prepare("UPDATE file_info SET version = ? WHERE dir_path = ? AND file_name = ?")?;
        Ok(statement.execute((file_info.version, &file_info.dir_path, &file_info.file_name))?)
    }

    pub fn insert_file_info(&self, file_info: &FileInfo) -> Result<(), DfrError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let inode_info_do_result = self.query_inode_info_do_by_inode(
            &tx,
            file_info.inode_info.dev_id,
            file_info.inode_info.inode,
        );
        let last_insert_id = if inode_info_do_result.is_err() {
            let query_error = inode_info_do_result.err().unwrap();
            match query_error {
                rusqlite::Error::QueryReturnedNoRows => debug!(
                    "Inode {} (dev_id: {}) not found in database, try to insert, error: {:?}",
                    file_info.inode_info.inode, file_info.inode_info.dev_id, query_error
                ),
                _ => error!(
                    "Query inode {} (dev_id: {}) error, try to insert, error: {:?}",
                    file_info.inode_info.inode, file_info.inode_info.dev_id, query_error
                ),
            }

            let sql = "INSERT OR REPLACE INTO inode_info (inode, dev_id, permissions, nlink, uid, gid, created, modified, md5, size) 
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
            let db_inode_info = inode_info_do_result?;
            if db_inode_info.inode_info != file_info.inode_info {
                info!("Need to update file {} inode info, db inode info: {:?}, current file inode info: {:?}",file_info.file_path, db_inode_info , file_info.inode_info);
                let sql = "
                UPDATE inode_info 
                SET inode=?1, dev_id=?2, permissions=?3, nlink=?4, uid=?5, gid=?6, created=?7, modified=?8, md5=?9, size=?10
                WHERE id=?11
                ";
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
                        db_inode_info.id,
                    ),
                )?;
            }
            db_inode_info.id
        };

        let sql = "INSERT OR REPLACE INTO file_info (inode_info_id, dir_path, file_name, file_extension, scan_time, version) 
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
        result?;
        tx.commit()?;
        Ok(())
    }

    pub fn query_inode_info_do_by_inode(
        &self,
        conn: &Connection,
        dev_id: u64,
        node_id: u64,
    ) -> Result<InodeInfoDO> {
        let sql = "
        SELECT inode, dev_id, permissions, nlink, uid, gid, created, modified, md5, size, id
        FROM inode_info
        WHERE dev_id = ? AND inode = ?
        ";
        self.query_inode_info_do(&conn, sql, [dev_id, node_id])
    }

    fn get_inode_info_do_by_id(&self, conn: &Connection, id: u64) -> Result<InodeInfoDO> {
        let sql = "
        SELECT inode, dev_id, permissions, nlink, uid, gid, created, modified, md5, size, id
        FROM inode_info
        WHERE id = ?
        ";
        self.query_inode_info_do(&conn, sql, [id])
    }

    fn query_inode_info_do(
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

    fn query_file_info_do_by_path(
        &self,
        conn: &Connection,
        dir_path: &str,
        file_name: &str,
    ) -> Result<FileInfoDO> {
        let sql = "SELECT inode_info_id, dir_path, file_name, file_extension, scan_time, version
        FROM file_info 
        WHERE dir_path = ? and file_name = ?";
        let mut stmt = conn.prepare(sql)?;
        let result = stmt.query_row([dir_path, file_name], |row| {
            Ok(FileInfoDO {
                inode_info_id: row.get(0)?,
                dir_path: row.get(1)?,
                file_name: row.get(2)?,
                file_extension: row.get(3)?,
                scan_time: row.get(4)?,
                version: row.get(5)?,
            })
        });
        result
    }

    pub fn get_file_by_path(&self, dir_path: &str, file_name: &str) -> Result<FileInfo, DfrError> {
        let conn = self.pool.get()?;
        let file_info_do = self.query_file_info_do_by_path(&conn, dir_path, file_name)?;
        let inode_info_do = self.get_inode_info_do_by_id(&conn, file_info_do.inode_info_id)?;
        let file_info = FileInfo::from_do(inode_info_do.inode_info, file_info_do);
        return Ok(file_info);
    }

    pub fn get_file_list_by_md5(&self, md5: &str) -> Result<Vec<FileInfo>, DfrError> {
        let conn = self.pool.get()?;
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

            Ok(FileInfo::from_do(
                inode_info,
                FileInfoDO {
                    inode_info_id: 0,
                    dir_path: row.get(10)?,
                    file_name: row.get(11)?,
                    file_extension: row.get(12)?,
                    scan_time: row.get(13)?,
                    version: row.get(14)?,
                },
            ))
        });
        let mut files = Vec::new();
        for item in inode_iter? {
            files.push(item?);
        }
        Ok(files)
    }

    fn _remove_file_by_path(
        &self,
        conn: &Connection,
        dir_path: &str,
        file_name: &str,
    ) -> Result<()> {
        let file_info_do = self.query_file_info_do_by_path(conn, dir_path, file_name)?;

        let sql = "SELECT COUNT(*) FROM file_info where inode_info_id = ?";
        let file_count = conn.query_row(sql, [file_info_do.inode_info_id], |row| {
            let file_count: u64 = row.get(0)?;
            Ok(file_count)
        })?;

        // delete file info from db
        let sql = "DELETE FROM file_info WHERE dir_path = ? AND file_name = ?";
        conn.execute(sql, (dir_path, file_name))?;

        if file_count <= 1 {
            // delete inode info from db
            self.remove_inode_by_id(conn, file_info_do.inode_info_id)?;
        }
        Ok(())
    }

    /// Remove file by path
    pub fn remove_file_by_path(&self, dir_path: &str, file_name: &str) -> Result<(), DfrError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        self._remove_file_by_path(&tx, dir_path, file_name)?;
        tx.commit()?;
        Ok(())
    }

    /// remove not existed files from database based on dir_path and version
    pub fn remove_deleted_files(&self, dir_path: &str, version: u64) -> Result<(), DfrError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let sql = "DELETE FROM file_info WHERE dir_path = ? and version != ?";
        let update_rows = tx.execute(sql, (dir_path, version))?;
        tx.commit()?;
        if update_rows > 0 {
            info!(
                "deleted {} rows in file_info by dir_path '{}' and version '{}'",
                update_rows, dir_path, version
            );
        }
        Ok(())
    }

    /// remove not existed files from database based on version
    pub fn remove_deleted_files_by_version(&self, version: u64) -> Result<(), DfrError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let sql = "DELETE FROM file_info WHERE version != ?";
        let update_rows = tx.execute(sql, [version])?;
        tx.commit()?;
        if update_rows > 0 {
            info!(
                "deleted {} rows in file_info by version '{}'",
                update_rows, version
            );
        }
        Ok(())
    }

    /// remove not existed inodes from database
    pub fn remove_deleted_inodes(&self) -> Result<(), DfrError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        let sql = "
            DELETE FROM inode_info 
            WHERE id NOT IN (
                SELECT DISTINCT inode_info_id 
                FROM file_info)";
        let update_rows = tx.execute(sql, ())?;
        tx.commit()?;
        if update_rows > 0 {
            info!("deleted {} rows in inode_info", update_rows);
        }
        Ok(())
    }

    /// remove not existed inodes from database
    fn remove_inode_by_id(&self, conn: &Connection, id: u64) -> Result<usize> {
        let sql = "
            DELETE FROM inode_info 
            WHERE id =?";
        let update_rows = conn.execute(sql, [id])?;

        if update_rows > 0 {
            info!("deleted inode_info by id {}", id);
        }
        Ok(update_rows)
    }

    pub fn move_file_to_trash(&self, file_info: &FileInfo) -> Result<(), DfrError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;
        let md5 = file_info.inode_info.md5.clone().unwrap();
        self._remove_file_by_path(
            &tx,
            file_info.dir_path.as_str(),
            file_info.file_name.as_str(),
        )?;

        let sql = "
            INSERT OR REPLACE INTO trash_info (dir_path, file_name, file_extension, remove_time, permissions, uid, gid, created, modified, md5, size) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)";
        tx.execute(
            sql,
            (
                &file_info.dir_path,
                &file_info.file_name,
                &file_info.file_extension,
                Local::now(),
                file_info.inode_info.permissions,
                file_info.inode_info.uid,
                file_info.inode_info.gid,
                &file_info.inode_info.created,
                &file_info.inode_info.modified,
                &md5,
                file_info.inode_info.size,
            ),
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn list_files(&self, query_list_params: &ListSettings) -> Result<FileInfoList, DfrError> {
        let mut conn = self.pool.get()?;
        let mut params: Vec<Arc<dyn ToSql>> = Vec::new();
        let mut sub_query_sql = String::from(
            "SELECT md5, COUNT(md5) AS md5_count
            FROM inode_info 
            WHERE 1=1",
        );
        if let Some(min_file_size) = query_list_params.min_file_size {
            params.push(Arc::new(min_file_size));
            sub_query_sql += " AND size >= ?";
        }
        if let Some(max_file_size) = query_list_params.max_file_size {
            params.push(Arc::new(max_file_size));
            sub_query_sql += " AND size < ?";
        }
        sub_query_sql += " GROUP BY md5";
        let mut filter_sub_query_sql = String::new();
        let mut filter_select_params = String::new();
        let mut filter_join_sql = String::new();
        let mut has_filter_md5_count = false;
        if let Some(filter_dup_file_in_dir_path) =
            query_list_params.filter_dup_file_by_dir_path.clone()
        {
            if filter_dup_file_in_dir_path {
                if query_list_params.dir_path.is_none() {
                    return Err(utils::error::DfrError::IoError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Dir path can not be empty while filter_dup_file_by_dir_path is true",
                    )));
                }
                let dir_path = query_list_params.dir_path.clone().unwrap();
                let mut sub_query_sql = String::from(
                    "SELECT b1.md5, COUNT(b1.md5) AS md5_count
                    FROM inode_info AS b1,
                        file_info AS b2
                    WHERE b1.id = b2.inode_info_id",
                );
                if let Some(min_file_size) = query_list_params.min_file_size {
                    params.push(Arc::new(min_file_size));
                    sub_query_sql += " AND b1.size >= ?";
                }
                if let Some(max_file_size) = query_list_params.max_file_size {
                    params.push(Arc::new(max_file_size));
                    sub_query_sql += " AND b1.size < ?";
                }

                params.push(Arc::new(format!("%{}%", dir_path)));
                sub_query_sql += " AND b2.dir_path LIKE ?";

                sub_query_sql += " GROUP BY b1.md5";
                filter_sub_query_sql = format!(", ({}) AS a4", sub_query_sql);
                filter_select_params = String::from(", a4.md5_count AS filter_md5_count");
                filter_join_sql = String::from("AND a4.md5 = a3.md5");
                has_filter_md5_count = true;
            }
        }

        let mut query_sql = format!(
            " FROM inode_info AS a1, 
                file_info AS a2, 
                ({}) AS a3
                {}
            WHERE 
                a1.id = a2.inode_info_id 
                AND a1.md5 = a3.md5
                {}",
            sub_query_sql, filter_sub_query_sql, filter_join_sql
        );
        if let Some(dir_path) = query_list_params.dir_path.clone() {
            query_sql += " AND a2.dir_path LIKE ?";
            params.push(Arc::new(format!("%{}%", dir_path)));
        }
        if let Some(file_name) = query_list_params.file_name.clone() {
            query_sql += " AND a2.file_name LIKE ?";
            params.push(Arc::new(format!("%{}%", file_name)));
        }
        if let Some(file_extension) = query_list_params.file_extension.clone() {
            query_sql += " AND a2.file_extension LIKE ?";
            params.push(Arc::new(format!("%{}%", file_extension)));
        }
        if let Some(file_extension_list) = query_list_params.file_extension_list.clone() {
            query_sql += " AND a2.file_extension IN (";
            let file_extensions: Vec<&str> = file_extension_list.split(',').collect();
            let mut file_extension_params = Vec::new();
            for file_extension in file_extensions.iter() {
                file_extension_params.push("?");
                params.push(Arc::new(String::from(*file_extension)));
            }
            query_sql += (file_extension_params.join(",") + ")").as_str();
        }
        if let Some(md5) = query_list_params.md5.clone() {
            query_sql += " AND a1.md5 = ?";
            params.push(Arc::new(md5));
        }
        if let Some(start_created_time) = query_list_params.start_created_time.clone() {
            query_sql += " AND a1.created >= ?";
            params.push(Arc::new(start_created_time));
        }
        if let Some(end_created_time) = query_list_params.end_created_time.clone() {
            query_sql += " AND a1.created <= ?";
            params.push(Arc::new(end_created_time));
        }

        if let Some(start_modified_time) = query_list_params.start_modified_time.clone() {
            query_sql += " AND a1.modified >= ?";
            params.push(Arc::new(start_modified_time));
            info!("start_modified_time: {:?}", start_modified_time);
        }
        if let Some(end_modified_time) = query_list_params.end_modified_time.clone() {
            query_sql += " AND a1.modified <= ?";
            params.push(Arc::new(end_modified_time));
            info!("end_modified_time: {:?}", end_modified_time);
        }
        if let Some(min_md5_count) = query_list_params.min_md5_count.clone() {
            query_sql += " AND a3.md5_count >= ?";
            params.push(Arc::new(min_md5_count));
        }

        if let Some(max_md5_count) = query_list_params.max_md5_count.clone() {
            query_sql += " AND a3.md5_count < ?";
            params.push(Arc::new(max_md5_count));
        }

        let count_sql = String::from("SELECT COUNT(*)") + &query_sql;
        let count_params = params.to_vec();
        info!("list file query count sql: {}", count_sql);

        let mut sql = String::from("SELECT a1.inode, a1.dev_id, a1.permissions, a1.nlink, a1.uid, a1.gid, a1.created, a1.modified, a1.md5, a1.size,
            a2.dir_path, a2.file_name, a2.file_extension, a2.scan_time, a2.version, a3.md5_count") +&filter_select_params+ &query_sql;

        // order by
        let mut order_by_list: Vec<String> = Vec::new();
        let mut order_asc = false;
        if let Some(_order_asc) = query_list_params.order_asc {
            order_asc = _order_asc;
        }
        if let Some(order_by) = query_list_params.order_by.clone() {
            if order_by == "size" {
                order_by_list.push(String::from(format!(
                    "a1.size {}",
                    if order_asc { "ASC" } else { "DESC" }
                )));
            }
        }
        //order by md5_count default
        order_by_list.push(String::from("a3.md5_count DESC"));
        sql += format!(" order by {}", order_by_list.join(",")).as_str();

        //  add limit
        sql += " LIMIT ? OFFSET ?;";
        params.push(Arc::new(query_list_params.page_count));
        params.push(Arc::new(
            (query_list_params.page_no - 1) * query_list_params.page_count,
        ));

        info!("List file query sql: {}", sql);

        let trans = conn.transaction()?;
        let mut stmt = trans.prepare(&count_sql)?;

        let count_result = stmt.query_map(params_from_iter(count_params.iter()), |row| {
            let count: u64 = row.get(0)?;
            Ok(count)
        });
        // get total count
        // FIXME thread 'actix-rt|system:0|arbiter:2' panicked at src/database/sqlite.rs:676:55:
        // called `Result::unwrap()` on an `Err` value: SqliteFailure(Error { code: DatabaseCorrupt, extended_code: 11 }, Some("database disk image is malformed"))
        let total_count = count_result?.next().unwrap()?;

        let mut stmt = trans.prepare(&sql)?;
        let file_iter = stmt.query_map(params_from_iter(params.iter()), |row| {
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
            let file_info = FileInfo::from_do(
                inode_info,
                FileInfoDO {
                    inode_info_id: 0,
                    dir_path: row.get(10)?,
                    file_name: row.get(11)?,
                    file_extension: row.get(12)?,
                    scan_time: row.get(13)?,
                    version: row.get(14)?,
                },
            );
            let filter_md5_count;
            if has_filter_md5_count {
                filter_md5_count = Some(row.get(16)?);
            } else {
                filter_md5_count = None;
            }
            Ok(FileInfoWithMd5Count {
                file_info,
                md5_count: row.get(15)?,
                filter_md5_count,
            })
        });
        let mut files = Vec::new();
        for item in file_iter? {
            files.push(item?);
        }
        Ok(FileInfoList {
            file_info_list: files,
            total_count,
        })
    }

    pub fn list_trash_files(
        &self,
        query_list_params: &TrashListSettings,
    ) -> Result<TrashFileInfoList, DfrError> {
        let mut conn = self.pool.get()?;
        let mut params: Vec<Arc<dyn ToSql>> = Vec::new();
        let mut query_sql = String::from(" FROM trash_info WHERE 1=1");
        if let Some(min_file_size) = query_list_params.min_file_size {
            params.push(Arc::new(min_file_size));
            query_sql += " AND size >= ?";
        }
        if let Some(max_file_size) = query_list_params.max_file_size {
            params.push(Arc::new(max_file_size));
            query_sql += " AND size < ?";
        }

        if let Some(dir_path) = query_list_params.dir_path.clone() {
            query_sql += " AND dir_path LIKE ?";
            params.push(Arc::new(format!("%{}%", dir_path)));
        }
        if let Some(file_name) = query_list_params.file_name.clone() {
            query_sql += " AND file_name LIKE ?";
            params.push(Arc::new(format!("%{}%", file_name)));
        }
        if let Some(file_extension) = query_list_params.file_extension.clone() {
            query_sql += " AND file_extension LIKE ?";
            params.push(Arc::new(format!("%{}%", file_extension)));
        }
        if let Some(file_extension_list) = query_list_params.file_extension_list.clone() {
            query_sql += " AND file_extension IN (";
            let file_extensions: Vec<&str> = file_extension_list.split(',').collect();
            let mut file_extension_params = Vec::new();
            for file_extension in file_extensions.iter() {
                file_extension_params.push("?");
                params.push(Arc::new(String::from(*file_extension)));
            }
            query_sql += (file_extension_params.join(",") + ")").as_str();
        }
        if let Some(md5) = query_list_params.md5.clone() {
            query_sql += " AND md5 = ?";
            params.push(Arc::new(md5));
        }
        if let Some(start_created_time) = query_list_params.start_created_time.clone() {
            query_sql += " AND created >= ?";
            params.push(Arc::new(start_created_time));
        }
        if let Some(end_created_time) = query_list_params.end_created_time.clone() {
            query_sql += " AND created <= ?";
            params.push(Arc::new(end_created_time));
        }

        if let Some(start_modified_time) = query_list_params.start_modified_time.clone() {
            query_sql += " AND modified >= ?";
            params.push(Arc::new(start_modified_time));
            info!("start_modified_time: {:?}", start_modified_time);
        }
        if let Some(end_modified_time) = query_list_params.end_modified_time.clone() {
            query_sql += " AND modified <= ?";
            params.push(Arc::new(end_modified_time));
            info!("end_modified_time: {:?}", end_modified_time);
        }

        if let Some(start_removed_time) = query_list_params.start_removed_time.clone() {
            query_sql += " AND remove_time >= ?";
            params.push(Arc::new(start_removed_time));
            info!("start_removed_time: {:?}", start_removed_time);
        }
        if let Some(end_removed_time) = query_list_params.end_removed_time.clone() {
            query_sql += " AND remove_time <= ?";
            params.push(Arc::new(end_removed_time));
            info!("end_removed_time: {:?}", end_removed_time);
        }

        let count_sql = String::from("SELECT COUNT(*)") + &query_sql;
        let count_params = params.to_vec();
        info!("list file query count sql: {}", count_sql);

        let mut sql = String::from("SELECT dir_path, file_name, file_extension, remove_time, permissions, uid, gid, created, modified, md5, size") + &query_sql;

        // order by
        let mut order_by_list: Vec<String> = Vec::new();
        let mut order_asc = false;
        if let Some(_order_asc) = query_list_params.order_asc {
            order_asc = _order_asc;
        }
        if let Some(order_by) = query_list_params.order_by.clone() {
            if order_by == "size" {
                order_by_list.push(String::from(format!(
                    "a1.size {}",
                    if order_asc { "ASC" } else { "DESC" }
                )));
            }
        }
        if !order_by_list.is_empty() {
            sql += format!(" ORDER BY {}", order_by_list.join(",")).as_str();
        }
        //  add limit
        sql += " LIMIT ? OFFSET ?;";
        params.push(Arc::new(query_list_params.page_count));
        params.push(Arc::new(
            (query_list_params.page_no - 1) * query_list_params.page_count,
        ));

        info!("List trash file query sql: {}", sql);

        let trans = conn.transaction()?;
        let mut stmt = trans.prepare(&count_sql)?;

        let count_iter = stmt.query_map(params_from_iter(count_params.iter()), |row| {
            let count: u64 = row.get(0)?;
            Ok(count)
        });
        // get total count

        let total_count = count_iter?.next().unwrap()?;

        let mut stmt = trans.prepare(&sql)?;
        let file_iter = stmt.query_map(params_from_iter(params.iter()), |row| {
            // dir_path, file_name, file_extension, remove_time, permissions, uid, gid, created, modified, md5, size
            let trash_file_info = TrashFileInfo {
                dir_path: row.get(0)?,
                file_name: row.get(1)?,
                file_extension: row.get(2)?,
                remove_time: row.get(3)?,
                permissions: row.get(4)?,
                uid: row.get(5)?,
                gid: row.get(6)?,
                created: row.get(7)?,
                modified: row.get(8)?,
                md5: row.get(9)?,
                size: row.get(10)?,
            };

            Ok(trash_file_info)
        });
        let mut files = Vec::new();
        for item in file_iter? {
            files.push(item?);
        }
        Ok(TrashFileInfoList {
            trash_file_info_list: files,
            total_count,
        })
    }

    pub fn get_trash_file_by_path(
        &self,
        dir_path: &str,
        file_name: &str,
    ) -> Result<TrashFileInfo, DfrError> {
        let conn = self.pool.get()?;
        let sql = "
            SELECT dir_path, file_name, file_extension, remove_time, permissions, uid, gid, created, modified, md5, size
            FROM trash_info 
            WHERE dir_path = ? AND file_name = ?";
        let mut stmt = conn.prepare(sql)?;
        let trash_file_info = stmt.query_row([dir_path, file_name], |row| {
            Ok(TrashFileInfo {
                dir_path: row.get(0)?,
                file_name: row.get(1)?,
                file_extension: row.get(2)?,
                remove_time: row.get(3)?,
                permissions: row.get(4)?,
                uid: row.get(5)?,
                gid: row.get(6)?,
                created: row.get(7)?,
                modified: row.get(8)?,
                md5: row.get(9)?,
                size: row.get(10)?,
            })
        });
        Ok(trash_file_info?)
    }

    pub fn remove_trash_file_by_path(
        &self,
        dir_path: &str,
        file_name: &str,
    ) -> Result<usize, DfrError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        let sql = "
            DELETE FROM trash_info 
            WHERE dir_path = ? AND file_name = ?";
        let usize = tx.execute(sql, (dir_path, file_name))?;
        tx.commit()?;
        Ok(usize)
    }

    pub fn remove_trash_file_by_md5(&self, md5: &str) -> Result<usize, DfrError> {
        let mut conn = self.pool.get()?;
        let tx = conn.transaction()?;

        let sql = "
            DELETE FROM trash_info 
            WHERE md5 = ?";
        let usize = tx.execute(sql, [md5])?;
        tx.commit()?;
        Ok(usize)
    }

    pub fn restore_trash_file_by_path(
        &self,
        trash_file_info: &TrashFileInfo,
    ) -> Result<(), DfrError> {
        let mut file_info = FileInfo::new(
            trash_file_info.get_file_path().clone().as_str(),
            0,
            Local::now(),
        )?;
        // set file md5 manually
        file_info.inode_info.md5 = Some(trash_file_info.md5.clone());
        // Restore the file to its original location
        self.insert_file_info(&file_info)?;
        self.remove_trash_file_by_path(
            trash_file_info.dir_path.as_str(),
            trash_file_info.file_name.as_str(),
        )?;
        Ok(())
    }
}
