use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use config::Config;
use serde::Deserialize;

// 引用配置加密模块（与运行时共享）
#[path = "src/core/config/encryption/key_derivation.rs"]
mod key_derivation;

#[path = "src/core/config/encryption/crypto.rs"]
mod crypto;

// =============================================================================
// 入口：构建脚本执行流程
// =============================================================================

fn main() {
    // 1. 仅在生产环境下下载 / 准备 webview-fixed 目录中的资源
    #[cfg(feature = "production")]
    {
        webview_assets::ensure_webview_fixed_downloaded();
    }

    // 2. 构建 Tauri 应用（处理 Windows manifest / 权限等）
    tauri_build_pipeline::build_tauri();

    // 3. 为前端构建写入环境标记文件（.build-env）
    frontend_env::prepare_frontend_build_env();

    // 4. 读取明文配置并生成加密后的二进制配置文件
    config_encrypt::generate_encrypted_config();
}

// =============================================================================
// 构建环境推导（供多个子模块复用）
// =============================================================================

/// 根据启用的 Cargo feature 推导出构建环境名
///
/// - 若启用 `test` feature -> "test"
/// - 若启用 `development` feature -> "development"
/// - 若启用 `production` feature 或未启用任何环境特性 -> "production"
///
/// 根据启用的 Cargo feature 推导出构建环境名
///
/// - 若启用 `test` feature -> "test"
/// - 若启用 `development` feature -> "development"
/// - 若启用 `production` feature 或未启用任何环境特性 -> "production"
pub(crate) fn detect_build_env_name() -> &'static str {
    if cfg!(feature = "test") {
        "test"
    } else if cfg!(feature = "development") {
        "development"
    } else {
        "production"
    }
}

/// 根据当前构建环境返回对应的配置文件名
pub(crate) fn current_config_file_name() -> &'static str {
    match detect_build_env_name() {
        "test" => "config.test.toml",
        "development" => "config.development.toml",
        _ => "config.production.toml",
    }
}

// =============================================================================
// 模块一：Webview 资源下载与解压
// =============================================================================

mod webview_assets {
    use super::*;

    /// Webview 配置结构体（用于 build.rs 中解析）
    #[derive(Deserialize)]
    struct WebviewConfig {
        /// 下载的 URL（注意：字段名保持与配置文件中的拼写一致：downlaod_url）
        #[serde(rename = "downlaod_url")]
        download_url: String,
    }

    /// 确保 `webview-fixed` 目录已经从远端 ZIP 包解压完成
    ///
    /// - 若目录已存在，则直接跳过，不重复下载
    /// - 若目录不存在，则从指定 URL 下载 zip 并解压到 `webview-fixed/`
    pub fn ensure_webview_fixed_downloaded() {
        let target_dir = Path::new("webview-fixed");

        // 若目录已存在，则认为资源已经就绪，避免每次构建都重新下载
        if target_dir.exists() {
            return;
        }

        // 优先尝试从当前环境的配置文件中读取下载地址
        let url = detect_webview_download_url().unwrap_or_else(|| {
            panic!(
                "[BUILD ERROR] Failed to detect webview download URL from config file '{}'.\n\
                 Please ensure the config file contains a valid [webview] section with 'downlaod_url' field.",
                super::current_config_file_name()
            );
        });

        if let Err(err) = download_and_extract_webview_fixed(&url, target_dir.to_path_buf()) {
            // 构建脚本失败时直接 panic，阻止继续构建，以避免产生不完整的产物
            panic!("failed to download and extract webview-fixed assets: {err}");
        }
    }

    /// 从当前构建环境对应的 `config.<env>.toml` 中解析 `[webview]` 段的 `downlaod_url`
    ///
    /// 使用 config crate 进行 TOML 解析，替代手动字符串解析，提高可靠性和可维护性。
    /// 解析失败时返回 `None`，由调用方决定是否回退到默认值。
    fn detect_webview_download_url() -> Option<String> {
        let config_file_name = super::current_config_file_name();

        // 使用 config crate 解析 TOML 文件
        let config = Config::builder()
            .add_source(config::File::with_name(config_file_name))
            .build()
            .map_err(|e| {
                eprintln!(
                    "[BUILD ERROR] Failed to load config file '{}': {}",
                    config_file_name, e
                );
                e
            })
            .ok()?;

        // 尝试获取 webview 配置段
        let webview_config: WebviewConfig = config
            .get("webview")
            .map_err(|e| {
                eprintln!(
                    "[BUILD ERROR] Failed to parse [webview] section in '{}': {}",
                    config_file_name, e
                );
                e
            })
            .ok()?;

        Some(webview_config.download_url)
    }

