use std::{fs, path::PathBuf};

use ::serde::{Deserialize, Serialize};
use clap::Parser;
use config::{Config, Environment, File};
use log::info;
use toml_edit::{DocumentMut, Item, Table};
use utoipa::ToSchema;

use crate::utils::error::DfrError;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "conf/config")]
    config_file_path: String,
}
/// System settings for the application. This struct is used to load and save settings from a configuration file.
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema)]
#[serde(default)]
pub struct SystemSettings {
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
    /// interval in seconds to clear trash
    pub clear_trash_interval_s: u32,
    /// trash path for deleted files
    pub trash_path: String,
}

/// User settings
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct UserSettings {
    /// login user name
    pub login_user_name: String,
    /// login password
    pub login_password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Settings {
    /// System settings
    pub system: SystemSettings,
    /// User settings
    pub user: UserSettings,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            config_file_path: "conf/config".to_string(),
            db_path: "conf/dfremover.db".to_string(),
            enable_ipv6: true,
            port: 8081,
            listen_addr_ipv4: "0.0.0.0".to_string(),
            listen_addr_ipv6: "::".to_string(),
            log_level: "info".to_string(),
            default_scan_path: "data/".to_string(),
            clear_trash_interval_s: 2592000, // 30 days in seconds
            trash_path: "data/dfr_trash".to_string(),
        }
    }
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            login_user_name: "admin".to_string(),
            login_password: "password".to_string(),
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
        settings.system.config_file_path = args.config_file_path.clone();
        Ok(settings)
    }

    fn save_system(&self, system: &SystemSettings, toml_doc: &mut DocumentMut) {
        if !toml_doc.contains_table("system") {
            toml_doc.insert("system", Item::Table(Table::new()));
        }
        let system_table = toml_doc["system"].as_table_mut().unwrap();
        let default_settings = SystemSettings::default();

        if system.db_path != default_settings.db_path || system_table.contains_key("db_path") {
            system_table["db_path"] = toml_edit::value(system.db_path.clone());
        }
        if system.enable_ipv6 != default_settings.enable_ipv6
            || system_table.contains_key("enable_ipv6")
        {
            system_table["enable_ipv6"] = toml_edit::value(system.enable_ipv6);
        }
        if system.listen_addr_ipv4 != default_settings.listen_addr_ipv4
            || system_table.contains_key("listen_addr_ipv4")
        {
            system_table["listen_addr_ipv4"] = toml_edit::value(system.listen_addr_ipv4.clone());
        }
        if system.listen_addr_ipv6 != default_settings.listen_addr_ipv6
            || system_table.contains_key("listen_addr_ipv6")
        {
            system_table["listen_addr_ipv6"] = toml_edit::value(system.listen_addr_ipv6.clone());
        }
        if system.port != default_settings.port || system_table.contains_key("port") {
            system_table["port"] = toml_edit::value(system.port as i64);
        }
        if system.log_level != default_settings.log_level || system_table.contains_key("log_level")
        {
            system_table["log_level"] = toml_edit::value(system.log_level.clone());
        }
        if system.default_scan_path != default_settings.default_scan_path
            || system_table.contains_key("default_scan_path")
        {
            system_table["default_scan_path"] = toml_edit::value(system.default_scan_path.clone());
        }
        if system.clear_trash_interval_s != default_settings.clear_trash_interval_s
            || system_table.contains_key("clear_trash_interval_s")
        {
            system_table["clear_trash_interval_s"] =
                toml_edit::value(system.clear_trash_interval_s as i64);
        }
        if system.trash_path != default_settings.trash_path
            || system_table.contains_key("trash_path")
        {
            system_table["trash_path"] = toml_edit::value(system.trash_path.clone());
        }
    }

    fn save_user(&self, user: &UserSettings, toml_doc: &mut DocumentMut) {
        if !toml_doc.contains_table("user") {
            toml_doc.insert("user", Item::Table(Table::new()));
        }
        let user_table = toml_doc["user"].as_table_mut().unwrap();
        let default_settings = UserSettings::default(); // Reset default settings to avoid conflicts

        if user.login_user_name != default_settings.login_user_name
            || user_table.contains_key("login_user_name")
        {
            user_table["login_user_name"] = toml_edit::value(user.login_user_name.clone());
        }
        if user.login_password != default_settings.login_password
            || user_table.contains_key("login_password")
        {
            user_table["login_password"] = toml_edit::value(user.login_password.clone());
        }
    }

    pub fn save(&self) -> Result<(), DfrError> {
        let mut config_file_path = PathBuf::from(self.system.config_file_path.as_str());
        config_file_path.set_extension("toml");
        // Save settings to config file
        let config_context = fs::read_to_string(&config_file_path)?;
        let mut toml_doc = config_context.parse::<DocumentMut>()?;
        // Update system settings
        self.save_system(&self.system, &mut toml_doc);
        self.save_user(&self.user, &mut toml_doc);

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
