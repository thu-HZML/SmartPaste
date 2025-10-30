// use tauri::Manager;
// use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
// use tauri_plugin_clipboard_manager::ClipboardExt;
// use std::thread;
// use std::time::Duration;
// use serde::{Deserialize, Serialize};
// use std::fs;
// use std::path::PathBuf;
// use chrono::Utc;
// use image::ColorType;
// use arboard::Clipboard;

// // 定义剪贴板内容的结构体 (保持不变)
// #[derive(Serialize, Deserialize, Debug, Clone)]
// struct ClipboardItem {
//     id: String,
//     txt: Option<String>,
//     image_path: Option<String>,
//     file_path: Option<String>,
//     is_favorite: bool,
//     notes: String,
//     timestamp: i64,
// }

// fn main() {
//     let result = tauri::Builder::default()
//         .plugin(tauri_plugin_clipboard_manager::init())
//         .setup(|app| {
//             let show_hide_shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyV);
//             let shortcut_for_handler = show_hide_shortcut.clone();
//             let handle = app.handle().clone();

//             // 快捷键处理逻辑 (保持不变)
//             app.handle().plugin(
//                 tauri_plugin_global_shortcut::Builder::new()
//                     .with_handler(move |_app, shortcut, event| {
//                         if shortcut == &shortcut_for_handler {
//                             if event.state() == ShortcutState::Pressed {
//                                 println!("✅ 按键被按下，执行窗口切换逻辑");
//                                 let window = handle.get_webview_window("main").unwrap();

//                                 if let Ok(minimized) = window.is_minimized() {
//                                     if minimized {
//                                         window.unminimize().unwrap();
//                                         window.set_focus().unwrap();
//                                         return;
//                                     }
//                                 }

//                                 if let Ok(visible) = window.is_visible() {
//                                     if visible {
//                                         window.hide().unwrap();
//                                     } else {
//                                         window.show().unwrap();
//                                         window.set_focus().unwrap();
//                                     }
//                                 }
//                             }
//                         }
//                     })
//                     .build()
//             )?;

//             app.global_shortcut().register(show_hide_shortcut)?;
//             println!("✅ 已注册全局快捷键 Alt+Shift+V");

//             // 剪贴板监听逻辑
//             let app_handle = app.handle().clone();

//             thread::spawn(move || {
//                 let mut last_text = String::new();
//                 let mut last_image_bytes: Vec<u8> = Vec::new();

//                 let app_dir = app_handle.path().app_data_dir().unwrap();
//                 let images_dir = app_dir.join("images");
//                 let json_path = app_dir.join("clipboard_history.json");

//                 if !images_dir.exists() {
//                     fs::create_dir_all(&images_dir).unwrap();
//                 }

//                 loop {
//                     // 监听文本 (逻辑不变)
//                     if let Ok(text) = app_handle.clipboard().read_text() {
//                         if !text.is_empty() && text != last_text {
//                             println!("检测到新的文本内容: {}", text);
//                             last_text = text.clone();
//                             last_image_bytes.clear();

//                             let new_item = ClipboardItem {
//                                 id: Utc::now().timestamp_millis().to_string(),
//                                 txt: Some(text),
//                                 image_path: None,
//                                 file_path: None,
//                                 is_favorite: false,
//                                 notes: "".to_string(),
//                                 timestamp: Utc::now().timestamp(),
//                             };
//                             save_to_json(&json_path, new_item);
//                         }
//                     }

//                     // 监听图片 (逻辑已修正)
//                     if let Ok(image) = app_handle.clipboard().read_image() {
//                         // 【修正】直接访问 .rgba 字段，并转换为 Vec<u8>
//                         let current_image_bytes = image.rgba().to_vec();

//                         if !current_image_bytes.is_empty() && current_image_bytes != last_image_bytes {
//                             println!("检测到新的图片内容");
//                             last_text.clear();

//                             let image_id = Utc::now().timestamp_millis().to_string();
//                             let image_path = images_dir.join(format!("{}.png", image_id));

//                             // 【修正】使用 image::save_buffer 来保存文件
//                             // 我们需要提供路径、字节数据、宽度、高度和颜色类型
//                             match image::save_buffer(
//                                 &image_path,
//                                 &current_image_bytes,
//                                 image.width() as u32,
//                                 image.height() as u32,
//                                 ColorType::Rgba8, // 颜色类型是 RGBA, 8位深度
//                             ) {
//                                 Ok(_) => {
//                                     println!("图片已保存到: {:?}", image_path);
//                                     let new_item = ClipboardItem {
//                                         id: image_id,
//                                         txt: None,
//                                         image_path: Some(image_path.to_str().unwrap().to_string()),
//                                         file_path: None,
//                                         is_favorite: false,
//                                         notes: "".to_string(),
//                                         timestamp: Utc::now().timestamp(),
//                                     };
//                                     save_to_json(&json_path, new_item);

