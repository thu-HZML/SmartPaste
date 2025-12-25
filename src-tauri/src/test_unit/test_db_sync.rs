use super::*;
use crate::db::sync::sync_cloud_data;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use rand::Rng;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

// 获取测试锁，防止并发测试导致数据库路径冲突
fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    crate::db::TEST_RUN_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

// 设置测试数据库路径
fn set_test_db_path() -> PathBuf {
    let mut p = std::env::temp_dir();
    let filename = format!("smartpaste_test_sync_{}.db", Uuid::new_v4());
    p.push(filename);
    set_db_path(p.clone());
    p
}

// 清理数据库文件
fn clear_db_file(p: &PathBuf) {
    if p.exists() {
        let _ = fs::remove_file(p);
    }
}

#[test]
fn test_sync_cloud_data_basic() {
    let _guard = test_lock();
    let db_path = set_test_db_path();

    // 确保数据库初始化
    let _ = init_db(&db_path);

    // 构造测试 JSON 数据
    let json_data = r#"{
        "data": [
            {
                "id": "item1",
                "item_type": "text",
                "content": "content1",
                "size": 8,
                "is_favorite": false,
                "notes": "note1",
                "timestamp": 1001
            },
            {
                "id": "item2",
                "item_type": "image",
                "content": "path/to/image",
                "size": 1024,
                "is_favorite": true,
                "notes": "note2",
                "timestamp": 1002
            }
        ],
        "folders": [
            {
                "id": "folder1",
                "name": "Folder 1",
                "num_items": 1
            }
        ],
        "folder_items": [
            {
                "folder_id": "folder1",
                "item_id": "item2"
            }
        ],
        "extended_data": [
            {
                "item_id": "item2",
                "ocr_text": "ocr result",
                "icon_data": "base64icon"
            }
        ]
    }"#;

    // 执行同步
    let result = sync_cloud_data(json_data);
    assert!(result.is_ok(), "Sync should succeed: {:?}", result.err());

    // 验证数据库内容
    let conn = Connection::open(&db_path).unwrap();

    // 验证 data 表
    let count: i64 = conn
        .query_row("SELECT count(*) FROM data", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 2, "Should have 2 items in data table");

    let content: String = conn
        .query_row("SELECT content FROM data WHERE id = 'item1'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(content, "content1");

    // 验证 folders 表
    let folder_name: String = conn
        .query_row("SELECT name FROM folders WHERE id = 'folder1'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(folder_name, "Folder 1");

    // 验证 folder_items 表
    let count_fi: i64 = conn
        .query_row(
            "SELECT count(*) FROM folder_items WHERE folder_id = 'folder1' AND item_id = 'item2'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count_fi, 1, "Relation should exist");

    // 验证 extended_data 表
    let ocr_text: String = conn
        .query_row(
            "SELECT ocr_text FROM extended_data WHERE item_id = 'item2'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(ocr_text, "ocr result");

    clear_db_file(&db_path);
}

#[test]
fn test_sync_cloud_data_ignore_existing() {
    let _guard = test_lock();
    let db_path = set_test_db_path();
    let _ = init_db(&db_path);
    let conn = Connection::open(&db_path).unwrap();

    // 预先插入一条数据
    conn.execute(
        "INSERT INTO data (id, item_type, content, size, is_favorite, notes, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params!["item1", "text", "original content", 10, 0, "", 1000],
    ).unwrap();

    // 构造包含相同 ID 但内容不同的 JSON 数据
    let json_data = r#"{
        "data": [
            {
                "id": "item1",
                "item_type": "text",
                "content": "new content",
                "size": 11,
                "is_favorite": false,
                "notes": "new note",
                "timestamp": 2000
            },
            {
                "id": "item3",
                "item_type": "text",
                "content": "content3",
                "size": 8,
                "is_favorite": false,
                "notes": "",
                "timestamp": 1003
            }
        ],
        "folders": [],
        "folder_items": [],
        "extended_data": []
    }"#;

    // 执行同步
    let result = sync_cloud_data(json_data);
    assert!(result.is_ok());

    // 验证 item1 应该保持原样 (INSERT OR IGNORE)
    let content: String = conn
        .query_row("SELECT content FROM data WHERE id = 'item1'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(
        content, "original content",
        "Existing item should not be overwritten"
    );

    // 验证 item3 应该被插入
    let content3: String = conn
        .query_row("SELECT content FROM data WHERE id = 'item3'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(content3, "content3", "New item should be inserted");

    clear_db_file(&db_path);
}

#[test]
fn test_sync_data_structs_coverage() {
    use crate::clipboard::{ClipboardItem, FolderItem};
    use crate::db::sync::{ExtendedData, FolderItemRelation, SyncData};

    let sync_data = SyncData {
        data: vec![ClipboardItem {
            id: "id".to_string(),
            item_type: "text".to_string(),
            content: "content".to_string(),
            size: Some(10),
            is_favorite: false,
            notes: "notes".to_string(),
            timestamp: 123,
        }],
        folders: vec![FolderItem {
            id: "fid".to_string(),
            name: "fname".to_string(),
            num_items: 0,
        }],
        folder_items: vec![FolderItemRelation {
            folder_id: "fid".to_string(),
            item_id: "id".to_string(),
        }],
        extended_data: vec![ExtendedData {
            item_id: "id".to_string(),
            ocr_text: Some("ocr".to_string()),
            icon_data: None,
        }],
    };

    // Test Debug
    let debug_str = format!("{:?}", sync_data);
    assert!(debug_str.contains("SyncData"));

    // Test Serialize
    let json = serde_json::to_string(&sync_data).unwrap();
    assert!(json.contains("id"));
}

#[test]
fn test_sync_conflict_handling() {
    let _guard = test_lock();
    let db_path = set_test_db_path();
    let _ = init_db(&db_path);

    // 1. Insert local data
    let conn = Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO data (id, item_type, content, size, is_favorite, notes, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params!["conflict-id", "text", "local content", 10, 0, "", 1000],
    ).unwrap();

    // 2. Sync data with same ID but different content
    let json_data = r#"{
        "data": [
            {
                "id": "conflict-id",
                "item_type": "text",
                "content": "cloud content",
                "size": 20,
                "is_favorite": false,
                "notes": "cloud note",
                "timestamp": 2000
            }
        ],
        "folders": [],
        "folder_items": [],
        "extended_data": []
    }"#;

    let result = sync_cloud_data(json_data);
    assert!(result.is_ok());

    // 3. Verify local data is preserved (INSERT OR IGNORE)
    let content: String = conn
        .query_row(
            "SELECT content FROM data WHERE id = 'conflict-id'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        content, "local content",
        "Local data should be preserved on conflict"
    );

    clear_db_file(&db_path);
}

#[test]
fn test_sync_db_errors() {
    let _guard = test_lock();

    // 1. Test with invalid DB path (directory instead of file)
    let temp_dir = std::env::temp_dir();
    let invalid_db_path = temp_dir.join(format!("invalid_db_{}", Uuid::new_v4()));
    std::fs::create_dir(&invalid_db_path).unwrap();

    set_db_path(invalid_db_path.clone());

    let json_data = r#"{ "data": [], "folders": [], "folder_items": [], "extended_data": [] }"#;

    // This should fail at init_db or Connection::open
    let result = sync_cloud_data(json_data);
    assert!(result.is_err());

    std::fs::remove_dir(invalid_db_path).ok();
}

#[test]
fn test_sync_invalid_json() {
    let _guard = test_lock();
    // No need to setup DB for this test as it fails before DB access

    let json_data = "{ invalid json }";
    let result = sync_cloud_data(json_data);
    assert!(result.is_err());
    assert!(result.err().unwrap().contains("JSON 解析失败"));
}

#[test]
fn test_sync_extended_data_optional_fields() {
    let _guard = test_lock();
    let db_path = set_test_db_path();
    let _ = init_db(&db_path);

    let json_data = r#"{
        "data": [
            {
                "id": "item_opt",
                "item_type": "text",
                "content": "content",
                "size": 10,
                "is_favorite": false,
                "notes": "",
                "timestamp": 1000
            }
        ],
        "folders": [],
        "folder_items": [],
        "extended_data": [
            {
                "item_id": "item_opt",
                "ocr_text": null,
                "icon_data": null
            }
        ]
    }"#;

    let result = sync_cloud_data(json_data);
    assert!(result.is_ok());

    let conn = Connection::open(&db_path).unwrap();
    let (ocr, icon): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT ocr_text, icon_data FROM extended_data WHERE item_id = 'item_opt'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert!(ocr.is_none());
    assert!(icon.is_none());

    clear_db_file(&db_path);
}

#[test]
fn test_sync_encrypted_cloud_data() {
    let _guard = test_lock();
    let db_path = set_test_db_path();
    let _ = init_db(&db_path);

    // 1. Generate DEK
    let dek_hex = crate::db::privacy::generate_dek();

    // 2. Prepare encrypted data
    // We need to manually encrypt some fields to simulate cloud data
    let plain_content = "Secret Content";
    let plain_note = "Secret Note";

    // Helper to encrypt string
    let encrypt_str = |s: &str| -> String {
        let mut iv = [0u8; 12];
        rand::rng().fill(&mut iv);
        let nonce = Nonce::from_slice(&iv);

        let key_bytes = hex::decode(&dek_hex).unwrap();
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).unwrap();

        let ciphertext = cipher.encrypt(nonce, s.as_bytes()).unwrap();

        let iv_b64 = base64::engine::general_purpose::STANDARD.encode(iv);
        let ct_b64 = base64::engine::general_purpose::STANDARD.encode(ciphertext);
        format!("{}:{}", iv_b64, ct_b64)
    };

    let enc_content = encrypt_str(plain_content);
    let enc_note = encrypt_str(plain_note);

    let json_data = format!(
        r#"{{
        "data": [
            {{
                "id": "enc_item_1",
                "item_type": "text",
                "content": "{}",
                "size": 100,
                "is_favorite": true,
                "notes": "{}",
                "timestamp": 1234567890
            }}
        ],
        "folders": [],
        "folder_items": [],
        "extended_data": []
    }}"#,
        enc_content, enc_note
    );

    // 3. Sync with decryption
    let result = crate::db::sync::sync_encrypted_cloud_data(&json_data, dek_hex.clone());
    assert!(result.is_ok(), "Encrypted sync failed: {:?}", result.err());

    // 4. Verify decrypted data in DB
    let conn = Connection::open(&db_path).unwrap();
    let (content, notes): (String, String) = conn
        .query_row(
            "SELECT content, notes FROM data WHERE id = 'enc_item_1'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    assert_eq!(content, plain_content);
    assert_eq!(notes, plain_note);

    clear_db_file(&db_path);
}

