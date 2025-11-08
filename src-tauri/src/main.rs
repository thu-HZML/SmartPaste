// src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 声明模块
mod app_setup;
mod clipboard;
mod db;


use tauri::Manager;
use arboard::Clipboard;
use std::fs;
use std::path::{Path, PathBuf};

#[tauri::command]
fn test_function() -> String {
    "这是来自 Rust 的测试信息".to_string()
}
#[tauri::command]
fn write_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn write_file_to_clipboard(
    app_handle: tauri::AppHandle,
    file_path: String,
) -> Result<(), String> {
    let path = Path::new(&file_path);
    
    // 检查文件是否存在
    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }
    
    // 检查是否是文件（不是目录）
    if !path.is_file() {
        return Err("路径指向的不是文件".to_string());
    }
    
    // 获取文件的绝对路径
    let absolute_path = fs::canonicalize(path)
        .map_err(|e| format!("无法获取文件绝对路径: {}", e))?;
    
    // 根据不同平台调用相应的文件复制方法
    copy_file_to_clipboard(absolute_path)
}
// 跨平台文件复制到剪贴板
#[tauri::command]
fn copy_file_to_clipboard(file_path: PathBuf) -> Result<(), String> {
    let file_path_str = file_path.to_str()
        .ok_or("文件路径包含非法字符")?;

    #[cfg(target_os = "windows")]
    {
        copy_file_to_clipboard_windows(file_path_str)
    }
    
    #[cfg(target_os = "macos")]
    {
        copy_file_to_clipboard_macos(file_path_str)
    }
    
    #[cfg(target_os = "linux")]
    {
        copy_file_to_clipboard_linux(file_path_str)
    }
}

#[cfg(target_os = "windows")]
fn copy_file_to_clipboard_windows(file_path: &str) -> Result<(), String> {
    use std::process::Command;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    let ps_script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Clipboard]::SetFileDropList(@('{}'))",
        file_path.replace("'", "''")
    );
    
    let output = Command::new("powershell")
        .args(&["-Command", &ps_script])
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        return Ok(());
    }
    
    Err("复制文件到剪贴板失败".to_string())
}

#[cfg(target_os = "macos")]
fn copy_file_to_clipboard_macos(file_path: &str) -> Result<(), String> {
    use std::process::Command;
    
    // 使用AppleScript复制文件
    let apple_script = format!(
        "set the clipboard to POSIX file \"{}\"",
        file_path.replace("\"", "\\\"")
    );
    
    let output = Command::new("osascript")
        .args(&["-e", &apple_script])
        .output()
        .map_err(|e| e.to_string())?;
    
    if output.status.success() {
        return Ok(());
    }
    
    Err("复制文件到剪贴板失败".to_string())
}

