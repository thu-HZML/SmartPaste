use super::*; // 关键：访问 app_setup.rs 中的所有内容（包括私有函数）
use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;
use crate::app_setup::normalize_shortcut_format;
use crate::app_setup::AppShortcutManager;
use crate::app_setup::SHORTCUT_CONFIGS;
use crate::app_setup::STORAGE_KEY_TO_HANDLER_KEY;
use crate::app_setup::copy_dir_all;
// --- 1. 快捷键格式化逻辑测试 (私有函数) ---
#[test]
fn test_normalize_shortcut_format() {
    // 基础大小写转换
    assert_eq!(normalize_shortcut_format("Ctrl+Alt+Delete"), "control+alt+delete");
    assert_eq!(normalize_shortcut_format("SHIFT+V"), "shift+v");

    // 特殊键名映射 (KeyA -> a)
    assert_eq!(normalize_shortcut_format("KeyA"), "a");
    assert_eq!(normalize_shortcut_format("KeyZ"), "z");
    assert_eq!(normalize_shortcut_format("Ctrl+KeyC"), "control+c");

    // 修饰键映射 (Cmd/Meta -> Super)
    assert_eq!(normalize_shortcut_format("Cmd+C"), "super+c");
    assert_eq!(normalize_shortcut_format("Meta+V"), "super+v");
    assert_eq!(normalize_shortcut_format("Command+Shift+3"), "super+shift+3");

    // 复杂组合验证
    assert_eq!(
        normalize_shortcut_format("Ctrl+Shift+KeyV"),
        "control+shift+v"
    );
}

// --- 2. 快捷键管理器状态测试 ---
#[test]
fn test_app_shortcut_manager_logic() {
    let manager = AppShortcutManager::new();

    // 初始状态应该为空
    {
        let map = manager.shortcuts.lock().unwrap();
        assert!(map.is_empty());
    }

    // 测试设置快捷键
    manager.set_shortcut("toggleWindow", "Alt+Space".to_string());
    assert_eq!(
        manager.get_shortcut("toggleWindow"),
        Some("Alt+Space".to_string())
    );

    // 测试覆盖设置
    manager.set_shortcut("toggleWindow", "Ctrl+Space".to_string());
    assert_eq!(
        manager.get_shortcut("toggleWindow"),
        Some("Ctrl+Space".to_string())
    );

    // 测试获取不存在的快捷键
    assert_eq!(manager.get_shortcut("nonexistent"), None);

    // 测试删除快捷键
    manager.remove_shortcut("toggleWindow");
    assert_eq!(manager.get_shortcut("toggleWindow"), None);
}

// --- 3. 静态配置映射一致性测试 (私有静态变量) ---
#[test]
fn test_shortcut_configs_integrity() {
    // 确保配置不为空
    assert!(!SHORTCUT_CONFIGS.is_empty());
    assert!(!STORAGE_KEY_TO_HANDLER_KEY.is_empty());

    // 遍历所有配置，验证反向映射是否存在
    for (handler_key, config) in SHORTCUT_CONFIGS.iter() {
        // 验证 storage_key 是否有对应的 handler_key 映射
        let mapped_handler_key = STORAGE_KEY_TO_HANDLER_KEY.get(config.storage_key);
        
        assert!(mapped_handler_key.is_some(), "Missing mapping for storage key: {}", config.storage_key);
        assert_eq!(mapped_handler_key.unwrap(), handler_key, "Mapping mismatch for {}", handler_key);
        
        // 验证默认值不为空
        assert!(!config.default_value.is_empty());
    }
    
    // 验证特定关键配置是否存在
    assert!(SHORTCUT_CONFIGS.contains_key("toggleWindow"));
    assert!(SHORTCUT_CONFIGS.contains_key("clearHistory"));
}

// --- 4. 文件递归复制测试 (私有函数) ---
#[test]
fn test_copy_dir_all() {
    // 创建临时源目录
    let src_dir = tempdir().expect("failed to create src temp dir");
    let dst_dir = tempdir().expect("failed to create dst temp dir");
    let src_path = src_dir.path();
    // 目标路径需要在 dst_dir 内部创建一个新名字
    let dst_path = dst_dir.path().join("backup");

    // 1. 在源目录创建文件结构
    // src/file1.txt
    let file1_path = src_path.join("file1.txt");
    let mut f1 = File::create(&file1_path).unwrap();
    writeln!(f1, "content1").unwrap();

    // src/subdir/
    let subdir_path = src_path.join("subdir");
    fs::create_dir(&subdir_path).unwrap();

    // src/subdir/file2.txt
    let file2_path = subdir_path.join("file2.txt");
    let mut f2 = File::create(&file2_path).unwrap();
    writeln!(f2, "content2").unwrap();

    // 2. 执行复制
    let copy_result = copy_dir_all(src_path, &dst_path);
    assert!(copy_result.is_ok());
    
    // 验证返回的大小是否大于0
    let total_bytes = copy_result.unwrap();
    assert!(total_bytes > 0);

    // 3. 验证目标目录结构
    assert!(dst_path.exists());
    assert!(dst_path.join("file1.txt").exists());
    assert!(dst_path.join("subdir").exists());
    assert!(dst_path.join("subdir").join("file2.txt").exists());

    // 验证内容
    let content1 = fs::read_to_string(dst_path.join("file1.txt")).unwrap();
    assert_eq!(content1, "content1\n");
}