use super::*;
use serde_json::json;

#[test]
fn test_update_simple_config_item() {
    // 确保配置已初始化
    let _ = init_config();

    // 1. 测试布尔值更新 (TrayIconVisible)
    let key = ConfigKey::TrayIconVisible;
    // 先设置为 true
    update_simple_config_item(&key, json!(true)).unwrap();
    assert_eq!(
        CONFIG.get().unwrap().read().unwrap().tray_icon_visible,
        true
    );

    // 设置为 false
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
    // 确保值未改变
    assert_eq!(CONFIG.get().unwrap().read().unwrap().max_history_items, 999);

    // 6. 测试 Autostart (应返回 Ok(false))
    let key = ConfigKey::Autostart;
    let res = update_simple_config_item(&key, json!(true));
    assert_eq!(res, Ok(false));
}
