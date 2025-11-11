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
use tauri_plugin_global_shortcut::{
    GlobalShortcutExt, Shortcut, ShortcutState as PluginShortcutState,
};

pub struct AppShortcutState {
    pub current_shortcut: Mutex<String>,
}

fn get_shortcut_config_path(handle: &AppHandle) -> PathBuf {
    let mut path = handle
        .path()
        .app_config_dir()
        .expect("无法获取应用配置目录");
    path.push("shortcut_config.txt");
    path
}

fn load_shortcut_from_storage(handle: &AppHandle) -> String {
    fs::read_to_string(get_shortcut_config_path(handle))
        .unwrap_or_else(|_| "Alt+Shift+V".to_string())
}

fn save_shortcut_to_storage(handle: &AppHandle, shortcut: &str) {
    if let Err(e) = fs::write(get_shortcut_config_path(handle), shortcut) {
        eprintln!("❌ 保存快捷键配置失败: {:?}", e);
    }
}

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
                            println!("✅ 快捷键触发，执行窗口切换逻辑");
                            toggle_window_visibility(&window);
                        }
                    }
                }
            })
            .build(),
    )?;

    // 2. 加载、存储并注册初始的快捷键
    let shortcut_str = load_shortcut_from_storage(&handle);
    println!("ℹ️ 正在尝试注册快捷键: {}", shortcut_str);

    if let Ok(shortcut) = Shortcut::from_str(&shortcut_str) {
        let manager = handle.global_shortcut();
        if let Err(e) = manager.register(shortcut) {
            eprintln!(
                "❌ 注册初始快捷键 {} 失败: {:?}. 用户可能需要重新设置。",
                shortcut_str, e
            );
        } else {
            println!("✅ 已成功注册全局快捷键: {}", shortcut_str);
        }
    } else {
        eprintln!("❌ 初始快捷键 '{}' 格式无效。", shortcut_str);
    }

    // 3. 将加载的快捷键字符串存入状态管理
    let state = handle.state::<AppShortcutState>();
    *state.current_shortcut.lock().unwrap() = shortcut_str;

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
            // ... 内部逻辑无改动 ...
            if let Ok(text) = app_handle.clipboard().read_text() {
                if !text.is_empty() && text != last_text {
                    println!("检测到新的文本内容: {}", text);
                    last_text = text.clone();
                    last_image_bytes.clear();
                    last_file_paths.clear();

                    let size = Some(text.chars().count() as u64);
                    let new_item = ClipboardItem {
                        id: Utc::now().timestamp_millis().to_string(),
                        item_type: "text".to_string(),
                        content: text,
                        size,
                        is_favorite: false,
                        notes: "".to_string(),
                        timestamp: Utc::now().timestamp(),
                    };
                    if let Err(e) = db::insert_received_data(new_item) {
                        eprintln!("❌ 保存文本数据到数据库失败: {:?}", e);
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

                    let image_id = Utc::now().timestamp_millis().to_string();
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
                        if let Err(e) = db::insert_received_data(new_item) {
                            eprintln!("❌ 保存图片数据到数据库失败: {:?}", e);
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
                                    id: timestamp.to_string(),
                                    item_type: "file".to_string(),
                                    content: dest_path.to_str().unwrap().to_string(),
                                    size: fs::metadata(&dest_path).ok().map(|m| m.len()),
                                    is_favorite: false,
                                    notes: "".to_string(),
                                    timestamp: Utc::now().timestamp(),
                                };
                                if let Err(e) = db::insert_received_data(new_item) {
                                    eprintln!("❌ 保存文件数据到数据库失败: {:?}", e);
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
