/// config 单元测试
use super::*;
use serde_json::json;
use std::sync::Mutex;
use uuid::Uuid;

// 使用互斥锁确保测试串行执行,避免全局状态冲突
static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_update_simple_config_item() {
    // 🔧 修复: 使用 unwrap_or_else 处理中毒的锁
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _ = init_config();

    // 1. 测试布尔值更新 (TrayIconVisible)
    let key = ConfigKey::TrayIconVisible;
    update_simple_config_item(&key, json!(true)).unwrap();
    assert_eq!(
        CONFIG.get().unwrap().read().unwrap().tray_icon_visible,
        true
    );

    let res = update_simple_config_item(&key, json!(false));
    assert_eq!(res, Ok(true));
    assert_eq!(
        CONFIG.get().unwrap().read().unwrap().tray_icon_visible,
        false
    );

    // 2. 测试数值更新 (MaxHistoryItems)
    let key = ConfigKey::MaxHistoryItems;
    let res = update_simple_config_item(&key, json!(999));
    assert_eq!(res, Ok(true));
    assert_eq!(CONFIG.get().unwrap().read().unwrap().max_history_items, 999);

    // 3. 测试字符串更新 (GlobalShortcut)
    let key = ConfigKey::GlobalShortcut;
    let res = update_simple_config_item(&key, json!("Ctrl+Alt+K"));
    assert_eq!(res, Ok(true));
    assert_eq!(
        CONFIG.get().unwrap().read().unwrap().global_shortcut,
        "Ctrl+Alt+K"
    );

    // 4. 测试 Option 类型更新 (AiApiKey)
    let key = ConfigKey::AiApiKey;
    let res = update_simple_config_item(&key, json!("sk-123456"));
    assert_eq!(res, Ok(true));
    assert_eq!(
        CONFIG.get().unwrap().read().unwrap().ai_api_key,
        Some("sk-123456".to_string())
    );

    let res = update_simple_config_item(&key, json!(null));
    assert_eq!(res, Ok(true));
    assert_eq!(CONFIG.get().unwrap().read().unwrap().ai_api_key, None);

    // 5. 测试类型错误
    let key = ConfigKey::MaxHistoryItems;
    let res = update_simple_config_item(&key, json!("not a number"));
    assert!(res.is_err());

    // 6. 测试 Autostart (应返回 Ok(false))
    let key = ConfigKey::Autostart;
    let res = update_simple_config_item(&key, json!(true));
    assert_eq!(res, Ok(false));
}

#[test]
fn test_get_config_item() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _ = init_config();

    // 重置为默认值以确保测试独立性
    update_simple_config_item(&ConfigKey::TrayIconVisible, json!(true)).unwrap();
    update_simple_config_item(&ConfigKey::MaxHistoryItems, json!(500)).unwrap();

    // 1. 测试获取布尔值 (TrayIconVisible)
    let result = get_config_item("tray_icon_visible");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!(true));

    // 2. 测试获取数值 (MaxHistoryItems)
    let result = get_config_item("max_history_items");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!(500));

    // 3. 测试获取字符串 (GlobalShortcut)
    let result = get_config_item("global_shortcut");
    assert!(result.is_ok());

    // 4. 测试获取 Option<String> (AiApiKey)
    let result = get_config_item("ai_api_key");
    assert!(result.is_ok());

    // 5. 测试获取 Vec<String> (IgnoredApps)
    let result = get_config_item("ignored_apps");
    assert!(result.is_ok());

    // 6. 测试无效的配置键
    let result = get_config_item("invalid_key");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid config key: invalid_key");

    // 7. 测试修改后再获取
    update_simple_config_item(&ConfigKey::MaxHistoryItems, json!(999)).unwrap();
    let result = get_config_item("max_history_items");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), json!(999));
}

