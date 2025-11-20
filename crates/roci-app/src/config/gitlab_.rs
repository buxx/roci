use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Gitlab {
    pub host: String,
    pub insecure: bool,
    pub cert_insecure: bool,
    pub project_ids: Vec<u64>,
}

impl Gitlab {
    pub fn empty(host: String, insecure: bool, cert_insecure: bool) -> Self {
        Self {
            host,
            insecure,
            cert_insecure,
            project_ids: vec![],
        }
    }

    pub fn protocol(&self) -> String {
        match self.insecure {
            true => "http://",
            false => "https://",
        }
        .to_string()
    }
}
