pub mod controller;
pub mod database;
pub mod model;
pub mod utils;

use std::{env, fs, ops::Deref, path::PathBuf};

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
use config::{Config, ConfigError, Environment, File};
use log::{info, warn};

use controller::{
    files::{delete_file, list_files},
    login::{get_captcha, login_account, logout_account},
    scan::{query_scan_status, start_scan, stop_scan},
    user::{get_current_user, get_notices, reject_anonymous_users},
};
use database::sqlite::PoolDatabaseManager;
use model::{
    common::{ErrorCode, RestResponse},
    scan::SharedScanStatus,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use toml_edit::DocumentMut;
use utils::{error::DfrError, network::check_ipv6_available};
use utoipa_actix_web::AppExt;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable as _};
use utoipa_scalar::{Scalar, Servable as _};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "conf/config")]
    config_file_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    /// Path to the configuration file. If not specified, a new one will be created in the "conf" directory.
    pub config_file_path: String,
    /// Path to the database file. If not specified, a new one will be created in the "conf" directory.
    pub db_path: String,
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
    /// default scan path for the server to start with
    pub default_scan_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            config_file_path: "conf/config".to_string(),
            db_path: "conf/dfremover.db".to_string(),
            enable_ipv6: true,
            port: 8081,
            listen_addr_ipv4: "0.0.0.0".to_string(),
            listen_addr_ipv6: "::".to_string(),
            log_level: "info".to_string(),
            login_user_name: "admin".to_string(),
            login_password: "password".to_string(),
            default_scan_path: "data/".to_string(),
        }
    }
}

impl Settings {
    pub fn new(args: &Args) -> Result<Self, DfrError> {
        //let config_file_path = PathBuf::from(args.config_file_path.as_str());
        //let config_file_path = config_file_path.canonicalize()?;
        info!("Loading config file from: {}", args.config_file_path.as_str());
        // Load settings from config file
        let config = Config::builder()
            .add_source(
                File::with_name(args.config_file_path.as_str())
                    .required(false),
            )
            .add_source(Environment::with_prefix("DFR"))
            .build()?;
        let mut settings = config.try_deserialize::<Settings>()?;
        settings.config_file_path = args.config_file_path.clone();
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), DfrError> {
        let mut config_file_path = PathBuf::from(self.config_file_path.as_str());
        config_file_path.set_extension("toml");
        // Save settings to config file
        let config_context = fs::read_to_string(&config_file_path)?;
        let mut toml_doc = config_context.parse::<DocumentMut>()?;
        let default_settings = Settings::default();
        if self.db_path != default_settings.db_path {
            toml_doc["db_path"] = toml_edit::value(self.db_path.clone());
        }
        if self.enable_ipv6 != default_settings.enable_ipv6 {
            toml_doc["enable_ipv6"] = toml_edit::value(self.enable_ipv6);
        }
        if self.listen_addr_ipv4 != default_settings.listen_addr_ipv4 {
            toml_doc["listen_addr_ipv4"] = toml_edit::value(self.listen_addr_ipv4.clone());
        }
        if self.listen_addr_ipv6 != default_settings.listen_addr_ipv6 {
            toml_doc["listen_addr_ipv6"] = toml_edit::value(self.listen_addr_ipv6.clone());
        }
        if self.port != default_settings.port {
            toml_doc["port"] = toml_edit::value(self.port as i64);
        }
        if self.log_level != default_settings.log_level {
            toml_doc["log_level"] = toml_edit::value(self.log_level.clone());
        }
        if self.login_user_name != default_settings.login_user_name {
            toml_doc["login_user_name"] = toml_edit::value(self.login_user_name.clone());
        }
        if self.login_password != default_settings.login_password {
            toml_doc["login_password"] = toml_edit::value(self.login_password.clone());
        }
        if self.default_scan_path != default_settings.default_scan_path {
            toml_doc["default_scan_path"] = toml_edit::value(self.default_scan_path.clone());
        }
        let toml_str = toml_doc.to_string();
        info!(
            "Saving config to: {}, content: {}",
            config_file_path.display(),
            toml_str
        );
        fs::write(&config_file_path, toml_str)?;
        Ok(())
    }
}

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
                    .service(delete_file),
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
