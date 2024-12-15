
use std::result;

use chrono::{NaiveDateTime, Utc};
use dup_file_remover::database::{file_info::FileInfo, sqlite::PoolDatabaseManager};

#[test]
fn test_create_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    // Your code here to create an SQLite database and perform operations
    let database_manager = PoolDatabaseManager::new("dfremover.db")?;
    database_manager.0.drop_tables()?;
    database_manager.0.create_tables()?;
    let file_info = FileInfo::new("dfremover.db", Utc::now().naive_utc())?;
    database_manager.0.insert_file_info(&file_info)?;
    println!("file_name: {}", file_info.file_name);
    //database_manager.0.drop_tables()?;
    Ok(())
}

#[test]
fn test_list_files() -> Result<(), Box<dyn std::error::Error>> {
    let database_manager = PoolDatabaseManager::new("dfremover.db")?;
    let result = database_manager.0.list_files(1, 100, 100, 1000);
    assert!(result.is_ok());
    let files = result.unwrap();
    for (index, file) in files.iter().enumerate() {
        println!("Index: {}, md5 count: {}, file info: {:?}", index, file.md5_count, file.file_info);
    }
    //database_manager.0.drop_tables()?;
    Ok(())
}