#[test]
fn test_parse_config_key() {
    // 此测试不需要锁,因为不修改全局状态
    assert_eq!(parse_config_key("autostart"), Some(ConfigKey::Autostart));
    assert_eq!(
        parse_config_key("tray_icon_visible"),
        Some(ConfigKey::TrayIconVisible)
    );
    assert_eq!(
        parse_config_key("max_history_items"),
        Some(ConfigKey::MaxHistoryItems)
    );
    assert_eq!(parse_config_key("ai_enabled"), Some(ConfigKey::AiEnabled));
    assert_eq!(
        parse_config_key("ocr_provider"),
        Some(ConfigKey::OcrProvider)
    );

    assert_eq!(parse_config_key("invalid_key"), None);
    assert_eq!(parse_config_key(""), None);
    assert_eq!(parse_config_key("AUTOSTART"), None);
}

#[test]
fn test_config_default_values() {
    let config = Config::default();

    assert_eq!(config.autostart, false);
    assert_eq!(config.tray_icon_visible, true);
    assert_eq!(config.minimize_to_tray, false);
    assert_eq!(config.auto_save, true);
    assert_eq!(config.retention_days, 30);
    assert_eq!(config.global_shortcut, "Shift+V");

    assert_eq!(config.max_history_items, 500);
    assert_eq!(config.ignore_short_text_len, 0);
    assert_eq!(config.ignore_big_file_mb, 5);
    assert_eq!(config.ignored_apps, Vec::<String>::new());
    assert_eq!(config.auto_classify, true);

    assert_eq!(config.ai_enabled, false);
    // assert_eq!(config.ai_service, None);
    assert_eq!(config.ai_api_key, None);

    assert_eq!(config.sensitive_filter, true);
    // assert_eq!(config.privacy_retention_days, 90);

    assert_eq!(config.cloud_sync_enabled, false);
    assert_eq!(config.sync_frequency, "5min");
}

#[test]
fn test_update_multiple_config_items() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _ = init_config();

    update_simple_config_item(&ConfigKey::TrayIconVisible, json!(false)).unwrap();
    update_simple_config_item(&ConfigKey::MaxHistoryItems, json!(1000)).unwrap();
    update_simple_config_item(&ConfigKey::AiEnabled, json!(true)).unwrap();
    update_simple_config_item(&ConfigKey::AiApiKey, json!("sk-test-key")).unwrap();

    let cfg = CONFIG.get().unwrap().read().unwrap();
    assert_eq!(cfg.tray_icon_visible, false);
    assert_eq!(cfg.max_history_items, 1000);
    assert_eq!(cfg.ai_enabled, true);
    assert_eq!(cfg.ai_api_key, Some("sk-test-key".to_string()));
}

#[test]
fn test_update_config_with_edge_cases() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _ = init_config();

    assert!(update_simple_config_item(&ConfigKey::MaxHistoryItems, json!(0)).is_ok());
    assert!(update_simple_config_item(&ConfigKey::MaxHistoryItems, json!(u32::MAX)).is_ok());

    assert!(update_simple_config_item(&ConfigKey::GlobalShortcut, json!("")).is_ok());

    assert!(update_simple_config_item(&ConfigKey::IgnoredApps, json!([])).is_ok());

    assert!(update_simple_config_item(&ConfigKey::AiApiKey, json!(null)).is_ok());
    assert_eq!(CONFIG.get().unwrap().read().unwrap().ai_api_key, None);
}

#[test]
fn test_config_serialization() {
    let config = Config::default();
    let json_str = config_to_json(&config);

    assert!(!json_str.is_empty());

    let deserialized: Config = serde_json::from_str(&json_str).unwrap();
    assert_eq!(config, deserialized);
}

#[test]
fn test_get_config_item_all_types() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _ = init_config();

    assert!(get_config_item("autostart").is_ok());
    assert!(get_config_item("ai_enabled").is_ok());

    assert!(get_config_item("retention_days").is_ok());
    assert!(get_config_item("max_history_items").is_ok());

    assert!(get_config_item("global_shortcut").is_ok());
    assert!(get_config_item("backup_frequency").is_ok());

    assert!(get_config_item("ai_api_key").is_ok());
    assert!(get_config_item("storage_path").is_ok());

    assert!(get_config_item("ignored_apps").is_ok());
    // assert!(get_config_item("privacy_records").is_ok());

    assert!(get_config_item("ocr_languages").is_ok());

    assert!(get_config_item("ocr_confidence_threshold").is_ok());

    assert!(get_config_item("ocr_timeout_secs").is_ok());
}

