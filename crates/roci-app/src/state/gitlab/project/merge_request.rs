use crate::state::gitlab::project::pipeline::Pipeline;
use derive_more::Constructor;
use serde::Deserialize;
use strum::{Display, EnumString};

pub const MERGE_STATUS_MERGEABLE: &str = "mergeable";

#[allow(dead_code)]
#[derive(Debug, Deserialize, Constructor)]
pub struct MergeRequestContainer {
    inner: MergeRequest,
    pub last_pipeline: Option<Pipeline>,
}

impl std::ops::Deref for MergeRequestContainer {
    type Target = MergeRequest;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct MergeRequest {
    pub id: u64,
    pub iid: u64,
    pub project_id: u64,
    pub title: String,
    pub state: MergeRequestState,
    pub web_url: String,
    pub created_at: String,
    pub detailed_merge_status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, EnumString, Display)]
#[serde(rename_all = "lowercase")]
pub enum MergeRequestState {
    Opened,
    Closed,
    Locked,
    Merged,
}
