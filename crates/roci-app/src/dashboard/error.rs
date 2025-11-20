use gitlab::{
    api::{
        projects::{
            issues::IssuesBuilderError,
            merge_requests::pipelines::MergeRequestPipelinesBuilderError,
            merge_requests::MergeRequestsBuilderError, pipelines::PipelinesBuilderError,
            ProjectBuilderError,
        },
        users::CurrentUserBuilderError,
        ApiError,
    },
    RestError,
};
use thiserror::Error;

use crate::state::gitlab::GitlabsError;

#[derive(Debug, Error)]
pub enum GitlabError {
    #[error("Gitlab error: {0}")]
    Gitlab(#[from] gitlab::GitlabError),
    #[error("Gitlabs error: {0}")]
    Gitlabs(#[from] GitlabsError),
    #[error("Api error: {0}")]
    Api(#[from] ApiError<RestError>),
    #[error("Project error: {0}")]
    Project(#[from] ProjectBuilderError),
    #[error("Pipelines error: {0}")]
    Pipelines(#[from] PipelinesBuilderError),
    #[error("Current user error: {0}")]
    User(#[from] CurrentUserBuilderError),
    #[error("Merge requests error: {0}")]
    MergeRequests(#[from] MergeRequestsBuilderError),
    #[error("Issues error: {0}")]
    Issues(#[from] IssuesBuilderError),
    #[error("Merge requests pipeline error: {0}")]
    MergeRequestPipelines(#[from] MergeRequestPipelinesBuilderError),
}
