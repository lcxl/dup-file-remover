use std::{fs, path::PathBuf};

use ::serde::{Deserialize, Serialize};
use clap::Parser;
use config::{Config, Environment, File};
use log::info;
use toml_edit::DocumentMut;
use utoipa::ToSchema;

use crate::utils::error::DfrError;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "conf/config")]
    config_file_path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
pub struct SettingsModel {
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
    /// default scan path for the server to start with
    pub default_scan_path: String,
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
        info!(
            "Loading config file from: {}",
            args.config_file_path.as_str()
        );
        // Load settings from config file
        let config = Config::builder()
            .add_source(File::with_name(args.config_file_path.as_str()).required(false))
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

    pub fn update(&mut self, settings: &SettingsModel) {
        let new_settings = settings.clone();
        self.db_path = new_settings.db_path;
        self.enable_ipv6 = new_settings.enable_ipv6;
        self.port = new_settings.port;
        self.listen_addr_ipv4 = new_settings.listen_addr_ipv4;
        self.listen_addr_ipv6 = new_settings.listen_addr_ipv6;
        self.log_level = new_settings.log_level;
        self.default_scan_path = new_settings.default_scan_path;
    }

    pub fn to_model(&self) -> SettingsModel {
        let _value = self.clone();
        let model = SettingsModel {
            config_file_path: _value.config_file_path,
            db_path: _value.db_path,
            enable_ipv6: _value.enable_ipv6,
            port: _value.port,
            listen_addr_ipv4: _value.listen_addr_ipv4,
            listen_addr_ipv6: _value.listen_addr_ipv6,
            log_level: _value.log_level,
            default_scan_path: _value.default_scan_path,
        };
        model
    }
}
