use std::time::Duration;

use gpui::*;
use gpui_component::select::SelectItem;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum RefreshEvery {
    X60Seconds,
    X5Minutes,
    X15Minutes,
    X30Minutes,
    X1Hour,
    X6Hours,
}
impl RefreshEvery {
    pub fn duration(&self) -> Duration {
        match self {
            RefreshEvery::X60Seconds => Duration::from_secs(60),
            RefreshEvery::X5Minutes => Duration::from_secs(60 * 5),
            RefreshEvery::X15Minutes => Duration::from_secs(60 * 15),
            RefreshEvery::X30Minutes => Duration::from_secs(60 * 30),
            RefreshEvery::X1Hour => Duration::from_secs(60 * 60 * 1),
            RefreshEvery::X6Hours => Duration::from_secs(60 * 60 * 6),
        }
    }
}

impl SelectItem for RefreshEvery {
    type Value = RefreshEvery;

    fn title(&self) -> SharedString {
        match self {
            RefreshEvery::X60Seconds => SharedString::new("Refresh every 60 seconds".to_string()),
            RefreshEvery::X5Minutes => SharedString::new("Refresh every 5 minutes".to_string()),
            RefreshEvery::X15Minutes => SharedString::new("Refresh every 15 minutes".to_string()),
            RefreshEvery::X30Minutes => SharedString::new("Refresh every 30 minutes".to_string()),
            RefreshEvery::X1Hour => SharedString::new("Refresh every 1 hour".to_string()),
            RefreshEvery::X6Hours => SharedString::new("Refresh every 6 hours".to_string()),
        }
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

impl Default for RefreshEvery {
    fn default() -> Self {
        Self::X1Hour
    }
}
