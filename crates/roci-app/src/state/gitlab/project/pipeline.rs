use serde::Deserialize;
use strum::{Display, EnumString};

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Pipeline {
    pub id: u64,
    pub iid: u64,
    pub project_id: u64,
    pub status: PipelineStatus,
    pub web_url: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, EnumString, Display)]
#[serde(rename_all = "lowercase")]
pub enum PipelineStatus {
    Running,
    Pending,
    Success,
    Failed,
    Canceled,
    Skipped,
    Created,
    Manual,
    Scheduled,
    Preparing,
    WaitingForResource,
}

impl PipelineStatus {
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed)
    }
}
