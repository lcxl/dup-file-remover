pub mod controller;
pub mod database;
pub mod model;
pub mod utils;

use std::{env, path::PathBuf};

use actix_server::Server;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, error, middleware::Logger, web, App, HttpResponse, HttpServer};
use clap::Parser;
use config::{Config, ConfigError, Environment, File};
use log::{info, warn};

use controller::{
    list::list_files,
    login::{get_captcha, login_account, logout_account},
    scan::{start_scan, stop_scan},
    user::{get_current_user, get_notices},
};
use database::sqlite::PoolDatabaseManager;
use serde::Deserialize;
use utils::network::check_ipv6_available;
use utoipa_actix_web::AppExt;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable as _};
use utoipa_scalar::{Scalar, Servable as _};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "./config.toml")]
    config_file_path: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Enable IPv6 support
    pub enable_ipv6: bool,
    /// port number for the server to bind to
    pub port: u16,
    /// listen ipv4 address for the server to bind to
    pub listen_addr_ipv4: String,
    /// listen ipv6 address for the server to bind to
    pub listen_addr_ipv6: String,
    /// access logs are printed with the INFO level so ensure it is enabled by default
    pub log_level: String,
    /// login user name
    pub login_user_name: String,
    /// login password
    pub login_password: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_ipv6: true,
            port: 8081,
            listen_addr_ipv4: "0.0.0.0".to_string(),
            listen_addr_ipv6: "::".to_string(),
            log_level: "debug".to_string(),
            login_user_name: "admin".to_string(),
            login_password: "password".to_string(),
        }
    }
}

impl Settings {
    pub fn new(args: &Args) -> Result<Self, ConfigError> {
        let config_file_path = PathBuf::from(args.config_file_path.as_str());
        info!("Loading config file from: {}", config_file_path.display());
        // Load settings from config file
        Config::builder()
            .add_source(
                File::with_name(config_file_path.to_string_lossy().to_string().as_str())
                    .required(false),
            )
            .add_source(Environment::with_prefix("DFR"))
            .build()?
            .try_deserialize::<Settings>()
    }
}

pub fn run() -> Result<Server, Box<dyn std::error::Error>> {
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

    let database_manager = PoolDatabaseManager::new("dfremover.db").unwrap();
    database_manager.0.create_tables().unwrap();
    let _settings = settings.clone();
    //start the server
    let mut http_server = HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .map(|app| app.wrap(Logger::default()))
            .app_data(web::Data::new(database_manager.clone()))
            .app_data(web::Data::new(_settings.clone()))
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
            .service(login_account)
            .service(logout_account)
            .service(get_current_user)
            .service(get_notices)
            .service(get_captcha)
            .openapi_service(|api| {
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api/openapi.json", api)
            })
            .openapi_service(|api| Redoc::with_url("/redoc", api))
            .openapi_service(|api| RapiDoc::with_url("/rapidoc", "/api/openapi.json", api))
            .openapi_service(|api| Scalar::with_url("/scalar", api))
            .into_app()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
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