// 新增测试

#[test]
fn test_vec_config_updates() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _ = init_config();

    // 测试 Vec<String> 更新
    let apps = vec!["app1".to_string(), "app2".to_string()];
    update_simple_config_item(&ConfigKey::IgnoredApps, json!(apps)).unwrap();

    let result = get_config_item("ignored_apps");
    assert_eq!(result.unwrap(), json!(["app1", "app2"]));
}

#[test]
fn test_option_vec_config() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _ = init_config();

    // 测试 Option<Vec<String>>
    let langs = vec!["en".to_string(), "zh".to_string()];
    update_simple_config_item(&ConfigKey::OcrLanguages, json!(langs)).unwrap();

    let result = get_config_item("ocr_languages");
    assert_eq!(result.unwrap(), json!(["en", "zh"]));

    // 测试设置为 null
    update_simple_config_item(&ConfigKey::OcrLanguages, json!(null)).unwrap();
    let result = get_config_item("ocr_languages");
    assert_eq!(result.unwrap(), json!(null));
}

#[test]
fn test_numeric_option_types() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _ = init_config();

    // 测试 Option<f32>
    update_simple_config_item(&ConfigKey::OcrConfidenceThreshold, json!(0.85)).unwrap();
    let result = get_config_item("ocr_confidence_threshold");

    // 验证结果是 f32 类型且值接近 0.85
    if let Ok(value) = result {
        if let Some(num) = value.as_f64() {
            let diff = (num - 0.85).abs();
            assert!(diff < 0.0001, "Expected ~0.85, got {}", num);
        } else {
            panic!("Expected number, got {:?}", value);
        }
    } else {
        panic!("Failed to get config item");
    }

    // 测试 Option<u64>
    update_simple_config_item(&ConfigKey::OcrTimeoutSecs, json!(30)).unwrap();
    let result = get_config_item("ocr_timeout_secs");
    assert_eq!(result.unwrap(), json!(30));
}

#[test]
fn test_config_file_operations() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    // 1. Setup temp config path
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join(format!("test_config_{}.json", Uuid::new_v4()));
    set_config_path(config_path.clone());

    // 2. Init config (should create file)
    let res = init_config();
    assert!(res.contains("initialized successfully") || res.contains("config json already exists"));
    assert!(config_path.exists());

    // Force reload global config from the file we just created
    // This ensures we are testing against a clean state regardless of other tests
    if let Some(lock) = CONFIG.get() {
        let content = std::fs::read_to_string(&config_path).unwrap();
        let new_cfg: Config = serde_json::from_str(&content).unwrap();
        let mut cfg = lock.write().unwrap();
        *cfg = new_cfg;
    }

    // 3. Verify default values
    let cfg = CONFIG.get().unwrap().read().unwrap();
    assert_eq!(cfg.max_history_items, 500);
    drop(cfg); // Release lock

    // 4. Modify and Save
    let mut cfg = CONFIG.get().unwrap().read().unwrap().clone();
    cfg.max_history_items = 1000;
    save_config(cfg.clone()).unwrap();

    // 5. Reload
    let reload_res = reload_config();
    assert_eq!(reload_res, "reloaded successfully");

    let cfg_reloaded = CONFIG.get().unwrap().read().unwrap();
    assert_eq!(cfg_reloaded.max_history_items, 1000);
    drop(cfg_reloaded);

    // 6. Test get_config_json
    let json_str = get_config_json();
    assert!(json_str.contains("\"max_history_items\": 1000"));

    // 7. Test set_config_item_internal
    set_config_item_internal("max_history_items", json!(2000)).unwrap();
    let cfg_internal = CONFIG.get().unwrap().read().unwrap();
    assert_eq!(cfg_internal.max_history_items, 2000);
    drop(cfg_internal);

    // 8. Test set_config_item_internal error
    let err = set_config_item_internal("invalid_key", json!(1));
    assert!(err.is_err());

    let err_type = set_config_item_internal("max_history_items", json!("string"));
    assert!(err_type.is_err());

    // Cleanup
    let _ = std::fs::remove_file(config_path);
}

