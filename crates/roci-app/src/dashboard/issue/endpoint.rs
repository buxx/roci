use derive_more::Constructor;
use gitlab::api::{Endpoint, QueryParams};
use serde::Serialize;

#[derive(Debug, Default, Serialize, Constructor)]
pub struct MyIssues<'a> {
    scope: &'a str,
    state: Option<&'a str>,
}

impl<'a> Endpoint for MyIssues<'a> {
    fn method(&self) -> reqwest::Method {
        reqwest::Method::GET
    }

    fn endpoint(&self) -> std::borrow::Cow<'static, str> {
        "issues".into()
    }

    fn parameters(&self) -> QueryParams<'_> {
        let mut params = QueryParams::default();
        params.push("scope", self.scope);
        if let Some(state) = self.state {
            params.push("state", state);
        }
        params
    }
}
