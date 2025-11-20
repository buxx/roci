use gpui::*;
use gpui_component::select::SelectItem;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(EnumIter, Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum ShowMergeRequest {
    OnlyMine,
    All,
}

impl SelectItem for ShowMergeRequest {
    type Value = ShowMergeRequest;

    fn title(&self) -> SharedString {
        match self {
            ShowMergeRequest::OnlyMine => SharedString::new("Show only my MRs".to_string()),
            ShowMergeRequest::All => SharedString::new("Show all MRs".to_string()),
        }
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

impl Default for ShowMergeRequest {
    fn default() -> Self {
        Self::OnlyMine
    }
}
