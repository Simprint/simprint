use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use config::Config;
use serde::Deserialize;

// =============================================================================
// 入口：构建脚本执行流程
// =============================================================================

fn main() {
    println!("cargo:rerun-if-env-changed=SIMPRINT_WEBVIEW_MODE");
    println!("cargo:rerun-if-changed=tauri.conf.json");

    let webview_mode =
        env::var("SIMPRINT_WEBVIEW_MODE").unwrap_or_else(|_| "embedBootstrapper".to_string());
    validate_selected_tauri_config(&webview_mode);
    println!("cargo:rustc-env=SIMPRINT_WEBVIEW_MODE={webview_mode}");

    // 1. 仅在生产环境下下载 / 准备 webview-fixed 目录中的资源
    #[cfg(feature = "production")]
    {
        if webview_mode == "fixed-runtime" {
            webview_assets::ensure_webview_fixed_downloaded();
        }
    }

    // 2. 构建 Tauri 应用（处理 Windows manifest / 权限等）
    tauri_build_pipeline::build_tauri();

    // 3. 为前端构建写入环境标记文件（.build-env）
    frontend_env::prepare_frontend_build_env();
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectedTauriConfig {
    bundle: SelectedBundleConfig,
    plugins: SelectedPluginConfig,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectedBundleConfig {
    create_updater_artifacts: bool,
    windows: SelectedWindowsConfig,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectedWindowsConfig {
    webview_install_mode: SelectedWebviewInstallMode,
}

#[derive(Deserialize)]
struct SelectedWebviewInstallMode {
    #[serde(rename = "type")]
    kind: String,
    path: Option<String>,
}

#[derive(Deserialize)]
struct SelectedPluginConfig {
    updater: SelectedUpdaterConfig,
}

#[derive(Deserialize)]
struct SelectedUpdaterConfig {
    endpoints: Vec<String>,
}

fn validate_selected_tauri_config(mode: &str) {
    let raw = fs::read_to_string("tauri.conf.json")
        .unwrap_or_else(|err| panic!("failed to read selected Tauri config: {err}"));
    let config: SelectedTauriConfig = serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("failed to parse selected Tauri config: {err}"));

    let (expected_install_mode, expected_manifest) = match mode {
        "embedBootstrapper" => ("embedBootstrapper", "latest.json"),
        "fixed-runtime" => ("fixedRuntime", "latest-fixed.json"),
        other => panic!(
            "unsupported SIMPRINT_WEBVIEW_MODE '{other}'; expected embedBootstrapper or fixed-runtime"
        ),
    };

    assert_eq!(
        config.bundle.windows.webview_install_mode.kind, expected_install_mode,
        "selected Tauri config does not match SIMPRINT_WEBVIEW_MODE '{mode}'"
    );
    assert!(
        config.bundle.create_updater_artifacts,
        "selected Tauri config must create signed updater artifacts"
    );

    let endpoint = config
        .plugins
        .updater
        .endpoints
        .first()
        .unwrap_or_else(|| panic!("selected Tauri config is missing an updater endpoint"));
    assert!(
        endpoint.ends_with(expected_manifest),
        "updater endpoint for mode '{mode}' must end with '{expected_manifest}'"
    );

    if mode == "fixed-runtime" {
        let expected_runtime_directory = fixed_runtime_directory_for_target_arch();
        let configured_path = config
            .bundle
            .windows
            .webview_install_mode
            .path
            .as_deref()
            .unwrap_or_else(|| panic!("fixed-runtime config is missing its WebView path"));
        let normalized_path = configured_path.replace('\\', "/");

        assert!(
            normalized_path.trim_end_matches('/').ends_with(expected_runtime_directory),
            "fixed-runtime path '{configured_path}' does not match target architecture directory '{expected_runtime_directory}'"
        );
    }
}

fn fixed_runtime_directory_for_target_arch() -> &'static str {
    match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86_64") => "Microsoft.WebView2.FixedVersionRuntime.151.0.4129.78.x64",
        Ok("aarch64") => "Microsoft.WebView2.FixedVersionRuntime.151.0.4129.78.arm64",
        Ok("x86") => "Microsoft.WebView2.FixedVersionRuntime.151.0.4129.78.x86",
        Ok(arch) => panic!("unsupported Windows target architecture '{arch}' for fixed-runtime"),
        Err(err) => panic!("CARGO_CFG_TARGET_ARCH is unavailable: {err}"),
    }
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
        x86_64_download_url: String,
        aarch64_download_url: String,
        x86_download_url: String,
    }

    struct TargetWebview {
        download_url: String,
        runtime_directory: &'static str,
    }

    /// 确保 `webview-fixed` 目录已经从远端 ZIP 包解压完成
    ///
    /// - 若当前目标架构的运行时目录已存在，则直接跳过
    /// - 否则从该架构的 URL 下载 zip，并解压到共享的 `webview-fixed/`
    pub fn ensure_webview_fixed_downloaded() {
        println!(
            "cargo:rerun-if-changed={}",
            super::current_config_file_name()
        );
        let target_dir = Path::new("webview-fixed");

        // 优先尝试从当前环境的配置文件中读取下载地址
        let target_webview = detect_target_webview().unwrap_or_else(|| {
            panic!(
                "[BUILD ERROR] Failed to detect the target WebView runtime from config file '{}'.\n\
                 Please ensure [webview] contains download URLs for x86_64, aarch64 and x86.",
                super::current_config_file_name()
            );
        });

        // 三种架构的运行时可以共存；仅当当前目标架构的目录已存在时才跳过下载。
        if target_dir.join(target_webview.runtime_directory).exists() {
            return;
        }

        if let Err(err) = download_and_extract_webview_fixed(
            &target_webview.download_url,
            target_dir.to_path_buf(),
        ) {
            // 构建脚本失败时直接 panic，阻止继续构建，以避免产生不完整的产物
            panic!("failed to download and extract webview-fixed assets: {err}");
        }

        if !target_dir.join(target_webview.runtime_directory).exists() {
            panic!(
                "downloaded WebView archive does not contain expected runtime directory '{}'",
                target_webview.runtime_directory
            );
        }
    }

    /// 从当前构建目标和 `config.<env>.toml` 中选择对应的 WebView 固定运行时。
    ///
    /// 使用 config crate 进行 TOML 解析，替代手动字符串解析，提高可靠性和可维护性。
    /// 解析失败时返回 `None`，由调用方决定是否回退到默认值。
    fn detect_target_webview() -> Option<TargetWebview> {
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

        match env::var("CARGO_CFG_TARGET_ARCH").ok()?.as_str() {
            "x86_64" => Some(TargetWebview {
                download_url: webview_config.x86_64_download_url,
                runtime_directory: super::fixed_runtime_directory_for_target_arch(),
            }),
            "aarch64" => Some(TargetWebview {
                download_url: webview_config.aarch64_download_url,
                runtime_directory: super::fixed_runtime_directory_for_target_arch(),
            }),
            "x86" => Some(TargetWebview {
                download_url: webview_config.x86_download_url,
                runtime_directory: super::fixed_runtime_directory_for_target_arch(),
            }),
            arch => {
                eprintln!("[BUILD ERROR] Unsupported Windows target architecture: {arch}");
                None
            }
        }
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
// 模块三：前端构建环境标记（.build-env）
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
// 模块四：Tauri 应用构建（Windows manifest / 权限等）
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
                // 开发环境：不需要管理员权限。
                tauri_build::build();
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            tauri_build::build();
        }
    }
}
