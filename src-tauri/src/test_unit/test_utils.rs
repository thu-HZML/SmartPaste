use base64::{engine::general_purpose, Engine as _};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

// 引入需要测试的模块
use crate::utils;
// 引入 config 模块
use crate::config::{init_config, ConfigKey};

/// 基于文件的全局锁
struct GlobalFileLock {
    path: PathBuf,
}

impl GlobalFileLock {
    fn acquire() -> Self {
        let mut lock_path = std::env::temp_dir();
        lock_path.push("smartpaste_test_global.lock");

        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < 30 {
            // create_new(true) 保证原子性
            if fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
                .is_ok()
            {
                return Self { path: lock_path };
            }
            thread::sleep(Duration::from_millis(100));
        }
        panic!("无法获取测试全局锁: 超时 (可能有其他测试正在运行或锁文件未清理)");
    }
}

impl Drop for GlobalFileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// 辅助函数：手动修改 config.json
fn manual_update_config_file(storage_path: &str) {
    // 使用全局配置路径，确保与 init_config/reload_config 一致
    let config_path = crate::config::get_config_path();

    // 如果文件不存在，先初始化一下
    if !config_path.exists() {
        let _ = init_config();
    }

    // 读取
    let content = fs::read_to_string(&config_path).unwrap_or_else(|_| "{}".to_string());
    let mut json_val: Value = serde_json::from_str(&content).unwrap_or(json!({}));

    // 修改 storage_path
    json_val["storage_path"] = json!(storage_path);

    // 写回
    let new_content = serde_json::to_string_pretty(&json_val).unwrap();
    fs::write(&config_path, new_content).expect("无法写入 config.json");
}

/// 辅助函数：设置测试环境
fn setup_test_env() -> (GlobalFileLock, PathBuf) {
    let lock = GlobalFileLock::acquire();

    // 1. 初始化 (确保 lazy_static / OnceLock 触发)
    let _ = init_config();

    // 2. 准备临时目录
    let temp_dir = std::env::temp_dir().join(format!("smartpaste_test_{}", uuid::Uuid::new_v4()));
    if temp_dir.exists() {
        let _ = fs::remove_dir_all(&temp_dir);
    }
    fs::create_dir_all(&temp_dir).expect("无法创建测试临时目录");

    // 3. 规范化路径 (Windows 下转为正斜杠，避免 JSON 转义地狱)
    let temp_dir_str = temp_dir.to_string_lossy().to_string().replace("\\", "/");

    println!("🛠️ [Test Setup] 目标临时路径: {}", temp_dir_str);

    // 4. 尝试通过 API 更新 (如果支持)
    // 注意：即使这里失败，后面的手动更新也会覆盖
    let _ = crate::config::update_simple_config_item(&ConfigKey::StoragePath, json!(temp_dir_str));

    // 5. 【关键】手动强制修改 config.json 文件
    // 因为 update_simple_config_item 可能不支持 StoragePath 或者逻辑有误
    manual_update_config_file(&temp_dir_str);

    // 6. 强制重载配置到内存
    let _ = crate::config::reload_config();

    // 7. 验证同步状态
    let mut synced = false;
    for _ in 0..10 {
        let current = crate::config::get_current_storage_path();
        let current_str = current.to_string_lossy().to_string().replace("\\", "/");

        // 检查路径是否包含我们的临时目录名 (处理 potential C:/ vs c:/ mismatch)
        if current_str.contains(&temp_dir_str)
            || (cfg!(windows)
                && current_str
                    .to_lowercase()
                    .contains(&temp_dir_str.to_lowercase()))
        {
            synced = true;
            break;
        }

        // 如果还没同步，再次尝试重载
        let _ = crate::config::reload_config();
        thread::sleep(Duration::from_millis(50));
    }

    if !synced {
        let current = crate::config::get_current_storage_path();
        panic!("❌ [Test Setup] 配置同步失败！\n期望: {}\n实际: {:?}\n请检查 config.rs 是否正确处理了 config.json 的读取。", 
            temp_dir_str, current);
    }

    (lock, temp_dir)
}

