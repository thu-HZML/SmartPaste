use crate::app_setup::ClipboardSourceState;
use arboard::Clipboard;
use base64::{engine::general_purpose, Engine as _};
use clipboard_rs::{Clipboard as ClipboardRsTrait, ClipboardContext};
use image::{ImageFormat, RgbaImage};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Cursor;
use std::io::{Read, Seek, Write};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};
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
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, GetIconInfo, GetSystemMetrics, HICON, ICONINFO, SM_CXSCREEN, SM_CYSCREEN,
};
use zip::write::FileOptions;
// main.rs 头部引入
use rdev::{listen, Button, EventType, Key};
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::OnceLock;
use std::thread;
use windows::Win32::System::Com::{CoInitialize, CoUninitialize};
use serde::Serialize; 
use serde_json::json;
use walkdir::WalkDir;

#[tauri::command]
pub fn test_function() -> String {
    "这是来自 Rust 的测试信息".to_string()
}
/// 辅助函数：递归压缩目录
fn zip_dir<T>(
    it: &mut zip::ZipWriter<T>,
    src_dir: &Path,
    prefix: &str,
    options: FileOptions,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    if !src_dir.exists() {
        return Ok(());
    }

    // 遍历目录
    for entry in std::fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();

        // 获取文件名
        let name = path.file_name().unwrap().to_string_lossy();

        // 组合 ZIP 中的路径 (例如: files/image.png)
        // 注意：ZIP 规范要求使用正斜杠 /，即使在 Windows 上
        let zip_entry_name = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", prefix, name)
        };

        if path.is_dir() {
            // 递归处理子文件夹
            // 在 ZIP 中显式添加目录条目是可选的，但为了结构清晰通常建议加上
            it.add_directory(&zip_entry_name, options)?;
            zip_dir(it, &path, &zip_entry_name, options)?;
        } else {
            // 这是一个文件，添加到 ZIP
            it.start_file(&zip_entry_name, options)?;
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            it.write_all(&buffer)?;
        }
    }
    Ok(())
}

