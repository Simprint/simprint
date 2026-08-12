use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::AppHandle;
use uuid::Uuid;

use crate::core::error::Result;
use crate::core::utils::hash::calculate_file_hash;
use crate::services::environment::ExtensionInfo;

static REGISTRY_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static EXTENSION_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-p]{32}$").expect("valid extension id regex"));
static HTML_TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?is)<[^>]+>").expect("valid html tag regex"));
static UPDATE_CHECK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<updatecheck\b[^>]*codebase="([^"]+)"[^>]*version="([^"]+)"[^>]*"#)
        .expect("valid updatecheck regex")
});
static UPDATE_HASH_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"hash_sha256="([^"]+)""#).expect("valid hash regex"));

const REGISTRY_FILENAME: &str = "local-extensions.json";
const LOCAL_SOURCE: &str = "local";
const LOCAL_SCOPE: &str = "user-local";
const CHROME_UPDATE_URL: &str = "https://clients2.google.com/service/update2/crx";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalExtensionDto {
    pub record_id: String,
    pub extension_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub browser: String,
    pub status: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rating: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_state: Option<String>,
    pub imported_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalExtensionCatalogRecord {
    record_id: String,
    extension_id: String,
    name: String,
    description: String,
    version: String,
    browser: String,
    author: Option<String>,
    homepage: Option<String>,
    icon_url: Option<String>,
    category: Option<String>,
    permissions: Vec<String>,
    hash: String,
    file_size: i64,
    downloads_count: Option<i64>,
    rating: Option<f64>,
    managed_crx_path: String,
    source: String,
    imported_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalExtensionInstallationRecord {
    record_id: String,
    scope: String,
    status: String,
    installed_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalExtensionRegistry {
    version: u32,
    catalog: Vec<LocalExtensionCatalogRecord>,
    installations: Vec<LocalExtensionInstallationRecord>,
}

impl Default for LocalExtensionRegistry {
    fn default() -> Self {
        Self {
            version: 1,
            catalog: Vec::new(),
            installations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedCrxMetadata {
    name: String,
    description: String,
    version: String,
    author: Option<String>,
    homepage: Option<String>,
    permissions: Vec<String>,
}

#[derive(Debug, Clone)]
struct ChromeUpdateInfo {
    extension_id: String,
    codebase: String,
    hash_sha256: Option<String>,
    store_url: String,
}

#[derive(Debug, Clone, Default)]
struct StorePageInfo {
    name: Option<String>,
    description: Option<String>,
    icon_url: Option<String>,
    category: Option<String>,
    downloads_count: Option<i64>,
    rating: Option<f64>,
}

#[derive(Debug, Clone)]
enum LocalImportSource {
    LocalFile,
    StoreUrl {
        extension_id: String,
        store_url: String,
        store_info: Option<StorePageInfo>,
    },
}

pub struct LocalExtensionService;

impl LocalExtensionService {
    pub fn import_crx(app: &AppHandle, path: String) -> Result<LocalExtensionDto> {
        let source_path = PathBuf::from(path.trim());
        if !source_path.exists() || !source_path.is_file() {
            return Err(format!("本地插件文件不存在: {}", source_path.display()).into());
        }

        import_crx_from_path(app, &source_path, LocalImportSource::LocalFile)
    }

    pub async fn import_store_url(app: &AppHandle, store_url: String) -> Result<LocalExtensionDto> {
        let normalized_store_url = store_url.trim().to_string();
        let extension_id = extract_extension_id(&normalized_store_url)?;

        if let Some(existing) = find_existing_store_import_result(app, &extension_id)? {
            return Ok(existing);
        }

        let store_info = fetch_store_page_info(&normalized_store_url).await.unwrap_or(None);
        let update_info = fetch_store_update_info(&normalized_store_url).await?;
        let temp_path = local_crx_dir(app)?.join(format!(
            ".tmp_{}_{}.crx",
            update_info.extension_id,
            Uuid::new_v4().simple()
        ));

        download_crx_to_path(&update_info.codebase, &temp_path).await?;

        let file_hash = calculate_file_hash(&temp_path)?;
        if let Some(expected_hash) = update_info.hash_sha256.as_deref() {
            if !expected_hash.eq_ignore_ascii_case(&file_hash) {
                let _ = fs::remove_file(&temp_path);
                return Err("下载的 CRX 文件校验失败".into());
            }
        }

        let import_result = import_crx_from_path(
            app,
            &temp_path,
            LocalImportSource::StoreUrl {
                extension_id: update_info.extension_id.clone(),
                store_url: update_info.store_url.clone(),
                store_info,
            },
        );

        let _ = fs::remove_file(&temp_path);
        import_result
    }

    pub fn list_extensions(app: &AppHandle) -> Result<Vec<LocalExtensionDto>> {
        let _guard = REGISTRY_LOCK.lock().map_err(|_| "local extension registry lock poisoned")?;

        let registry = read_registry(app)?;
        Ok(registry
            .catalog
            .iter()
            .map(|record| to_dto(record, installation_status(&registry, &record.record_id)))
            .collect())
    }

    pub fn install_extension(app: &AppHandle, record_id: String) -> Result<LocalExtensionDto> {
        update_installation_status(app, &record_id, "active", true)
    }

    pub fn uninstall_extension(app: &AppHandle, record_id: String) -> Result<LocalExtensionDto> {
        update_installation_status(app, &record_id, "inactive", false)
    }

    pub fn remove_extension(app: &AppHandle, record_id: String) -> Result<()> {
        let _guard = REGISTRY_LOCK.lock().map_err(|_| "local extension registry lock poisoned")?;

        let mut registry = read_registry(app)?;
        let record_index = registry
            .catalog
            .iter()
            .position(|item| item.record_id == record_id)
            .ok_or_else(|| format!("未找到本地插件记录: {record_id}"))?;

        let record = registry.catalog.remove(record_index);
        registry.installations.retain(|item| item.record_id != record.record_id);

        write_registry(app, &registry)?;
        let _ = fs::remove_file(&record.managed_crx_path);
        Ok(())
    }

    pub fn disable_extension(app: &AppHandle, record_id: String) -> Result<LocalExtensionDto> {
        update_installation_status(app, &record_id, "disabled", true)
    }

    pub fn enable_extension(app: &AppHandle, record_id: String) -> Result<LocalExtensionDto> {
        update_installation_status(app, &record_id, "active", true)
    }

    pub fn list_active_extension_infos(app: &AppHandle) -> Result<Vec<ExtensionInfo>> {
        let _guard = REGISTRY_LOCK.lock().map_err(|_| "local extension registry lock poisoned")?;

        let registry = read_registry(app)?;
        let active_ids = registry
            .installations
            .iter()
            .filter(|item| item.scope == LOCAL_SCOPE && item.status == "active")
            .map(|item| item.record_id.as_str())
            .collect::<HashSet<_>>();

        Ok(registry
            .catalog
            .iter()
            .filter(|record| active_ids.contains(record.record_id.as_str()))
            .map(|record| ExtensionInfo {
                extension_id: record.extension_id.clone(),
                name: record.name.clone(),
                version: record.version.clone(),
                download_url: None,
                managed_crx_path: Some(record.managed_crx_path.clone()),
                hash: record.hash.clone(),
                icon_url: None,
            })
            .collect())
    }
}

fn import_crx_from_path(
    app: &AppHandle,
    source_path: &Path,
    import_source: LocalImportSource,
) -> Result<LocalExtensionDto> {
    let _guard = REGISTRY_LOCK.lock().map_err(|_| "local extension registry lock poisoned")?;

    let hash = calculate_file_hash(source_path)?;
    let parsed = parse_crx_metadata(source_path)?;
    let mut registry = read_registry(app)?;

    if let Some(existing_index) = find_existing_record_index(&registry, &import_source, &hash) {
        let record = &registry.catalog[existing_index];
        let status = installation_status(&registry, &record.record_id);
        let import_state = if matches!(status, "active" | "disabled") {
            "already_installed"
        } else {
            "already_exists"
        };

        return Ok(with_import_state(to_dto(record, status), import_state));
    }

    let record_id = format!("local_{}", Uuid::new_v4().simple());
    let managed_path = local_crx_dir(app)?.join(format!("{record_id}.crx"));
    fs::copy(source_path, &managed_path)?;

    let file_size = fs::metadata(&managed_path)?.len() as i64;
    let now = chrono::Utc::now().to_rfc3339();
    let extension_id = match &import_source {
        LocalImportSource::LocalFile => record_id.clone(),
        LocalImportSource::StoreUrl { extension_id, .. } => extension_id.clone(),
    };
    let homepage = match &import_source {
        LocalImportSource::LocalFile => parsed.homepage.clone(),
        LocalImportSource::StoreUrl { store_url, .. } => Some(store_url.clone()),
    };
    let store_info = match &import_source {
        LocalImportSource::LocalFile => None,
        LocalImportSource::StoreUrl { store_info, .. } => store_info.as_ref(),
    };
    let name = store_info.and_then(|info| info.name.clone()).unwrap_or(parsed.name.clone());
    let description = store_info
        .and_then(|info| info.description.clone())
        .unwrap_or(parsed.description.clone());

    let record = LocalExtensionCatalogRecord {
        record_id: record_id.clone(),
        extension_id,
        name,
        description,
        version: parsed.version,
        browser: "chrome".to_string(),
        author: parsed.author,
        homepage,
        icon_url: store_info.and_then(|info| info.icon_url.clone()),
        category: store_info.and_then(|info| info.category.clone()),
        permissions: parsed.permissions,
        hash,
        file_size,
        downloads_count: store_info.and_then(|info| info.downloads_count),
        rating: store_info.and_then(|info| info.rating),
        managed_crx_path: managed_path.to_string_lossy().to_string(),
        source: LOCAL_SOURCE.to_string(),
        imported_at: now.clone(),
        updated_at: now,
    };

    let dto = with_import_state(to_dto(&record, "available"), "imported");
    registry.catalog.push(record);
    write_registry(app, &registry)?;
    Ok(dto)
}

fn find_existing_record_index(
    registry: &LocalExtensionRegistry,
    import_source: &LocalImportSource,
    hash: &str,
) -> Option<usize> {
    match import_source {
        LocalImportSource::LocalFile => {
            registry.catalog.iter().position(|record| record.hash == hash)
        }
        LocalImportSource::StoreUrl { extension_id, .. } => registry
            .catalog
            .iter()
            .position(|record| record.extension_id == *extension_id)
            .or_else(|| registry.catalog.iter().position(|record| record.hash == hash)),
    }
}

fn find_existing_store_import_result(
    app: &AppHandle,
    extension_id: &str,
) -> Result<Option<LocalExtensionDto>> {
    let _guard = REGISTRY_LOCK.lock().map_err(|_| "local extension registry lock poisoned")?;
    let registry = read_registry(app)?;

    let Some(record_index) =
        registry.catalog.iter().position(|record| record.extension_id == extension_id)
    else {
        return Ok(None);
    };

    let record = &registry.catalog[record_index];
    let status = installation_status(&registry, &record.record_id);
    let import_state = if matches!(status, "active" | "disabled") {
        "already_installed"
    } else {
        "already_exists"
    };

    Ok(Some(with_import_state(
        to_dto(record, status),
        import_state,
    )))
}

fn extract_extension_id(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if EXTENSION_ID_REGEX.is_match(trimmed) {
        return Ok(trimmed.to_string());
    }

    let parsed = reqwest::Url::parse(trimmed).map_err(|_| "无效的 Google 商店地址")?;
    let host = parsed.host_str().ok_or("无效的 Google 商店地址")?.to_ascii_lowercase();

    if host != "chromewebstore.google.com" && host != "chrome.google.com" {
        return Err("暂仅支持 Google Chrome 插件商店地址".into());
    }

    if let Some(segments) = parsed.path_segments() {
        for segment in segments {
            if EXTENSION_ID_REGEX.is_match(segment) {
                return Ok(segment.to_string());
            }
        }
    }

    Err("无法从地址中提取扩展 ID".into())
}

async fn fetch_store_update_info(store_url: &str) -> Result<ChromeUpdateInfo> {
    let extension_id = extract_extension_id(store_url)?;
    log::info!(
        "[local-extension-import] fetch store update info: url={}, extension_id={}",
        store_url,
        extension_id
    );
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let response = client
        .get(CHROME_UPDATE_URL)
        .query(&[
            ("response", "updatecheck"),
            ("acceptformat", "crx3"),
            ("prodversion", "120.0.0.0"),
            ("x", &format!("id={extension_id}&uc")),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("获取商店插件下载信息失败: HTTP {}", response.status()).into());
    }

    let xml = response.text().await?;
    log::info!(
        "[local-extension-import] update api raw xml: url={}, extension_id={}, xml={}",
        store_url,
        extension_id,
        xml
    );
    parse_update_response(&extension_id, store_url, &xml)
}

async fn fetch_store_page_info(store_url: &str) -> Result<Option<StorePageInfo>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let response = client.get(store_url).send().await?;
    if !response.status().is_success() {
        return Ok(None);
    }

    let html = response.text().await?;
    Ok(parse_store_page(&html))
}

fn parse_store_page(html: &str) -> Option<StorePageInfo> {
    let name = extract_store_name(html);
    let description = extract_store_description(html);
    let icon_url = extract_store_icon_url(html);
    let category = extract_store_category(html);
    let downloads_count = extract_store_downloads_count(html);
    let rating = extract_store_rating(html);

    let info = StorePageInfo {
        name,
        description,
        icon_url,
        category,
        downloads_count,
        rating,
    };

    if info.name.is_none()
        && info.description.is_none()
        && info.icon_url.is_none()
        && info.category.is_none()
        && info.downloads_count.is_none()
        && info.rating.is_none()
    {
        None
    } else {
        Some(info)
    }
}

fn extract_store_name(html: &str) -> Option<String> {
    extract_meta_content(html, "property", "og:title")
        .or_else(|| extract_meta_content(html, "name", "title"))
        .or_else(|| extract_tag_text(html, "h1"))
        .or_else(|| extract_tag_text(html, "title"))
}

fn extract_store_description(html: &str) -> Option<String> {
    extract_meta_content(html, "property", "og:description")
        .or_else(|| extract_meta_content(html, "name", "description"))
}

fn extract_store_icon_url(html: &str) -> Option<String> {
    extract_meta_content(html, "property", "og:image")
        .or_else(|| extract_link_href(html, "icon"))
        .or_else(|| extract_link_href(html, "shortcut icon"))
}

fn extract_store_category(html: &str) -> Option<String> {
    extract_meta_content(html, "property", "product:category")
        .or_else(|| extract_meta_content(html, "name", "category"))
        .or_else(|| extract_itemprop_content(html, "applicationCategory"))
        .or_else(|| extract_json_field_string(html, "applicationCategory"))
}

fn extract_store_downloads_count(html: &str) -> Option<i64> {
    extract_json_number_like_field(html, "userCount").or_else(|| {
        extract_number_from_patterns(
            html,
            &[
                r#"(?i)([\d,]+)\s*(?:users?|用户)"#,
                r#"(?i)"userCount"\s*:\s*"?(\\?[\d,]+)"?"#,
            ],
        )
    })
}

fn extract_store_rating(html: &str) -> Option<f64> {
    extract_float_from_patterns(
        html,
        &[
            "(?i)\"ratingValue\"\\s*:\\s*\"?(\\\\?\\d+\\.?\\d*)\"?",
            r#"(?i)(\d+\.?\d*)\s*(?:out of 5|/5|星|stars?)"#,
            r#"(?i)Average rating[:\s]*(\d+\.?\d*)"#,
        ],
    )
    .filter(|value| (0.0..=5.0).contains(value))
}

fn extract_meta_content(html: &str, attr_name: &str, attr_value: &str) -> Option<String> {
    let escaped_attr_value = regex::escape(attr_value);
    let tag_pattern = format!(
        r#"(?is)<meta\b[^>]*\b{}\s*=\s*["']{}["'][^>]*>"#,
        attr_name, escaped_attr_value
    );
    let content_pattern = r#"(?is)\bcontent\s*=\s*["']([^"']+)["']"#;

    let tag_regex = Regex::new(&tag_pattern).ok()?;
    let content_regex = Regex::new(content_pattern).ok()?;
    let tag = tag_regex.find(html)?.as_str();
    let content = content_regex.captures(tag)?.get(1)?.as_str();
    normalize_html_text(content)
}

fn extract_link_href(html: &str, rel_value: &str) -> Option<String> {
    let escaped_rel_value = regex::escape(rel_value);
    let tag_pattern = format!(
        r#"(?is)<link\b[^>]*\brel\s*=\s*["']{}["'][^>]*>"#,
        escaped_rel_value
    );
    let href_pattern = r#"(?is)\bhref\s*=\s*["']([^"']+)["']"#;

    let tag_regex = Regex::new(&tag_pattern).ok()?;
    let href_regex = Regex::new(href_pattern).ok()?;
    let tag = tag_regex.find(html)?.as_str();
    let href = href_regex.captures(tag)?.get(1)?.as_str();
    normalize_html_text(href)
}

fn extract_itemprop_content(html: &str, itemprop: &str) -> Option<String> {
    let escaped_itemprop = regex::escape(itemprop);
    let tag_pattern = format!(
        r#"(?is)<[^>]+\bitemprop\s*=\s*["']{}["'][^>]*>"#,
        escaped_itemprop
    );
    let content_attr_pattern = r#"(?is)\bcontent\s*=\s*["']([^"']+)["']"#;

    let tag_regex = Regex::new(&tag_pattern).ok()?;
    let content_attr_regex = Regex::new(content_attr_pattern).ok()?;
    let tag = tag_regex.find(html)?.as_str();

    if let Some(content) = content_attr_regex
        .captures(tag)
        .and_then(|captures| captures.get(1).map(|value| value.as_str()))
    {
        return normalize_html_text(content);
    }

    let text = HTML_TAG_REGEX.replace_all(tag, "");
    normalize_html_text(&text)
}

fn extract_tag_text(html: &str, tag_name: &str) -> Option<String> {
    let escaped_tag = regex::escape(tag_name);
    let pattern = format!(r#"(?is)<{0}\b[^>]*>(.*?)</{0}>"#, escaped_tag);
    let regex = Regex::new(&pattern).ok()?;
    let raw_text = regex.captures(html)?.get(1)?.as_str();
    let text = HTML_TAG_REGEX.replace_all(raw_text, "");
    normalize_html_text(&text)
}

fn extract_json_field_string(html: &str, field_name: &str) -> Option<String> {
    let escaped_field_name = regex::escape(field_name);
    let pattern = format!("(?is)\"{}\"\\s*:\\s*\"([^\"]+)\"", escaped_field_name);
    let regex = Regex::new(&pattern).ok()?;
    let value = regex.captures(html)?.get(1)?.as_str();
    normalize_html_text(value)
}

fn extract_json_number_like_field(html: &str, field_name: &str) -> Option<i64> {
    let escaped_field_name = regex::escape(field_name);
    let pattern = format!(r#"(?is)"{}"\s*:\s*"?(\\?[\d,]+)"?"#, escaped_field_name);
    let regex = Regex::new(&pattern).ok()?;
    let value = regex.captures(html)?.get(1)?.as_str();
    parse_i64_like(value)
}

fn extract_number_from_patterns(html: &str, patterns: &[&str]) -> Option<i64> {
    for pattern in patterns {
        let regex = Regex::new(pattern).ok()?;
        if let Some(value) = regex
            .captures(html)
            .and_then(|captures| captures.get(1).map(|match_value| match_value.as_str()))
            .and_then(parse_i64_like)
        {
            return Some(value);
        }
    }
    None
}

fn extract_float_from_patterns(html: &str, patterns: &[&str]) -> Option<f64> {
    for pattern in patterns {
        let regex = Regex::new(pattern).ok()?;
        if let Some(value) = regex
            .captures(html)
            .and_then(|captures| captures.get(1).map(|match_value| match_value.as_str()))
            .and_then(parse_f64_like)
        {
            return Some(value);
        }
    }
    None
}

fn normalize_html_text(value: &str) -> Option<String> {
    let normalized = value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ");
    let collapsed = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_i64_like(value: &str) -> Option<i64> {
    let digits = value
        .replace("\\", "")
        .chars()
        .filter(|char_value| char_value.is_ascii_digit())
        .collect::<String>();
    digits.parse::<i64>().ok()
}

fn parse_f64_like(value: &str) -> Option<f64> {
    value.replace("\\", "").trim().parse::<f64>().ok()
}

fn parse_update_response(
    extension_id: &str,
    store_url: &str,
    xml: &str,
) -> Result<ChromeUpdateInfo> {
    if xml.contains("error-unknownApplication") || xml.contains("status=\"noupdate\"") {
        return Err("未找到该 Google 商店插件".into());
    }

    let captures = UPDATE_CHECK_REGEX.captures(xml).ok_or("未能解析 Google 商店插件下载信息")?;
    let matched = captures
        .get(0)
        .map(|value| value.as_str())
        .ok_or("未能解析 Google 商店插件下载信息")?;

    let codebase = captures
        .get(1)
        .map(|value| value.as_str().to_string())
        .ok_or("缺少插件下载地址")?;
    let hash_sha256 = UPDATE_HASH_REGEX
        .captures(matched)
        .and_then(|captures| captures.get(1).map(|value| value.as_str().to_string()));

    Ok(ChromeUpdateInfo {
        extension_id: extension_id.to_string(),
        codebase,
        hash_sha256,
        store_url: store_url.trim().to_string(),
    })
}

async fn download_crx_to_path(download_url: &str, output_path: &Path) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client.get(download_url).send().await?;
    if !response.status().is_success() {
        return Err(format!("下载商店插件失败: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes().await?;
    fs::write(output_path, &bytes)?;
    Ok(())
}

fn update_installation_status(
    app: &AppHandle,
    record_id: &str,
    status: &str,
    keep_record: bool,
) -> Result<LocalExtensionDto> {
    let _guard = REGISTRY_LOCK.lock().map_err(|_| "local extension registry lock poisoned")?;

    let mut registry = read_registry(app)?;
    let now = chrono::Utc::now().to_rfc3339();
    let dto = {
        let record = registry
            .catalog
            .iter_mut()
            .find(|item| item.record_id == record_id)
            .ok_or_else(|| format!("未找到本地插件记录: {record_id}"))?;

        if !Path::new(&record.managed_crx_path).exists() {
            return Err(format!("本地插件文件不存在: {}", record.managed_crx_path).into());
        }

        record.updated_at = now.clone();
        to_dto(record, status)
    };

    match registry
        .installations
        .iter_mut()
        .find(|item| item.record_id == record_id && item.scope == LOCAL_SCOPE)
    {
        Some(installation) => {
            installation.status = status.to_string();
            installation.updated_at = now.clone();
            if keep_record && installation.installed_at.trim().is_empty() {
                installation.installed_at = now.clone();
            }
        }
        None if keep_record => registry.installations.push(LocalExtensionInstallationRecord {
            record_id: record_id.to_string(),
            scope: LOCAL_SCOPE.to_string(),
            status: status.to_string(),
            installed_at: now.clone(),
            updated_at: now.clone(),
        }),
        None => {}
    }

    write_registry(app, &registry)?;
    Ok(LocalExtensionDto {
        status: installation_status(&registry, record_id).to_string(),
        updated_at: now,
        ..dto
    })
}

fn installation_status<'a>(registry: &'a LocalExtensionRegistry, record_id: &str) -> &'a str {
    registry
        .installations
        .iter()
        .find(|item| item.record_id == record_id && item.scope == LOCAL_SCOPE)
        .map(|item| match item.status.as_str() {
            "active" => "active",
            "disabled" => "disabled",
            _ => "available",
        })
        .unwrap_or("available")
}

fn to_dto(record: &LocalExtensionCatalogRecord, status: &str) -> LocalExtensionDto {
    LocalExtensionDto {
        record_id: record.record_id.clone(),
        extension_id: record.extension_id.clone(),
        name: record.name.clone(),
        description: record.description.clone(),
        version: record.version.clone(),
        browser: record.browser.clone(),
        status: status.to_string(),
        source: record.source.clone(),
        author: record.author.clone(),
        homepage: record.homepage.clone(),
        icon_url: record.icon_url.clone(),
        category: record.category.clone(),
        permissions: if record.permissions.is_empty() {
            None
        } else {
            Some(record.permissions.clone())
        },
        hash: Some(record.hash.clone()),
        file_size: Some(record.file_size),
        downloads_count: record.downloads_count,
        rating: record.rating,
        import_state: None,
        imported_at: record.imported_at.clone(),
        updated_at: record.updated_at.clone(),
    }
}

fn with_import_state(mut dto: LocalExtensionDto, import_state: &str) -> LocalExtensionDto {
    dto.import_state = Some(import_state.to_string());
    dto
}

fn registry_path() -> Result<PathBuf> {
    Ok(crate::core::paths::PathManager::get_config_dir()?.join(REGISTRY_FILENAME))
}

fn local_crx_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = crate::core::paths::PathManager::get_local_dir(app)?
        .join("extensions")
        .join("crx");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn read_registry(app: &AppHandle) -> Result<LocalExtensionRegistry> {
    let path = registry_path()?;
    if !path.exists() {
        let registry = LocalExtensionRegistry::default();
        write_registry(app, &registry)?;
        return Ok(registry);
    }

    let content = fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(LocalExtensionRegistry::default());
    }

    Ok(serde_json::from_str(&content)?)
}

fn write_registry(_app: &AppHandle, registry: &LocalExtensionRegistry) -> Result<()> {
    let path = registry_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(registry)?;
    fs::write(path, content)?;
    Ok(())
}

fn parse_crx_metadata(path: &Path) -> Result<ParsedCrxMetadata> {
    let mut archive = open_crx_zip_archive(path)?;
    let mut manifest_file = archive.by_name("manifest.json")?;
    let mut manifest_content = String::new();
    manifest_file.read_to_string(&mut manifest_content)?;
    drop(manifest_file);

    let manifest: Value = serde_json::from_str(&manifest_content)?;
    let name = manifest
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Local Extension")
        .to_string();
    let description = manifest
        .get("description")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    let version = manifest
        .get("version")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("0.0.0")
        .to_string();
    let author = manifest.get("author").and_then(parse_manifest_author);
    let homepage = manifest
        .get("homepage_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    let mut permissions = Vec::new();
    for key in ["permissions", "host_permissions", "optional_permissions"] {
        if let Some(items) = manifest.get(key).and_then(Value::as_array) {
            for value in items {
                if let Some(permission) = value.as_str() {
                    let trimmed = permission.trim();
                    if !trimmed.is_empty() && !permissions.iter().any(|item| item == trimmed) {
                        permissions.push(trimmed.to_string());
                    }
                }
            }
        }
    }

    Ok(ParsedCrxMetadata {
        name,
        description,
        version,
        author,
        homepage,
        permissions,
    })
}

fn parse_manifest_author(value: &Value) -> Option<String> {
    if let Some(author) = value.as_str().map(str::trim).filter(|author| !author.is_empty()) {
        return Some(author.to_string());
    }

    if let Some(author_object) = value.as_object() {
        for key in ["name", "email"] {
            if let Some(author) = author_object
                .get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|author| !author.is_empty())
            {
                return Some(author.to_string());
            }
        }
    }

    None
}

fn open_crx_zip_archive(path: &Path) -> Result<zip::ZipArchive<fs::File>> {
    let mut file = fs::File::open(path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    if &magic != b"Cr24" {
        return Err("不是有效的 CRX3 文件格式".into());
    }

    let mut version_bytes = [0u8; 4];
    file.read_exact(&mut version_bytes)?;
    let version = u32::from_le_bytes(version_bytes);
    if version != 3 {
        return Err(format!("暂不支持的 CRX 版本: {version}").into());
    }

    let mut header_len_bytes = [0u8; 4];
    file.read_exact(&mut header_len_bytes)?;
    let header_len = u32::from_le_bytes(header_len_bytes);
    let zip_offset = 4 + 4 + 4 + header_len as u64;
    file.seek(std::io::SeekFrom::Start(zip_offset))?;

    Ok(zip::ZipArchive::new(file)?)
}
