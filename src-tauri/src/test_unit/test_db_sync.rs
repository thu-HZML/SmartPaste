use super::*;
use crate::db::sync::sync_cloud_data;
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
