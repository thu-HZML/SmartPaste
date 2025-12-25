use super::*;
use crate::clipboard::ClipboardItem;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

// --- 测试辅助函数 ---

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    crate::db::TEST_RUN_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

use uuid::Uuid;

fn set_test_db_path() {
    let mut p = std::env::temp_dir();
    let filename = format!("smartpaste_test_private_{}.db", Uuid::new_v4());
    p.push(filename); // 使用独立的文件名
    set_db_path(p);
    let _ = crate::clipboard::take_last_inserted();
}

fn clear_db_file() {
    let p: PathBuf = get_db_path();
    if p.exists() {
        for _ in 0..5 {
            if fs::remove_file(&p).is_ok() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        // Try one last time and panic if fails
        fs::remove_file(&p).expect("failed to remove test db file");
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
    // 匹配中文notes - 中文不再强制要求 \b 边界
    let item5 = make_item("pw-5", "普通文本", "这是一个密码");

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
    let json_result =
        comprehensive_search("", Some("private"), None, None).expect("comprehensive_search failed");
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

#[test]
fn test_check_and_mark_private_item() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. Prepare item
    let item = make_item("p-item-1", "content", "This is a secret password");
    insert_received_db_data(item.clone()).unwrap();

    // 2. Test marking as private (password flag = true)
    let result = check_and_mark_private_item(item.clone(), true, false, false, false);
    assert!(result.is_ok());
    assert!(is_item_private(&item.id));

    // 3. Test unmarking (password flag = false)
    // Note: check_and_mark_private_item logic says: if match and flag false, delete.
    let result = check_and_mark_private_item(item.clone(), false, false, false, false);
    assert!(result.is_ok());
    assert!(!is_item_private(&item.id));

    // 4. Test non-matching item
    let item2 = make_item("p-item-2", "content", "Just normal text");
    insert_received_db_data(item2.clone()).unwrap();
    let result = check_and_mark_private_item(item2.clone(), true, true, true, true);
    assert!(result.is_ok());
    assert!(!is_item_private(&item2.id));
}

#[test]
fn test_unmark_privacy() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. Prepare items
    // "password 123" matches \bpassword\b
    let item_pwd = make_item("p-1", "content", "password 123");
    // Valid Visa: 4000 0000 0000 0002
    let item_card = make_item("p-2", "4000 0000 0000 0002", "notes");

    insert_received_db_data(item_pwd.clone()).unwrap();
    insert_received_db_data(item_card.clone()).unwrap();

    // 2. Mark them first
    let c1 = mark_passwords_as_private(true).unwrap();
    assert_eq!(c1, 1, "Should mark 1 password");

    let c2 = mark_bank_cards_as_private(true).unwrap();
    assert_eq!(c2, 1, "Should mark 1 card");

    assert!(is_item_private(&item_pwd.id), "pwd should be private");
    assert!(is_item_private(&item_card.id), "card should be private");

    // 3. Unmark passwords
    let count = mark_passwords_as_private(false).unwrap();
    assert_eq!(count, 1);
    assert!(!is_item_private(&item_pwd.id));
    assert!(is_item_private(&item_card.id)); // Card should still be private

    // 4. Unmark cards
    let count2 = mark_bank_cards_as_private(false).unwrap();
    assert_eq!(count2, 1);
    assert!(!is_item_private(&item_card.id));
}

#[test]
fn test_luhn_algorithm_cases() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // Valid Visa: 4000 0000 0000 0002
    // Invalid: 4000 0000 0000 0003
    let valid_card = make_item("c-valid", "4000 0000 0000 0002", "");
    let invalid_card = make_item("c-invalid", "4000 0000 0000 0003", "");
    let mixed_text = make_item("c-mixed", "My card is 4000-0000-0000-0002 ok?", "");

    insert_received_db_data(valid_card.clone()).unwrap();
    insert_received_db_data(invalid_card.clone()).unwrap();
    insert_received_db_data(mixed_text.clone()).unwrap();

    let count = mark_bank_cards_as_private(true).unwrap();
    assert_eq!(count, 2); // valid_card and mixed_text should be marked

    assert!(is_item_private(&valid_card.id));
    assert!(!is_item_private(&invalid_card.id));
    assert!(is_item_private(&mixed_text.id));
}

