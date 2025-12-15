use super::*;
use crate::clipboard::ClipboardItem;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

// --- 测试辅助函数 ---

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn set_test_db_path() {
    let mut p = std::env::temp_dir();
    p.push("smartpaste_test_private.db"); // 使用独立的文件名
    set_db_path(p);
    let _ = crate::clipboard::take_last_inserted();
}

fn clear_db_file() {
    let p: PathBuf = get_db_path();
    if p.exists() {
        let _ = fs::remove_file(p);
    }
}

fn make_item(id: &str, content: &str, notes: &str) -> ClipboardItem {
    ClipboardItem {
        id: id.to_string(),
        item_type: "text".to_string(),
        content: content.to_string(),
        size: Some(content.len() as u64),
        is_favorite: false,
        notes: notes.to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    }
}

/// 检查指定 ID 是否存在于 private_data 表中
fn is_item_private(id: &str) -> bool {
    let db_path = get_db_path();
    let conn = Connection::open(db_path).expect("failed to open db");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM private_data WHERE item_id = ?1",
            [id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    count > 0
}

// --- 测试用例 ---

#[test]
fn test_mark_passwords() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. 准备数据
    // 修正：mark_passwords_as_private 只检查 notes 字段

    // 匹配 notes (原 content 匹配改为 notes 匹配)
    let item1 = make_item("pw-1", "some content", "My password is 123456");
    // 匹配 notes
    let item2 = make_item("pw-2", "Just some text", "This is a secret key");
    // 不匹配
    let item3 = make_item("pw-3", "Hello world", "Just a note");
    // 匹配其他关键词 (login) - 移至 notes
    let item4 = make_item("pw-4", "some content", "Login credentials for site");
    // 匹配中文notes - 注意：正则使用 \b 边界，中文前后通常需要分隔符才能匹配 \b
    let item5 = make_item("pw-5", "普通文本", "这是一个 密码");

    insert_received_db_data(item1.clone()).unwrap();
    insert_received_db_data(item2.clone()).unwrap();
    insert_received_db_data(item3.clone()).unwrap();
    insert_received_db_data(item4.clone()).unwrap();
    insert_received_db_data(item5.clone()).unwrap();

    // 2. 执行标记 (添加)
    let count = mark_passwords_as_private(true).expect("mark passwords failed");

    // 3. 验证结果
    assert_eq!(count, 4, "Should mark 4 items as private");
    assert!(
        is_item_private(&item1.id),
        "item1 (password in notes) should be private"
    );
    assert!(
        is_item_private(&item2.id),
        "item2 (secret key in notes) should be private"
    );
    assert!(!is_item_private(&item3.id), "item3 should NOT be private");
    assert!(
        is_item_private(&item4.id),
        "item4 (login in notes) should be private"
    );
    assert!(
        is_item_private(&item5.id),
        "item5 (密码 in notes) should be private"
    );

    // 4. 执行取消标记 (删除)
    let count_removed = mark_passwords_as_private(false).expect("unmark passwords failed");
    assert_eq!(count_removed, 4, "Should unmark 4 items");
    assert!(
        !is_item_private(&item1.id),
        "item1 should no longer be private"
    );
    assert!(
        !is_item_private(&item2.id),
        "item2 should no longer be private"
    );
}

#[test]
fn test_mark_bank_cards() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. 准备数据
    // 有效的 Visa 卡号 (4000 0000 0000 0002 符合 Luhn 算法: 8+0+0+0... + 2 = 10)
    let valid_visa = "4000 0000 0000 0002";
    let item_valid = make_item("card-valid", &format!("Payment: {}", valid_visa), "");

    // 无效的卡号 (Luhn 校验失败)
    let item_invalid = make_item("card-invalid", "4000 0000 0000 0003", "");

    // 普通数字文本
    let item_text = make_item("card-text", "My number is 1234567890", "");

    insert_received_db_data(item_valid.clone()).unwrap();
    insert_received_db_data(item_invalid.clone()).unwrap();
    insert_received_db_data(item_text.clone()).unwrap();

    // 2. 执行标记
    let count = mark_bank_cards_as_private(true).expect("mark bank cards failed");

    // 3. 验证结果
    assert_eq!(count, 1, "Should mark 1 valid card");
    assert!(
        is_item_private(&item_valid.id),
        "Valid card should be private"
    );
    assert!(
        !is_item_private(&item_invalid.id),
        "Invalid Luhn card should NOT be private"
    );
    assert!(
        !is_item_private(&item_text.id),
        "Random numbers should NOT be private"
    );

    // 4. 取消标记
    mark_bank_cards_as_private(false).unwrap();
    assert!(
        !is_item_private(&item_valid.id),
        "Valid card should be unmarked"
    );
}

