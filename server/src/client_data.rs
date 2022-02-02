use pwd_dl_zkp_core::core::Choice;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct ClientTest {
    pub c: Option<String>,
    pub choice: Option<Choice>,
    pub valid: Option<bool>,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub(crate) struct ClientData {
    pub created_at: String,
    pub p: Option<String>,
    pub g: Option<String>,
    pub y: Option<String>,
    pub tests: Vec<ClientTest>,
    pub auth: Option<bool>,
}

impl ClientData {
    pub fn new() -> Self {
        Self {
            created_at: "".to_string(), // format!("{:?}", Utc::now()),
            ..Default::default()
        }
    }

    pub fn should_continue(&self) -> bool {
        self.tests
            .iter()
            .filter(|t| t.valid.unwrap_or(false))
            .count()
            < 10
    }
}
