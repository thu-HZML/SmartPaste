use crate::clipboard::ClipboardItem;
use crate::config::{self, CONFIG};
use crate::db;
use crate::ocr;
use crate::utils;
use chrono::Utc;
use image::ColorType;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Emitter, Manager, State, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{
    GlobalShortcutExt, Shortcut, ShortcutState as PluginShortcutState,
};
use uuid::Uuid;

// 全局静态变量存储托盘图标的句柄
static TRAY_ICON_GLOBAL: OnceLock<TrayIcon> = OnceLock::new();

pub struct ClipboardSourceState {
    pub is_frontend_copy: Mutex<bool>,
}
/// 管理应用的主快捷键状态
pub struct AppShortcutManager {
    pub shortcuts: Mutex<std::collections::HashMap<String, String>>,
}
impl AppShortcutManager {
    pub fn new() -> Self {
        Self {
            shortcuts: Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn get_shortcut(&self, shortcut_type: &str) -> Option<String> {
        self.shortcuts.lock().unwrap().get(shortcut_type).cloned()
    }

    pub fn set_shortcut(&self, shortcut_type: &str, shortcut: String) {
        self.shortcuts
            .lock()
            .unwrap()
            .insert(shortcut_type.to_string(), shortcut);
    }

    pub fn remove_shortcut(&self, shortcut_type: &str) {
        self.shortcuts.lock().unwrap().remove(shortcut_type);
    }
}
// 快捷键配置定义
#[derive(Clone)]
pub struct ShortcutConfig {
    pub storage_key: &'static str,
    pub default_value: &'static str,
    pub handler: fn(&AppHandle, &str),
}

// 快捷键配置映射
lazy_static::lazy_static! {
    static ref SHORTCUT_CONFIGS: std::collections::HashMap<&'static str, ShortcutConfig> = {
        let mut m = std::collections::HashMap::new();
        m.insert("toggleWindow", ShortcutConfig {
            storage_key: "global_shortcut",
            default_value: "Shift+V",
            handler: |app, _shortcut| {
                println!("🎯 执行主窗口切换");
                if let Some(window) = app.get_webview_window("main") {
                    toggle_window_visibility(&window);
                }
            },
        });
        m.insert("pasteWindow", ShortcutConfig {
            storage_key: "global_shortcut_2",
            default_value: "Shift+Alt+C",
            handler: |app, shortcut| {
                println!("🎯 执行剪贴板窗口切换，快捷键: {}", shortcut);
                if let Some(window) = app.get_webview_window("main") {
                    match window.eval(
                        "if (typeof toggleClipboardWindow === 'function') { console.log('Rust: 调用剪贴板窗口切换'); toggleClipboardWindow(); } else { console.error('Rust: toggleClipboardWindow 未找到'); }"
                    ) {
                        Ok(_) => println!("✅ JavaScript 执行命令发送成功"),
                        Err(e) => println!("❌ JavaScript 执行失败: {:?}", e),
                    }
                }
            },
        });
        m.insert("AIWindow", ShortcutConfig {
            storage_key: "global_shortcut_3",
            default_value: "Shift+Ctrl+A",
            handler: |app, shortcut| {
                println!("🤖 执行AI窗口切换，快捷键: {}", shortcut);
                if let Some(window) = app.get_webview_window("main") {
                    match window.eval(
                        "if (typeof toggleAIWindow === 'function') { console.log('Rust: 调用AI窗口切换'); toggleAIWindow(); } else { console.error('Rust: toggleAIWindow 未找到'); }"
                    ) {
                        Ok(_) => println!("✅ AI窗口切换命令发送成功"),
                        Err(e) => println!("❌ AI窗口切换执行失败: {:?}", e),
                    }
                }
            },
        });
        m.insert("setWindow", ShortcutConfig {
            storage_key: "global_shortcut_4",
            default_value: "Shift+Ctrl+V",
            handler: |app, shortcut| {
                println!("⚙️ 执行设置窗口切换，快捷键: {}", shortcut);
                if let Some(window) = app.get_webview_window("main") {
                    match window.eval(
                        "if (typeof toggleSetWindow === 'function') { console.log('Rust: 调用设置页面切换'); toggleSetWindow(); } else { console.error('Rust: toggleSetWindow 未找到'); }"
                    ) {
                        Ok(_) => println!("✅ 设置窗口切换命令发送成功"),
                        Err(e) => println!("❌ 设置窗口切换执行失败: {:?}", e),
                    }
                }
            },
        });
        m.insert("clearHistory", ShortcutConfig {
            storage_key: "global_shortcut_5",
            default_value: "Shift+Ctrl+Delete",
            handler: |app, shortcut| {
                println!("🗑️ 执行清空历史，快捷键: {}", shortcut);
                if let Some(window) = app.get_webview_window("main") {
                    match window.eval(
                        "if (typeof clearClipboardHistory === 'function') { console.log('Rust: 调用清空历史'); clearClipboardHistory(); } else { console.error('Rust: clearClipboardHistory 未找到'); }"
                    ) {
                        Ok(_) => println!("✅ 清空历史命令发送成功"),
                        Err(e) => println!("❌ 清空历史执行失败: {:?}", e),
                    }
                }
            },
        });
        m
    };
    // 通过 Storage Key 查找 Handler Key 的反向映射
    static ref STORAGE_KEY_TO_HANDLER_KEY: std::collections::HashMap<&'static str, &'static str> = {
        let mut m = std::collections::HashMap::new();
        for (handler_key, config) in SHORTCUT_CONFIGS.iter() {
            m.insert(config.storage_key, *handler_key);
        }
        m
    };
}

/// 从 Config 中加载快捷键配置
fn load_shortcut_from_storage(shortcut_type: &str) -> String {
    // 确保我们能通过 storage_key 找到对应的配置，以获取默认值
    if let Some(handler_key) = STORAGE_KEY_TO_HANDLER_KEY.get(shortcut_type) {
        if let Some(config) = SHORTCUT_CONFIGS.get(handler_key) {
            // 拿到对应的配置对象
            if let Some(lock) = CONFIG.get() {
                let cfg = lock.read().unwrap();
                // 简化匹配，直接使用传入的 storage_key
                match shortcut_type {
                    "global_shortcut" => cfg.global_shortcut.clone(),
                    "global_shortcut_2" => cfg.global_shortcut_2.clone(),
                    "global_shortcut_3" => cfg.global_shortcut_3.clone(),
                    "global_shortcut_4" => cfg.global_shortcut_4.clone(),
                    "global_shortcut_5" => cfg.global_shortcut_5.clone(),
                    _ => config.default_value.to_string(),
                }
            } else {
                config.default_value.to_string()
            }
        } else {
            // fallback to default if config map lookup fails
            // Since we use STORAGE_KEY_TO_HANDLER_KEY, this path is unlikely
            "".to_string()
        }
    } else {
        // Unknown shortcut type
        "".to_string()
    }
}

/// 保存快捷键到 Config
fn save_shortcut_to_storage(shortcut_type: &str, shortcut: &str) {
    if let Some(config) = SHORTCUT_CONFIGS.get(shortcut_type) {
        let value = serde_json::Value::String(shortcut.to_string());
        if let Err(e) = config::set_config_item_internal(config.storage_key, value) {
            eprintln!("Failed to save shortcut: {}", e);
        }
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
    shortcut_type: String,
    new_shortcut_str: String,
    handle: AppHandle,
    state: State<AppShortcutManager>,
) -> Result<(), String> {
    let manager = handle.global_shortcut();

    // 1. 获取旧的快捷键并注销
    let old_shortcut_str = state.get_shortcut(&shortcut_type).unwrap_or_default();
    if !old_shortcut_str.is_empty() {
        if let Ok(old_shortcut) = Shortcut::from_str(&old_shortcut_str) {
            if let Err(e) = manager.unregister(old_shortcut) {
                eprintln!("⚠️ 注销旧快捷键 {} 可能失败: {:?}", old_shortcut_str, e);
            }
        }
    }

    // 2. 尝试注册新的快捷键
    let new_shortcut = Shortcut::from_str(&new_shortcut_str).map_err(|e| e.to_string())?;
    if let Err(e) = manager.register(new_shortcut.clone()) {
        // 注册失败，尝试恢复旧的快捷键
        if !old_shortcut_str.is_empty() {
            if let Ok(old_shortcut_revert) = Shortcut::from_str(&old_shortcut_str) {
                manager.register(old_shortcut_revert).ok();
            }
        }
        return Err(format!("注册快捷键失败，可能已被占用: {}", e));
    }

    // 3. 更新状态并保存
    println!(
        "✅ 已成功更新快捷键 {}: {}",
        shortcut_type, new_shortcut_str
    );
    state.set_shortcut(&shortcut_type, new_shortcut_str.clone());
    save_shortcut_to_storage(&shortcut_type, &new_shortcut_str);

    Ok(())
}
/// 获取当前快捷键
#[tauri::command]
pub fn get_current_shortcut(
    shortcut_type: String,
    state: State<AppShortcutManager>,
) -> Result<String, String> {
    state
        .get_shortcut(&shortcut_type)
        .ok_or_else(|| "快捷键未找到".to_string())
}
/// 获取所有快捷键
#[tauri::command]
pub fn get_all_shortcuts(
    state: State<AppShortcutManager>,
) -> Result<std::collections::HashMap<String, String>, String> {
    Ok(state.shortcuts.lock().unwrap().clone())
}

/// 创建系统托盘图标和菜单
pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let last_click_time = Arc::new(Mutex::new(Instant::now()));
    let show_hide = MenuItem::with_id(app, "show_hide", "显示/隐藏", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::new(app)?;
    menu.append(&show_hide)?;
    menu.append(&quit)?;
    let tray_handle = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("SmartPaste")
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
    // 存储 handle
    if TRAY_ICON_GLOBAL.set(tray_handle).is_err() {
        eprintln!("⚠️ 托盘图标句柄重复设置失败");
    }
    Ok(())
}

// 供 config.rs 调用的获取句柄函数
pub fn get_tray_icon_handle() -> Option<&'static TrayIcon> {
    TRAY_ICON_GLOBAL.get()
}

pub fn setup_global_shortcuts(handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut_manager = handle.state::<AppShortcutManager>();

    // 1. 设置统一的全局事件处理器
    handle.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state() != PluginShortcutState::Pressed {
                    return;
                }

                let shortcut_str = shortcut.to_string();
                let manager = app.state::<AppShortcutManager>();
                let shortcuts = manager.shortcuts.lock().unwrap();

                // 统一快捷键格式进行比较
                let normalized_received = normalize_shortcut_format(&shortcut_str);

                // 检查所有注册的快捷键
                // storage_key 是 &String 类型，需要 .as_str() 才能用作 HashMap<&str, ...> 的查找键
                for (storage_key, registered_shortcut) in shortcuts.iter() {
                    let normalized_registered = normalize_shortcut_format(registered_shortcut);

                    if normalized_received == normalized_registered {
                        println!("✅ 匹配到快捷键: {} - {}", storage_key, registered_shortcut);

                        // 使用 storage_key.as_str() 转换为 &str 进行查找
                        if let Some(handler_key) =
                            STORAGE_KEY_TO_HANDLER_KEY.get(storage_key.as_str())
                        {
                            // 找到对应的处理器配置并执行
                            if let Some(config) = SHORTCUT_CONFIGS.get(handler_key) {
                                println!("🚀 执行处理器: {}", handler_key);
                                (config.handler)(app, registered_shortcut);
                            } else {
                                println!("❌ 未找到处理器配置 (Handler Key: {})", handler_key);
                            }
                        } else {
                            // 错误：找不到与存储键对应的处理器
                            println!("❌ 未找到处理器: {}", storage_key);
                        }
                        return;
                    }
                }

                println!("❌ 未找到匹配的快捷键处理器");
            })
            .build(),
    )?;

    // 2. 初始化并注册所有快捷键
    // 迭代 SHORTCUT_CONFIGS 的值，确保使用 config.storage_key 作为 AppShortcutManager 的键
    for config in SHORTCUT_CONFIGS.values() {
        let shortcut_type = config.storage_key; // shortcut_type 即为 storage_key (e.g., "global_shortcut")
        let shortcut_str = load_shortcut_from_storage(shortcut_type);
        println!("ℹ️ 正在尝试注册快捷键 {}: {}", shortcut_type, shortcut_str);

        if let Ok(shortcut) = Shortcut::from_str(&shortcut_str) {
            let manager = handle.global_shortcut();
            if let Err(e) = manager.register(shortcut) {
                eprintln!(
                    "❌ 注册快捷键 {} {} 失败: {:?}. 用户可能需要重新设置。",
                    shortcut_type, shortcut_str, e
                );
            } else {
                println!("✅ 已成功注册快捷键 {}: {}", shortcut_type, shortcut_str);
                // 使用 Storage Key (shortcut_type) 存储到 AppShortcutManager
                shortcut_manager.set_shortcut(shortcut_type, shortcut_str);
            }
        } else {
            eprintln!("❌ 快捷键 {} '{}' 格式无效。", shortcut_type, shortcut_str);
        }
    }

    Ok(())
}

