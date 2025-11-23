use gpui::SharedString;
use gpui_component::notification::NotificationType;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};
use thiserror::Error;

use crate::config::{merge_request::ShowMergeRequest, refresh::RefreshEvery, theme::ThemeMode};

pub mod gitlab_;
pub mod merge_request;
pub mod refresh;
pub mod theme;

const KEYRING_SERVICE_NAME: &str = "roci";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    pub gitlabs: Vec<gitlab_::Gitlab>,
    pub refresh_every: RefreshEvery,
    pub show_merge_request: ShowMergeRequest,
    #[serde(default)]
    pub theme_mode: ThemeMode,
}

impl Config {
    fn path() -> Result<PathBuf, ConfigError> {
        Ok(homedir::my_home()?
            .ok_or(ConfigError::NoHome)?
            .join(".roci"))
    }

    pub fn from_env() -> Result<(Self, Option<ConfigLoadInfo>), ConfigError> {
        let path = Self::path()?;

        match fs::read_to_string(&path) {
            Ok(raw) => match ron::from_str(&raw) {
                Ok(config) => Ok((config, None)),
                Err(error) => Ok((
                    Self::default(),
                    Some(ConfigLoadInfo::Invalid(error.to_string())),
                )),
            },
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => Ok((Self::default(), Some(ConfigLoadInfo::NoOne(path)))),
                _ => Err(ConfigError::Io(error.kind())),
            },
        }
    }

    pub fn persist(&self) -> Result<(), ConfigError> {
        let path = Self::path()?;
        let raw =
            ron::to_string(self).map_err(|error| ConfigError::Unexpected(error.to_string()))?;
        fs::write(path, raw).map_err(|error| ConfigError::Io(error.kind()))?;

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Can't determine home dir: {0}")]
    Home(#[from] homedir::GetHomeError),
    #[error("No home dir for current user")]
    NoHome,
    #[error("Io error: {0}")]
    Io(io::ErrorKind),
    #[error("Unexpected: {0}")]
    Unexpected(String),
    #[error("Token secret access error: {0}")]
    Keyring(#[from] keyring::Error),
}

#[derive(Debug, Error)]
pub enum ConfigLoadInfo {
    #[error("New config file created at {0}")]
    NoOne(PathBuf),
    #[error("Invalid config found, new one crated")]
    Invalid(String),
}

impl ConfigLoadInfo {
    pub fn into_notification(&self) -> (NotificationType, SharedString) {
        match self {
            ConfigLoadInfo::NoOne(_) => (
                NotificationType::Info,
                SharedString::new(format!("No config found")),
            ),
            ConfigLoadInfo::Invalid(message) => (
                NotificationType::Warning,
                SharedString::new(format!(
                    "Invalid config found ({}), new one crated",
                    message
                )),
            ),
        }
    }
}

pub fn set_password(host: &str, token: &str) -> Result<(), keyring::Error> {
    keyring::Entry::new(KEYRING_SERVICE_NAME, host)?.set_password(token)?;
    Ok(())
}

pub fn get_password(host: &str) -> Result<String, keyring::Error> {
    Ok(keyring::Entry::new(KEYRING_SERVICE_NAME, host)?.get_password()?)
}
