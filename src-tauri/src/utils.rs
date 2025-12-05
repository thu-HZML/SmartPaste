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
use tauri::{Emitter, Manager, State,AppHandle};
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
use zip::write::FileOptions;
// main.rs 头部引入
use windows::Win32::System::Com::{CoInitialize, CoUninitialize};
use rdev::{listen, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::thread;
use serde_json::json;
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
// 控制开关：是否向前端发送数据
static IS_MONITORING: AtomicBool = AtomicBool::new(false);
// 保证线程只启动一次
static MONITOR_THREAD_STARTED: AtomicBool = AtomicBool::new(false);

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
                // 1. 检查开关，如果前端没让开始，就什么都不做
                if !IS_MONITORING.load(Ordering::SeqCst) {
                    return;
                }

                // 2. 匹配事件类型
                let (key_name, event_type) = match event.event_type {
                    EventType::KeyPress(key) => (format!("{:?}", key), "down"),
                    EventType::KeyRelease(key) => (format!("{:?}", key), "up"),
                    _ => return, // 忽略鼠标等其他事件
                };

                // 3. 动态构建 JSON 数据 (不使用结构体)
                let payload = json!({
                    "key": key_name,   // 例如 "KeyA", "ControlLeft"
                    "type": event_type // "down" 或 "up"
                });

                // 4. 发送事件给前端
                // 前端需要监听 'key-monitor-event'
                if let Err(e) = app.emit("key-monitor-event", payload) {
                    eprintln!("❌ 发送事件失败: {}", e);
                }
            }) {
                eprintln!("❌ 键盘监听线程错误: {:?}", error);
            }
        });
    }
}

/// 停止监听：前端调用此方法后，Rust 暂停发送事件
#[tauri::command]
pub fn stop_key_listener() {
    println!("⏸️ 暂停键盘监听");
    IS_MONITORING.store(false, Ordering::SeqCst);
}