#[test]
fn test_config_coverage_extensions() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _ = init_config();

    // Cover all ConfigKey variants in update_simple_config_item
    // Boolean fields
    let bool_keys = vec![
        ConfigKey::MinimizeToTray,
        ConfigKey::AutoSave,
        ConfigKey::AutoClassify,
        ConfigKey::OcrAutoRecognition,
        ConfigKey::DeleteConfirmation,
        ConfigKey::KeepFavoritesOnDelete,
        ConfigKey::AutoSort,
        ConfigKey::AiEnabled,
        ConfigKey::AiAutoTag,
        ConfigKey::AiAutoSummary,
        ConfigKey::AiTranslation,
        ConfigKey::AiWebSearch,
        ConfigKey::SensitiveFilter,
        ConfigKey::FilterPasswords,
        ConfigKey::FilterBankCards,
        ConfigKey::FilterIdCards,
        ConfigKey::FilterPhoneNumbers,
        ConfigKey::AutoBackup,
        ConfigKey::CloudSyncEnabled,
        ConfigKey::EncryptCloudData,
        ConfigKey::SyncOnlyWifi,
    ];

    for key in bool_keys {
        update_simple_config_item(&key, json!(true)).unwrap();
        update_simple_config_item(&key, json!(false)).unwrap();
    }

    // String fields
    let string_keys = vec![
        ConfigKey::GlobalShortcut2,
        ConfigKey::GlobalShortcut3,
        ConfigKey::GlobalShortcut4,
        ConfigKey::GlobalShortcut5,
        ConfigKey::AiProvider,
        ConfigKey::AiModel,
        ConfigKey::BackupFrequency,
        ConfigKey::SyncFrequency,
        ConfigKey::SyncContentType,
    ];

    for key in string_keys {
        update_simple_config_item(&key, json!("test_value")).unwrap();
    }

    // Option<String> fields
    let opt_string_keys = vec![
        ConfigKey::AiBaseUrl,
        ConfigKey::StoragePath,
        ConfigKey::LastBackupPath,
        ConfigKey::Username,
        ConfigKey::Email,
        ConfigKey::Bio,
        ConfigKey::AvatarPath,
        ConfigKey::OcrProvider,
    ];

    for key in opt_string_keys {
        update_simple_config_item(&key, json!("some_val")).unwrap();
        update_simple_config_item(&key, json!(null)).unwrap();
    }

    // u32 fields
    let u32_keys = vec![
        ConfigKey::RetentionDays,
        ConfigKey::IgnoreShortTextLen,
        ConfigKey::IgnoreBigFileMb,
    ];
    for key in u32_keys {
        update_simple_config_item(&key, json!(10)).unwrap();
    }

    // f32 fields
    update_simple_config_item(&ConfigKey::AiTemperature, json!(0.5)).unwrap();

    // Cover get_config_item for all keys
    let all_keys = vec![
        "minimize_to_tray",
        "auto_save",
        "retention_days",
        "global_shortcut_2",
        "global_shortcut_3",
        "global_shortcut_4",
        "global_shortcut_5",
        "ignore_short_text_len",
        "ignore_big_file_mb",
        "auto_classify",
        "ocr_auto_recognition",
        "delete_confirmation",
        "keep_favorites_on_delete",
        "auto_sort",
        "ai_provider",
        "ai_model",
        "ai_base_url",
        "ai_temperature",
        "ai_auto_tag",
        "ai_auto_summary",
        "ai_translation",
        "ai_web_search",
        "sensitive_filter",
        "filter_passwords",
        "filter_bank_cards",
        "filter_id_cards",
        "filter_phone_numbers",
        "storage_path",
        "auto_backup",
        "backup_frequency",
        "last_backup_path",
        "cloud_sync_enabled",
        "sync_frequency",
        "sync_content_type",
        "encrypt_cloud_data",
        "sync_only_wifi",
        "username",
        "email",
        "bio",
        "avatar_path",
        "ocr_provider",
    ];

    for key in all_keys {
        assert!(get_config_item(key).is_ok(), "Failed to get key: {}", key);
    }
}

