use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalApiBrowserKernelVersion {
    pub kernel_id: String,
    pub type_code: String,
    pub resource_name: String,
    pub version: String,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
    pub hash: String,
    pub signature: String,
    pub file_size: Option<i32>,
    pub is_latest: bool,
    pub status: String,
    pub arch: Option<String>,
    pub package_format: Option<String>,
    pub requires_extract: bool,
    pub entrypoint_template: Option<String>,
    pub extract_root: Option<String>,
    pub installed: bool,
}

pub type LocalApiBrowserKernelListResponse = HashMap<String, Vec<LocalApiBrowserKernelVersion>>;