#[test]
fn test_is_valid_luhn_direct() {
    // Test empty (Line 101 coverage)
    assert!(!is_valid_luhn(""));
    // Test non-digit (Line 101 coverage)
    assert!(!is_valid_luhn("123a"));
    // Test valid
    assert!(is_valid_luhn("4000000000000002"));

    // Test high digit (Line 115 coverage)
    // 4000 0000 0000 0085
    // Reversed: 5, 8, 0...
    // Index 1 is 8. 8*2 = 16 > 9. 16-9=7. This triggers the digit -= 9 branch.
    // Sum: 5 + 7 + 0... + 8 (from 4*2) = 20. Valid.
    assert!(is_valid_luhn("4000000000000085"));
}

#[test]
fn test_luhn_coverage_high_digit() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 4000 0000 0000 0085
    // Reversed: 5, 8, 0...
    // Index 1 is 8. 8*2 = 16 > 9. 16-9=7. This triggers the digit -= 9 branch.
    // Sum: 5 + 7 + 0... + 8 (from 4*2) = 20. Valid.
    let item = make_item("c-high", "4000 0000 0000 0085", "");
    insert_received_db_data(item.clone()).unwrap();

    let count = mark_bank_cards_as_private(true).unwrap();
    assert_eq!(count, 1);
    assert!(is_item_private(&item.id));
}

#[test]
fn test_auto_mark_private_data_coverage() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let item_pwd = make_item("auto-1", "content", "password 123");
    let item_card = make_item("auto-2", "4000 0000 0000 0002", "");
    let item_id = make_item("auto-3", "110101199003071234", ""); // Valid ID regex
    let item_phone = make_item("auto-4", "13800138000", ""); // Valid Phone regex

    insert_received_db_data(item_pwd.clone()).unwrap();
    insert_received_db_data(item_card.clone()).unwrap();
    insert_received_db_data(item_id.clone()).unwrap();
    insert_received_db_data(item_phone.clone()).unwrap();

    // Enable all flags
    let count = auto_mark_private_data(true, true, true, true).unwrap();
    assert_eq!(count, 4);

    assert!(is_item_private(&item_pwd.id));
    assert!(is_item_private(&item_card.id));
    assert!(is_item_private(&item_id.id));
    assert!(is_item_private(&item_phone.id));
}

#[test]
fn test_check_and_mark_private_item_coverage() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. Password
    let item_pwd = make_item("chk-1", "content", "password 123");
    insert_received_db_data(item_pwd.clone()).unwrap();

    // Mark
    let res = check_and_mark_private_item(item_pwd.clone(), true, false, false, false).unwrap();
    assert!(res);
    assert!(is_item_private(&item_pwd.id));

    // Unmark
    let res = check_and_mark_private_item(item_pwd.clone(), false, false, false, false).unwrap();
    assert!(!res);
    assert!(!is_item_private(&item_pwd.id));

    // 2. Bank Card
    let item_card = make_item("chk-2", "4000 0000 0000 0002", "");
    insert_received_db_data(item_card.clone()).unwrap();

    // Mark
    let res = check_and_mark_private_item(item_card.clone(), false, true, false, false).unwrap();
    assert!(res);
    assert!(is_item_private(&item_card.id));

    // Unmark
    let res = check_and_mark_private_item(item_card.clone(), false, false, false, false).unwrap();
    assert!(!res);
    assert!(!is_item_private(&item_card.id));

    // 3. ID Number
    let item_id = make_item("chk-3", "110101199003071234", "");
    insert_received_db_data(item_id.clone()).unwrap();

    // Mark
    let res = check_and_mark_private_item(item_id.clone(), false, false, true, false).unwrap();
    assert!(res);
    assert!(is_item_private(&item_id.id));

    // Unmark
    let res = check_and_mark_private_item(item_id.clone(), false, false, false, false).unwrap();
    assert!(!res);
    assert!(!is_item_private(&item_id.id));

    // 4. Phone Number
    let item_phone = make_item("chk-4", "13800138000", "");
    insert_received_db_data(item_phone.clone()).unwrap();

    // Mark
    let res = check_and_mark_private_item(item_phone.clone(), false, false, false, true).unwrap();
    assert!(res);
    assert!(is_item_private(&item_phone.id));

    // Unmark
    let res = check_and_mark_private_item(item_phone.clone(), false, false, false, false).unwrap();
    assert!(!res);
    assert!(!is_item_private(&item_phone.id));
}

