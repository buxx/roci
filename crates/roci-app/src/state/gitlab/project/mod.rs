use serde::Deserialize;

pub mod merge_request;
pub mod pipeline;

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub default_branch: String,
}
