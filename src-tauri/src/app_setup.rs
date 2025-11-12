use crate::clipboard::ClipboardItem;
use crate::db;
use chrono::Utc;
use image::ColorType;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr; // 修正：导入 FromStr trait 以使用 .parse()
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;
use uuid::Uuid;
use tauri_plugin_global_shortcut::{
    GlobalShortcutExt, Shortcut, ShortcutState as PluginShortcutState,
};
pub struct ClipboardSourceState {
    pub is_frontend_copy: Mutex<bool>,
}
pub struct AppShortcutState {
    pub current_shortcut: Mutex<String>,
}
pub struct AppShortcutState2 {
    pub current_shortcut: Mutex<String>,
}
/// 构建并返回主快捷键配置文件的完整路径。
/// 该文件名为 `shortcut_config.txt`，存储在应用的配置目录下。
/// # Param
/// handle: &AppHandle - Tauri 的应用句柄，用于获取应用目录。
/// # Returns
/// PathBuf - 指向配置文件的路径对象。
/// # Panics
/// 如果无法获取应用配置目录，程序会 panic。
fn get_shortcut_config_path(handle: &AppHandle) -> PathBuf {
    let mut path = handle
        .path()
        .app_config_dir()
        .expect("无法获取应用配置目录");
    path.push("shortcut_config.txt");
    path
}
/// 构建并返回第二个界面快捷键配置文件的完整路径。
/// 该文件名为 `shortcut_config2.txt`，存储在应用的配置目录下。
/// # Param
/// handle: &AppHandle - Tauri 的应用句柄，用于获取应用目录。
/// # Returns
/// PathBuf - 指向配置文件的路径对象。
/// # Panics
/// 如果无法获取应用配置目录，程序会 panic。
fn get_shortcut_config_path2(handle: &AppHandle) -> PathBuf {
    let mut path = handle
        .path()
        .app_config_dir()
        .expect("无法获取应用配置目录");
    path.push("shortcut_config2.txt");
    path
}
/// 从本地存储中加载主快捷键配置。
/// 如果配置文件不存在或读取失败，将返回默认快捷键 "Alt+Shift+V"。
/// # Param
/// handle: &AppHandle - Tauri 的应用句柄，用于定位配置文件。
/// # Returns
/// String - 从文件中读取到的快捷键字符串，或默认值。
fn load_shortcut_from_storage(handle: &AppHandle) -> String {
    fs::read_to_string(get_shortcut_config_path(handle))
        .unwrap_or_else(|_| "Alt+Shift+V".to_string())
}
/// 从本地存储中加载第二个界面的快捷键配置。
/// 如果配置文件不存在或读取失败，将返回默认快捷键 "Alt+Shift+C"。
/// # Param
/// handle: &AppHandle - Tauri 的应用句柄，用于定位配置文件。
/// # Returns
/// String - 从文件中读取到的快捷键字符串，或默认值。
fn load_shortcut_from_storage2(handle: &AppHandle) -> String {
    fs::read_to_string(get_shortcut_config_path2(handle))
        .unwrap_or_else(|_| "Alt+Shift+C".to_string())
}