/// 导出数据为 ZIP。作为 Tauri Command 暴露给前端。
#[tauri::command]
pub fn export_to_zip() -> Result<String, String> {
    // 1. 获取当前存储根目录
    let root_path = crate::config::get_current_storage_path();

    // 2. 生成 ZIP 文件名 (backup_时间戳.zip)
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let zip_filename = format!("backup_{}.zip", timestamp);
    let zip_path = root_path.join(&zip_filename);

    // 3. 创建 ZIP 文件
    let file = File::create(&zip_path).map_err(|e| format!("无法创建 ZIP 文件: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);

    // 设置压缩选项 (Deflated 压缩率较高)
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // 4. 定义需要打包的目标列表
    let targets = vec![
        ("config.json", false), // (文件名, 是否是文件夹)
        ("smartpaste.db", false),
        ("files", true),
    ];

    for (target_name, is_dir) in targets {
        let target_path = root_path.join(target_name);

        if target_path.exists() {
            if is_dir {
                // 压缩文件夹
                zip.add_directory(target_name, options)
                    .map_err(|e| e.to_string())?;
                zip_dir(&mut zip, &target_path, target_name, options)
                    .map_err(|e| format!("压缩目录 {} 失败: {}", target_name, e))?;
            } else {
                // 压缩单个文件
                zip.start_file(target_name, options)
                    .map_err(|e| e.to_string())?;
                // 读取文件内容
                // 注意：如果数据库正在被频繁写入，这里可能会有读取冲突，但一般备份操作能接受
                let mut f = File::open(&target_path).map_err(|e| e.to_string())?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
                zip.write_all(&buffer).map_err(|e| e.to_string())?;
            }
        }
    }

    // 5. 完成写入
    zip.finish().map_err(|e| format!("ZIP 写入失败: {}", e))?;

    println!("✅ 数据已备份至: {}", zip_path.display());

    // 返回生成的 ZIP 文件名或完整路径
    Ok(zip_path.to_string_lossy().to_string())
}
/// 从当前目录下的最新备份 ZIP 恢复数据
/// 要求 ZIP 中必须包含 config.json, smartpaste.db 和 files/ 文件夹
#[tauri::command]
pub fn import_data_from_zip(app: tauri::AppHandle) -> Result<String, String> {
    // 1. 获取当前存储路径
    let root_path = crate::config::get_current_storage_path();
    println!("🔍 开始在 {} 查找备份文件...", root_path.display());

    // 2. 扫描目录下所有以 backup_ 开头 .zip 结尾的文件，并找到最新的一个
    let mut zip_files: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(&root_path).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("backup_") && name.ends_with(".zip") {
                    zip_files.push(path);
                }
            }
        }
    }

    // 如果没有找到备份
    if zip_files.is_empty() {
        return Err("未找到任何以 backup_ 开头的 zip 备份文件".to_string());
    }

    // 按文件名排序（因为文件名包含时间戳，排序后最后一个就是最新的）
    zip_files.sort();
    let latest_zip_path = zip_files.last().unwrap();
    println!("📦 找到最新备份: {}", latest_zip_path.display());

    // 3. 预检查 ZIP 内容
    let file = fs::File::open(latest_zip_path).map_err(|e| format!("无法打开 ZIP: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("ZIP 格式错误: {}", e))?;

    let mut has_config = false;
    let mut has_db = false;
    let mut has_files_dir = false;

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name();

        // 检查关键文件是否存在
        if name == "config.json" {
            has_config = true;
        } else if name == "smartpaste.db" {
            has_db = true;
        }
        // 只要有任何文件或目录以 files/ 开头，就认为包含 files 文件夹
        else if name.starts_with("files/") || name.starts_with("files\\") {
            has_files_dir = true;
        }
    }

    if !has_config || !has_db || !has_files_dir {
        return Err(format!(
            "备份文件不完整! 检查结果: config.json={}, db={}, files={}",
            has_config, has_db, has_files_dir
        ));
    }

    println!("✅ 备份文件校验通过，准备恢复...");

    // 4. 清理旧数据 (Config, DB, Files)
    // 注意：Windows 下如果文件被占用这里会报错，建议前端做个 loading 状态

    let target_config = root_path.join("config.json");
    let target_db = root_path.join("smartpaste.db");
    let target_files_dir = root_path.join("files");

    // 尝试删除旧配置
    if target_config.exists() {
        fs::remove_file(&target_config).map_err(|e| format!("无法删除旧 config.json: {}", e))?;
    }

    // 尝试删除旧数据库
    // ⚠️ 警告：如果数据库连接未释放，这里会失败。
    // db.rs 是按需打开连接的，理论上只要没有正在进行的查询就可以删除。
    if target_db.exists() {
        fs::remove_file(&target_db)
            .map_err(|e| format!("无法删除旧 smartpaste.db (可能正在使用中): {}", e))?;
    }

    // 尝试删除旧 files 目录
    if target_files_dir.exists() {
        fs::remove_dir_all(&target_files_dir)
            .map_err(|e| format!("无法删除旧 files 目录: {}", e))?;
    }

    // 5. 解压文件
    println!("🔄 正在解压...");
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;

        // 获取输出路径
        // ⚠️ 安全检查：防止 Zip Slip 漏洞 (文件名包含 ../ 试图跳出目录)
        let outpath = match file.enclosed_name() {
            Some(path) => root_path.join(path),
            None => continue, // 跳过非法路径
        };

        // 只解压我们需要的那三个目标，防止 ZIP 里有垃圾文件
        let file_name_str = file.name();
        if file_name_str != "config.json"
            && file_name_str != "smartpaste.db"
            && !file_name_str.starts_with("files/")
            && !file_name_str.starts_with("files\\")
        {
            continue;
        }

        if (*file.name()).ends_with('/') || (*file.name()).ends_with('\\') {
            fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
            io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }
    println!("🔧 正在修正 config.json 中的存储路径...");
    let config_file_path = root_path.join("config.json");

    if config_file_path.exists() {
        // 1. 读取解压出来的配置文件
        let config_content =
            fs::read_to_string(&config_file_path).map_err(|e| format!("读取配置失败: {}", e))?;

        // 2. 解析 JSON
        let mut json_val: serde_json::Value =
            serde_json::from_str(&config_content).map_err(|e| format!("解析配置失败: {}", e))?;

        // 3. 获取当前的物理路径字符串
        let current_path_str = root_path.to_string_lossy().to_string();

        // 4. 规范化路径 (Windows下强制使用反斜杠，防止混合斜杠Bug复发)
        #[cfg(target_os = "windows")]
        let final_path_str = current_path_str.replace("\\", "/");

        #[cfg(not(target_os = "windows"))]
        let final_path_str = current_path_str;

        println!("📍 将 storage_path 修正为: {}", final_path_str);

        // 5. 修改字段
        json_val["storage_path"] = serde_json::Value::String(final_path_str);

        // 6. 写回文件
        let new_content = serde_json::to_string_pretty(&json_val)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        fs::write(&config_file_path, new_content).map_err(|e| format!("写入配置失败: {}", e))?;

        println!("✅ storage_path 修正完成");
    } else {
        eprintln!("⚠️ 警告: 解压后未找到 config.json，跳过路径修正");
    }
    // 6. 恢复完成后，必须重新加载配置到内存
    println!("🔄 恢复完成，正在刷新配置...");
    let reload_msg = crate::config::reload_config();
    println!("配置刷新结果: {}", reload_msg);

    // 7. 发送事件通知前端刷新页面 (可选)
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("data-restored", "success");
    }

    Ok(format!(
        "恢复成功！已从 {} 还原数据。",
        latest_zip_path.file_name().unwrap().to_string_lossy()
    ))
}
#[tauri::command]
pub fn write_to_clipboard(
    text: String,
    _app_handle: tauri::AppHandle,
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
pub async fn write_file_to_clipboard(
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

/// 将文件的相对路径按配置设置转化为绝对路径
/// Param:
/// relative_path: &PathBuf - 相对路径
/// Returns:
/// PathBuf - 绝对路径
pub fn resolve_absolute_path(relative_path: &PathBuf) -> PathBuf {
    let storage_path = crate::config::get_current_storage_path();
    storage_path.join(relative_path)
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
pub async fn write_files_to_clipboard(
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
pub fn copy_file_to_clipboard(file_path: PathBuf) -> Result<(), String> {
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
pub fn copy_file_to_clipboard_windows(file_path: &str) -> Result<(), String> {
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
pub fn copy_file_to_clipboard_macos(file_path: &str) -> Result<(), String> {
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
pub fn copy_file_to_clipboard_linux(file_path: &str) -> Result<(), String> {
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
pub async fn get_file_icon(path: String) -> Result<String, String> {
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
pub fn extract_icon_base64(path: &str) -> Result<String, String> {
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
            return Err(format!(
                "SHGetFileInfoW 失败或未找到图标，路径: {}",
                clean_path
            ));
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

// 在静态变量区域添加以下内容
static IS_MONITORING: AtomicBool = AtomicBool::new(false);
static IS_MOUSE_BUTTON_MONITORING: AtomicBool = AtomicBool::new(false);
static IS_MOUSE_MOVE_MONITORING: AtomicBool = AtomicBool::new(false);
static MONITOR_THREAD_STARTED: AtomicBool = AtomicBool::new(false);

// 用于存储屏幕尺寸，方便坐标归一化
static SCREEN_WIDTH: AtomicU32 = AtomicU32::new(0);
static SCREEN_HEIGHT: AtomicU32 = AtomicU32::new(0);

// 控制开关：是否向前端发送数据
/// 开始监听：前端调用此方法后，Rust 开始向前端 emit 事件
#[tauri::command]
pub fn start_key_listener(app: AppHandle) {
    println!("▶️ 开启键盘监听");
    IS_MONITORING.store(true, Ordering::SeqCst);

    // 如果线程还没启动，则启动它
    if !MONITOR_THREAD_STARTED.load(Ordering::SeqCst) {
        MONITOR_THREAD_STARTED.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            // rdev::listen 是阻塞的，会一直运行
            if let Err(error) = listen(move |event| {
                // 处理键盘事件
                if IS_MONITORING.load(Ordering::SeqCst) {
                    let (key_name, event_type) = match event.event_type {
                        EventType::KeyPress(key) => (format!("{:?}", key), "down"),
                        EventType::KeyRelease(key) => (format!("{:?}", key), "up"),
                        _ => ("".to_string(), ""), // 返回空字符串
                    };

                    if !key_name.is_empty() {
                        let payload = json!({
                            "key": key_name,
                            "type": event_type
                        });

                        if let Err(e) = app.emit("key-monitor-event", payload) {
                            eprintln!("❌ 发送键盘事件失败: {}", e);
                        }
                    }
                }

                // 处理鼠标事件
                handle_mouse_event(&app, &event);
            }) {
                eprintln!("❌ 监听线程错误: {:?}", error);
            }
        });
        println!("🚀 启动了全局监听线程");
    }
}

/// 停止键盘监听：前端调用此方法后，Rust 暂停发送键盘事件
#[tauri::command]
pub fn stop_key_listener() {
    println!("⏸️ 暂停键盘监听");
    IS_MONITORING.store(false, Ordering::SeqCst);
}

/// 开始监听鼠标按下/松开事件
#[tauri::command]
pub fn start_mouse_button_listener(app: AppHandle) {
    println!("▶️ 开启鼠标按钮监听");
    IS_MOUSE_BUTTON_MONITORING.store(true, Ordering::SeqCst);

    // 确保监听线程已启动
    if !MONITOR_THREAD_STARTED.load(Ordering::SeqCst) {
        // 如果监听线程没有启动，就启动它
        start_key_listener(app.clone());
    }
}

/// 开始监听鼠标移动事件（实时位置）
#[tauri::command]
pub fn start_mouse_move_listener(app: AppHandle) {
    println!("▶️ 开启鼠标移动监听");
    IS_MOUSE_MOVE_MONITORING.store(true, Ordering::SeqCst);

    // 获取屏幕尺寸用于坐标归一化
    update_screen_size();

    // 确保监听线程已启动
    if !MONITOR_THREAD_STARTED.load(Ordering::SeqCst) {
        // 如果监听线程没有启动，就启动它
        start_key_listener(app.clone());
    }
}

/// 停止所有鼠标监听
#[tauri::command]
pub fn stop_mouse_listener() {
    println!("⏸️ 停止所有鼠标监听");
    IS_MOUSE_BUTTON_MONITORING.store(false, Ordering::SeqCst);
    IS_MOUSE_MOVE_MONITORING.store(false, Ordering::SeqCst);
}

/// 获取屏幕尺寸（用于坐标归一化）
#[cfg(target_os = "windows")]
fn update_screen_size() {
    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN) as u32;
        let height = GetSystemMetrics(SM_CYSCREEN) as u32;
        SCREEN_WIDTH.store(width, Ordering::SeqCst);
        SCREEN_HEIGHT.store(height, Ordering::SeqCst);
        println!("📐 屏幕尺寸: {}x{}", width, height);
    }
}

#[cfg(target_os = "macos")]
fn update_screen_size() {
    // macOS 实现
    use cocoa::appkit::NSScreen;
    use cocoa::base::{id, nil};

    unsafe {
        let main_screen = NSScreen::mainScreen(nil);
        let frame = NSScreen::frame(main_screen);

        let width = frame.size.width as u32;
        let height = frame.size.height as u32;

        SCREEN_WIDTH.store(width, Ordering::SeqCst);
        SCREEN_HEIGHT.store(height, Ordering::SeqCst);
        println!("📐 屏幕尺寸: {}x{}", width, height);
    }
}

#[cfg(target_os = "linux")]
fn update_screen_size() {
    // Linux 实现（使用 xrandr）
    use std::process::Command;

    match Command::new("xrandr").arg("--current").output() {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // 解析 xrandr 输出获取主屏幕尺寸
            for line in output_str.lines() {
                if line.contains(" connected") {
                    // 寻找分辨率部分
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    for part in parts {
                        if part.contains('x') {
                            if let Some((width_str, rest)) = part.split_once('x') {
                                if let Some((height_str, _)) = rest.split_once('+') {
                                    if let (Ok(width), Ok(height)) =
                                        (width_str.parse::<u32>(), height_str.parse::<u32>())
                                    {
                                        SCREEN_WIDTH.store(width, Ordering::SeqCst);
                                        SCREEN_HEIGHT.store(height, Ordering::SeqCst);
                                        println!("📐 屏幕尺寸: {}x{}", width, height);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 如果没有找到，使用默认值
            SCREEN_WIDTH.store(1920, Ordering::SeqCst);
            SCREEN_HEIGHT.store(1080, Ordering::SeqCst);
            println!("⚠️ 无法获取屏幕尺寸，使用默认值 1920x1080");
        }
        Err(e) => {
            eprintln!("❌ 获取屏幕尺寸失败: {}", e);
            // 如果命令失败，使用默认值
            SCREEN_WIDTH.store(1920, Ordering::SeqCst);
            SCREEN_HEIGHT.store(1080, Ordering::SeqCst);
            println!("⚠️ 使用默认屏幕尺寸 1920x1080");
        }
    }
}

/// 归一化鼠标坐标（转换为0-1范围，左下角为原点）
fn normalize_mouse_position(x: f64, y: f64) -> (f64, f64) {
    let width = SCREEN_WIDTH.load(Ordering::SeqCst) as f64;
    let height = SCREEN_HEIGHT.load(Ordering::SeqCst) as f64;

    if width == 0.0 || height == 0.0 {
        // 如果没获取到屏幕尺寸，返回0.5, 0.5
        return (0.5, 0.5);
    }

    // 归一化到 [0, 1]
    let normalized_x = x / width;
    // 翻转Y轴，使左下角为原点
    let normalized_y = 1.0 - (y / height);

    // 确保在[0, 1]范围内
    let clamped_x = normalized_x.clamp(0.0, 1.0);
    let clamped_y = normalized_y.clamp(0.0, 1.0);

    (clamped_x, clamped_y)
}

/// 处理鼠标事件
fn handle_mouse_event(app: &AppHandle, event: &rdev::Event) {
    match &event.event_type {
        // 鼠标按钮按下
        EventType::ButtonPress(button) => {
            if IS_MOUSE_BUTTON_MONITORING.load(Ordering::SeqCst) {
                let button_str = match button {
                    Button::Left => "left",
                    Button::Right => "right",
                    Button::Middle => "middle",
                    Button::Unknown(code) => {
                        // 处理未知按钮（通常是鼠标侧键）
                        if *code == 4 {
                            "back"
                        } else if *code == 5 {
                            "forward"
                        } else if *code == 6 {
                            "task"
                        } else {
                            "unknown"
                        }
                    }
                    _ => "other",
                };

                let payload = json!({
                    "button": button_str,
                    "type": "down"
                });

                if let Err(e) = app.emit("mouse-button-event", payload) {
                    eprintln!("❌ 发送鼠标按钮事件失败: {}", e);
                }
            }
        }

        // 鼠标按钮松开
        EventType::ButtonRelease(button) => {
            if IS_MOUSE_BUTTON_MONITORING.load(Ordering::SeqCst) {
                let button_str = match button {
                    Button::Left => "left",
                    Button::Right => "right",
                    Button::Middle => "middle",
                    Button::Unknown(code) => {
                        if *code == 4 {
                            "back"
                        } else if *code == 5 {
                            "forward"
                        } else if *code == 6 {
                            "task"
                        } else {
                            "unknown"
                        }
                    }
                    _ => "other",
                };

                let payload = json!({
                    "button": button_str,
                    "type": "up"
                });

                if let Err(e) = app.emit("mouse-button-event", payload) {
                    eprintln!("❌ 发送鼠标按钮事件失败: {}", e);
                }
            }
        }

        // 鼠标移动
        EventType::MouseMove { x, y } => {
            if IS_MOUSE_MOVE_MONITORING.load(Ordering::SeqCst) {
                let (normalized_x, normalized_y) = normalize_mouse_position(*x, *y);

                let payload = json!({
                    "x": normalized_x,
                    "y": normalized_y,
                    "raw_x": x,
                    "raw_y": y,
                    "type": "move"
                });

                if let Err(e) = app.emit("mouse-move-event", payload) {
                    eprintln!("❌ 发送鼠标移动事件失败: {}", e);
                }
            }
        }

        // 忽略滚轮事件
        EventType::Wheel { .. } => {}

        // 忽略其他事件
        _ => {}
    }
}
#[tauri::command]
pub fn get_utils_dir_path(_app: AppHandle) -> Result<String, String> {
    // 方法1: 使用当前模块文件的路径（编译时确定）
    #[cfg(debug_assertions)]
    {
        // 调试模式下，尝试使用源码路径
        let current_file_path = Path::new(file!());
        if let Some(dir_path) = current_file_path.parent() {
            if let Ok(absolute_path) = dir_path.canonicalize() {
                return Ok(absolute_path.to_string_lossy().replace("\\", "/"));
            }
        }
    }

    // 方法2: 使用当前可执行文件所在目录（适用于所有环境）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir_path) = exe_path.parent() {
            // 获取绝对路径并标准化分隔符
            let canonical_path = dir_path.canonicalize().unwrap_or(dir_path.to_path_buf());
            return Ok(canonical_path.to_string_lossy().replace("\\", "/"));
        }
    }

    // 方法3: 使用当前工作目录作为备选
    if let Ok(current_dir) = std::env::current_dir() {
        return Ok(current_dir.to_string_lossy().replace("\\", "/"));
    }

    Err("无法获取当前目录路径".to_string())
}

/**
 * 【新增命令】读取本地文件并返回 Base64 编码的字符串。
 * 用于将前端选择的本地图片文件转换为可上传的格式。
 * @param file_path: String - 文件的绝对路径。
 * @returns Base64 编码的字符串。
 */
#[tauri::command]
pub async fn read_file_base64(file_path: String) -> Result<String, String> {
    use std::fs;

    // 文件 I/O 是阻塞操作，因此使用 spawn_blocking 避免阻塞 Tauri 运行时
    tauri::async_runtime::spawn_blocking(move || {
        let path = Path::new(&file_path);

        if !path.exists() {
            return Err(format!("文件路径不存在: {}", file_path));
        }

        let mut file = fs::File::open(path).map_err(|e| format!("无法打开文件: {}", e))?;
        let mut buffer = Vec::new();

        // 读取文件内容到缓冲区
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("读取文件内容失败: {}", e))?;

        // Base64 编码
        let base64_content = general_purpose::STANDARD.encode(buffer);

        Ok(base64_content)
    })
    .await
    .map_err(|e| format!("异步任务执行失败: {}", e))?
}

/**
 * 【新增命令】读取本地 config.json 文件内容，返回 JSON 字符串。
 * 对应前端调用: readLocalConfigContent
 */
#[tauri::command]
pub async fn read_local_config_content() -> Result<String, String> {
    use std::fs;
    tauri::async_runtime::spawn_blocking(move || {
        let config_path = crate::config::get_config_path(); // 假设 config 模块提供此函数
        fs::read_to_string(config_path)
            .map_err(|e| format!("读取 config.json 失败: {}", e))
    })
    .await
    .map_err(|e| format!("异步任务执行失败: {}", e))?
}

/**
 * 将配置内容字符串写入本地配置文件。
 * 用于实现登录成功后从云端同步配置到本地。
 * @param content: String - 配置文件的内容 (JSON 字符串)。
 * 对应前端调用: writeLocalConfigContent
 */
#[tauri::command]
pub async fn write_local_config_file(content: String) -> Result<(), String> {
    use std::fs;

    // 文件 I/O 是阻塞操作
    tauri::async_runtime::spawn_blocking(move || {
        let config_path = crate::config::get_config_path(); // 假设 crate::config 模块提供了 get_config_path

        fs::write(&config_path, content).map_err(|e| format!("写入本地配置文件失败: {}", e))?;

        // 【关键】写入新配置后，需要重新加载配置到内存中，以便立即生效
        let reload_msg = crate::config::reload_config();
        println!("同步配置写入后，配置刷新结果: {}", reload_msg);

        Ok(())
    })
    .await
    .map_err(|e| format!("异步任务执行失败: {}", e))?
}

/**
 * 【新增命令】读取本地 smartpaste.db 文件内容，返回 Base64 字符串。
 * 对应前端调用: readDbFileBase64
 */
#[tauri::command]
pub async fn read_db_file_base64() -> Result<String, String> {
    use base64::{engine::general_purpose, Engine as _};
    use std::fs;
    use std::io::Read;

    tauri::async_runtime::spawn_blocking(move || {
        let root_path = crate::config::get_current_storage_path();
        let db_path = root_path.join("smartpaste.db");
        
        if !db_path.exists() {
            return Err("本地数据库文件不存在".to_string());
        }

        let mut file = fs::File::open(db_path).map_err(|e| format!("无法打开数据库文件: {}", e))?;
        let mut buffer = Vec::new();
        
        file.read_to_end(&mut buffer).map_err(|e| format!("读取数据库文件失败: {}", e))?;
        
        let base64_content = general_purpose::STANDARD.encode(buffer);
        
        Ok(base64_content)
    })
    .await
    .map_err(|e| format!("异步任务执行失败: {}", e))?
}

/**
 * 【新增命令】将 Base64 内容解码并替换本地 smartpaste.db 文件。
 * 对应前端调用: replaceLocalDbFile
 */
#[tauri::command]
pub async fn replace_local_db_file(base64_content: String) -> Result<(), String> {
    use base64::{engine::general_purpose, Engine as _};
    use std::fs;
    use std::io::Write;

    tauri::async_runtime::spawn_blocking(move || {
        let root_path = crate::config::get_current_storage_path();
        let db_path = root_path.join("smartpaste.db");
        
        // 1. Base64 解码
        let decoded_bytes = general_purpose::STANDARD.decode(base64_content)
            .map_err(|e| format!("Base64 解码失败: {}", e))?;
        
        // 2. 写入文件
        fs::write(&db_path, decoded_bytes)
            .map_err(|e| format!("写入数据库文件失败: {} (文件可能被占用)", e))?;

        println!("✅ 本地数据库文件已更新");
        Ok(())
    })
    .await
    .map_err(|e| format!("异步任务执行失败: {}", e))?
}

/**
 * 【新增命令】通知数据库模块重新加载连接。（占位实现）
 * 对应前端调用: refreshDatabaseConnection
 */
#[tauri::command]
pub fn refresh_database_connection() -> Result<(), String> {
    println!("⚠️ 尝试刷新数据库连接 (需要实现 crate::db::refresh_connection)");
    Ok(())
}

// -----------------------------------------------------
// 文件同步相关辅助结构体
// -----------------------------------------------------

/// 用于前端接收本地文件列表，包含相对路径和绝对路径
#[derive(Debug, Serialize)]
pub struct LocalFileInfo {
    relative_path: String,
    file_path: String,
}

/**
 * 【新增命令】获取本地剪贴板文件目录(files/)中的所有文件列表。
 * 返回一个包含相对路径和绝对路径的结构体列表。
 * 对应前端调用: getLocalFilesToUpload
 */
#[tauri::command]
pub async fn get_local_files_to_upload() -> Result<Vec<LocalFileInfo>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let root_path = crate::config::get_current_storage_path();
        let files_dir = root_path.join("files");
        let mut file_list: Vec<LocalFileInfo> = Vec::new();

        if !files_dir.exists() {
            return Ok(file_list); // 目录不存在，返回空列表
        }

        // 遍历 files 目录
        for entry in walkdir::WalkDir::new(&files_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            
            // 忽略目录本身
            if path.is_dir() {
                continue;
            }

            // 计算相对于 files_dir 的相对路径
            let relative_path_os = path.strip_prefix(&files_dir)
                .map_err(|e| format!("计算相对路径失败: {}", e))?;
            
            let relative_path = relative_path_os.to_string_lossy().to_string().replace("\\", "/");
            let absolute_path = path.to_string_lossy().to_string().replace("\\", "/");
            
            file_list.push(LocalFileInfo {
                relative_path,
                file_path: absolute_path,
            });
        }

        Ok(file_list)
    })
    .await
    .map_err(|e| format!("异步任务执行失败: {}", e))?
}

/**
 * 【新增命令】将 Base64 内容解码并保存到本地剪贴板文件目录。
 * @param relative_path: String - 相对于 files/ 目录的路径。
 * @param base64_content: String - 文件的 Base64 内容。
 * 对应前端调用: saveClipboardFile
 */
#[tauri::command]
pub async fn save_clipboard_file(relative_path: String, base64_content: String) -> Result<(), String> {
    use base64::{engine::general_purpose, Engine as _};
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    tauri::async_runtime::spawn_blocking(move || {
        let root_path = crate::config::get_current_storage_path();
        let files_dir = root_path.join("files");
        
        // 目标绝对路径: {ROOT}/files/{RELATIVE_PATH}
        let file_path = files_dir.join(&relative_path);
        
        // 安全检查：防止相对路径包含 '..' 试图跳出目录 (Zip Slip 风险)
        if file_path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err("相对路径包含非法字符 '..'".to_string());
        }

        // 1. 确保父目录存在
        if let Some(parent_dir) = file_path.parent() {
            fs::create_dir_all(parent_dir)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }

        // 2. Base64 解码
        let decoded_bytes = general_purpose::STANDARD.decode(base64_content)
            .map_err(|e| format!("Base64 解码失败: {}", e))?;
        
        // 3. 写入文件
        fs::write(&file_path, decoded_bytes)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        
        println!("💾 文件保存成功: {}", relative_path);
        Ok(())
    })
    .await
    .map_err(|e| format!("异步任务执行失败: {}", e))?
}



/// 前端文件结构体，包含文件名、Base64 数据和 MIME 类型。
/// 用于将本地文件信息传递给前端。
#[derive(serde::Serialize)]
pub struct FrontendFile {
    /// 文件名
    name: String,
    /// Base64 编码的数据
    data: String,
    /// MIME 类型
    mime: String,
}

/// 读取本地文件并返回给前端（Base64 编码），包括文件名和 MIME 类型。
/// 作为 Tauri command 暴露给前端调用。
/// # Param
/// file_path: String - 文件的绝对路径。
/// # Returns
/// Result<FrontendFile, String> - 成功返回包含文件信息的结构体，失败返回错误信息。
/// # Example
/// 前端使用示例：
/// ```javascript
/// import { invoke } from '@tauri-apps/api/tauri';
/// ...
/// async function getFileFromPath(filePath) {
///     const { name, data, mime } = await invoke('read_file_to_frontend', { filePath });
///     const res = await fetch(`data:${mime};base64,${data}`);
///     const blob = await res.blob();
///     return new File([blob], name, { type: mime });
/// }
/// ```
#[tauri::command]
pub async fn read_file_to_frontend(file_path: String) -> Result<FrontendFile, String> {
    let path_buf = std::path::PathBuf::from(&file_path);
    let path = path_buf.as_path();

    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    let name = path
        .file_name()
        .ok_or("无法获取文件名")?
        .to_string_lossy()
        .to_string();

    // 简单的 MIME 推断
    let mime = match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("txt") => "text/plain",
        Some("pdf") => "application/pdf",
        Some("json") => "application/json",
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("zip") => "application/zip",
        _ => "application/octet-stream",
    }
    .to_string();

    // 异步读取文件
    let path_clone = path_buf.clone();
    let content = tauri::async_runtime::spawn_blocking(move || {
        let mut file = fs::File::open(path_clone).map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        use std::io::Read;
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
        Ok::<Vec<u8>, String>(buffer)
    })
    .await
    .map_err(|e| e.to_string())??;

    let base64_data = general_purpose::STANDARD.encode(content);

    Ok(FrontendFile {
        name,
        data: base64_data,
        mime,
    })
}
