// src/app_setup.rs

use crate::clipboard::ClipboardItem;
use crate::db;
use chrono::Utc;
use image::ColorType;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, WebviewWindow};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

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
    // 1. 定义主要快捷键和备用快捷键
    let primary_shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyV);
    let alternative_shortcut = Shortcut::new(
        Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
        Code::KeyV,
    );

    // 2. 为闭包提前克隆变量
    let handle_for_closure = handle.clone();
    let primary_for_handler = primary_shortcut.clone();
    let alternative_for_handler = alternative_shortcut.clone();

    // 3. 设置能够响应任一快捷键的处理器
    handle.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, shortcut, event| {
                // 判断按下的快捷键是否是我们定义的两个之一
                if (shortcut == &primary_for_handler || shortcut == &alternative_for_handler)
                    && event.state() == ShortcutState::Pressed
                {
                    if let Some(window) = handle_for_closure.get_webview_window("main") {
                        println!("✅ 快捷键触发，执行窗口切换逻辑");
                        toggle_window_visibility(&window);
                    }
                }
            })
            .build(),
    )?;

    // 4. 尝试注册快捷键，并在失败时提供备用方案
    let manager = handle.global_shortcut();
    match manager.register(primary_shortcut) {
        Ok(_) => {
            println!("✅ 已注册全局快捷键: Alt-Shift-V");
        }
        Err(e) => {
            eprintln!("⚠️ 注册 Alt-Shift-V 失败: {:?}. 正在尝试备用快捷键...", e);
            // 尝试注册备用快捷键
            match manager.register(alternative_shortcut) {
                Ok(_) => {
                    println!("✅ 已成功注册备用全局快捷键: Ctrl-Alt-Shift-V");
                }
                Err(e2) => {
                    eprintln!("❌ 注册备用快捷键也失败了: {:?}", e2);
                }
            }
        }
    };
    Ok(())
}
/// 启动后台线程以监控剪贴板
pub fn start_clipboard_monitor(app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        let mut last_text = String::new();
        let mut last_image_bytes: Vec<u8> = Vec::new();
        let mut last_file_paths: Vec<PathBuf> = Vec::new();

        let app_dir = app_handle.path().app_data_dir().unwrap();
        let files_dir = app_dir.join("files");
        fs::create_dir_all(&files_dir).unwrap();

        loop {
            // --- 1. 监听文本 ---
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

            // --- 2. 监听图片 ---
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

            // --- 3. 监听文件 ---
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
