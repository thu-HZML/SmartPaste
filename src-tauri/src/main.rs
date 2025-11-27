// src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 声明模块
mod app_setup;
mod clipboard;
mod config;
mod db;
mod ocr;

// 注册性能测试模块 (仅在测试模式下编译)
#[cfg(test)]
mod test_performance;

use app_setup::{
    update_shortcut, update_shortcut2, AppShortcutState, AppShortcutState2, ClipboardSourceState,
};
use arboard::Clipboard;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_autostart::MacosLauncher;
use std::ffi::OsStr;
use std::io::Cursor;
use std::os::windows::ffi::OsStrExt;
use base64::{engine::general_purpose, Engine as _};
use image::{ImageFormat, RgbaImage};
use windows::core::{PCWSTR};
use windows::Win32::Foundation::{HWND};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC,BITMAP, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, GetIconInfo, HICON, ICONINFO,
};
#[tauri::command]
fn test_function() -> String {
    "这是来自 Rust 的测试信息".to_string()
}



#[tauri::command]
fn write_to_clipboard(
    text: String,
    app_handle: tauri::AppHandle,
    state: State<'_, ClipboardSourceState>,
) -> Result<(), String> {
    // 设置标志，表示这是前端触发的复制
    *state.is_frontend_copy.lock().unwrap() = true;

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;

    Ok(())
}
/// 将指定的文本写入系统剪贴板。作为 Tauri command 暴露给前端调用。
/// 此函数会设置一个状态标志，以区分是前端主动复制还是由其他程序引起的剪贴板变化。
/// # Param
/// text: String - 需要写入剪贴板的文本内容。
/// app_handle: tauri::AppHandle - Tauri 的应用句柄。
/// state: State<'_,ClipboardSourceState> - 用于管理剪贴板来源状态的 Tauri 状态。
/// # Returns
/// Result<(), String> - 操作成功则返回 Ok(())，失败则返回包含错误信息的 Err。
#[tauri::command]
async fn write_file_to_clipboard(
    app_handle: tauri::AppHandle,
    file_path: String,
    state: State<'_, ClipboardSourceState>,
) -> Result<(), String> {
    // 设置标志，表示这是前端触发的复制
    *state.is_frontend_copy.lock().unwrap() = true;
    let path = Path::new(&file_path);

    // 检查文件是否存在
    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    // 检查是否是文件（不是目录）
    // if !path.is_file() {
    //     return Err("路径指向的不是文件".to_string());
    // }

    // 获取文件的绝对路径
    let absolute_path =
        fs::canonicalize(path).map_err(|e| format!("无法获取文件绝对路径: {}", e))?;

    let mut final_path_str = absolute_path.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        // 去除 Rust canonicalize 产生的 \\?\ 前缀
        const VERBATIM_PREFIX: &str = r"\\?\";
        if final_path_str.starts_with(VERBATIM_PREFIX) {
            final_path_str = final_path_str[VERBATIM_PREFIX.len()..].to_string();
        }
    }
    // 根据不同平台调用相应的文件复制方法
    copy_file_to_clipboard(PathBuf::from(final_path_str))
}

/// 跨平台地将文件复制到系统剪贴板。作为 Tauri command 暴露给前端调用。
/// 此函数会根据编译的目标操作系统（Windows, macOS, Linux）调用相应的底层实现。
/// # Param
/// file_path: PathBuf - 要复制的文件的路径。
/// # Returns
/// Result<(), String> - 操作成功则返回 Ok(())，失败（如路径非法或底层实现出错）则返回包含错误信息的 Err。
#[tauri::command]
fn copy_file_to_clipboard(file_path: PathBuf) -> Result<(), String> {
    let file_path_str = file_path.to_str().ok_or("文件路径包含非法字符")?;

    #[cfg(target_os = "windows")]
    {
        copy_file_to_clipboard_windows(file_path_str)
    }

    #[cfg(target_os = "macos")]
    {
        copy_file_to_clipboard_macos(file_path_str)
    }

    #[cfg(target_os = "linux")]
    {
        copy_file_to_clipboard_linux(file_path_str)
    }
}

