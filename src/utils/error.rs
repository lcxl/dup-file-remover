use std::fmt::{self, Display};

use actix_web::ResponseError;

/// Custom error type for the application.
#[derive(Debug)]
pub enum DfrError {
    /// An I/O error occurred.
    IoError(std::io::Error),
    /// A JSON serialization/deserialization error occurred.
    JsonError(serde_json::Error),
    /// A Rusqlite database error occurred.
    RusqliteError(rusqlite::Error),
    /// A configuration error occurred.
    ConfigError(config::ConfigError),
    /// A TOML edit error occurred.
    TomlEditError(toml_edit::TomlError),
    /// A TOML ser error occurred.
    TomlError(toml::ser::Error),
    /// A connection pool error occurred.
    R2d2Error(r2d2::Error),
}

impl Display for DfrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DfrError::IoError(error) => error.fmt(f),
            DfrError::JsonError(error) => error.fmt(f),
            DfrError::RusqliteError(error) => error.fmt(f),
            DfrError::ConfigError(error) => error.fmt(f),
            DfrError::TomlEditError(error) => error.fmt(f),
            DfrError::TomlError(error) => error.fmt(f),
            DfrError::R2d2Error(error) => error.fmt(f),
        }
    }
}

impl From<std::io::Error> for DfrError {
    fn from(err: std::io::Error) -> Self {
        DfrError::IoError(err)
    }
}

impl From<serde_json::Error> for DfrError {
    fn from(err: serde_json::Error) -> Self {
        DfrError::JsonError(err)
    }
}

impl From<rusqlite::Error> for DfrError {
    fn from(err: rusqlite::Error) -> Self {
        DfrError::RusqliteError(err)
    }
}

impl From<config::ConfigError> for DfrError {
    fn from(err: config::ConfigError) -> Self {
        DfrError::ConfigError(err)
    }
}

impl From<toml_edit::TomlError> for DfrError {
    fn from(err: toml_edit::TomlError) -> Self {
        DfrError::TomlEditError(err)
    }
}

impl From<toml::ser::Error> for DfrError {
    fn from(err: toml::ser::Error) -> Self {
        DfrError::TomlError(err)
    }
}

impl From<r2d2::Error> for DfrError {
    fn from(err: r2d2::Error) -> Self {
        DfrError::R2d2Error(err)
    }
}

impl ResponseError for DfrError {}
