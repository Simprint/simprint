use std::{borrow::Cow, collections::BTreeMap};

use rmcp::{
    handler::server::router::tool::{AsyncTool, ToolBase, ToolRoute, ToolRouter},
    schemars::JsonSchema,
};
use serde::{Deserialize, Serialize};

use crate::{
    local_api::entitys::LocalApiBrowserKernelVersion,
    mcp::{bridge::LocalApiListBrowserKernelsRequest, error::McpToolError, server::McpServer},
};

pub fn routes() -> Vec<ToolRoute<McpServer>> {
    ToolRouter::new()
        .with_async_tool::<ListBrowserKernelsTool>()
        .into_iter()
        .collect()
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct ListBrowserKernelsInput {
    pub platform: Option<String>,
    pub type_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ListBrowserKernelsOutput {
    pub items: Vec<BrowserKernelGroupSummary>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BrowserKernelGroupSummary {
    pub type_code: String,
    pub versions: Vec<BrowserKernelVersionSummary>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct BrowserKernelVersionSummary {
    pub kernel_id: String,
    pub resource_name: String,
    pub version: String,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub platform: Option<String>,
    pub file_size: Option<i32>,
    pub is_latest: bool,
    pub status: String,
    pub arch: Option<String>,
    pub package_format: Option<String>,
    pub requires_extract: bool,
}

struct ListBrowserKernelsTool;

impl ToolBase for ListBrowserKernelsTool {
    type Parameter = ListBrowserKernelsInput;
    type Output = ListBrowserKernelsOutput;
    type Error = McpToolError;

    fn name() -> Cow<'static, str> {
        "simprint_list_browser_kernels".into()
    }

    fn description() -> Option<Cow<'static, str>> {
        Some("List Simprint browser kernel versions by platform or kernel type.".into())
    }
}

impl AsyncTool<McpServer> for ListBrowserKernelsTool {
    async fn invoke(
        service: &McpServer,
        param: Self::Parameter,
    ) -> Result<Self::Output, Self::Error> {
        let items = service
            .bridge()
            .list_browser_kernels(LocalApiListBrowserKernelsRequest {
                platform: param.platform.map(normalize_non_empty),
                type_code: param.type_code.map(normalize_non_empty),
            })
            .await?;

        Ok(ListBrowserKernelsOutput {
            items: sort_kernel_groups(items),
        })
    }
}

fn sort_kernel_groups(
    groups: std::collections::HashMap<String, Vec<LocalApiBrowserKernelVersion>>,
) -> Vec<BrowserKernelGroupSummary> {
    groups
        .into_iter()
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(type_code, versions)| BrowserKernelGroupSummary {
            type_code,
            versions: versions.into_iter().map(map_browser_kernel_version).collect(),
        })
        .collect()
}

fn map_browser_kernel_version(
    version: LocalApiBrowserKernelVersion,
) -> BrowserKernelVersionSummary {
    BrowserKernelVersionSummary {
        kernel_id: version.kernel_id,
        resource_name: version.resource_name,
        version: version.version,
        name: version.name,
        notes: version.notes,
        platform: version.platform,
        file_size: version.file_size,
        is_latest: version.is_latest,
        status: version.status,
        arch: version.arch,
        package_format: version.package_format,
        requires_extract: version.requires_extract,
    }
}

fn normalize_non_empty(value: String) -> String {
    value.trim().to_string()
}
