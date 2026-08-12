//! 对象存储 URL 获取模块
//!
//! 提供获取对象存储资源访问 URL 的功能

fn ensure_rooted_path(root: &str, object_path: &str) -> String {
    let root = root.trim_matches('/');
    let object_path = object_path.trim_start_matches('/');
    if object_path.is_empty() {
        return root.to_string();
    }
    if object_path == root || object_path.starts_with(&format!("{}/", root)) {
        object_path.to_string()
    } else {
        format!("{}/{}", root, object_path)
    }
}

/// 获取对象的完整 URL
///
/// # Arguments
/// - `public_base_url`: 对象存储资源访问基础 URL
/// - `object_path`: 对象路径
///
/// # Returns
/// 返回完整的访问 URL
pub fn get_object_url(public_base_url: &str, object_path: &str) -> String {
    format!(
        "{}/{}",
        public_base_url.trim_end_matches('/'),
        object_path.trim_start_matches('/')
    )
}

/// 获取扩展 CRX 文件的完整 URL
///
/// # Arguments
/// - `public_base_url`: 对象存储资源访问基础 URL
/// - `extension_root`: 扩展对象根路径
/// - `object_path`: CRX 对象路径（数据库中存储的路径）
pub fn get_extension_crx_url(
    public_base_url: &str,
    extension_root: &str,
    object_path: &str,
) -> String {
    get_object_url(public_base_url, &ensure_rooted_path(extension_root, object_path))
}

/// 获取扩展图标的完整 URL
///
/// # Arguments
/// - `public_base_url`: 对象存储资源访问基础 URL
/// - `extension_root`: 扩展对象根路径
/// - `object_path`: 图标对象路径（数据库中存储的路径）
pub fn get_extension_icon_url(
    public_base_url: &str,
    extension_root: &str,
    object_path: &str,
) -> String {
    get_object_url(public_base_url, &ensure_rooted_path(extension_root, object_path))
}

/// 获取头像的完整 URL
///
/// # Arguments
/// - `public_base_url`: 对象存储资源访问基础 URL
/// - `avatar_root`: 头像对象根路径
/// - `resource_hash`: 头像文件哈希
pub fn get_avatar_url(public_base_url: &str, avatar_root: &str, resource_hash: &str) -> String {
    get_object_url(public_base_url, &ensure_rooted_path(avatar_root, resource_hash))
}

/// 获取版本资源的完整 URL
///
/// # Arguments
/// - `public_base_url`: 对象存储资源访问基础 URL
/// - `version_root`: 版本资源对象根路径
/// - `object_path`: 版本资源对象路径（数据库中存储的路径）
pub fn get_version_resource_url(
    public_base_url: &str,
    version_root: &str,
    object_path: &str,
) -> String {
    get_object_url(public_base_url, &ensure_rooted_path(version_root, object_path))
}