#[cfg(target_os = "windows")]
fn copy_file_to_clipboard_windows(file_path: &str) -> Result<(), String> {
    use std::process::Command;

    let ps_script = format!(
        "$sc = New-Object System.Collections.Specialized.StringCollection; $sc.Add('{}'); Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Clipboard]::SetFileDropList($sc);",
        file_path.replace("'", "''") // 转义 PowerShell 中的单引号
    );

    // 使用 -NoProfile 加快启动速度，-WindowStyle Hidden 隐藏窗口闪烁
    let output = Command::new("powershell")
        .args(&[
            "-NoProfile",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &ps_script,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    // 如果失败，读取 stderr 获取详细错误信息（方便调试）
    let err_msg = String::from_utf8_lossy(&output.stderr);
    Err(format!("复制文件到剪贴板失败: {}", err_msg))
}

#[cfg(target_os = "macos")]
fn copy_file_to_clipboard_macos(file_path: &str) -> Result<(), String> {
    use std::process::Command;

    // 使用AppleScript复制文件
    let apple_script = format!(
        "set the clipboard to POSIX file \"{}\"",
        file_path.replace("\"", "\\\"")
    );

    let output = Command::new("osascript")
        .args(&["-e", &apple_script])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    Err("复制文件到剪贴板失败".to_string())
}

#[cfg(target_os = "linux")]
fn copy_file_to_clipboard_linux(file_path: &str) -> Result<(), String> {
    use std::process::Command;

    // Linux上的文件复制比较复杂，尝试多种方法

    // 方法1: 使用xclip复制文件URI
    let file_uri = format!("file://{}", file_path);
    let output = Command::new("xclip")
        .args(&["-selection", "clipboard", "-t", "text/uri-list"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?
        .stdin
        .unwrap()
        .write_all(file_uri.as_bytes())
        .map_err(|e| e.to_string())?;

    // 检查xclip是否成功
    if Command::new("xclip")
        .args(&["-selection", "clipboard", "-o"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Ok(());
    }

    Err("Linux系统文件复制功能受限，请确保已安装xclip".to_string())
}
#[tauri::command]
fn get_current_shortcut(state: tauri::State<AppShortcutState>) -> String {
    state.current_shortcut.lock().unwrap().clone()
}

#[tauri::command]
fn get_current_shortcut2(state: tauri::State<AppShortcutState2>) -> String {
    state.current_shortcut.lock().unwrap().clone()
}

/// 获取文件的系统图标（Base64 格式，不包含文件夹）
#[tauri::command]
async fn get_file_icon(path: String) -> Result<String, String> {
    let p = Path::new(&path);

    // 1. 检查路径是否存在
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    #[cfg(target_os = "windows")]
    {
        // 调用 unsafe 的帮助函数来处理 Win32 API
        let icon_base64 = tauri::async_runtime::spawn_blocking(move || {
            extract_icon_base64(&path)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))??;

        Ok(format!("data:image/png;base64,{}", icon_base64))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("当前系统暂不支持图标提取".to_string())
    }
}


#[cfg(target_os = "windows")]
fn extract_icon_base64(path: &str) -> Result<String, String> {
    unsafe {
        let wide_path: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        
        let mut shfi = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0), 
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        if result == 0 || shfi.hIcon.is_invalid() {
            return Err("SHGetFileInfoW 失败或未找到图标".to_string());
        }

        let hicon = shfi.hIcon;
        let _icon_guard = ScopeGuard(hicon, |h| { let _ = DestroyIcon(h); });

        hicon_to_png_base64(hicon)
    }
}

#[cfg(target_os = "windows")]
unsafe fn hicon_to_png_base64(hicon: HICON) -> Result<String, String> {
    let mut icon_info = ICONINFO::default();
    GetIconInfo(hicon, &mut icon_info)
        .map_err(|e| format!("GetIconInfo 失败: {}", e))?;
    
    let _color_bmp_guard = ScopeGuard(icon_info.hbmColor, |h| { let _ = DeleteObject(h); });
    let _mask_bmp_guard = ScopeGuard(icon_info.hbmMask, |h| { let _ = DeleteObject(h); });

    let hdc_screen = GetDC(HWND(std::ptr::null_mut()));
    let _dc_guard = ScopeGuard(hdc_screen, |h| { let _ = ReleaseDC(HWND(std::ptr::null_mut()), h); });
    
    let mut bmp: BITMAP = std::mem::zeroed();
    
    // GetObjectW 参数转换
    if GetObjectW(
        windows::Win32::Graphics::Gdi::HGDIOBJ(icon_info.hbmColor.0), 
        std::mem::size_of::<BITMAP>() as i32, 
        Some(&mut bmp as *mut _ as *mut _)
    ) == 0 {
        return Err("GetObjectW 失败".to_string());
    }

    let width = bmp.bmWidth;
    let height = bmp.bmHeight;
    let pixel_count = (width * height) as usize;

    let mut bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, 
            biPlanes: 1,
            biBitCount: 32, 
            // BI_RGB 是 BI_COMPRESSION 类型，需要 .0 取出 u32
            biCompression: BI_RGB.0, 
            ..Default::default()
        },
        ..Default::default()
    };

    let mut pixels: Vec<u8> = vec![0; pixel_count * 4];

    if GetDIBits(
        hdc_screen,
        icon_info.hbmColor,
        0,
        height as u32,
        Some(pixels.as_mut_ptr() as *mut _),
        &mut bi,
        DIB_RGB_COLORS,
    ) == 0 {
        return Err("GetDIBits 失败".to_string());
    }

    // BGRA -> RGBA 转换
    for chunk in pixels.chunks_mut(4) {
        let b = chunk[0];
        let r = chunk[2];
        chunk[0] = r;
        chunk[2] = b;
    }

    let img_buffer = RgbaImage::from_raw(width as u32, height as u32, pixels)
        .ok_or("无法构建图像缓冲区")?;

    let mut png_data = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);
    
    // 使用 ImageFormat::Png
    img_buffer
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| format!("图片编码失败: {}", e))?;

    Ok(general_purpose::STANDARD.encode(png_data))
}