//                                     // 保存成功后，才更新 last_image_bytes
//                                     last_image_bytes = current_image_bytes;
//                                 }
//                                 Err(e) => {
//                                     eprintln!("❌ 保存图片失败: {:?}", e);
//                                 }
//                             }
//                         }
//                     }
                    
//                     thread::sleep(Duration::from_millis(500));
//                 }
//             });
//             Ok(())
//         })
//         .run(tauri::generate_context!());

//     if let Err(e) = result {
//         eprintln!("❌ 启动 Tauri 应用失败: {:?}", e);
//     }
// }

// // save_to_json 函数 (保持不变)
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
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_clipboard_manager::ClipboardExt;
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::Utc;
use image::ColorType;

// 剪贴板项目结构体
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClipboardItem {
    id: String,
    txt: Option<String>,
    image_path: Option<String>,
    file_path: Option<String>,
    is_favorite: bool,
    notes: String,
    timestamp: i64,
}

fn main() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let show_hide_shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyV);
            let shortcut_for_handler = show_hide_shortcut.clone();
            let handle = app.handle().clone();

            // 快捷键处理 
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if shortcut == &shortcut_for_handler && event.state() == ShortcutState::Pressed {
                            println!("✅ 按键被按下，执行窗口切换逻辑");
                            let window = handle.get_webview_window("main").unwrap();
                            if let Ok(minimized) = window.is_minimized() {
                                if minimized { window.unminimize().unwrap(); window.set_focus().unwrap(); return; }
                            }
                            if let Ok(visible) = window.is_visible() {
                                if visible { window.hide().unwrap(); } else { window.show().unwrap(); window.set_focus().unwrap(); }
                            }
                        }
                    })
                    .build()
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
                let images_dir = app_dir.join("images");
                let files_dir = app_dir.join("files");
                let json_path = app_dir.join("clipboard_history.json");

                fs::create_dir_all(&images_dir).unwrap();
                fs::create_dir_all(&files_dir).unwrap();

                loop {
                    if let Ok(text) = app_handle.clipboard().read_text() {
                        if !text.is_empty() && text != last_text {
                            println!("检测到新的文本内容: {}", text);
                            last_text = text.clone();
                            last_image_bytes.clear();
                            last_file_paths.clear();
                            let new_item = ClipboardItem {
                                id: Utc::now().timestamp_millis().to_string(),
                                txt: Some(text),
                                image_path: None, file_path: None, is_favorite: false,
                                notes: "".to_string(), timestamp: Utc::now().timestamp(),
                            };
                            save_to_json(&json_path, new_item);
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
                            let image_path = images_dir.join(format!("{}.png", image_id));
                            if image::save_buffer(&image_path, &image.rgba(), image.width() as u32, image.height() as u32, ColorType::Rgba8).is_ok() {
                                println!("图片已保存到: {:?}", image_path);
                                let new_item = ClipboardItem {
                                    id: image_id,
                                    txt: None, image_path: Some(image_path.to_str().unwrap().to_string()),
                                    file_path: None, is_favorite: false,
                                    notes: "".to_string(), timestamp: Utc::now().timestamp(),
                                };
                                save_to_json(&json_path, new_item);
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
                                if let Some(file_name) = original_path.file_name().and_then(|n| n.to_str()) {
                                    let timestamp = Utc::now().timestamp_millis();
                                    let new_file_name = format!("{}-{}", timestamp, file_name);
                                    let destination_path = files_dir.join(&new_file_name);

                                    if fs::copy(&original_path, &destination_path).is_ok() {
                                        println!("文件已复制到: {:?}", destination_path);
                                        let new_item = ClipboardItem {
                                            id: timestamp.to_string(),
                                            txt: None, image_path: None,
                                            file_path: Some(destination_path.to_str().unwrap().to_string()),
                                            is_favorite: false,
                                            notes: "".to_string(),
                                            timestamp: Utc::now().timestamp(),
                                        };
                                        save_to_json(&json_path, new_item);
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

fn save_to_json(path: &PathBuf, new_item: ClipboardItem) {
    let mut history: Vec<ClipboardItem> = if path.exists() {
        let file_content = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
        serde_json::from_str(&file_content).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };
    history.insert(0, new_item);
    let json_string = serde_json::to_string_pretty(&history).unwrap();
    fs::write(path, json_string).expect("无法写入 JSON 文件");
    println!("剪贴板历史已更新");
}