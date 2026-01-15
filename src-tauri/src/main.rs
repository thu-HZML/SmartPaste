// src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 声明模块
mod app_setup;
mod clipboard;
mod config;
mod db;
mod ocr;
mod qrcode;
mod utils;

// 注册性能测试模块 (仅在测试模式下编译)
#[cfg(test)]
#[path = "test_unit/test_performance.rs"]
mod test_performance;

use app_setup::{
    get_all_shortcuts, get_current_shortcut, update_shortcut, AppShortcutManager,
    ClipboardSourceState,
};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_notification;
use utils::EncryptionState;

fn main() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init()) // 文件系统插件
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]), // 可以传递启动参数，这里为空
        ))
        .plugin(tauri_plugin_notification::init())
        .manage(AppShortcutManager::new())
        .manage(ClipboardSourceState {
            is_frontend_copy: Mutex::new(false),
        })
        .manage(EncryptionState {
            dek: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            utils::test_function,
            utils::write_to_clipboard,
            utils::write_file_to_clipboard,
            utils::copy_file_to_clipboard,
            utils::start_key_listener,
            utils::stop_key_listener,
            update_shortcut,
            get_current_shortcut,
            get_all_shortcuts,
            utils::get_file_icon,
            utils::write_files_to_clipboard,
            utils::export_to_zip,
            utils::import_data_from_zip,
            utils::start_mouse_button_listener,
            utils::start_mouse_move_listener,
            utils::stop_mouse_listener,
            utils::get_utils_dir_path,
            utils::read_file_to_frontend,
            utils::get_screen_resolution,
            db::insert_received_text_data,
            db::insert_received_data,
            db::get_all_data,
            db::get_latest_data,
            db::get_data_by_id,
            db::delete_all_data,
            db::delete_data,
            db::delete_data_by_id,
            db::update_data_content_by_id,
            db::set_favorite_status_by_id,
            db::favorite_data_by_id,
            db::unfavorite_data_by_id,
            db::filter_data_by_favorite,
            db::get_favorite_data_count,
            db::add_notes_by_id,
            db::filter_data_by_type,
            db::comprehensive_search,
            db::create_new_folder,
            db::rename_folder,
            db::delete_folder,
            db::get_all_folders,
            db::add_item_to_folder,
            db::remove_item_from_folder,
            db::filter_data_by_folder,
            db::get_folders_by_item_id,
            db::get_ocr_text_by_item_id,
            db::search_data_by_ocr_text,
            db::get_icon_data_by_item_id,
            db::mark_passwords_as_private,
            db::prepare_encrypted_db_upload,
            db::restore_from_encrypted_db,
            db::mark_bank_cards_as_private,
            db::mark_identity_numbers_as_private,
            db::mark_phone_numbers_as_private,
            db::clear_all_private_data,
            db::auto_mark_private_data,
            db::check_and_mark_private_item,
            db::trigger_cleanup,
            db::sync_cloud_data,
            db::sync_encrypted_cloud_data,
            db::encrypt_file,
            db::decrypt_file,
            db::generate_salt,
            db::generate_dek,
            db::derive_mk,
            db::wrap_dek,
            db::unwrap_dek,
            db::delete_temp_encrypted_file,
            db::top_data_by_id,
            ocr::configure_ocr,
            ocr::ocr_image,
            config::get_config_json,
            config::set_config_item,
            config::get_config_item,
            config::sync_and_apply_config,
            config::force_set_storage_path_to_config_dir,
            config::get_config_directory_path,
            qrcode::recognize_qrcode,
            utils::read_file_base64,
            utils::get_local_files_to_upload,
            utils::read_db_file_base64,
            utils::save_clipboard_file,
            utils::download_cloud_file,
            utils::set_dek_state,
            utils::get_dek_state,
            utils::clear_dek_state
        ])
        .setup(move |app| {
            // 1. 获取系统默认的应用数据目录
            let app_default_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            if !app_default_dir.exists() {
                std::fs::create_dir_all(&app_default_dir).expect("无法创建默认应用目录");
            }

            // 2. 初始化引导配置 - 先从默认位置加载
            let default_config_path = app_default_dir.join("config.json");
            config::set_config_path(default_config_path.clone());
            let init_result = config::init_config();
            println!("配置初始化结果: {}", init_result);

            // 3. 确定最终的数据存储根目录
            let mut data_root = app_default_dir.clone();
            let custom_storage_path: Option<String> = if let Some(lock) = config::CONFIG.get() {
                let cfg = lock.read().unwrap();
                cfg.storage_path.clone()
            } else {
                None
            };
            // 接着使用提取出来的字符串进行逻辑处理
            if let Some(ref path_str) = custom_storage_path {
                // 规范化路径逻辑
                #[cfg(target_os = "windows")]
                let custom_path = PathBuf::from(path_str.replace("/", "\\"));
                #[cfg(not(target_os = "windows"))]
                let custom_path = PathBuf::from(path_str);

                if !path_str.trim().is_empty() {
                    println!("✅ 检测到配置的存储路径: {}", custom_path.display());

                    // 检查自定义路径是否存在，如果不存在则创建
                    if !custom_path.exists() {
                        println!("📁 创建存储路径: {}", custom_path.display());
                        if let Err(e) = std::fs::create_dir_all(&custom_path) {
                            eprintln!("❌ 创建存储路径失败: {}", e);
                        } else {
                            data_root = custom_path.clone();
                        }
                    } else {
                        data_root = custom_path.clone();
                    }

                    // 检查新路径下是否有配置文件
                    let new_config_path = data_root.join("config.json");
                    if new_config_path.exists() {
                        println!(
                            "📄 检测到新路径下的配置文件，切换到: {}",
                            new_config_path.display()
                        );
                        config::set_config_path(new_config_path.clone());

                        // 🔥 这里现在可以安全地调用 reload_config 了，因为外面没有持有读锁
                        let reload_result = config::reload_config();
                        println!("重新加载配置结果: {}", reload_result);
                    } else {
                        println!("ℹ️ 新路径下没有配置文件，将使用默认配置路径");
                        // 如果新路径没有配置文件，但存储路径已设置，我们创建一个
                        println!("📝 在新路径创建配置文件");

                        // 这里需要再次获取读锁来复制配置，但这没问题，因为上面的锁已经释放了
                        if let Some(lock) = config::CONFIG.get() {
                            let config_to_save = lock.read().unwrap().clone();
                            config::set_config_path(new_config_path.clone());
                            if let Err(e) = config::save_config(config_to_save) {
                                eprintln!("❌ 创建新路径配置文件失败: {}", e);
                                // 恢复默认路径
                                config::set_config_path(default_config_path.clone());
                            } else {
                                println!("✅ 新路径配置文件创建成功");
                            }
                        }
                    }
                }
            }

            // 4. 配置各类文件的最终路径
            let final_db_path = data_root.join("smartpaste.db");
            let final_files_dir = data_root.join("files");

            // 5. 确保 files 文件夹存在
            if !final_files_dir.exists() {
                std::fs::create_dir_all(&final_files_dir).expect("无法创建 files 文件夹");
            }

            // 6. 设置数据库路径
            println!("📂 数据库路径设置为: {}", final_db_path.to_string_lossy());
            db::set_db_path(final_db_path);

            // 6.1 执行初始化清理（清除过期数据）
            if let Some(lock) = config::CONFIG.get() {
                let retention_days = lock.read().unwrap().retention_days;
                println!("🧹 执行初始化清理，保留天数: {} 天", retention_days);
                match db::clear_data_expired(retention_days) {
                    Ok(deleted) => {
                        if deleted > 0 {
                            println!("   ✅ 初始化清理: 删除了 {} 条过期记录", deleted);
                        } else {
                            println!("   ✅ 初始化清理: 没有过期数据");
                        }
                    }
                    Err(e) => eprintln!("   ❌ 初始化清理失败: {}", e),
                }
            }

            // 6.2 启动后台清理线程（实时监听和定期清理）
            app_setup::start_cleanup_worker();

            // 7. 打印最终使用的配置路径
            let current_config_path = config::get_config_path();
            println!("📄 最终配置文件路径: {}", current_config_path.display());

            // 8. 根据配置自动标记隐私数据
            if let Some(lock) = config::CONFIG.get() {
                let cfg = lock.read().unwrap();
                println!("🔒 正在根据配置初始化隐私数据标记...");
                match db::auto_mark_private_data(
                    cfg.filter_passwords,
                    cfg.filter_bank_cards,
                    cfg.filter_id_cards,
                    cfg.filter_phone_numbers,
                ) {
                    Ok(count) => println!("✅ 初始化隐私标记完成，受影响记录数: {}", count),
                    Err(e) => eprintln!("❌ 初始化隐私标记失败: {}", e),
                }
            }

            // 9. 获取OCR配置并初始化OCR引擎
            if let Some(lock) = config::CONFIG.get() {
                let cfg = lock.read().unwrap();
                println!("👁️ 正在初始化 OCR 引擎...");

                let provider = cfg.ocr_provider.clone();
                let languages = cfg.ocr_languages.clone();

                // 转换 Vec<String> 为 Vec<&str> 以匹配 configure_ocr 签名
                let languages_ref: Option<Vec<&str>> = if let Some(ref langs) = languages {
                    Some(langs.iter().map(|s| s.as_str()).collect())
                } else {
                    None
                };

                let confidence = cfg.ocr_confidence_threshold;
                let timeout = cfg.ocr_timeout_secs;

                match ocr::configure_ocr(provider, languages_ref, confidence, timeout) {
                    Ok(msg) => println!("✅ OCR引擎初始化成功: {}", msg),
                    Err(e) => eprintln!("❌ OCR引擎初始化失败: {}", e),
                }
            }

            // 打印当前配置的存储路径用于验证
            if let Some(lock) = config::CONFIG.get() {
                let cfg = lock.read().unwrap();
                println!("📍 配置中记录的存储路径: {:?}", cfg.storage_path);
                println!("📍 最终数据根目录: {}", data_root.display());

                // 验证存储路径是否与最终数据根目录一致
                if let Some(ref storage_path) = cfg.storage_path {
                    let storage_path_buf = PathBuf::from(storage_path);
                    if storage_path_buf != data_root {
                        println!("⚠️ 警告: 配置中的存储路径与最终数据根目录不一致");
                        println!("  配置存储路径: {}", storage_path);
                        println!("  实际数据根目录: {}", data_root.display());
                    }
                }
            }

            let tray_icon_visible = if let Some(lock) = config::CONFIG.get() {
                lock.read().unwrap().tray_icon_visible
            } else {
                true // 默认显示
            };

            if tray_icon_visible {
                // 只有在 visible 为 true 时才创建托盘图标
                app_setup::setup_tray(app)?;
                println!("✅ 托盘图标已创建");
            } else {
                // 如果是 false，则不创建托盘图标
                println!("🚫 托盘图标配置为不可见，跳过创建");
            }
            app_setup::setup_global_shortcuts(app.handle().clone())?;

            let handle = app.handle().clone();
            app_setup::start_clipboard_monitor(handle);

            if let Some(window) = app.get_webview_window("main") {
                window.hide()?;
            }

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
