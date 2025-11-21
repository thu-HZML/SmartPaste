use crate::clipboard::ClipboardItem;
use crate::config::{self, CONFIG};
use crate::db;
use crate::ocr;
use chrono::Utc;
use image::ColorType;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{
    GlobalShortcutExt, Shortcut, ShortcutState as PluginShortcutState,
};
use uuid::Uuid;
pub struct ClipboardSourceState {
    pub is_frontend_copy: Mutex<bool>,
}
pub struct AppShortcutState {
    pub current_shortcut: Mutex<String>,
}
pub struct AppShortcutState2 {
    pub current_shortcut: Mutex<String>,
}

/// 从 Config 中加载主快捷键配置
/// 不再需要 handle 参数来找路径，但为了保持函数签名兼容性或方便后续修改，可以留着或去掉
fn load_shortcut_from_storage(_handle: &AppHandle) -> String {
    if let Some(lock) = CONFIG.get() {
        let cfg = lock.read().unwrap();
        cfg.global_shortcut.clone()
    } else {
        "Alt+Shift+V".to_string()
    }
}

/// 从 Config 中加载第二个界面的快捷键配置
fn load_shortcut_from_storage2(_handle: &AppHandle) -> String {
    if let Some(lock) = CONFIG.get() {
        let cfg = lock.read().unwrap();
        cfg.global_shortcut_2.clone()
    } else {
        "Alt+Shift+C".to_string()
    }
}

/// 将主快捷键保存到 Config
fn save_shortcut_to_storage(_handle: &AppHandle, shortcut: &str) {
    config::set_global_shortcut_internal(shortcut.to_string());
}

