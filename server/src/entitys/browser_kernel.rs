use serde::{Deserialize, Serialize};

/// 查询浏览器内核列表请求
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ListBrowserKernelsRequest {
    /// 平台过滤，如 windows/darwin/linux，为空表示不过滤
    pub platform: Option<String>,
    /// 版本类型过滤，如 SIMPRINT_KERNEL_CHROMIUM，为空表示查询所有 SIMPRINT_KERNEL_* 类型
    pub type_code: Option<String>,
}