/// 将主快捷键配置字符串保存到本地文件中。
/// 如果保存失败，会向 stderr 打印一条错误信息。
/// # Param
/// handle: &AppHandle - Tauri 的应用句柄，用于定位配置文件。
/// shortcut: &str - 需要保存的快捷键字符串。
/// # Returns
/// ()
fn save_shortcut_to_storage(handle: &AppHandle, shortcut: &str) {
    if let Err(e) = fs::write(get_shortcut_config_path(handle), shortcut) {
        eprintln!("❌ 保存快捷键配置失败: {:?}", e);
    }
}
/// 将第二个界面的快捷键配置字符串保存到本地文件中。
/// 如果保存失败，会向 stderr 打印一条错误信息。
/// # Param
/// handle: &AppHandle - Tauri 的应用句柄，用于定位配置文件。
/// shortcut: &str - 需要保存的快捷键字符串。
/// # Returns
/// ()
fn save_shortcut_to_storage2(handle: &AppHandle, shortcut: &str) {
    if let Err(e) = fs::write(get_shortcut_config_path2(handle), shortcut) {
        eprintln!("❌ 保存第二个界面快捷键配置失败: {:?}", e);
    }
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
                        if let Some(window) = handle_for_closure.get_webview_window("second_window") {
                            println!("✅ 第二个界面快捷键触发，执行窗口切换逻辑");
                            toggle_window_visibility(&window);
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

/// 启动后台线程以监控剪贴板
pub fn start_clipboard_monitor(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        let mut last_text = String::new();
        let mut last_image_bytes: Vec<u8> = Vec::new();
        let mut last_file_paths: Vec<PathBuf> = Vec::new();

        // 修正 #2: 确保这里也使用正确的 .path() 方法
        let app_dir = app_handle.path().app_data_dir().unwrap();
        let files_dir = app_dir.join("files");
        fs::create_dir_all(&files_dir).unwrap();

        loop {
            // 获取当前是否是前端复制状态
            let is_frontend_copy = {
                let state = app_handle.state::<ClipboardSourceState>();
                let mut flag = state.is_frontend_copy.lock().unwrap();
                let current = *flag;
                // 重置标志，以便下次检测
                *flag = false;
                current
            };
            // ... 内部逻辑无改动 ...
            if let Ok(text) = app_handle.clipboard().read_text() {
                if !text.is_empty() && text != last_text {
                    println!("检测到新的文本内容: {}", text);
                    last_text = text.clone();
                    last_image_bytes.clear();
                    last_file_paths.clear();

                    let size = Some(text.chars().count() as u64);
                    let new_item = ClipboardItem {
                        id: Uuid::new_v4().to_string(),
                        item_type: "text".to_string(),
                        content: text,
                        size,
                        is_favorite: false,
                        notes: "".to_string(),
                        timestamp: Utc::now().timestamp(),
                    };
                    
                    if !is_frontend_copy {
                        if let Err(e) = db::insert_received_data(new_item) {
                            eprintln!("❌ 保存文本数据到数据库失败: {:?}", e);
                        }
                    }
                }
            }

            if let Ok(image) = app_handle.clipboard().read_image() {
                let current_image_bytes = image.rgba().to_vec();
                if !current_image_bytes.is_empty() && current_image_bytes != last_image_bytes {
                    println!("检测到新的图片内容");
                    last_image_bytes = current_image_bytes.clone();
                    last_text.clear();
                    last_file_paths.clear();

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
                            id: image_id,
                            item_type: "image".to_string(),
                            content: dest_path.to_str().unwrap().to_string(),
                            size: fs::metadata(&dest_path).ok().map(|m| m.len()),
                            is_favorite: false,
                            notes: "".to_string(),
                            timestamp: Utc::now().timestamp(),
                        };
                        if !is_frontend_copy {
                            if let Err(e) = db::insert_received_data(new_item) {
                                eprintln!("❌ 保存图片数据到数据库失败: {:?}", e);
                            }
                        }
                    }
                }
            }

            if let Ok(paths) = clipboard_files::read() {
                if !paths.is_empty() && paths != last_file_paths {
                    println!("检测到新的文件复制: {:?}", paths);
                    last_file_paths = paths.clone();
                    last_text.clear();
                    last_image_bytes.clear();

                    for path in paths {
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            let timestamp = Utc::now().timestamp_millis();
                            let new_file_name = format!("{}-{}", timestamp, file_name);
                            let dest_path = files_dir.join(&new_file_name);

                            if fs::copy(&path, &dest_path).is_ok() {
                                let new_item = ClipboardItem {
                                    id: Uuid::new_v4().to_string(),
                                    item_type: "file".to_string(),
                                    content: dest_path.to_str().unwrap().to_string(),
                                    size: fs::metadata(&dest_path).ok().map(|m| m.len()),
                                    is_favorite: false,
                                    notes: "".to_string(),
                                    timestamp: Utc::now().timestamp(),
                                };
                                if !is_frontend_copy {
                                    if let Err(e) = db::insert_received_data(new_item) {
                                        eprintln!("❌ 保存文件数据到数据库失败: {:?}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            thread::sleep(Duration::from_millis(500));
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