/// 将第二个快捷键保存到 Config
fn save_shortcut_to_storage2(_handle: &AppHandle, shortcut: &str) {
    config::set_global_shortcut_2_internal(shortcut.to_string());
}
/// 动态更新并注册应用的主全局快捷键。作为 Tauri command 暴露给前端调用。
///
/// 该函数会执行以下操作：
/// 1. 从状态中获取并注销当前已注册的快捷键。
/// 2. 尝试注册用户提供的新快捷键。
/// 3. 如果注册失败（例如快捷键已被占用），则会尝试恢复注册旧的快捷键，并返回错误。
/// 4. 如果注册成功，则更新应用状态，并将新快捷键持久化到本地存储中。
///
/// # Param
/// new_shortcut_str: String - 新的快捷键组合字符串，例如 "CmdOrCtrl+Shift+V"。
/// handle: AppHandle - Tauri 的应用句柄，用于访问全局快捷键管理器。
/// state: State<AppShortcutState> - 存储当前主快捷键的 Tauri 状态。
/// # Returns
/// Result<(), String> - 操作成功则返回 Ok(())，失败则返回包含错误信息的 Err。
#[tauri::command]
pub fn update_shortcut(
    new_shortcut_str: String,
    handle: AppHandle,
    state: State<AppShortcutState>,
) -> Result<(), String> {
    let mut current_shortcut_str = state.current_shortcut.lock().unwrap();
    let manager = handle.global_shortcut();

    // 1. 注销旧的快捷键 (先解析成 Shortcut 对象)
    if !current_shortcut_str.is_empty() {
        if let Ok(old_shortcut) = Shortcut::from_str(&*current_shortcut_str) {
            if let Err(e) = manager.unregister(old_shortcut) {
                eprintln!(
                    "⚠️ 注销旧快捷键 {} 可能失败: {:?}",
                    &*current_shortcut_str, e
                );
            }
        }
    }

    // 2. 尝试注册新的快捷键 (先解析成 Shortcut 对象)
    let new_shortcut = Shortcut::from_str(&new_shortcut_str).map_err(|e| e.to_string())?;
    if let Err(e) = manager.register(new_shortcut.clone()) {
        // 如果注册失败，尝试恢复旧的快捷键
        if !current_shortcut_str.is_empty() {
            if let Ok(old_shortcut_revert) = Shortcut::from_str(&*current_shortcut_str) {
                manager.register(old_shortcut_revert).ok();
            }
        }
        return Err(format!("注册新快捷键失败，可能已被占用: {}", e));
    }

    // 3. 成功后，更新状态并保存
    println!("✅ 已成功更新并注册快捷键: {}", new_shortcut_str);
    *current_shortcut_str = new_shortcut_str.clone();
    save_shortcut_to_storage(&handle, &new_shortcut_str);

    Ok(())
}
/// 动态更新并注册应用的第二个全局快捷键。作为 Tauri command 暴露给前端调用。
///
/// 功能与 `update_shortcut` 类似，但针对的是第二个独立的快捷键。
/// 它会注销旧的、注册新的，并在失败时回滚。成功后会更新对应的状态 `AppShortcutState2`
/// 并调用 `save_shortcut_to_storage2` 进行持久化。
///
/// # Param
/// new_shortcut_str: String - 新的快捷键组合字符串。
/// handle: AppHandle - Tauri 的应用句柄，用于访问全局快捷键管理器。
/// state: State<AppShortcutState2> - 存储当前第二个快捷键的 Tauri 状态。
/// # Returns
/// Result<(), String> - 操作成功则返回 Ok(())，失败则返回包含错误信息的 Err。
#[tauri::command]
pub fn update_shortcut2(
    new_shortcut_str: String,
    handle: AppHandle,
    state: State<AppShortcutState2>,
) -> Result<(), String> {
    let mut current_shortcut_str = state.current_shortcut.lock().unwrap();
    let manager = handle.global_shortcut();

    // 1. 注销旧的快捷键 (先解析成 Shortcut 对象)
    if !current_shortcut_str.is_empty() {
        if let Ok(old_shortcut) = Shortcut::from_str(&*current_shortcut_str) {
            if let Err(e) = manager.unregister(old_shortcut) {
                eprintln!(
                    "⚠️ 注销第二个界面旧快捷键 {} 可能失败: {:?}",
                    &*current_shortcut_str, e
                );
            }
        }
    }

    // 2. 尝试注册新的快捷键 (先解析成 Shortcut 对象)
    let new_shortcut = Shortcut::from_str(&new_shortcut_str).map_err(|e| e.to_string())?;
    if let Err(e) = manager.register(new_shortcut.clone()) {
        // 如果注册失败，尝试恢复旧的快捷键
        if !current_shortcut_str.is_empty() {
            if let Ok(old_shortcut_revert) = Shortcut::from_str(&*current_shortcut_str) {
                manager.register(old_shortcut_revert).ok();
            }
        }
        return Err(format!("注册第二个界面新快捷键失败，可能已被占用: {}", e));
    }

    // 3. 成功后，更新状态并保存
    println!("✅ 已成功更新并注册第二个界面快捷键: {}", new_shortcut_str);
    *current_shortcut_str = new_shortcut_str.clone();
    save_shortcut_to_storage2(&handle, &new_shortcut_str);

    Ok(())
}

/// 创建系统托盘图标和菜单
pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let last_click_time = Arc::new(Mutex::new(Instant::now()));
    let show_hide = MenuItem::with_id(app, "show_hide", "显示/隐藏", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::new(app)?;
    menu.append(&show_hide)?;
    menu.append(&quit)?;
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("桌面宠物")
        .on_menu_event(move |app, event| {
            if let Some(window) = app.get_webview_window("main") {
                match event.id().as_ref() {
                    "show_hide" => toggle_window_visibility(&window),
                    "quit" => std::process::exit(0),
                    _ => {}
                }
            }
        })
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                let now = Instant::now();
                let mut last_time = last_click_time.lock().unwrap();
                if now.duration_since(*last_time) < Duration::from_millis(200) {
                    return;
                }
                *last_time = now;
                if let tauri::tray::MouseButton::Left = button {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        toggle_window_visibility(&window);
                    }
                }
            }
        })
        .build(app)?;
    println!("✅ 托盘图标创建成功");
    Ok(())
}

