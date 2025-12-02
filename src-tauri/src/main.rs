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
    get_all_shortcuts, get_current_shortcut, update_shortcut, AppShortcutManager,
    ClipboardSourceState,
};
use arboard::Clipboard;
use base64::{engine::general_purpose, Engine as _};
use clipboard_rs::{Clipboard as ClipboardRsTrait, ClipboardContext};
use image::{ImageFormat, RgbaImage};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Cursor;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_autostart::MacosLauncher;
use uuid::Uuid;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};
// main.rs 头部引入
use windows::Win32::System::Com::{CoInitialize, CoUninitialize, COINIT_APARTMENTTHREADED};
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
/// 将文件写入剪贴板（去除时间戳前缀）
#[tauri::command]
async fn write_file_to_clipboard(
    _app_handle: tauri::AppHandle,
    file_path: String,
    state: State<'_, ClipboardSourceState>,
) -> Result<(), String> {
    *state.is_frontend_copy.lock().unwrap() = true;

    // 直接复用修复后的处理逻辑，它现在支持文件夹且没有权限问题
    let final_path = process_file_for_clipboard(&file_path)?;

    // 写入剪贴板 (复用列表逻辑，只不过列表里只有一个)
    copy_files_list_to_clipboard(vec![final_path])
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    // 如果目标文件夹不存在，创建它
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    // 遍历源文件夹
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            // 如果是子文件夹，递归调用
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            // 如果是文件，直接复制
            fs::copy(&entry.path(), &dest_path)?;
        }
    }
    Ok(())
}
/// 更新剪贴板监控的文件目录（需要修改 app_setup.rs）
fn update_clipboard_monitor_path(app_handle: &tauri::AppHandle, data_root: &Path) {
    // 这里需要修改 app_setup.rs 中的 start_clipboard_monitor 函数
    // 使其能够接收和使用 data_root 路径，而不是硬编码的 app_dir
    println!("📁 剪贴板监控使用目录: {}", data_root.to_string_lossy());
}
// --- 辅助函数：处理单个文件（去除时间戳，复制到临时目录，返回绝对路径） ---
fn process_file_for_clipboard(file_path: &str) -> Result<PathBuf, String> {
    let path = Path::new(file_path);

    // 1. 检查是否存在
    if !path.exists() {
        return Err(format!("路径不存在: {}", file_path));
    }

    // 2. 解析原始文件名
    let file_name_os = path.file_name().ok_or("无法获取名称")?;
    let file_name_str = file_name_os.to_string_lossy();

    // 解析时间戳逻辑
    let clean_file_name = if let Some((prefix, name)) = file_name_str.split_once('-') {
        if prefix.len() == 13 && prefix.chars().all(char::is_numeric) {
            name.to_string()
        } else {
            file_name_str.to_string()
        }
    } else {
        file_name_str.to_string()
    };

    // 3. 【关键修改】创建唯一的父级临时目录
    // 结构变为: %TEMP% / {UUID} / {CleanFileName}
    let temp_root = env::temp_dir();
    let unique_sub_dir = temp_root.join(Uuid::new_v4().to_string());

    // 创建这个唯一的文件夹
    if let Err(e) = fs::create_dir_all(&unique_sub_dir) {
        return Err(format!("无法创建临时容器目录: {}", e));
    }

    // 真正的目标路径
    let temp_target_path = unique_sub_dir.join(&clean_file_name);

    // 4. 执行复制
    if path.is_dir() {
        // 复制文件夹
        if let Err(e) = copy_dir_all(path, &temp_target_path) {
            return Err(format!("复制文件夹失败: {}", e));
        }
    } else {
        // 复制文件
        if let Err(e) = fs::copy(path, &temp_target_path) {
            return Err(format!("复制文件失败: {}", e));
        }
    }

    // 5. 获取绝对路径并处理 Windows 前缀
    let absolute_path =
        fs::canonicalize(&temp_target_path).map_err(|e| format!("无法获取绝对路径: {}", e))?;

    #[cfg(target_os = "windows")]
    let final_path = {
        let mut s = absolute_path.to_string_lossy().to_string();
        const VERBATIM_PREFIX: &str = r"\\?\";
        if s.starts_with(VERBATIM_PREFIX) {
            s = s[VERBATIM_PREFIX.len()..].to_string();
        }
        PathBuf::from(s)
    };

    #[cfg(not(target_os = "windows"))]
    let final_path = absolute_path;

    Ok(final_path)
}