#[test]
fn test_sync_schema_errors() {
    let _guard = test_lock();

    // Case 1: Broken data table
    let db_path = set_test_db_path();
    let conn = Connection::open(&db_path).unwrap();
    // Create table with missing columns to cause INSERT failure
    conn.execute("CREATE TABLE data (id TEXT PRIMARY KEY)", [])
        .unwrap();
    conn.close().unwrap();
    let json_data = r#"{ "data": [{"id": "1", "item_type": "text", "content": "c", "size": 0, "is_favorite": false, "notes": "", "timestamp": 1}], "folders": [], "folder_items": [], "extended_data": [] }"#;
    assert!(sync_cloud_data(json_data).is_err());
    clear_db_file(&db_path);

    // Case 2: Broken folders table
    let db_path = set_test_db_path();
    let _ = init_db(&db_path); // Create correct tables first
    let conn = Connection::open(&db_path).unwrap();
    conn.execute("DROP TABLE folders", []).unwrap();
    conn.execute("CREATE TABLE folders (id TEXT PRIMARY KEY)", [])
        .unwrap(); // Wrong schema
    conn.close().unwrap();
    let json_data = r#"{ "data": [], "folders": [{"id": "f1", "name": "n", "num_items": 0}], "folder_items": [], "extended_data": [] }"#;
    assert!(sync_cloud_data(json_data).is_err());
    clear_db_file(&db_path);

    // Case 3: Broken folder_items table
    let db_path = set_test_db_path();
    let _ = init_db(&db_path);
    let conn = Connection::open(&db_path).unwrap();
    conn.execute("DROP TABLE folder_items", []).unwrap();
    conn.execute("CREATE TABLE folder_items (id TEXT PRIMARY KEY)", [])
        .unwrap(); // Wrong schema
    conn.close().unwrap();
    let json_data = r#"{ "data": [], "folders": [], "folder_items": [{"folder_id": "f1", "item_id": "i1"}], "extended_data": [] }"#;
    assert!(sync_cloud_data(json_data).is_err());
    clear_db_file(&db_path);

    // Case 4: Broken extended_data table
    let db_path = set_test_db_path();
    let _ = init_db(&db_path);
    let conn = Connection::open(&db_path).unwrap();
    conn.execute("DROP TABLE extended_data", []).unwrap();
    conn.execute("CREATE TABLE extended_data (id TEXT PRIMARY KEY)", [])
        .unwrap(); // Wrong schema
    conn.close().unwrap();
    let json_data = r#"{ "data": [], "folders": [], "folder_items": [], "extended_data": [{"item_id": "i1", "ocr_text": null, "icon_data": null}] }"#;
    assert!(sync_cloud_data(json_data).is_err());
    clear_db_file(&db_path);
}