    /// 从远程下载 webview-fixed.zip 并解压到指定目录
    fn download_and_extract_webview_fixed(
        url: &str,
        target_dir: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::{self, Cursor};

        // 确保目标目录存在
        if !target_dir.exists() {
            fs::create_dir_all(&target_dir)?;
        }

        println!(
            "cargo:warning=Downloading webview-fixed assets from {}",
            url
        );

        // 使用 blocking 客户端下载 ZIP 文件（build.rs 不能是 async）
        let response = reqwest::blocking::get(url)?;
        if !response.status().is_success() {
            return Err(format!("download failed, status: {}", response.status()).into());
        }

        let mut bytes: Vec<u8> = Vec::new();
        let mut reader = response;
        reader.copy_to(&mut bytes)?;

        // 使用 zip crate 解压缩
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name();

            // 大多数发布包会在 ZIP 内部自带一层 `webview-fixed/` 根目录。
            // 为避免解压后出现 `webview-fixed/webview-fixed/...` 的双层目录，
            // 这里如果发现路径以 `webview-fixed/` 开头，就去掉这一级目录。
            let relative_name = file_name.strip_prefix("webview-fixed/").unwrap_or(file_name);

            let mut out_path = target_dir.clone();
            out_path.push(relative_name);

            if file_name.ends_with('/') || relative_name.is_empty() {
                // 目录条目
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let mut outfile = File::create(&out_path)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }
}

// =============================================================================
// 模块三：配置加密（将 TOML 加工为加密二进制）
// =============================================================================

mod config_encrypt {
    use super::*;

    /// 从 config.toml 生成加密后的二进制配置文件
    pub fn generate_encrypted_config() {
        // 不同环境使用不同的配置文件
        // 当对应的配置文件发生变化时重新运行构建脚本
        println!("cargo:rerun-if-changed=config.development.toml");
        println!("cargo:rerun-if-changed=config.test.toml");
        println!("cargo:rerun-if-changed=config.production.toml");

        // 根据当前构建环境选择对应的配置文件
        let config_file_name = current_config_file_name();

        // 读取配置文件并生成加密的二进制文件，避免在可执行文件中直接出现明文配置
        let out_dir = env::var("OUT_DIR").unwrap_or_else(|e| {
            panic!(
                "[BUILD ERROR] OUT_DIR environment variable is not set: {}\n\
                 This build script must be run by Cargo, not directly.",
                e
            );
        });

        let config_path = Path::new(config_file_name);
        let config_bytes = fs::read(config_path).unwrap_or_else(|e| {
            panic!(
                "[BUILD ERROR] Failed to read config file '{}': {}\n\
                 Please ensure the config file exists and is readable.",
                config_path.display(),
                e
            );
        });

        std::str::from_utf8(&config_bytes).unwrap_or_else(|e| {
            panic!(
                "[BUILD ERROR] Config file '{}' is not valid UTF-8: {}\n\
                 Please ensure workflow/local scripts write this file with UTF-8 encoding.",
                config_path.display(),
                e
            );
        });

        let encrypted = crypto::encrypt(&config_bytes).expect("Failed to encrypt config");

        let out_path = Path::new(&out_dir).join("config_encrypted.bin");
        println!(
            "cargo:warning=Config encrypted file path: {}",
            out_path.display()
        );
        fs::write(&out_path, &encrypted).unwrap_or_else(|e| {
            panic!(
                "[BUILD ERROR] Failed to write encrypted config to '{}': {}\n\
                 Please check write permissions for OUT_DIR.",
                out_path.display(),
                e
            );
        });
    }
}

// =============================================================================
// 模块四：前端构建环境标记（.build-env）
// =============================================================================

mod frontend_env {
    use super::*;

    /// 准备前端构建所需的环境标记文件
    pub fn prepare_frontend_build_env() {
        // 根据当前启用的 feature，推导出前端构建使用的环境名称
        let env_name = detect_build_env_name();
        write_frontend_env_hint(env_name);
    }

    /// 将推导出的构建环境名写到前端目录，供前端构建脚本读取
    fn write_frontend_env_hint(env_name: &str) {
        // 前端在 .. 目录下
        let hint_path = Path::new("..").join(".build-env");

        if let Some(parent) = hint_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "[BUILD WARNING] Failed to create parent directory for '{}': {}",
                    hint_path.display(),
                    e
                );
            }
        }

        if let Err(e) = fs::write(&hint_path, env_name.as_bytes()) {
            // 失败不会中断构建，但会输出一条提示，前端将退回到默认 production
            println!("cargo:warning=failed to write frontend build env hint: {e}");
        }
    }
}

// =============================================================================
// 模块五：Tauri 应用构建（Windows manifest / 权限等）
// =============================================================================

mod tauri_build_pipeline {
    /// 构建 tauri 应用，处理不同平台下的 manifest / 权限等
    pub fn build_tauri() {
        // 开发环境不需要管理员权限，发布环境需要管理员权限
        #[cfg(target_os = "windows")]
        {
            // let is_dev = cfg!(debug_assertions);
            let is_dev = true; // 暂时跳过软件管理员申请，再后续评估再决定是否需要管理员。

            if !is_dev {
                // 发布环境：需要管理员权限. (updater.exe manifest)
                embed_resource::compile("windows/updater.rc", embed_resource::NONE);

                // 发布环境：需要管理员权限. (主程序manifest)
                let manifest = include_str!("windows/main.manifest");
                let window_attributes =
                    tauri_build::WindowsAttributes::new().app_manifest(manifest);
                let attrs = tauri_build::Attributes::new().windows_attributes(window_attributes);
                tauri_build::try_build(attrs).unwrap_or_else(|e| {
                    panic!(
                        "[BUILD ERROR] Tauri build failed: {}\n\
                         Please check your Tauri configuration and dependencies.",
                        e
                    );
                });
            } else {
                // 开发环境：不需要管理员权限. (updater.exe manifest)
                tauri_build::build();
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            tauri_build::build();
        }
    }
}
