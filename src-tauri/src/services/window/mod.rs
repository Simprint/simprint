use crate::core::error::Result;
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

/// 一键排列计算得到的单个窗口区域
#[derive(Debug, Clone, Serialize)]
pub struct WindowLayoutCell {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// 获取主屏幕的分辨率（像素）
fn get_primary_screen_size() -> (i32, i32) {
    #[cfg(target_os = "windows")]
    {
        // SAFETY: GetSystemMetrics 是线程安全的 Windows API，用于获取系统指标
        unsafe {
            let w = GetSystemMetrics(SM_CXSCREEN);
            let h = GetSystemMetrics(SM_CYSCREEN);
            (w, h)
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        (0, 0)
    }
}

pub struct WindowService;

impl WindowService {
    fn get_webview_data_dir() -> Result<PathBuf> {
        crate::core::paths::PathManager::get_webview_data_dir().map_err(Into::into)
    }

    /// 计算一键排列时的窗口布局
    pub fn calculate_window_layout(window_count: usize) -> Vec<WindowLayoutCell> {
        let (screen_width, screen_height) = get_primary_screen_size();
        if screen_width <= 0 || screen_height <= 0 || window_count == 0 {
            return Vec::new();
        }

        let (rows, cols) = if window_count <= 4 {
            match window_count {
                1 => (1, 1),
                2 => (1, 2),
                3 => (1, 3),
                4 => (2, 2),
                _ => (1, 1),
            }
        } else {
            let cell_count = ((window_count + 3) / 4) * 4;
            let mut rows = (cell_count as f64).sqrt().floor() as usize;
            if rows < 1 {
                rows = 1;
            }
            while cell_count % rows != 0 {
                rows = rows.saturating_sub(1);
                if rows == 0 {
                    rows = 1;
                    break;
                }
            }
            let cols = cell_count / rows;
            let (r, c) = (rows, cols);
            if r > c { (c, r) } else { (r, c) }
        };

        let cell_count = rows * cols;
        let mut cells = Vec::with_capacity(cell_count);

        match (rows, cols) {
            (1, 1) => {
                cells.push(WindowLayoutCell {
                    x: 0,
                    y: 0,
                    width: screen_width,
                    height: screen_height,
                });
            }
            (1, 2) => {
                let w1 = screen_width / 2;
                let w2 = screen_width - w1;
                cells.push(WindowLayoutCell {
                    x: 0,
                    y: 0,
                    width: w1,
                    height: screen_height,
                });
                cells.push(WindowLayoutCell {
                    x: w1,
                    y: 0,
                    width: w2,
                    height: screen_height,
                });
            }
            (1, 3) => {
                let w1 = screen_width / 3;
                let w2 = screen_width / 3;
                let w3 = screen_width - w1 - w2;
                let h = screen_height;

                let mut x = 0;
                for w in [w1, w2, w3] {
                    cells.push(WindowLayoutCell {
                        x,
                        y: 0,
                        width: w,
                        height: h,
                    });
                    x += w;
                }
            }
            (2, 2) => {
                let w1 = screen_width / 2;
                let w2 = screen_width - w1;
                let h1 = screen_height / 2;
                let h2 = screen_height - h1;

                cells.push(WindowLayoutCell {
                    x: 0,
                    y: 0,
                    width: w1,
                    height: h1,
                });
                cells.push(WindowLayoutCell {
                    x: w1,
                    y: 0,
                    width: w2,
                    height: h1,
                });
                cells.push(WindowLayoutCell {
                    x: 0,
                    y: h1,
                    width: w1,
                    height: h2,
                });
                cells.push(WindowLayoutCell {
                    x: w1,
                    y: h1,
                    width: w2,
                    height: h2,
                });
            }
            _ => {
                let cell_width = screen_width / (cols as i32);
                let cell_height = screen_height / (rows as i32);

                let mut y = 0;
                for _row in 0..rows {
                    let mut x = 0;
                    for _col in 0..cols {
                        cells.push(WindowLayoutCell {
                            x,
                            y,
                            width: cell_width,
                            height: cell_height,
                        });
                        x += cell_width;
                    }
                    y += cell_height;
                }
            }
        }

        cells
    }

    /// 一键排列：根据选中的环境 UUID 列表，为每个环境计算并下发窗口布局指令
    pub async fn arrange_environments(env_ids: Vec<String>) -> Result<()> {
        if env_ids.is_empty() {
            return Ok(());
        }

        let connected_env_ids =
            crate::services::environment::KernelService::get_connected_environments().await?;
        let connected_env_set: std::collections::HashSet<_> =
            connected_env_ids.into_iter().collect();

        let mut connected_envs = Vec::new();
        for env_id in env_ids.into_iter() {
            if connected_env_set.contains(&env_id) {
                connected_envs.push(env_id);
            }
        }

        let count = connected_envs.len();
        if count == 0 {
            return Ok(());
        }

        let cells = Self::calculate_window_layout(count);
        if cells.is_empty() {
            return Err("无法计算窗口布局".into());
        }

        for (index, env_id) in connected_envs.into_iter().enumerate() {
            if index >= cells.len() {
                break;
            }
            let cell = &cells[index];

            crate::services::environment::KernelService::set_window_bounds(
                env_id,
                cell.x,
                cell.y,
                cell.width,
                cell.height,
            )
            .await?;
        }

        Ok(())
    }

    /// 获取窗口的宽高
    pub fn get_window_size(app_handle: &AppHandle) -> (f64, f64) {
        if let Some(window) = app_handle.get_webview_window("main") {
            if let Ok(size) = window.inner_size() {
                (size.width as f64, size.height as f64)
            } else {
                (0.0, 0.0)
            }
        } else {
            (0.0, 0.0)
        }
    }

    /// 隐藏主窗口
    pub fn hide_window(app_handle: &AppHandle) -> Result<()> {
        if let Some(window) = app_handle.get_webview_window("main") {
            window.hide()?;
            Ok(())
        } else {
            Err("无法找到主窗口".into())
        }
    }

    /// 显示主窗口
    pub fn show_window(app_handle: &AppHandle) -> Result<()> {
        if let Some(window) = app_handle.get_webview_window("main") {
            window.show()?;
            window.set_focus()?;
            Ok(())
        } else {
            Err("无法找到主窗口".into())
        }
    }

    /// 创建主窗口
    pub fn create_main_window(app_handle: &AppHandle) -> Result<()> {
        if app_handle.get_webview_window("main").is_some() {
            log::debug!("主窗口已存在，跳过创建");
            return Ok(());
        }

        let build_started_at = std::time::Instant::now();
        log::info!("Main window build started");

        #[cfg(feature = "production")]
        let devtools_enabled = false;
        #[cfg(not(feature = "production"))]
        let devtools_enabled = true;

        let _window =
            WebviewWindowBuilder::new(app_handle, "main", WebviewUrl::App("index.html".into()))
                .data_directory(Self::get_webview_data_dir()?)
                .title("Simprint")
                .inner_size(1438.0, 828.0)
                .min_inner_size(1038.0, 688.0)
                .resizable(true)
                .maximizable(true)
                .fullscreen(false)
                .center()
                .decorations(false)
                .visible(false)
                .drag_and_drop(false)
                .devtools(devtools_enabled)
                .build()?;

        log::info!(
            "Main window built in {:.1} ms",
            build_started_at.elapsed().as_secs_f64() * 1_000.0
        );

        log::trace!("主窗口已创建");
        Ok(())
    }

    /// 创建同步器窗口
    pub async fn create_syncer_window(app_handle: &AppHandle) -> Result<()> {
        if let Some(window) = app_handle.get_webview_window("syncer") {
            log::debug!("同步器窗口已存在，将其带到前台");
            window.show()?;
            window.set_focus()?;
            return Ok(());
        }

        #[cfg(feature = "production")]
        let devtools_enabled = false;

        #[cfg(not(feature = "production"))]
        let devtools_enabled = true;

        let window =
            WebviewWindowBuilder::new(app_handle, "syncer", WebviewUrl::App("syncer.html".into()))
                .data_directory(Self::get_webview_data_dir()?)
                .title("同步器")
                .inner_size(400.0, 600.0)
                .resizable(true)
                .decorations(false)
                .drag_and_drop(false)
                .devtools(devtools_enabled)
                .build()?;

        window.show()?;
        window.set_focus()?;

        Ok(())
    }
}
