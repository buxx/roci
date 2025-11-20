use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub target_url: String,
    pub body: String,
}