pub fn setup_global_shortcuts(handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let handle_for_closure = handle.clone();

    // 1. 设置一个全局的、唯一的事件处理器
    handle.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, shortcut, event| {
                let state = handle_for_closure.state::<AppShortcutState>();
                let active_shortcut_str = state.current_shortcut.lock().unwrap();

                if let Ok(active_shortcut) = Shortcut::from_str(&active_shortcut_str) {
                    if shortcut == &active_shortcut && event.state() == PluginShortcutState::Pressed
                    {
                        if let Some(window) = handle_for_closure.get_webview_window("main") {
                            println!("✅ 第一个界面快捷键触发，执行窗口切换逻辑");
                            toggle_window_visibility(&window);
                        }
                    }
                }

                // 添加第二个界面的快捷键处理
                let state2 = handle_for_closure.state::<AppShortcutState2>();
                let active_shortcut_str2 = state2.current_shortcut.lock().unwrap();

                if let Ok(active_shortcut2) = Shortcut::from_str(&active_shortcut_str2) {
                    if shortcut == &active_shortcut2 && event.state() == PluginShortcutState::Pressed
                    {
                        if let Some(window) = handle_for_closure.get_webview_window("main") {
                            println!("🎯 执行前端 toggleClipboardWindow 函数");
                            match window.eval(
                                "if (typeof toggleClipboardWindow === 'function') { console.log('Rust: 调用剪贴板窗口切换'); toggleClipboardWindow(); } else { console.error('Rust: toggleClipboardWindow 未找到'); }"
                            ) {
                                Ok(_) => println!("✅ JavaScript 执行命令发送成功"),
                                Err(e) => println!("❌ JavaScript 执行失败: {:?}", e),
                            }
                        } else {
                            println!("❌ 主窗口未找到，无法执行前端函数");
                        }
                    }
                }
            })
            .build(),
    )?;

    // 2. 加载、存储并注册第一个界面的初始快捷键
    let shortcut_str = load_shortcut_from_storage(&handle);
    println!("ℹ️ 正在尝试注册第一个界面快捷键: {}", shortcut_str);

    if let Ok(shortcut) = Shortcut::from_str(&shortcut_str) {
        let manager = handle.global_shortcut();
        if let Err(e) = manager.register(shortcut) {
            eprintln!(
                "❌ 注册第一个界面初始快捷键 {} 失败: {:?}. 用户可能需要重新设置。",
                shortcut_str, e
            );
        } else {
            println!("✅ 已成功注册第一个界面全局快捷键: {}", shortcut_str);
        }
    } else {
        eprintln!("❌ 第一个界面初始快捷键 '{}' 格式无效。", shortcut_str);
    }

    // 3. 将加载的快捷键字符串存入状态管理
    let state = handle.state::<AppShortcutState>();
    *state.current_shortcut.lock().unwrap() = shortcut_str;

    // 4. 加载、存储并注册第二个界面的初始快捷键
    let shortcut_str2 = load_shortcut_from_storage2(&handle);
    println!("ℹ️ 正在尝试注册第二个界面快捷键: {}", shortcut_str2);

    if let Ok(shortcut2) = Shortcut::from_str(&shortcut_str2) {
        let manager = handle.global_shortcut();
        if let Err(e) = manager.register(shortcut2) {
            eprintln!(
                "❌ 注册第二个界面初始快捷键 {} 失败: {:?}. 用户可能需要重新设置。",
                shortcut_str2, e
            );
        } else {
            println!("✅ 已成功注册第二个界面全局快捷键: {}", shortcut_str2);
        }
    } else {
        eprintln!("❌ 第二个界面初始快捷键 '{}' 格式无效。", shortcut_str2);
    }

    // 5. 将加载的第二个界面快捷键字符串存入状态管理
    let state2 = handle.state::<AppShortcutState2>();
    *state2.current_shortcut.lock().unwrap() = shortcut_str2;

    Ok(())
}