fn normalize_shortcut_format(shortcut: &str) -> String {
    let mut normalized = shortcut.to_lowercase();

    // 替换常见的格式差异
    normalized = normalized.replace("keya", "a");
    normalized = normalized.replace("keyb", "b");
    normalized = normalized.replace("keyc", "c");
    normalized = normalized.replace("keyd", "d");
    normalized = normalized.replace("keye", "e");
    normalized = normalized.replace("keyf", "f");
    normalized = normalized.replace("keyg", "g");
    normalized = normalized.replace("keyh", "h");
    normalized = normalized.replace("keyi", "i");
    normalized = normalized.replace("keyj", "j");
    normalized = normalized.replace("keyk", "k");
    normalized = normalized.replace("keyl", "l");
    normalized = normalized.replace("keym", "m");
    normalized = normalized.replace("keyn", "n");
    normalized = normalized.replace("keyo", "o");
    normalized = normalized.replace("keyp", "p");
    normalized = normalized.replace("keyq", "q");
    normalized = normalized.replace("keyr", "r");
    normalized = normalized.replace("keys", "s");
    normalized = normalized.replace("keyt", "t");
    normalized = normalized.replace("keyu", "u");
    normalized = normalized.replace("keyv", "v");
    normalized = normalized.replace("keyw", "w");
    normalized = normalized.replace("keyx", "x");
    normalized = normalized.replace("keyy", "y");
    normalized = normalized.replace("keyz", "z");

    // 统一修饰键名称
    normalized = normalized.replace("ctrl", "control");
    normalized = normalized.replace("cmd", "super");
    normalized = normalized.replace("command", "super");
    normalized = normalized.replace("meta", "super");

    normalized
}

