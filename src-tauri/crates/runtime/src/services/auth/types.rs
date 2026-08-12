use crate::infrastructure::eventbus::AuthInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum AuthCommandRequest {
    SetAuthState { auth_info: AuthInfo },
    ClearAuthState,
    GetAuthState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthCommandResponse {
    Ack,
    State { auth_info: AuthInfo },
}