pub fn start_clipboard_monitor(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        let mut last_text = String::new();
        let mut last_image_bytes: Vec<u8> = Vec::new();
        let mut last_file_paths: Vec<PathBuf> = Vec::new();

        let mut is_first_run = true;
        let mut frontend_ignore_countdown = 0;

        let app_dir = app_handle.path().app_data_dir().unwrap();
        let files_dir = app_dir.join("files");
        fs::create_dir_all(&files_dir).unwrap();

        loop {
            {
                let state = app_handle.state::<ClipboardSourceState>();
                let mut flag = state.is_frontend_copy.lock().unwrap();
                if *flag {
                    frontend_ignore_countdown = 9; // 0.9秒倒计时
                    *flag = false; // 重置状态
                    println!("前端触发复制，启动忽略倒计时...");
                }
            }

            // 只要倒计时大于0，就认为是前端复制状态
            let is_frontend_copy = frontend_ignore_countdown > 0;
            if frontend_ignore_countdown > 0 {
                frontend_ignore_countdown -= 1;
            }

            if is_first_run {
                if let Ok(text) = app_handle.clipboard().read_text() {
                    if !text.is_empty() {
                        last_text = text;
                    }
                }
                if let Ok(image) = app_handle.clipboard().read_image() {
                    let current = image.rgba().to_vec();
                    if !current.is_empty() {
                        last_image_bytes = current;
                    }
                }
                if let Ok(paths) = clipboard_files::read() {
                    if !paths.is_empty() {
                        last_file_paths = paths;
                    }
                }
                is_first_run = false;
                thread::sleep(Duration::from_millis(1000));
                continue;
            }

            // --- 图片监控 ---
            if let Ok(image) = app_handle.clipboard().read_image() {
                let current_image_bytes = image.rgba().to_vec();
                if !current_image_bytes.is_empty() && current_image_bytes != last_image_bytes {
                    println!("检测到新的图片内容");
                    // 立即更新 last 状态，防止重复检测
                    last_image_bytes = current_image_bytes.clone();
                    last_text.clear();
                    last_file_paths.clear();
                    if is_frontend_copy {
                        println!("忽略前端触发的图片变更");
                    } else {
                        // 只有是非前端复制时，才执行保存文件和数据库操作
                        let image_id = Uuid::new_v4().to_string();
                        let dest_path = files_dir.join(format!("{}.png", image_id));

                        if image::save_buffer(
                            &dest_path,
                            &image.rgba(),
                            image.width(),
                            image.height(),
                            ColorType::Rgba8,
                        )
                        .is_ok()
                        {
                            let new_item = ClipboardItem {
                                id: image_id.clone(),
                                item_type: "image".to_string(),
                                content: dest_path.to_str().unwrap().to_string(),
                                size: fs::metadata(&dest_path).ok().map(|m| m.len()),
                                is_favorite: false,
                                notes: "".to_string(),
                                timestamp: Utc::now().timestamp_millis(),
                            };

                            if let Err(e) = db::insert_received_db_data(new_item) {
                                eprintln!("❌ 保存图片数据到数据库失败: {:?}", e);
                            } else {
                                // OCR识别（异步）
                                let ocr_path = dest_path.clone().to_str().unwrap().to_string();
                                let ocr_item_id = image_id.clone();
                                tauri::async_runtime::spawn(async move {
                                    match ocr::ocr_image(ocr_path).await {
                                        Ok(res) => {
                                            // 识别成功，保存结果到数据库
                                            let ocr_text =
                                                match serde_json::from_str::<Vec<Value>>(&res) {
                                                    Ok(json_array) => json_array
                                                        .iter()
                                                        .filter_map(|v| {
                                                            v.get("text").and_then(|t| t.as_str())
                                                        })
                                                        .collect::<Vec<&str>>()
                                                        .join("\n"),
                                                    Err(_) => res.clone(),
                                                };
                                            if let Err(e) =
                                                db::insert_ocr_text(&ocr_item_id, &ocr_text)
                                            {
                                                eprintln!("❌ 保存OCR结果到数据库失败: {:?}", e);
                                            }
                                        }
                                        Err(err) => eprintln!("OCR error: {}", err),
                                    }
                                });

                                // 通知前端
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    let _ = window.emit("clipboard-updated", "");
                                }
                            }
                        }
                    }
                }
            }
            // --- 文件监控 ---
            else if let Ok(paths) = clipboard_files::read() {
                if !paths.is_empty() && paths != last_file_paths {
                    println!("检测到新的文件复制: {:?}", paths);
                    last_file_paths = paths.clone();
                    last_text.clear();
                    last_image_bytes.clear();

                    if is_frontend_copy {
                        println!("忽略前端触发的文件变更");
                    } else {
                        let mut has_new_files = false;
                        const IMAGE_EXTENSIONS: &[&str] =
                            &["png", "jpg", "jpeg", "gif", "bmp", "webp", "ico"];

                        for path in paths {
                            // 1. 判断类型：如果是目录则为 "folder"，否则按扩展名判断
                            let item_type = if path.is_dir() {
                                "folder".to_string()
                            } else {
                                path.extension()
                                    .and_then(|ext| ext.to_str())
                                    .map(|ext_str| {
                                        if IMAGE_EXTENSIONS
                                            .contains(&ext_str.to_lowercase().as_str())
                                        {
                                            "image".to_string()
                                        } else {
                                            "file".to_string()
                                        }
                                    })
                                    .unwrap_or_else(|| "file".to_string())
                            };

                            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                let timestamp = Utc::now().timestamp_millis();
                                let new_file_name = format!("{}-{}", timestamp, file_name);
                                let dest_path = files_dir.join(&new_file_name);

                                // 2. 根据是文件夹还是文件执行不同的复制操作
                                let copy_result = if path.is_dir() {
                                    copy_dir_all(&path, &dest_path)
                                } else {
                                    fs::copy(&path, &dest_path)
                                };

                                match copy_result {
                                    Ok(bytes_copied) => {
                                        has_new_files = true;

                                        // ✅ 直接使用复制时计算出的大小
                                        let size = Some(bytes_copied);

                                        let new_item = ClipboardItem {
                                            id: Uuid::new_v4().to_string(),
                                            item_type: item_type,
                                            content: dest_path.to_str().unwrap().to_string(),
                                            size: size,
                                            is_favorite: false,
                                            notes: "".to_string(),
                                            timestamp: Utc::now().timestamp_millis(),
                                        };

                                        if let Err(e) = db::insert_received_db_data(new_item) {
                                            eprintln!("❌ 保存数据到数据库失败: {:?}", e);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("❌ 复制 {:?} 失败: {}", path, e);
                                    }
                                }
                            }
                        }

                        if has_new_files {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.emit("clipboard-updated", "");
                            }
                        }
                    }
                }
            } else if let Ok(text) = app_handle.clipboard().read_text() {
                if !text.is_empty() && text != last_text {
                    println!("检测到新的文本内容");
                    last_text = text.clone();
                    last_image_bytes.clear();
                    last_file_paths.clear();

                    if is_frontend_copy {
                        println!("忽略前端触发的文本变更");
                    } else {
                        let size = Some(text.chars().count() as u64);
                        let new_item = ClipboardItem {
                            id: Uuid::new_v4().to_string(),
                            item_type: "text".to_string(),
                            content: text,
                            size,
                            is_favorite: false,
                            notes: "".to_string(),
                            timestamp: Utc::now().timestamp_millis(),
                        };

                        if let Err(e) = db::insert_received_db_data(new_item) {
                            eprintln!("❌ 保存文本数据到数据库失败: {:?}", e);
                        } else {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.emit("clipboard-updated", "");
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    });
}
/// 切换窗口的显示与隐藏状态
fn toggle_window_visibility(window: &WebviewWindow) {
    if let Ok(is_visible) = window.is_visible() {
        if is_visible {
            if let Err(e) = window.hide() {
                eprintln!("❌ 隐藏窗口失败: {:?}", e);
            }
        } else {
            if let Err(e) = window.show() {
                eprintln!("❌ 显示窗口失败: {:?}", e);
            }
            if let Err(e) = window.set_focus() {
                eprintln!("⚠️ 设置窗口焦点失败: {:?}", e);
            }
        }
    }
}

/// 递归复制文件夹，并返回复制的总字节数
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<u64> {
    fs::create_dir_all(&dst)?;
    let mut total_size: u64 = 0;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            // 递归调用，加上子文件夹的大小
            total_size += copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            // fs::copy 返回的是复制的字节数 (u64)
            total_size += fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(total_size)
}