#[test]
fn test_sync_init_db_error() {
    let _guard = test_lock();
    let mut p = std::env::temp_dir();
    let filename = format!("smartpaste_test_sync_err_{}", Uuid::new_v4());
    p.push(&filename);

    // Create a directory at the path where DB should be
    std::fs::create_dir(&p).unwrap();
    set_db_path(p.clone());

    let json_data = r#"{ "data": [], "folders": [], "folder_items": [], "extended_data": [] }"#;
    let result = sync_cloud_data(json_data);
    assert!(result.is_err());

    std::fs::remove_dir(&p).unwrap();
}

#[test]
fn test_sync_encrypted_decryption_failures() {
    let _guard = test_lock();
    let db_path = set_test_db_path();
    let _ = init_db(&db_path);
    let dek_hex = crate::db::privacy::generate_dek();

    // 1. Invalid format (no colon)
    // 2. Invalid Base64 IV
    // 3. Invalid Base64 Ciphertext
    // 4. Decryption failure (wrong key/tag)

    let json_data = r#"{
        "data": [
            { "id": "fail1", "item_type": "text", "content": "no_colon", "size": 0, "is_favorite": false, "notes": "", "timestamp": 1 },
            { "id": "fail2", "item_type": "text", "content": "!!!:validbase64", "size": 0, "is_favorite": false, "notes": "", "timestamp": 2 },
            { "id": "fail3", "item_type": "text", "content": "validbase64:!!!", "size": 0, "is_favorite": false, "notes": "", "timestamp": 3 },
            { "id": "fail4", "item_type": "text", "content": "YWJjZGVmZ2hpamts:YWJjZGVmZ2hpamts", "size": 0, "is_favorite": false, "notes": "", "timestamp": 4 }
        ],
        "folders": [], "folder_items": [], "extended_data": []
    }"#;

    let result = crate::db::sync::sync_encrypted_cloud_data(json_data, dek_hex);
    assert!(result.is_ok()); // Should succeed but store original strings

    let conn = Connection::open(&db_path).unwrap();

    let c1: String = conn
        .query_row("SELECT content FROM data WHERE id='fail1'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(c1, "no_colon");

    let c2: String = conn
        .query_row("SELECT content FROM data WHERE id='fail2'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(c2, "!!!:validbase64");

    let c3: String = conn
        .query_row("SELECT content FROM data WHERE id='fail3'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(c3, "validbase64:!!!");

    let c4: String = conn
        .query_row("SELECT content FROM data WHERE id='fail4'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(c4, "YWJjZGVmZ2hpamts:YWJjZGVmZ2hpamts");

    clear_db_file(&db_path);
}

#[test]
fn test_sync_encrypted_invalid_dek() {
    let _guard = test_lock();
    let db_path = set_test_db_path();
    let _ = init_db(&db_path);

    let json_data = r#"{ "data": [], "folders": [], "folder_items": [], "extended_data": [] }"#;

    // Invalid hex
    let res = crate::db::sync::sync_encrypted_cloud_data(json_data, "invalid_hex".to_string());
    assert!(res.is_err());
    assert!(res.err().unwrap().contains("Invalid DEK hex"));

    // Invalid length
    let res = crate::db::sync::sync_encrypted_cloud_data(json_data, "123456".to_string());
    assert!(res.is_err());
    assert!(res.err().unwrap().contains("DEK must be 32 bytes"));

    clear_db_file(&db_path);
}