#[test]
fn test_encrypted_db_workflow() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. 准备数据
    let item = make_item("enc-1", "Sensitive Content", "Secret Note");
    insert_received_db_data(item.clone()).unwrap();

    // 2. 生成密钥
    let dek_hex = crate::db::privacy::generate_dek();

    // 3. 准备加密上传
    let encrypted_b64 = crate::db::privacy::prepare_encrypted_db_upload(dek_hex.clone())
        .expect("Failed to prepare encrypted upload");

    assert!(!encrypted_b64.is_empty());

    // 4. 模拟清空数据库（或在新环境）
    clear_db_file();
    let _ = init_db(get_db_path().as_path());

    // 验证数据已清空
    let conn = Connection::open(get_db_path()).unwrap();
    let count: i64 = conn
        .query_row("SELECT count(*) FROM data", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
    drop(conn); // 释放连接以便 restore

    // 5. 从加密数据恢复
    crate::db::privacy::restore_from_encrypted_db(dek_hex, encrypted_b64)
        .expect("Failed to restore from encrypted db");

    // 6. 验证数据已恢复
    let conn = Connection::open(get_db_path()).unwrap();
    let content: String = conn
        .query_row("SELECT content FROM data WHERE id = 'enc-1'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(content, "Sensitive Content");
    drop(conn);

    clear_db_file();
}

#[test]
fn test_file_encryption() {
    let _g = test_lock();
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join(format!("test_plain_{}.txt", Uuid::new_v4()));
    let enc_path = temp_dir.join(format!("test_enc_{}.bin", Uuid::new_v4()));
    let dec_path = temp_dir.join(format!("test_dec_{}.txt", Uuid::new_v4()));

    // 1. 创建源文件
    let original_content = "This is a secret file content.";
    fs::write(&input_path, original_content).unwrap();

    // 2. 生成密钥
    let dek_hex = crate::db::privacy::generate_dek();

    // 3. 加密
    crate::db::privacy::encrypt_file(
        input_path.to_string_lossy().to_string(),
        enc_path.to_string_lossy().to_string(),
        dek_hex.clone(),
    )
    .expect("Encryption failed");

    assert!(enc_path.exists());
    let enc_content = fs::read(&enc_path).unwrap();
    assert_ne!(enc_content, original_content.as_bytes());

    // 4. 解密
    crate::db::privacy::decrypt_file(
        enc_path.to_string_lossy().to_string(),
        dec_path.to_string_lossy().to_string(),
        dek_hex,
    )
    .expect("Decryption failed");

    // 5. 验证
    let dec_content = fs::read_to_string(&dec_path).unwrap();
    assert_eq!(dec_content, original_content);

    // Cleanup
    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(enc_path);
    let _ = fs::remove_file(dec_path);
}

#[test]
fn test_key_management() {
    // 1. Salt & DEK Generation
    let salt = crate::db::privacy::generate_salt();
    let dek = crate::db::privacy::generate_dek();
    assert!(salt.len() > 32);
    assert_eq!(dek.len(), 64); // 32 bytes hex -> 64 chars

    // 2. Derive MK
    let password = "my_secure_password";
    let mk = crate::db::privacy::derive_mk(password, &salt).expect("Derive MK failed");
    assert_eq!(mk.len(), 64);

    // 3. Wrap DEK
    let wrapped_dek = crate::db::privacy::wrap_dek(&dek, &mk).expect("Wrap DEK failed");
    assert_ne!(wrapped_dek, dek);

    // 4. Unwrap DEK
    let unwrapped_dek =
        crate::db::privacy::unwrap_dek(&wrapped_dek, &mk).expect("Unwrap DEK failed");
    assert_eq!(unwrapped_dek, dek);
}

#[test]
fn test_delete_temp_encrypted_file() {
    let _g = test_lock();
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("temp_del_{}.bin", Uuid::new_v4()));

    // 1. Test non-existent file
    let res =
        crate::db::privacy::delete_temp_encrypted_file(file_path.to_string_lossy().to_string());
    assert!(res.is_err());

    // 2. Test small file
    fs::write(&file_path, "short").unwrap();
    let res =
        crate::db::privacy::delete_temp_encrypted_file(file_path.to_string_lossy().to_string());
    assert!(res.is_err()); // Too small

    // 3. Test SQLite file
    let mut sqlite_header = b"SQLite format 3\0".to_vec();
    sqlite_header.extend_from_slice(&[0u8; 100]);
    fs::write(&file_path, &sqlite_header).unwrap();
    let res =
        crate::db::privacy::delete_temp_encrypted_file(file_path.to_string_lossy().to_string());
    assert!(res.is_err()); // Is SQLite

    // 4. Test valid encrypted file (mock)
    let mut valid_data = vec![0u8; 12]; // Nonce
    valid_data.extend_from_slice(b"some encrypted data");
    fs::write(&file_path, &valid_data).unwrap();
    let res =
        crate::db::privacy::delete_temp_encrypted_file(file_path.to_string_lossy().to_string());
    assert!(res.is_ok());
    assert!(!file_path.exists());
}
