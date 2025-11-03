use chrono::Utc;
use image::ColorType;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClipboardItem {
    id: String,
    item_type: String, // 数据类型：text/image/file
    content: String, // 对text类型，存储文本内容；对其他类型，存储文件路径  txt:// txt: Option<String>,  file// _path: Option<String>,
    is_favorite: bool,
    notes: String,
    timestamp: i64,
}

mod db;

fn main() {
    let result = tauri::Builder::default()
        // 注册 Tauri commands
        .invoke_handler(tauri::generate_handler![
            // db::clipboard_item_to_json,
            // db::clipboard_items_to_json,
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
        .setup(|app| {
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
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("❌ 启动 Tauri 应用失败: {:?}", e);
    }
}

// save_to_json 函数无需修改，它会自动适应新的结构体
// fn save_to_json(path: &PathBuf, new_item: ClipboardItem) {
//     let mut history: Vec<ClipboardItem> = if path.exists() {
//         let file_content = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
//         serde_json::from_str(&file_content).unwrap_or_else(|_| vec![])
//     } else {
//         vec![]
//     };
//     history.insert(0, new_item);
//     let json_string = serde_json::to_string_pretty(&history).unwrap();
//     fs::write(path, json_string).expect("无法写入 JSON 文件");
//     println!("剪贴板历史已更新");
// }