// --- 核心 helper：将路径列表写入剪贴板 ---
fn copy_files_list_to_clipboard(paths: Vec<PathBuf>) -> Result<(), String> {
    let ctx = ClipboardContext::new().map_err(|e| e.to_string())?;

    // 将 PathBuf 转换为 String 列表
    let paths_str: Vec<String> = paths
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    ctx.set_files(paths_str).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn write_files_to_clipboard(
    _app_handle: tauri::AppHandle,
    file_paths: Vec<String>,
    state: State<'_, ClipboardSourceState>,
) -> Result<(), String> {
    *state.is_frontend_copy.lock().unwrap() = true;

    if file_paths.is_empty() {
        return Err("未选择任何内容".to_string());
    }

    let mut final_paths: Vec<PathBuf> = Vec::new();

    for path_str in file_paths {
        // 这里调用修改后的 process_file_for_clipboard
        match process_file_for_clipboard(&path_str) {
            Ok(clean_path) => final_paths.push(clean_path),
            Err(e) => {
                println!("处理失败 [{}]: {}", path_str, e);
            }
        }
    }

    if final_paths.is_empty() {
        return Err("所有内容处理失败".to_string());
    }

    // 写入剪贴板 (复用之前的函数)
    copy_files_list_to_clipboard(final_paths)?;

    Ok(())
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
        let icon_base64 = tauri::async_runtime::spawn_blocking(move || extract_icon_base64(&path))
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
        // 1. 初始化 COM
        let com_init = CoInitialize(None);
        let _com_guard = ScopeGuard((), |_| {
            if com_init.is_ok() {
               CoUninitialize();
            }
        });

        // 2. 路径规范化：强制将所有正斜杠 '/' 替换为反斜杠 '\'
        // Windows API 对混合斜杠非常敏感
        let normalized_path = path.replace("/", "\\");

        // 3. 处理 UNC 前缀 (\\?\)
        // 如果规范化后的路径以 \\?\ 开头，则去掉它，因为 SHGetFileInfoW 有时对这个前缀处理不好
        let clean_path = if normalized_path.starts_with(r"\\?\") {
            &normalized_path[4..]
        } else {
            &normalized_path
        };

        // 调试日志（可选，确认路径变正常了）
        // println!("🔧 提取图标使用的路径: {}", clean_path);

        let wide_path: Vec<u16> = OsStr::new(clean_path)
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
            return Err(format!("SHGetFileInfoW 失败或未找到图标，路径: {}", clean_path));
        }

        let hicon = shfi.hIcon;
        let _icon_guard = ScopeGuard(hicon, |h| {
            let _ = DestroyIcon(h);
        });

        hicon_to_png_base64(hicon)
    }
}

#[cfg(target_os = "windows")]
unsafe fn hicon_to_png_base64(hicon: HICON) -> Result<String, String> {
    let mut icon_info = ICONINFO::default();
    GetIconInfo(hicon, &mut icon_info).map_err(|e| format!("GetIconInfo 失败: {}", e))?;

    let _color_bmp_guard = ScopeGuard(icon_info.hbmColor, |h| {
        let _ = DeleteObject(h);
    });
    let _mask_bmp_guard = ScopeGuard(icon_info.hbmMask, |h| {
        let _ = DeleteObject(h);
    });

    let hdc_screen = GetDC(HWND(std::ptr::null_mut()));
    let _dc_guard = ScopeGuard(hdc_screen, |h| {
        let _ = ReleaseDC(HWND(std::ptr::null_mut()), h);
    });

    let mut bmp: BITMAP = std::mem::zeroed();

    // GetObjectW 参数转换
    if GetObjectW(
        windows::Win32::Graphics::Gdi::HGDIOBJ(icon_info.hbmColor.0),
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bmp as *mut _ as *mut _),
    ) == 0
    {
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
    ) == 0
    {
        return Err("GetDIBits 失败".to_string());
    }

    // BGRA -> RGBA 转换
    for chunk in pixels.chunks_mut(4) {
        let b = chunk[0];
        let r = chunk[2];
        chunk[0] = r;
        chunk[2] = b;
    }

    let img_buffer =
        RgbaImage::from_raw(width as u32, height as u32, pixels).ok_or("无法构建图像缓冲区")?;

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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]), // 可以传递启动参数，这里为空
        ))
        .manage(AppShortcutManager::new())
        .manage(ClipboardSourceState {
            is_frontend_copy: Mutex::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            test_function,
            write_to_clipboard,
            write_file_to_clipboard,
            copy_file_to_clipboard,
            update_shortcut,
            get_current_shortcut,
            get_all_shortcuts,
            get_file_icon,
            write_files_to_clipboard,
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
            db::search_data,
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
            config::set_config_item,
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
            
            // 读取配置中的 storage_path
            if let Some(lock) = config::CONFIG.get() {
                let cfg = lock.read().unwrap();
                if let Some(ref path_str) = cfg.storage_path {
                    let custom_path = PathBuf::from(path_str);
                    if !path_str.trim().is_empty() {
                        println!("✅ 检测到配置的存储路径: {}", path_str);
                        
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
                            println!("📄 检测到新路径下的配置文件，切换到: {}", new_config_path.display());
                            config::set_config_path(new_config_path.clone());
                            
                            // 重新加载配置
                            let reload_result = config::init_config();
                            println!("重新加载配置结果: {}", reload_result);
                        } else {
                            println!("ℹ️ 新路径下没有配置文件，将使用默认配置路径");
                            // 如果新路径没有配置文件，但存储路径已设置，我们创建一个
                            println!("📝 在新路径创建配置文件");
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

            // 7. 打印最终使用的配置路径
            let current_config_path = config::get_config_path();
            println!("📄 最终配置文件路径: {}", current_config_path.display());
            
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
