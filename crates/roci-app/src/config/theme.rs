use gpui::SharedString;
use gpui_component::{notification::NotificationType, select::SelectItem, ThemeConfig, ThemeSet};
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use thiserror::Error;

const AYU: &str = include_str!("../../themes/ayu.json");

#[derive(EnumIter, Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum ThemeMode {
    System,
    Dark,
    Light,
}

impl ThemeMode {
    pub fn resolve(&self) -> Result<Self, dark_light::Error> {
        Ok(match self {
            ThemeMode::System => match dark_light::detect()? {
                dark_light::Mode::Dark => Self::Dark,
                dark_light::Mode::Light => Self::Light,
                dark_light::Mode::Unspecified => Self::Light,
            },
            ThemeMode::Dark => ThemeMode::Dark,
            ThemeMode::Light => ThemeMode::Light,
        })
    }

    pub fn theme_name(&self) -> Result<&str, dark_light::Error> {
        match self.resolve()? {
            ThemeMode::System => unreachable!(),
            ThemeMode::Dark => Ok("Ayu Dark"),
            ThemeMode::Light => Ok("Ayu Light"),
        }
    }
}

pub fn load_theme(
    mode: ThemeMode,
) -> Result<(ThemeConfig, Option<DarkLightError>), LoadthemeError> {
    let theme_set = serde_json::from_str::<ThemeSet>(AYU)?;
    let (theme_name, dark_light_error) = match mode.theme_name() {
        Ok(theme_name) => (theme_name, None),
        Err(error) => ("Ayu Light", Some(DarkLightError(error))),
    };

    let theme = theme_set
        .themes
        .into_iter()
        .find(|theme| theme.name == SharedString::new(theme_name))
        .ok_or(LoadthemeError::UnknownTheme(theme_name.to_string()))?;
    Ok((theme, dark_light_error))
}

#[derive(Debug, Error)]
pub enum LoadthemeError {
    #[error("Malformed theme JSON: {0}")]
    Malformed(#[from] serde_json::Error),
    #[error("Unknown theme '{0}'")]
    UnknownTheme(String),
}

pub struct DarkLightError(dark_light::Error);

impl DarkLightError {
    pub fn into_notification(&self) -> (NotificationType, SharedString) {
        (
            NotificationType::Warning,
            SharedString::new(format!(
                "Can't determine system dark/light preference: {}",
                self.0
            )),
        )
    }
}

impl SelectItem for ThemeMode {
    type Value = ThemeMode;

    fn title(&self) -> SharedString {
        match self {
            ThemeMode::System => SharedString::new("System theme".to_string()),
            ThemeMode::Dark => SharedString::new("Dark theme".to_string()),
            ThemeMode::Light => SharedString::new("Light theme".to_string()),
        }
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::System
    }
}
