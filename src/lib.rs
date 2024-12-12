pub mod controller;
pub mod database;
pub mod model;

use actix_files as fs;

use actix_web::{error, middleware::Logger, web, App, HttpResponse, HttpServer};
use actix_server::Server;
use log::warn;

use controller::scan::scan_files;
use database::sqlite::{Pool, PoolDatabaseManager};
use r2d2_sqlite::SqliteConnectionManager;


pub  fn run() -> Server {
    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_manager = PoolDatabaseManager::new("dfremover.db").unwrap();
    //start the server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(database_manager.clone()))
            .app_data(
                web::JsonConfig::default()
                    .limit(4096 * 1024 << 2)
                    .error_handler(|err, req| {
                        // <- create custom error response
                        warn!("progress request {} err: {}", req.path(), err);
                        return error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().finish(),
                        )
                        .into();
                    }),
            ) // <- limit size of the payload (global configuration)
            .service(web::resource("/scan/start").route(web::post().to(scan_files)))
        //.service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8081)).unwrap()
    //.bind(("[::]", 8080))?
    .run()
    
}