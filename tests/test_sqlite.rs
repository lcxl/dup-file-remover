use chrono::Local;
use dup_file_remover::{
    database::{file_info::FileInfo, sqlite::PoolDatabaseManager},
    model::settings::ListSettings,
    utils::error::DfrError,
};

#[test]
fn test_create_sqlite() -> Result<(), DfrError> {
    // Your code here to create an SQLite database and perform operations
    let database_manager = PoolDatabaseManager::new("dfremover.db")?;
    database_manager.drop_tables()?;
    database_manager.create_tables()?;
    let file_info = FileInfo::new("dfremover.db", 1, Local::now())?;
    database_manager.insert_file_info(&file_info)?;
    println!("file_name: {}", file_info.file_name);
    //database_manager.drop_tables()?;
    Ok(())
}

#[test]
fn test_list_files() -> Result<(), DfrError> {
    let database_manager = PoolDatabaseManager::new("dfremover.db")?;
    let query_list_params = ListSettings {
        page_no: 1,
        page_count: 100,
        min_file_size: Some(100),
        max_file_size: Some(1000),
        ..Default::default()
    };
    let result = database_manager.list_files(&query_list_params);
    assert!(result.is_ok());
    let files = result.unwrap();
    println!("Total file count: {}", files.total_count);
    for (index, file) in files.file_info_list.iter().enumerate() {
        println!(
            "Index: {}, md5 count: {}, file info: {:?}",
            index, file.md5_count, file.file_info
        );
    }

    //database_manager.drop_tables()?;
    Ok(())
}
