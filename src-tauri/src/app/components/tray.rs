use tauri::image::Image;
use tauri::menu::MenuEvent;
use tauri::{
    App,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
};
use tauri::{AppHandle, Manager};

/// 获取tary 图标
pub fn get_icon() -> Result<Image<'static>, tauri::Error> {
    let bytes = include_bytes!("../../../icons/tray-icon.png");
    Image::from_bytes(bytes)
}

// 获取tray 提示文本
pub fn tooltip_text(app: &App) -> String {
    app.config().product_name.clone().unwrap_or("simprint".to_string())
}

/// 创建tray 菜单
pub fn menu(app: &mut App) -> Result<TrayIcon, tauri::Error> {
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    let icon = get_icon()?;

    let tray = TrayIconBuilder::new()
        .on_menu_event(menu_handler)
        .on_tray_icon_event(tray_handler)
        .tooltip(tooltip_text(app))
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .build(app)?;

    Ok(tray)
}

/// 检查应用是否已初始化完成
fn is_app_ready() -> bool {
    let app_init_state = crate::app::init_state::read_app_init_state();
    app_init_state.is_initialized && !app_init_state.is_updating
}

/// 菜单事件处理
pub fn menu_handler(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        "show" => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                if is_app_ready() {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                } else {
                    // 应用未准备好，显示提示或忽略
                    println!("应用正在初始化中，请稍候...");
                }
            });
        }
        "quit" => {
            println!("quit menu item was clicked");
            app.cleanup_before_exit();
            std::process::exit(0);
        }
        _ => {
            println!("menu item {:?} not handled", event.id);
        }
    }
}

/// 托盘事件处理
pub fn tray_handler(tray: &TrayIcon, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => {
            let app = tray.app_handle();
            let app_handle = app.clone();

            tauri::async_runtime::spawn(async move {
                if is_app_ready() {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        if window.is_minimized().expect("get window minimized failed") {
                            let _ = window.unminimize();
                        }
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                } else {
                    println!("应用正在初始化中，请稍候...");
                }
            });
        }
        _ => {}
    }
}
