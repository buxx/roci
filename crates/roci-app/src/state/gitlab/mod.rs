use gitlab::{AsyncGitlab, GitlabBuilder};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

use crate::config::get_password;

pub mod issue;
pub mod project;
pub mod todo;
pub mod user;

#[derive(Clone, Default)]
pub struct Gitlabs(Arc<Mutex<Vec<(String, AsyncGitlab)>>>);

impl Gitlabs {
    pub async fn get(
        &self,
        config: &crate::config::gitlab_::Gitlab,
    ) -> Result<AsyncGitlab, GitlabsError> {
        let mut values = self.0.lock().await;

        if values
            .iter()
            .find(|(host, _)| host == &config.host)
            .is_none()
        {
            let token = get_password(&config.host)?;
            let mut builder = GitlabBuilder::new(&config.host, token);
            if config.insecure {
                builder.insecure();
            }
            if config.cert_insecure {
                builder.cert_insecure();
            }
            let gitlab = builder.build_async().await?;
            values.push((config.host.clone(), gitlab));
        }

        Ok(values
            .iter()
            .find(|(host, _)| host == &config.host)
            .expect("Inserted just before")
            .1
            .clone())
    }

    pub fn invalidate(&self, host: &str) {
        let mut values = self.0.blocking_lock();
        values.retain(|(host_, _)| host_ != host);
    }
}

#[derive(Debug, Error)]
pub enum GitlabsError {
    #[error("{0}")]
    Gitlab(#[from] gitlab::GitlabError),
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
}
