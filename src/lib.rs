pub mod controller;
pub mod database;
pub mod model;
pub mod utils;

use actix_server::Server;
use actix_web::{error, middleware::Logger, web, App, HttpResponse, HttpServer};
use log::{info, warn, error};

use controller::scan::{start_scan, stop_scan};
use database::sqlite::PoolDatabaseManager;
use utils::network::check_ipv6_available;

pub fn run() ->  std::io::Result<Server> {
    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database_manager = PoolDatabaseManager::new("dfremover.db").unwrap();
    database_manager.0.create_tables();
    //start the server
    let mut http_server = HttpServer::new(move || {
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
            .service(web::resource("/api/scan/start").route(web::post().to(start_scan)))
            .service(web::resource("/api/scan/stop").route(web::post().to(stop_scan)))
        //.service(fs::Files::new("/", "./static").index_file("index.html"))
    });

    if check_ipv6_available() {
        http_server = http_server.bind(("[::]:8081"))?;
        info!("Server started at http://[::]:8081");
    } else {
        http_server = http_server.bind(("0.0.0.0", 8081))?;
        info!("Server started at http://0.0.0.0:8081");
    }
    

    Ok(http_server.run())
}