#[test]
fn test_mark_identity_numbers() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. 准备数据
    // 18位身份证
    let item18 = make_item("id-18", "ID: 110101199003071234", "");
    // 18位带X
    let item18x = make_item("id-18x", "11010119900307123X", "");
    // 15位身份证
    let item15 = make_item("id-15", "320102800101123", "");
    // 错误长度
    let item_short = make_item("id-short", "123456", "");

    insert_received_db_data(item18.clone()).unwrap();
    insert_received_db_data(item18x.clone()).unwrap();
    insert_received_db_data(item15.clone()).unwrap();
    insert_received_db_data(item_short.clone()).unwrap();

    // 2. 执行标记
    let count = mark_identity_numbers_as_private(true).expect("mark id failed");

    // 3. 验证结果
    assert_eq!(count, 3, "Should mark 3 ID numbers");
    assert!(is_item_private(&item18.id));
    assert!(is_item_private(&item18x.id));
    assert!(is_item_private(&item15.id));
    assert!(!is_item_private(&item_short.id));

    // 4. 取消标记
    mark_identity_numbers_as_private(false).unwrap();
    assert!(!is_item_private(&item18.id));
}

#[test]
fn test_mark_phone_numbers() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. 准备数据
    // 正常手机号
    let item_phone = make_item("ph-1", "Call 13800138000 now", "");
    // 另一手机号
    let item_phone2 = make_item("ph-2", "18912345678", "");
    // 非手机号 (110)
    let item_short = make_item("ph-3", "110", "");
    // 11位数字但非手机号开头 (例如 100...)
    // 注意：正则 \b1[3-9]\d{9}\b 限制了第二位必须是 3-9
    let item_fake = make_item("ph-4", "10012345678", "");

    insert_received_db_data(item_phone.clone()).unwrap();
    insert_received_db_data(item_phone2.clone()).unwrap();
    insert_received_db_data(item_short.clone()).unwrap();
    insert_received_db_data(item_fake.clone()).unwrap();

    // 2. 执行标记
    let count = mark_phone_numbers_as_private(true).expect("mark phone failed");

    // 3. 验证结果
    assert_eq!(count, 2, "Should mark 2 phone numbers");
    assert!(is_item_private(&item_phone.id));
    assert!(is_item_private(&item_phone2.id));
    assert!(!is_item_private(&item_short.id));
    assert!(!is_item_private(&item_fake.id));

    // 4. 取消标记
    mark_phone_numbers_as_private(false).unwrap();
    assert!(!is_item_private(&item_phone.id));
}

#[test]
fn test_get_all_private_data() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. 准备数据
    // 修正：将 "password: 123" 放入 notes，因为 mark_passwords_as_private 只查 notes
    let item1 = make_item("p-1", "some content", "password: 123"); // 应被标记 (密码)
    let item2 = make_item("p-2", "normal text", ""); // 不应被标记
    let item3 = make_item("p-3", "13800138000", ""); // 应被标记 (手机号，查 content)

    insert_received_db_data(item1.clone()).unwrap();
    insert_received_db_data(item2.clone()).unwrap();
    insert_received_db_data(item3.clone()).unwrap();

    // 2. 标记隐私数据
    mark_passwords_as_private(true).unwrap();
    mark_phone_numbers_as_private(true).unwrap();

    // 3. 获取所有隐私数据
    let json_result = get_all_private_data().expect("get_all_private_data failed");
    let items: Vec<ClipboardItem> =
        serde_json::from_str(&json_result).expect("failed to parse json");

    // 4. 验证
    assert_eq!(items.len(), 2, "Should return exactly 2 private items");
    let ids: Vec<String> = items.iter().map(|i| i.id.clone()).collect();
    assert!(ids.contains(&item1.id), "Should contain password item");
    assert!(ids.contains(&item3.id), "Should contain phone item");
    assert!(!ids.contains(&item2.id), "Should NOT contain normal item");
}

#[test]
fn test_clear_all_private_data() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. 准备数据并标记
    // 修正：将 "password: 123" 放入 notes
    let item1 = make_item("c-1", "some content", "password: 123");
    insert_received_db_data(item1.clone()).unwrap();
    mark_passwords_as_private(true).unwrap();

    assert!(
        is_item_private(&item1.id),
        "Item should be private initially"
    );

    // 2. 清除隐私标记
    let count = clear_all_private_data().expect("clear_all_private_data failed");
    assert_eq!(count, 1, "Should clear 1 record");

    // 3. 验证 private_data 表为空
    assert!(
        !is_item_private(&item1.id),
        "Item should no longer be marked private"
    );

    // 4. 验证原始数据仍然存在 (clear_all_private_data 只清除标记，不删除原数据)
    let json = get_data_by_id(&item1.id).unwrap();
    assert_ne!(json, "null", "Actual data item should still exist");
}
