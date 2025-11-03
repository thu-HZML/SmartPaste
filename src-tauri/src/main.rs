

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClipboardItem {
    id: String,
    item_type: String, // 数据类型：text/image/file
    content: String, // 对text类型，存储文本内容；对其他类型，存储文件路径  txt:// txt: Option<String>,  file// _path: Option<String>,
    is_favorite: bool,
    notes: String,
    timestamp: i64,
}

use chrono::Utc;
use image::ColorType;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::thread;
use tauri::{menu::{Menu, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent}, Manager, PhysicalPosition};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

mod db;


fn main() {
    // 使用共享状态来防止重复点击
    let last_click_time = Arc::new(Mutex::new(Instant::now()));
    
    let result = tauri::Builder::default()
        // 注册 Tauri commands
        .invoke_handler(tauri::generate_handler![
            db::insert_received_data,
            db::get_all_data,
            db::get_data_by_id,
            db::delete_data,
            db::delete_data_by_id,
            db::favorite_data_by_id,
            db::search_text_content,
            db::add_notes_by_id
        ])
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(move |app| {
            let click_time_clone = Arc::clone(&last_click_time);
            
            // 创建托盘菜单
            let menu = Menu::new(app)?;
            
            // 创建菜单项
            let show_hide = MenuItem::with_id(app, "show_hide", "显示/隐藏", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            
            // 添加菜单项到菜单
            menu.append(&show_hide)?;
            menu.append(&quit)?;

            // 创建托盘图标
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("桌面宠物")
                .on_menu_event(move |app, event| {
                    println!("🖱️ 菜单项点击: {}", event.id().as_ref());
                    
                    if let Some(window) = app.get_webview_window("main") {
                        match event.id().as_ref() {
                            "show_hide" => {
                                toggle_window_visibility(&window);
                            }
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
                            
                            // 防抖处理：如果距离上次点击太近（小于200毫秒），忽略这次点击
                            if now.duration_since(*last_time) < Duration::from_millis(200) {
                                println!("⏰ 忽略重复点击");
                                return;
                            }
                            
                            *last_time = now;
                            
                            println!("🎯 托盘点击事件: {:?}", button);
                            
                            match button {
                                tauri::tray::MouseButton::Left => {
                                    // 左键点击：只切换显示/隐藏，不显示菜单
                                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                                        toggle_window_visibility(&window);
                                    }
                                    // 重要：左键点击后不显示菜单
                                }
                                tauri::tray::MouseButton::Right => {
                                    // 右键点击：显示菜单
                                    println!("📋 右键点击，显示菜单");
                                    // 右键菜单由系统自动处理
                                }
                                _ => {}
                            }
                        }
                        TrayIconEvent::DoubleClick { .. } => {
                            println!("🖱️ 托盘双击事件");
                            // 双击也可以用来切换显示/隐藏
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                toggle_window_visibility(&window);
                            }
                        }
                        _ => {
                            // 移除了其他事件的日志输出
                        }
                    }
                })
                .build(app)?;

            println!("✅ 托盘图标创建成功");
            /*
            // 设置窗口初始位置到右下角
            if let Some(window) = app.get_webview_window("main") {
                window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: 150, height: 150 }))?;
                
                if let Ok(monitor) = window.current_monitor() {
                    if let Some(monitor) = monitor {
                        let screen_size = monitor.size();
                        let x = screen_size.width as i32 - 150 - 20;
                        let y = screen_size.height as i32 - 150 - 20;
                        window.set_position(tauri::Position::Physical(PhysicalPosition { x, y }))?;
                        println!("📍 设置窗口位置: x={}, y={}", x, y);
                    }
                } else {
                    let x = 100;
                    let y = 100;
                    window.set_position(tauri::Position::Physical(PhysicalPosition { x, y }))?;
                    println!("⚠️ 使用默认窗口位置: x={}, y={}", x, y);
                }

                window.show()?;
                println!("🪟 窗口初始显示状态设置完成");
            }
            */
            // 全局快捷键设置
            // 取得 app_data_dir 并设置到 db 模块
            let app_dir = app.path().app_data_dir().unwrap();
            let db_path = app_dir.join("smartpaste.db");
            // 确保目录存在
            std::fs::create_dir_all(&app_dir).ok();
            db::set_db_path(db_path);

            // 现有快捷键 / 线程 / 文件路径逻辑继续使用 app_dir
            let files_dir = app_dir.join("files");
            std::fs::create_dir_all(&files_dir).unwrap();

            let show_hide_shortcut =
                Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyV);
            let shortcut_for_handler = show_hide_shortcut.clone();
            let handle = app.handle().clone();

            // 快捷键处理
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if shortcut == &shortcut_for_handler
                            && event.state() == ShortcutState::Pressed
                        {
                            println!("✅ 按键被按下，执行窗口切换逻辑");
                            let window = handle.get_webview_window("main").unwrap();
                            if let Ok(minimized) = window.is_minimized() {
                                if minimized {
                                    window.unminimize().unwrap();
                                    window.set_focus().unwrap();
                                    return;
                                }
                            }
                            if let Ok(visible) = window.is_visible() {
                                if visible {
                                    window.hide().unwrap();
                                } else {
                                    window.show().unwrap();
                                    window.set_focus().unwrap();
                                }
                            }
                        }
                    })
                    .build(),
            )?;
            app.global_shortcut().register(show_hide_shortcut)?;
            println!("✅ 已注册全局快捷键 Alt-Shift-V");

            // 剪贴板监听线程
            let app_handle = app.handle().clone();
            thread::spawn(move || {
                let mut last_text = String::new();
                let mut last_image_bytes: Vec<u8> = Vec::new();
                let mut last_file_paths: Vec<PathBuf> = Vec::new();

                let app_dir = app_handle.path().app_data_dir().unwrap();
                let files_dir = app_dir.join("files");
                // let json_path = app_dir.join("clipboard_history.json");
                fs::create_dir_all(&files_dir).unwrap();

                loop {
                    // --- 1. 监听文本 ---
                    if let Ok(text) = app_handle.clipboard().read_text() {
                        if !text.is_empty() && text != last_text {
                            println!("检测到新的文本内容: {}", text);
                            last_text = text.clone();
                            last_image_bytes.clear();
                            last_file_paths.clear();
                            let new_item = ClipboardItem {
                                id: Utc::now().timestamp_millis().to_string(),

                                item_type: "text".to_string(),
                                content: text.clone(),
                                //   txt://  Some(text),
                                // file// _path: None,
                                is_favorite: false,
                                notes: "".to_string(),
                                timestamp: Utc::now().timestamp(),
                            };
                            db::insert_received_data(new_item.clone()).unwrap();
                            match db::insert_received_data(new_item.clone()) {
                                Ok(_) => println!("文本数据已保存到数据库"),
                                Err(e) => eprintln!("❌ 保存文本数据到数据库失败: {:?}", e),
                            }
                            // save_to_json(&json_path, new_item);
                        }
                    }

                    // --- 2. 监听图片 (并作为文件处理) ---
                    if let Ok(image) = app_handle.clipboard().read_image() {
                        let current_image_bytes = image.rgba().to_vec();
                        if !current_image_bytes.is_empty()
                            && current_image_bytes != last_image_bytes
                        {
                            println!("检测到新的图片内容");
                            last_image_bytes = current_image_bytes.clone();
                            last_text.clear();
                            last_file_paths.clear();

                            let image_id = Utc::now().timestamp_millis().to_string();
                            let destination_path = files_dir.join(format!("{}.png", image_id));

                            if image::save_buffer(
                                &destination_path,
                                &image.rgba(),
                                image.width() as u32,
                                image.height() as u32,
                                ColorType::Rgba8,
                            )
                            .is_ok()
                            {
                                println!("图片已作为文件保存到: {:?}", destination_path);
                                let new_item = ClipboardItem {
                                    id: image_id,
                                    item_type: "image".to_string(),
                                    content: destination_path.to_str().unwrap().to_string(),
                                    is_favorite: false,
                                    notes: "".to_string(),
                                    timestamp: Utc::now().timestamp(),
                                };
                                db::insert_received_data(new_item.clone()).unwrap();
                                match db::insert_received_data(new_item.clone()) {
                                    Ok(_) => println!("图片数据已保存到数据库"),
                                    Err(e) => {
                                        eprintln!("❌ 保存图片数据到数据库失败: {:?}", e)
                                    }
                                }
                                // save_to_json(&json_path, new_item);
                            }
                        }
                    }

                    if let Ok(paths) = clipboard_files::read() {
                        if !paths.is_empty() && paths != last_file_paths {
                            println!("检测到新的文件复制: {:?}", paths);
                            last_file_paths = paths.clone();
                            last_text.clear();
                            last_image_bytes.clear();

                            for original_path in paths {
                                if let Some(file_name) =
                                    original_path.file_name().and_then(|n| n.to_str())
                                {
                                    let timestamp = Utc::now().timestamp_millis();
                                    let new_file_name = format!("{}-{}", timestamp, file_name);
                                    let destination_path = files_dir.join(&new_file_name);

                                    if fs::copy(&original_path, &destination_path).is_ok() {
                                        println!("文件已复制到: {:?}", destination_path);
                                        let new_item = ClipboardItem {
                                            id: timestamp.to_string(),
                                            item_type: "file".to_string(),
                                            content: destination_path.to_str().unwrap().to_string(),
                                            is_favorite: false,
                                            notes: "".to_string(),
                                            timestamp: Utc::now().timestamp(),
                                        };
                                        db::insert_received_data(new_item.clone()).unwrap();
                                        match db::insert_received_data(new_item.clone()) {
                                            Ok(_) => println!("文件数据已保存到数据库"),
                                            Err(e) => {
                                                eprintln!("❌ 保存文件数据到数据库失败: {:?}", e)
                                            }
                                        }
                                        // save_to_json(&json_path, new_item);
                                    }
                                }
                            }
                        }
                    }

                    thread::sleep(Duration::from_millis(500));
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("❌ 启动 Tauri 应用失败: {:?}", e);
    }
}

// 切换窗口显示/隐藏的辅助函数
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
                }
                if let Err(e) = window.set_focus() {
                    eprintln!("⚠️ 设置窗口焦点失败: {:?}", e);
                } else {
                    println!("👀 显示桌宠窗口");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 获取窗口可见性失败: {:?}", e);
        }
    }
}