/// config 单元测试
use super::*;
use serde_json::json;
use std::sync::Mutex;

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
    assert_eq!(config.global_shortcut, "Alt+Shift+V");

    assert_eq!(config.max_history_items, 500);
    assert_eq!(config.ignore_short_text_len, 0);
    assert_eq!(config.ignore_big_file_mb, 5);
    assert_eq!(config.ignored_apps, Vec::<String>::new());
    assert_eq!(config.auto_classify, true);

    assert_eq!(config.ai_enabled, false);
    assert_eq!(config.ai_service, None);
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
    assert!(get_config_item("privacy_records").is_ok());

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