pub fn start_clipboard_monitor(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        // 获取配置的存储路径
        // 初始变量状态
        let mut last_text = String::new();
        let mut last_image_bytes: Vec<u8> = Vec::new();
        let mut last_file_paths: Vec<PathBuf> = Vec::new();

        let mut is_first_run = true;
        let mut frontend_ignore_countdown = 0;

        // 定义相对路径根目录 (保持不变，因为这是存入数据库的相对路径)
        let db_root_dir = PathBuf::from("files");
        // 辅助函数
        fn get_path_size(path: &Path) -> u64 {
            if path.is_dir() {
                // 递归计算文件夹大小
                let mut total = 0;
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        total += get_path_size(&entry.path());
                    }
                }
                total
            } else {
                // 文件大小
                fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            }
        }
        loop {
            // 每次循环都重新读取配置，以支持运行时修改
            let size_limit_mb = {
                if let Some(lock) = CONFIG.get() {
                    let cfg = lock.read().unwrap();
                    cfg.ignore_big_file_mb
                } else {
                    5 // 默认值 5MB
                }
            };
            let size_limit_bytes = size_limit_mb as u64 * 1024 * 1024;

            let current_storage_path = crate::config::get_current_storage_path();
            let files_dir = current_storage_path.join("files");

            // 确保目录存在 (防止路径刚切换，文件夹还没建好，或者被意外删除)
            if !files_dir.exists() {
                if let Err(e) = fs::create_dir_all(&files_dir) {
                    eprintln!("❌ 无法创建文件存储目录 {:?}: {}", files_dir, e);
                    // 如果目录创建失败，本次循环暂停，避免后续报错
                    thread::sleep(Duration::from_millis(1000));
                    continue;
                }
            }
            {
                let state = app_handle.state::<ClipboardSourceState>();
                let mut flag = state.is_frontend_copy.lock().unwrap();
                if *flag {
                    frontend_ignore_countdown = 30; // 3秒倒计时
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
                        // let dest_path = files_dir.join(format!("{}.png", image_id));
                        let dest_relative_path = db_root_dir.join(format!("{}.png", image_id));
                        let dest_absolute_path = utils::resolve_absolute_path(&dest_relative_path);
                        if image::save_buffer(
                            &dest_absolute_path,
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
                                content: dest_relative_path.to_str().unwrap().to_string(),
                                size: fs::metadata(&dest_absolute_path).map(|m| m.len()).ok(),
                                is_favorite: false,
                                notes: "".to_string(),
                                timestamp: Utc::now().timestamp_millis(),
                            };

                            // println!("✅ 图片保存到文件: {:?}", dest_path);
                            if let Err(e) = db::insert_received_db_data(new_item) {
                                eprintln!("❌ 保存图片数据到数据库失败: {:?}", e);
                            } else {
                                // OCR识别（异步）
                                let ocr_path =
                                    dest_absolute_path.clone().to_str().unwrap().to_string();
                                let ocr_item_id = image_id.clone();
                                tauri::async_runtime::spawn(async move {
                                    match ocr::ocr_image(ocr_path).await {
                                        Ok(res) => {
                                            println!("✅ OCR识别成功: {}", res);
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
                    //println!("检测到新的文件复制: {:?}", paths);
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
                            // 检查文件/文件夹大小是否超过限制
                            let path_size = get_path_size(&path);
                            if size_limit_mb > 0 && path_size > size_limit_bytes {
                                println!(
                                    "❌ 文件/文件夹大小超过限制: {:?} ({} MB > {} MB)，跳过复制",
                                    path,
                                    path_size as f64 / (1024.0 * 1024.0),
                                    size_limit_mb
                                );
                                continue; // 跳过这个文件/文件夹
                            }
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
                            if item_type == "image" {
                                println!("检测到图片复制: {:?}", path);
                            } else if item_type == "file" {
                                println!("检测到文件复制: {:?}", path);
                            } else if item_type == "folder" {
                                println!("检测到文件夹复制: {:?}", path);
                            }
                            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                let timestamp = Utc::now().timestamp_millis();
                                let new_file_name = format!("{}-{}", timestamp, file_name);
                                let dest_path = files_dir.join(&new_file_name);
                                let dest_relative_path = db_root_dir.join(&new_file_name);

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
                                            // content: dest_path.to_str().unwrap().to_string(),
                                            content: dest_relative_path
                                                .to_str()
                                                .unwrap()
                                                .to_string(),
                                            size: size,
                                            is_favorite: false,
                                            notes: "".to_string(),
                                            timestamp: Utc::now().timestamp_millis(),
                                        };

                                        // 先保存 id 与路径副本，new_item 会被 move 到 insert_received_db_data
                                        let item_id_for_icon = new_item.id.clone();
                                        let dest_path_for_icon =
                                            dest_path.clone().to_str().unwrap().to_string();

                                        // 记录数据库插入开始时间
                                        let db_insert_start = Instant::now();

                                        if let Err(e) = db::insert_received_db_data(new_item) {
                                            eprintln!("❌ 保存数据到数据库失败: {:?}", e);
                                        } else {
                                            println!(
                                                "[Main] 数据库插入耗时: {:?}",
                                                db_insert_start.elapsed()
                                            );

                                            // 记录调度时间
                                            let schedule_time = Instant::now();

                                            // 异步提取系统图标并存入 extended_data.icon_data
                                            tauri::async_runtime::spawn(async move {
                                                // 记录开始时间
                                                let task_start = Instant::now();
                                                println!(
                                                    "[Async] 图标获取任务启动延迟： {:?}",
                                                    task_start.duration_since(schedule_time)
                                                );

                                                // 记录图标提取开始时间
                                                let icon_extract_start = Instant::now();

                                                match utils::get_file_icon(
                                                    dest_path_for_icon.clone(),
                                                )
                                                .await
                                                {
                                                    Ok(data_uri) => {
                                                        println!(
                                                            "[Async] 图标提取耗时: {:?}",
                                                            icon_extract_start.elapsed()
                                                        );

                                                        // 记录图标插入数据库开始时间
                                                        let db_icon_insert_start = Instant::now();

                                                        if let Err(err) = db::insert_icon_data(
                                                            &item_id_for_icon,
                                                            &data_uri,
                                                        ) {
                                                            eprintln!(
                                                                "❌ insert_icon_data 失败: {:?}",
                                                                err
                                                            );
                                                        }
                                                        println!(
                                                            "[Async] 图标数据插入耗时: {:?}",
                                                            db_icon_insert_start.elapsed()
                                                        );
                                                        println!(
                                                            "[Async] 图标任务总耗时: {:?}",
                                                            task_start.elapsed()
                                                        );
                                                    }
                                                    Err(err) => {
                                                        eprintln!("⚠️ get_file_icon 失败: {}", err);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("❌ 复制 {:?} 失败: {}", path, e);
                                    }
                                }
                            }
                        }

                        if has_new_files {
                            let emit_start = Instant::now();
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.emit("clipboard-updated", "");
                            }
                            println!("[Main] 事件发送耗时: {:?}", emit_start.elapsed());
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

                        // 能否被插入，取决于配置中的筛选条件
                        let can_insert = {
                            if let Some(lock) = CONFIG.get() {
                                let cfg = lock.read().unwrap();
                                cfg.ignore_short_text_len == 0 // 0 表示不限制（不忽略短文本）
                                    || size.unwrap_or(0) >= cfg.ignore_short_text_len as u64
                            } else {
                                true // 默认允许插入
                            }
                        };
                        if can_insert {
                            if let Err(e) = db::insert_received_db_data(new_item) {
                                eprintln!("❌ 保存文本数据到数据库失败: {:?}", e);
                            } else {
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    let _ = window.emit("clipboard-updated", "");
                                }
                            }
                        } else {
                            println!("⚠️ 文本长度不足，忽略插入");
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

/// 启动后台数据库清理线程
/// **功能**：
/// - 在收到插入通知后进行去抖并执行清理
/// - 定期（每5分钟）自动执行一次清理
/// - 根据配置执行过期数据清理和数量限制清理
pub fn start_cleanup_worker() {
    use std::sync::mpsc::channel;

    let (tx, rx) = channel();

    // 将 Sender 设置到 db 模块
    db::set_cleanup_sender(tx);

    std::thread::spawn(move || {
        println!("🧹 后台清理线程已启动");

        // 去抖：在收到通知后等待短时间合并多次通知
        let debounce = Duration::from_millis(500);
        // 定期检查间隔（防止长时间无人触发时也做一次清理）
        let periodic = Duration::from_secs(60 * 5); // 5 分钟

        loop {
            let start = Instant::now();
            match rx.recv_timeout(periodic) {
                Ok(_) => {
                    // 收到触发，短暂去抖等待更多触发
                    thread::sleep(debounce);
                    // 清空通道中可能积累的其他通知
                    while rx.try_recv().is_ok() {}
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // 周期性唤醒，继续执行清理
                }
                Err(_) => {
                    // 通道已断开，退出线程
                    println!("🛑 后台清理线程退出");
                    break;
                }
            }

            // 读取配置
            let (max_items, retention_days) = if let Some(lock) = CONFIG.get() {
                let cfg = lock.read().unwrap();
                (cfg.max_history_items, cfg.retention_days)
            } else {
                (500u32, 30u32) // 默认值
            };

            // 执行过期清理
            match db::clear_data_expired(retention_days) {
                Ok(deleted) => {
                    if deleted > 0 {
                        println!("🧹 后台清理: 删除了 {} 条过期记录", deleted);
                    }
                }
                Err(e) => eprintln!("❌ 后台清理: 过期数据清理失败: {}", e),
            }

            // 执行数量限制清理
            match db::enforce_max_history_items(max_items) {
                Ok(deleted) => {
                    if deleted > 0 {
                        println!("🧹 后台清理: 删除了 {} 条超量记录", deleted);
                    }
                }
                Err(e) => eprintln!("❌ 后台清理: 数量限制清理失败: {}", e),
            }

            // 如果上次 recv 很快就返回，保证循环不会 100% 占用 CPU
            let elapsed = start.elapsed();
            if elapsed < Duration::from_millis(100) {
                thread::sleep(Duration::from_millis(100) - elapsed);
            }
        }
    });
}