#[cfg(target_os = "linux")]
fn copy_file_to_clipboard_linux(file_path: &str) -> Result<(), String> {
    use std::process::Command;
    
    // Linux上的文件复制比较复杂，尝试多种方法
    
    // 方法1: 使用xclip复制文件URI
    let file_uri = format!("file://{}", file_path);
    let output = Command::new("xclip")
        .args(&["-selection", "clipboard", "-t", "text/uri-list"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?
        .stdin
        .unwrap()
        .write_all(file_uri.as_bytes())
        .map_err(|e| e.to_string())?;
    
    // 检查xclip是否成功
    if Command::new("xclip")
        .args(&["-selection", "clipboard", "-o"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Ok(());
    }

    
    Err("Linux系统文件复制功能受限，请确保已安装xclip".to_string())
}


fn main() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            test_function,
            write_to_clipboard,
            write_file_to_clipboard,
            copy_file_to_clipboard,
            db::insert_received_data,
            db::get_all_data,
            db::get_latest_data,
            db::get_data_by_id,
            db::delete_data,
            db::delete_data_by_id,
            db::set_favorite_status_by_id,
            db::search_text_content,
            db::add_notes_by_id,
            db::filter_data_by_type,
            db::create_new_folder,
            db::rename_folder,
            db::delete_folder,
            db::add_item_to_folder,
            db::remove_item_from_folder,
            db::filter_data_by_folder,
        ])
        .setup(|app| {
            // 初始化数据库路径
            let app_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            if !app_dir.exists() {
                std::fs::create_dir_all(&app_dir).expect("无法创建应用数据目录");
            }
            let db_path = app_dir.join("smartpaste.db");
            db::set_db_path(db_path);

            // 调试：读取并打印数据库中所有记录
            match db::get_all_data() {
                Ok(json) => println!("DEBUG get_all_data: {}", json),
                Err(e) => eprintln!("DEBUG get_all_data error: {}", e),
            }

            // 现有快捷键 / 线程 / 文件路径逻辑继续使用 app_dir
            let files_dir = app_dir.join("files");
            std::fs::create_dir_all(&files_dir).unwrap();
            // 设置系统托盘
            app_setup::setup_tray(app)?;

            // 注册全局快捷键
            app_setup::setup_global_shortcuts(app.handle().clone())?;

            // 启动剪贴板监控
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, PhysicalPosition,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

// ✅ 新增命令：动态设置窗口鼠标穿透
#[tauri::command]
fn set_mouse_passthrough(passthrough: bool, window: tauri::Window, state: tauri::State<'_, AppState>) {
    let mut is_passthrough = state.is_passthrough.lock().unwrap();
    
    if let Err(e) = window.set_ignore_cursor_events(passthrough) {
        eprintln!("⚠️ 设置鼠标穿透失败: {:?}", e);
    } else {
        *is_passthrough = passthrough;
        println!(
            "🎯 已设置窗口鼠标穿透状态为: {}",
            if passthrough { "开启" } else { "关闭" }
        );
    }
}

#[derive(Default)]
struct AppState {
    pet_position: Mutex<PhysicalPosition<f64>>,
    pet_size: Mutex<(f64, f64)>,
    is_passthrough: Mutex<bool>, // 跟踪当前穿透状态
}

#[tauri::command]
fn update_pet_position(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
) {
    let mut pet_pos = state.pet_position.lock().unwrap();
    let mut pet_size = state.pet_size.lock().unwrap();
    
    *pet_pos = PhysicalPosition::new(x, y);
    *pet_size = (width, height);
    
    println!("📌 更新桌宠位置: ({}, {}), 大小: {}x{}", x, y, width, height);
}

fn main() {
    // 防抖控制点击频率
    let last_click_time = Arc::new(Mutex::new(Instant::now()));
    let app_state = Arc::new(AppState::default());

    let result = tauri::Builder::default()
        .manage(app_state.clone())
        .setup(move |app| {
            let click_time_clone = Arc::clone(&last_click_time);

            // 创建托盘菜单
            let menu = Menu::new(app)?;
            let show_hide = MenuItem::with_id(app, "show_hide", "显示/隐藏", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            menu.append(&show_hide)?;
            menu.append(&quit)?;

            // 托盘图标
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("SmartPaste")
                .on_menu_event(move |app, event| {
                    println!("🖱️ 菜单项点击: {}", event.id().as_ref());
                    if let Some(window) = app.get_webview_window("main") {
                        match event.id().as_ref() {
                            "show_hide" => toggle_window_visibility(&window),
                            "quit" => {
                                println!("🚪 退出应用");
                                std::process::exit(0);
                            }
                            _ => {}
                        }
                    }
                })
                .on_tray_icon_event(move |tray, event| {
                    match event {
                        TrayIconEvent::Click { button, .. } => {
                            let now = Instant::now();
                            let mut last_time = click_time_clone.lock().unwrap();

                            // 防抖：200ms 内的重复点击忽略
                            if now.duration_since(*last_time) < Duration::from_millis(200) {
                                println!("⏰ 忽略重复点击");
                                return;
                            }
                            *last_time = now;

                            println!("🎯 托盘点击事件: {:?}", button);
                            match button {
                                tauri::tray::MouseButton::Left => {
                                    if let Some(window) = tray.app_handle().get_webview_window("main")
                                    {
                                        toggle_window_visibility(&window);
                                    }
                                }
                                tauri::tray::MouseButton::Right => {
                                    println!("📋 右键点击，显示菜单");
                                }
                                _ => {}
                            }
                        }
                        TrayIconEvent::DoubleClick { .. } => {
                            println!("🖱️ 托盘双击事件");
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                toggle_window_visibility(&window);
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            println!("✅ 托盘图标创建成功");

            // 设置主窗口为透明 + 穿透
            if let Some(window) = app.get_webview_window("main") {
                
                window.show()?;
            }

            // 全局快捷键 Alt+Shift+V 显示/隐藏窗口
            let show_hide_shortcut =
                Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyV);
            let shortcut_for_handler = show_hide_shortcut.clone();
            let handle = app.handle().clone();
            app_setup::start_clipboard_monitor(handle);

            // 初始隐藏主窗口，避免启动时闪烁
            if let Some(window) = app.get_webview_window("main") {
                window.hide()?;
            }

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if shortcut == &shortcut_for_handler {
                            if event.state() == ShortcutState::Pressed {
                                println!("⌨️ Alt+Shift+V 被按下，切换窗口可见性");
                                if let Some(window) = handle.get_webview_window("main") {
                                    toggle_window_visibility(&window);
                                }
                            }
                        }
                    })
                    .build(),
            )?;

            app.global_shortcut().register(show_hide_shortcut)?;
            println!("✅ 已注册全局快捷键 Alt+Shift+V");
            //start_mouse_detection(app.handle().clone(), app_state.clone());

            Ok(())
        })
        // ✅ 注册前端命令
        .invoke_handler(tauri::generate_handler![set_mouse_passthrough, update_pet_position])
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("❌ 启动 Tauri 应用失败: {:?}", e);
    }
}


// 辅助函数：切换窗口显示/隐藏
fn toggle_window_visibility(window: &tauri::WebviewWindow) {
    match window.is_visible() {
        Ok(visible) => {
            if visible {
                if let Err(e) = window.hide() {
                    eprintln!("❌ 隐藏窗口失败: {:?}", e);
                } else {
                    println!("👻 隐藏桌宠窗口");
                }
            } else {
                if let Err(e) = window.show() {
                    eprintln!("❌ 显示窗口失败: {:?}", e);
                } else {
                    println!("👀 显示桌宠窗口");
                }
            }
        }
        Err(e) => eprintln!("❌ 获取窗口可见性失败: {:?}", e),
    }
}
