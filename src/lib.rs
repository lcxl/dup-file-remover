pub mod controller;
pub mod database;
pub mod model;
pub mod utils;

use std::{env, ops::Deref};

use actix_server::Server;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    error,
    middleware::{from_fn, Logger},
    web::{self},
    App, HttpResponse, HttpServer,
};
use clap::Parser;
use log::{info, warn};

use controller::{
    files::{delete_file, list_files},
    login::{change_password, get_captcha, login_account, logout_account},
    scan::{query_scan_status, start_scan, stop_scan},
    settings::{query_settings, update_settings},
    user::{get_current_user, get_notices, reject_anonymous_users},
};
use database::sqlite::PoolDatabaseManager;
use model::{
    common::{ErrorCode, RestResponse},
    scan::SharedScanStatus,
    settings::{Args, Settings},
};
use tokio::sync::Mutex;
use utils::{error::DfrError, network::check_ipv6_available};
use utoipa_actix_web::AppExt;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable as _};
use utoipa_scalar::{Scalar, Servable as _};
use utoipa_swagger_ui::SwaggerUi;

pub struct SharedSettings(pub Mutex<Settings>);

impl SharedSettings {
    pub fn from(setting: Settings) -> Self {
        SharedSettings(Mutex::new(setting))
    }
}

impl Deref for SharedSettings {
    type Target = Mutex<Settings>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn run() -> Result<Server, DfrError> {
    let args = Args::parse();
    let settings = Settings::new(&args)?;

    env_logger::init_from_env(
        env_logger::Env::new().default_filter_or(settings.log_level.as_str()),
    );
    info!("Server args: {:?}", args);
    info!("Server settings: {:?}", settings);
    let mut file_path = env::current_exe()?;
    info!("Server file path: {:?}", file_path);
    file_path.pop();
    file_path.push("static");
    info!("Server static path: {:?}", file_path);

    let secret_key = Key::generate();

    let database_manager = PoolDatabaseManager::new(&settings.db_path)?;
    database_manager.create_tables()?;
    // create shared scan status for scan progress tracking
    let scan_status_data = web::Data::new(SharedScanStatus::new());
    let shared_settings = web::Data::new(SharedSettings::from(settings.clone()));
    //start the server
    let mut http_server = HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .map(|app| app.wrap(Logger::default()))
            .app_data(web::Data::new(database_manager.clone()))
            .app_data(shared_settings.clone())
            .app_data(scan_status_data.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(4096 * 1024 << 2)
                    .error_handler(|err, req| {
                        // <- create custom error response
                        warn!("progress request {} err: {}", req.path(), err);
                        let err_message = err.to_string();
                        return error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest()
                                .json(RestResponse::failed(ErrorCode::UNKNOWN_ERROR, err_message)),
                        )
                        .into();
                    }),
            ) // <- limit size of the payload (global configuration)
            // no need to login for these routes
            .service(login_account)
            .service(logout_account)
            .service(get_current_user)
            .service(get_notices)
            .service(get_captcha)
            .service(
                // need to login for these routes
                utoipa_actix_web::scope("/api/dfr")
                    .wrap(from_fn(reject_anonymous_users))
                    .service(start_scan)
                    .service(stop_scan)
                    .service(query_scan_status)
                    .service(list_files)
                    .service(delete_file)
                    .service(change_password)
                    .service(query_settings)
                    .service(update_settings),
            )
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api/openapi.json", api)
            })
            .openapi_service(|api| Redoc::with_url("/redoc", api))
            .openapi_service(|api| RapiDoc::with_url("/rapidoc", "/api/openapi.json", api))
            .openapi_service(|api| Scalar::with_url("/scalar", api))
            .into_app()
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_secure(false)
                    .build(),
            )
            .service(
                actix_files::Files::new("/", file_path.to_string_lossy().to_string().as_str())
                    .index_file("index.html"),
            )
    });

    if settings.enable_ipv6 && check_ipv6_available() {
        let addr = format!("{}:{}", settings.listen_addr_ipv6, settings.port);
        http_server = http_server.bind(addr.as_str())?;
        info!("Server started at http://{}", addr);
    } else {
        http_server = http_server.bind((settings.listen_addr_ipv4.as_str(), settings.port))?;
        info!(
            "Server started at http://{}:{}",
            settings.listen_addr_ipv4, settings.port
        );
    }

    Ok(http_server.run())
}
