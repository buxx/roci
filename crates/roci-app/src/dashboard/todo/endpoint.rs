use gitlab::api::{Endpoint, QueryParams};
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct MyTodos;

impl Endpoint for MyTodos {
    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }

    fn endpoint(&self) -> std::borrow::Cow<'static, str> {
        "todos".into()
    }

    fn parameters(&self) -> QueryParams<'_> {
        let mut params = QueryParams::default();
        params.push("state", "pending");
        params
    }
}