struct ScopeGuard<T: Copy, F: FnMut(T)>(T, F);

impl<T: Copy, F: FnMut(T)> Drop for ScopeGuard<T, F> {
    fn drop(&mut self) {
        (self.1)(self.0);
    }
}

fn main() {
    let result = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]), // 可以传递启动参数，这里为空
        ))
        .manage(AppShortcutState {
            current_shortcut: Mutex::new(String::new()),
        })
        .manage(AppShortcutState2 {
            current_shortcut: Mutex::new(String::new()),
        })
        .manage(ClipboardSourceState {
            is_frontend_copy: Mutex::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            test_function,
            write_to_clipboard,
            write_file_to_clipboard,
            copy_file_to_clipboard,
            update_shortcut,
            update_shortcut2,
            get_current_shortcut,
            get_current_shortcut2,
            get_file_icon,
            db::insert_received_text_data,
            db::insert_received_data,
            db::get_all_data,
            db::get_latest_data,
            db::get_data_by_id,
            db::delete_all_data,
            db::delete_unfavorited_data,
            db::delete_data,
            db::delete_data_by_id,
            db::update_data_content_by_id,
            db::set_favorite_status_by_id,
            db::favorite_data_by_id,
            db::unfavorite_data_by_id,
            db::filter_data_by_favorite,
            db::get_favorite_data_count,
            db::search_text_content,
            db::add_notes_by_id,
            db::filter_data_by_type,
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
            ocr::configure_ocr,
            ocr::ocr_image,
            config::get_config_json,
            config::set_autostart,
            config::is_autostart_enabled,
            config::set_tray_icon_visible,
            config::set_minimize_to_tray,
            config::set_auto_save,
            config::set_retention_days,
            config::set_max_history_items,
            config::set_ignore_short_text,
            config::set_ignore_big_file,
            config::add_ignored_app,
            config::remove_ignored_app,
            config::clear_all_ignored_apps,
            config::set_auto_classify,
            config::set_ocr_auto_recognition,
            config::set_delete_confirmation,
            config::set_keep_favorites,
            config::set_auto_sort,
            config::set_ai_enabled,
            config::set_ai_service,
            config::set_ai_api_key,
            config::set_ai_auto_tag,
            config::set_ai_auto_summary,
            config::set_ai_translation,
            config::set_ai_web_search,
            config::set_sensitive_filter,
            config::set_filter_passwords,
            config::set_filter_bank_cards,
            config::set_filter_id_cards,
            config::set_filter_phone_numbers,
            config::set_privacy_retention_days,
            config::get_privacy_records,
            config::delete_all_privacy_records,
            config::set_storage_path,
            config::set_auto_backup,
            config::set_backup_frequency,
            config::set_last_backup_path,
            config::set_cloud_sync_enabled,
            config::set_sync_frequency,
            config::set_sync_content_type,
            config::set_encrypt_cloud_data,
            config::set_sync_only_wifi,
            config::set_username,
            config::set_email,
            config::set_bio,
            config::set_avatar_path,
            config::set_ocr_provider,
            config::set_ocr_languages,
            config::set_ocr_confidence_threshold,
            config::set_ocr_timeout_secs,
        ])
        .setup(move |app| {
            // 初始化数据库路径
            let app_dir = app.path().app_data_dir().expect("无法获取应用数据目录");
            if !app_dir.exists() {
                std::fs::create_dir_all(&app_dir).expect("无法创建应用数据目录");
            }

            // 初始化配置文件
            let config_path = app_dir.join("config.json");
            config::set_config_path(config_path.clone());
            let init_result = config::init_config();
            println!("配置初始化结果: {}", init_result);

            // 设置数据库路径
            let mut db_path = app_dir.join("smartpaste.db");
            // db::set_db_path(db_path.clone());

            // 获取配置文件中的存储路径设置
            if let Some(lock) = config::CONFIG.get() {
                let cfg = lock.read().unwrap();
                // 如果配置中没有存储路径，则使用默认的 app_dir
                if cfg.storage_path.is_none() {
                    drop(cfg); // 释放读锁
                    config::set_storage_path(app_dir.to_string_lossy().to_string());
                }
                // 否则，使用配置中的存储路径
                else if let Some(ref path_str) = cfg.storage_path {
                    let custom_path = PathBuf::from(path_str);
                    if custom_path.exists() && custom_path.is_dir() {
                        drop(cfg); // 释放读锁
                        config::set_storage_path(custom_path.to_string_lossy().to_string());
                        db_path = custom_path.join("smartpaste.db");
                    } else {
                        eprintln!(
                            "⚠️ 配置的存储路径无效，使用默认路径: {}",
                            app_dir.to_string_lossy()
                        );
                        drop(cfg); // 释放读锁
                        config::set_storage_path(app_dir.to_string_lossy().to_string());
                    }
                }
            }

            // 以现有数据库路径，修改 Config 中的数据存储路径
            // let set_db_path_result = config::set_db_storage_path(db_path.clone());

            // 设置数据库路径并打印结果
            println!("设置数据库路径结果: {}", db_path.to_string_lossy());
            db::set_db_path(db_path.clone());
            // 调试：读取并打印数据库中所有记录
            /*
            match db::get_all_data() {
                Ok(json) => println!("DEBUG get_all_data: {}", json),
                Err(e) => eprintln!("DEBUG get_all_data error: {}", e),
            }
            */
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

            // 设置主窗口为透明 + 穿透
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

// 辅助函数：切换窗口显示/隐藏
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
                } else {
                    println!("👀 显示桌宠窗口");
                }
            }
        }
        Err(e) => eprintln!("❌ 获取窗口可见性失败: {:?}", e),
    }
}
