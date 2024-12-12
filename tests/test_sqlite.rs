
use chrono::{NaiveDateTime, Utc};
use dup_file_remover::database::{file_info::FileInfo, sqlite::PoolDatabaseManager};

#[test]
fn test_create_sqlite() -> Result<(), Box<dyn std::error::Error>> {
    // Your code here to create an SQLite database and perform operations
    let database_manager = PoolDatabaseManager::new("dfremover.db")?;
    database_manager.0.create_tables()?;
    database_manager.0.drop_tables()?;
    let file_info = FileInfo::new("test.txt", Utc::now().naive_utc())?;
    database_manager.0.insert_file_info(&file_info)?;
    println!("file_name: {}", file_info.file_name);
    Ok(())
}