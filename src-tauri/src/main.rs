// src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 声明模块
mod app_setup;
mod clipboard;
mod db;

use app_setup::{ClipboardSourceState,update_shortcut,update_shortcut2, AppShortcutState, AppShortcutState2};
use arboard::Clipboard;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
fn test_function() -> String {
    "这是来自 Rust 的测试信息".to_string()
}

/// 设置或取消应用的开机自启。作为 Tauri command 暴露给前端调用。
/// # Param
/// app: tauri::AppHandle - Tauri 的应用句柄，用于访问应用相关功能。
/// enable: bool - true表示启用开机自启，false表示禁用。
/// # Returns
/// Result<(), String> - 操作成功则返回 Ok(())，失败则返回包含错误信息的 Err。
#[tauri::command]
async fn set_autostart(app: tauri::AppHandle, enable: bool) -> Result<(), String> {
    let autolaunch = app.autolaunch();
    
    if enable {
        autolaunch
            .enable()
            .map_err(|e| format!("启用开机自启失败: {}", e))?;
    } else {
        autolaunch
            .disable()
            .map_err(|e| format!("禁用开机自启失败: {}", e))?;
    }
    
    Ok(())
}

/// 检查应用是否已设置为开机自启。作为 Tauri command 暴露给前端调用。
/// # Param
/// app: tauri::AppHandle - Tauri 的应用句柄，用于访问应用相关功能。
/// # Returns
/// Result<bool, String> - 操作成功则返回 Ok(bool)，其中 true 表示已启用自启，false 表示未启用。失败则返回包含错误信息的 Err。
#[tauri::command]
async fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let autolaunch = app.autolaunch();
    
    autolaunch
        .is_enabled()
        .map_err(|e| format!("检查自启状态失败: {}", e))
}


#[tauri::command]
fn write_to_clipboard(
    text: String, 
    app_handle: tauri::AppHandle,
    state: State<'_,ClipboardSourceState>
) -> Result<(), String> {
    // 设置标志，表示这是前端触发的复制
    *state.is_frontend_copy.lock().unwrap() = true;
    
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    
    Ok(())
}
/// 将指定的文本写入系统剪贴板。作为 Tauri command 暴露给前端调用。
/// 此函数会设置一个状态标志，以区分是前端主动复制还是由其他程序引起的剪贴板变化。
/// # Param
/// text: String - 需要写入剪贴板的文本内容。
/// app_handle: tauri::AppHandle - Tauri 的应用句柄。
/// state: State<'_,ClipboardSourceState> - 用于管理剪贴板来源状态的 Tauri 状态。
/// # Returns
/// Result<(), String> - 操作成功则返回 Ok(())，失败则返回包含错误信息的 Err。
#[tauri::command]
async fn write_file_to_clipboard(
    app_handle: tauri::AppHandle,
    file_path: String,
    state: State<'_,ClipboardSourceState>
) -> Result<(), String> {
    // 设置标志，表示这是前端触发的复制
    *state.is_frontend_copy.lock().unwrap() = true;
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
    let absolute_path =
        fs::canonicalize(path).map_err(|e| format!("无法获取文件绝对路径: {}", e))?;

    // 根据不同平台调用相应的文件复制方法
    copy_file_to_clipboard(absolute_path)
}


/// 跨平台地将文件复制到系统剪贴板。作为 Tauri command 暴露给前端调用。
/// 此函数会根据编译的目标操作系统（Windows, macOS, Linux）调用相应的底层实现。
/// # Param
/// file_path: PathBuf - 要复制的文件的路径。
/// # Returns
/// Result<(), String> - 操作成功则返回 Ok(())，失败（如路径非法或底层实现出错）则返回包含错误信息的 Err。
#[tauri::command]
fn copy_file_to_clipboard(file_path: PathBuf) -> Result<(), String> {
    let file_path_str = file_path.to_str().ok_or("文件路径包含非法字符")?;

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
    use std::io::Write;
    use std::process::Command;
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
#[tauri::command]
fn get_current_shortcut(state: tauri::State<AppShortcutState>) -> String {
    state.current_shortcut.lock().unwrap().clone()
}

#[tauri::command]
fn get_current_shortcut2(state: tauri::State<AppShortcutState2>) -> String {
    state.current_shortcut.lock().unwrap().clone()
}
fn main() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]), // 可以传递启动参数，这里为空
        ))
        .manage(AppShortcutState {
            current_shortcut: Mutex::new(String::new()),
        })
        .manage(AppShortcutState2 {
            current_shortcut: Mutex::new(String::new()),
        }).manage(ClipboardSourceState { // 新增的状态
            is_frontend_copy: Mutex::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            test_function,
            write_to_clipboard,
            write_file_to_clipboard,
            copy_file_to_clipboard,
            update_shortcut,
            update_shortcut2,
            get_current_shortcut, 
            get_current_shortcut2,
            set_autostart,
            is_autostart_enabled,
            db::insert_received_data,
            db::get_all_data,
            db::get_latest_data,
            db::get_data_by_id,
            db::delete_all_data,
            db::delete_data,
            db::delete_data_by_id,
            db::update_data_content_by_id,
            db::set_favorite_status_by_id,
            db::search_text_content,
            db::add_notes_by_id,
            db::filter_data_by_type,
            db::create_new_folder,
            db::rename_folder,
            db::delete_folder,
            db::get_all_folders,
            db::add_item_to_folder,
            db::remove_item_from_folder,
            db::filter_data_by_folder,
        ])
        .setup(move|app| {
            // 初始化数据库路径
            let app_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            if !app_dir.exists() {
                std::fs::create_dir_all(&app_dir).expect("无法创建应用数据目录");
            }
            let db_path = app_dir.join("smartpaste.db");
            db::set_db_path(db_path);

            // 调试：读取并打印数据库中所有记录
            /*
            match db::get_all_data() {
                Ok(json) => println!("DEBUG get_all_data: {}", json),
                Err(e) => eprintln!("DEBUG get_all_data error: {}", e),
            }
            */
            // 现有快捷键 / 线程 / 文件路径逻辑继续使用 app_dir
            let files_dir = app_dir.join("files");
            std::fs::create_dir_all(&files_dir).unwrap();
            // 设置系统托盘
            app_setup::setup_tray(app)?;

            // 注册全局快捷键
            app_setup::setup_global_shortcuts(app.handle().clone())?;

            // 启动剪贴板监控
            let handle = app.handle().clone();
            app_setup::start_clipboard_monitor(handle);

            // 初始隐藏主窗口，避免启动时闪烁
            if let Some(window) = app.get_webview_window("main") {
                window.hide()?;
            }           

            // 设置主窗口为透明 + 穿透
            if let Some(window) = app.get_webview_window("main") {               
                window.show()?;
            }

            Ok(())
        })
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
