// src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 声明模块
mod app_setup;
mod clipboard;
mod db;

use tauri::Manager;

#[tauri::command]
fn test_function() -> String {
    "这是来自 Rust 的测试信息".to_string()
}

fn main() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            test_function,
            db::insert_received_data,
            db::get_all_data,
            db::get_latest_data,
            db::get_data_by_id,
            db::delete_data,
            db::delete_data_by_id,
            db::set_favorite_status_by_id,
            db::search_text_content,
            db::add_notes_by_id,
            db::create_new_folder,
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
            let handle = app.handle().clone();
            app_setup::start_clipboard_monitor(handle);

            // 初始隐藏主窗口，避免启动时闪烁
            if let Some(window) = app.get_webview_window("main") {
                window.hide()?;
            }

            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("❌ 启动 Tauri 应用失败: {:?}", e);
    }
}