#[test]
fn test_private_functions_coverage() {
    // Test normalize_to_forward_slashes
    assert_eq!(normalize_to_forward_slashes("a\\b\\c"), "a/b/c");
    assert_eq!(normalize_to_forward_slashes("a/b/c"), "a/b/c");

    // Test copy_dir_all (basic file op)
    let temp_dir = std::env::temp_dir().join(format!("test_copy_{}", Uuid::new_v4()));
    let src = temp_dir.join("src");
    let dst = temp_dir.join("dst");

    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("file.txt"), "content").unwrap();
    std::fs::create_dir(src.join("subdir")).unwrap();
    std::fs::write(src.join("subdir").join("subfile.txt"), "sub").unwrap();

    // Test copy
    crate::config::copy_dir_all(&src, &dst).unwrap();

    assert!(dst.join("file.txt").exists());
    assert!(dst.join("subdir").join("subfile.txt").exists());

    // Test failure: dst exists but is file
    let bad_dst = temp_dir.join("bad_dst");
    std::fs::write(&bad_dst, "I am a file").unwrap();
    let res = crate::config::copy_dir_all(&src, &bad_dst);
    assert!(res.is_err());

    std::fs::remove_dir_all(temp_dir).ok();
}

#[test]
fn test_migrate_data() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let temp_dir = std::env::temp_dir().join(format!("test_migrate_{}", Uuid::new_v4()));
    let old_path = temp_dir.join("old");
    let new_path = temp_dir.join("new");

    // Setup old path
    std::fs::create_dir_all(&old_path).unwrap();
    std::fs::write(old_path.join("smartpaste.db"), "db content").unwrap();
    std::fs::create_dir(old_path.join("files")).unwrap();
    std::fs::write(old_path.join("files").join("img.png"), "img").unwrap();

    // Test migration
    let res = crate::config::migrate_data_to_new_path(&old_path, &new_path);
    assert!(res.is_ok());

    // Verify new path
    assert!(new_path.join("smartpaste.db").exists());
    assert!(new_path.join("files").join("img.png").exists());

    // Verify old path cleanup (files dir should be gone)
    assert!(!old_path.join("files").exists());

    // Test failure: new path creation fails (invalid path)
    // On Windows, using a reserved name or invalid char might work
    // But simpler to just pass a file as new_path
    let file_as_dir = temp_dir.join("file_as_dir");
    std::fs::write(&file_as_dir, "content").unwrap();
    let res = crate::config::migrate_data_to_new_path(&old_path, &file_as_dir);
    assert!(res.is_err());

    std::fs::remove_dir_all(temp_dir).ok();
}

#[test]
fn test_set_config_item_internal_error() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let _ = init_config();

    // Autostart requires AppHandle, so internal set should fail
    let res = set_config_item_internal("autostart", json!(true));
    assert!(res.is_err());
    assert!(res.err().unwrap().contains("requires AppHandle"));

    // Invalid key
    let res = set_config_item_internal("invalid_key_xyz", json!(true));
    assert!(res.is_err());
}

#[test]
fn test_reload_config_errors() {
    let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());

    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join(format!("test_config_err_{}.json", Uuid::new_v4()));
    set_config_path(config_path.clone());

    // 1. File not found
    let res = reload_config();
    assert_eq!(res, "File not found");

    // 2. Parse error
    std::fs::write(&config_path, "{ invalid json").unwrap();
    let res = reload_config();
    assert!(res.contains("Parse error"));

    std::fs::remove_file(config_path).ok();
}
