use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
}
