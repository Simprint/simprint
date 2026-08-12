use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GetLocalApiConfigRequest {}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ResetLocalApiKeyRequest {}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalApiConfigRequest {
    pub enabled: Option<bool>,
    pub port: Option<i32>,
    pub remote_access: Option<bool>,
    pub cors_origins: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateLocalApiKeyRequest {
    pub api_key: String,
    pub permission_code: String,
}
