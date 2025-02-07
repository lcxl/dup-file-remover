pub mod controller;
pub mod database;
pub mod model;
pub mod utils;

use std::env;

use actix_server::Server;
use actix_web::{error, middleware::Logger, web, App, HttpResponse, HttpServer};
use log::{info, warn};

use controller::{
    list::list_files,
    scan::{start_scan, stop_scan},
};
use database::sqlite::PoolDatabaseManager;
use utils::network::check_ipv6_available;
use utoipa_actix_web::AppExt;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable as _};
use utoipa_scalar::{Scalar, Servable as _};
use utoipa_swagger_ui::SwaggerUi;

pub fn run() -> std::io::Result<Server> {
    // access logs are printed with the INFO level so ensure it is enabled by default
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    //
    let mut file_path = env::current_exe()?;
    info!("Server file path: {:?}", file_path);
    file_path.pop();
    file_path.push("static");
    info!("Server static path: {:?}", file_path);
    
    let database_manager = PoolDatabaseManager::new("dfremover.db").unwrap();
    database_manager.0.create_tables().unwrap();
    //start the server
    let mut http_server = HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .map(|app| app.wrap(Logger::default()))
            //.wrap(Logger::default())
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
            .service(start_scan)
            .service(stop_scan)
            .service(list_files)
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api/openapi.json", api)
            })
            .openapi_service(|api| Redoc::with_url("/redoc", api))
            .openapi_service(|api| RapiDoc::with_url("/rapidoc", "/api/openapi.json", api))
            .openapi_service(|api| Scalar::with_url("/scalar", api))
            .into_app()
        .service(actix_files::Files::new("/", file_path.to_string_lossy().to_string().as_str()).index_file("index.html"))
    });

    if check_ipv6_available() {
        http_server = http_server.bind("[::]:8081")?;
        info!("Server started at http://[::]:8081");
    } else {
        http_server = http_server.bind(("0.0.0.0", 8081))?;
        info!("Server started at http://0.0.0.0:8081");
    }

    Ok(http_server.run())
}