fn teardown_test_env(path: PathBuf) {
    if path.exists() {
        // 多次尝试删除，应对 Windows 文件占用
        for _ in 0..5 {
            if fs::remove_dir_all(&path).is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

#[test]
fn test_basic_function() {
    assert_eq!(utils::test_function(), "这是来自 Rust 的测试信息");
}

#[test]
fn test_get_utils_dir_path() {
    if let Ok(path) = std::env::current_dir() {
        assert!(path.exists());
    }
}

#[tokio::test]
async fn test_read_file_base64() {
    let (_lock, temp_dir) = setup_test_env();
    let file_path = temp_dir.join("test_image.txt");

    fs::write(&file_path, "Hello World").unwrap();

    let result = utils::read_file_base64(file_path.to_string_lossy().to_string()).await;
    assert_eq!(result.unwrap(), "SGVsbG8gV29ybGQ=");

    // 测试不存在的文件
    let bad_path = temp_dir.join("none.txt");
    let bad_result = utils::read_file_base64(bad_path.to_string_lossy().to_string()).await;
    assert!(bad_result.is_err());

    teardown_test_env(temp_dir);
}

#[tokio::test]
async fn test_read_file_to_frontend() {
    let (_lock, temp_dir) = setup_test_env();

    let file_path = temp_dir.join("test.png");
    let content = vec![1, 2, 3, 4];
    fs::write(&file_path, &content).unwrap();

    let result = utils::read_file_to_frontend(file_path.to_string_lossy().to_string()).await;
    let file = result.unwrap();

    assert_eq!(file.name, "test.png");
    assert_eq!(file.mime, "image/png");
    assert_eq!(file.data, general_purpose::STANDARD.encode(&content));

    teardown_test_env(temp_dir);
}

#[tokio::test]
async fn test_save_clipboard_file_and_get_list() {
    let (_lock, temp_dir) = setup_test_env();

    let relative_path = "subdir/test.txt";
    let content = "Save Me";
    let base64_content = general_purpose::STANDARD.encode(content);

    // 1. 保存
    let save_res = utils::save_clipboard_file(relative_path.to_string(), base64_content).await;
    assert!(save_res.is_ok(), "保存失败: {:?}", save_res.err());

    // 2. 验证物理文件 (使用 files 子目录)
    let expected_file_path = temp_dir.join("files").join("subdir").join("test.txt");

    // 等待文件系统写入
    for _ in 0..20 {
        if expected_file_path.exists() {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    if !expected_file_path.exists() {
        println!(
            "❌ 文件未找到。当前配置路径: {:?}",
            crate::config::get_current_storage_path()
        );
        // 尝试列出 temp_dir 内容辅助调试
        if let Ok(entries) = fs::read_dir(&temp_dir) {
            println!("📂 临时目录内容:");
            for entry in entries {
                println!(" - {:?}", entry.unwrap().path());
            }
        }
    }

    assert!(
        expected_file_path.exists(),
        "文件未创建在预期路径: {:?}",
        expected_file_path
    );
    assert_eq!(fs::read_to_string(&expected_file_path).unwrap(), content);

    // 3. 获取列表
    let list = utils::get_local_files_to_upload().await.unwrap();
    let found = list
        .iter()
        .find(|f| f.file_path.replace("\\", "/").contains("subdir/test.txt"));
    assert!(found.is_some(), "列表中未找到文件");

    teardown_test_env(temp_dir);
}

#[tokio::test]
async fn test_save_clipboard_file_security() {
    let (_lock, temp_dir) = setup_test_env();

    let result = utils::save_clipboard_file("../hack.txt".to_string(), "".to_string()).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("非法字符"));

    teardown_test_env(temp_dir);
}

#[test]
fn test_resolve_absolute_path() {
    let (_lock, temp_dir) = setup_test_env();

    let relative = PathBuf::from("files/image.png");
    let absolute = utils::resolve_absolute_path(&relative);

    let abs_str = absolute.to_string_lossy().to_string().replace("\\", "/");
    let temp_str = temp_dir.to_string_lossy().to_string().replace("\\", "/");

    // Windows 大小写不敏感比较
    let contains = if cfg!(windows) {
        abs_str.to_lowercase().contains(&temp_str.to_lowercase())
    } else {
        abs_str.contains(&temp_str)
    };

    if !contains {
        println!("❌ 路径解析失败。");
        println!("期望包含: {}", temp_str);
        println!("实际路径: {}", abs_str);
    }

    assert!(contains, "绝对路径解析错误，未包含临时目录");

    teardown_test_env(temp_dir);
}

#[test]
fn test_process_file_for_clipboard_logic() {
    // 假设此函数不依赖 config，如果依赖，setup_test_env 会处理
    let (_lock, temp_dir) = setup_test_env();

    let original_name = "1234567890123-realname.txt";
    let file_path = temp_dir.join(original_name);
    fs::write(&file_path, "content").unwrap();

    let result = utils::process_file_for_clipboard(file_path.to_str().unwrap());
    let final_path = result.unwrap();

    assert_eq!(
        final_path.file_name().unwrap().to_string_lossy(),
        "realname.txt"
    );
    assert!(final_path.exists());

    teardown_test_env(temp_dir);
}
