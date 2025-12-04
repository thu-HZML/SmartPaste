use super::*;
use serde_json;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    // 如果 mutex 被 poison，恢复并返回被污染时的 guard（避免测试间直接失败）
    TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn set_test_db_path() {
    // 在临时目录下使用独立数据库文件，避免污染真实数据
    let mut p = std::env::temp_dir();
    p.push("smartpaste_test.db");
    // 覆盖全局 OnceLock（只会在第一次调用设置）
    set_db_path(p);
    // 确保清理全局 last_inserted，避免跨测试遗留状态导致断言失败
    let _ = crate::clipboard::take_last_inserted();
}

fn clear_db_file() {
    let p: PathBuf = get_db_path();
    let _ = fs::remove_file(p);
}

fn make_item(id: &str, item_type: &str, content: &str) -> ClipboardItem {
    ClipboardItem {
        id: id.to_string(),
        item_type: item_type.to_string(),
        content: content.to_string(),
        size: Some(content.len() as u64),
        is_favorite: false,
        notes: "".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    }
}

#[test]
fn test_clipboard_item_to_json_roundtrip() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let item = make_item("json-ut-1", "text", "roundtrip");
    let json = clipboard_item_to_json(item.clone()).expect("serialize failed");
    let parsed: ClipboardItem = serde_json::from_str(&json).expect("deserialize failed");
    assert_eq!(parsed.id, item.id);
    assert_eq!(parsed.content, item.content);
    assert_eq!(parsed.item_type, item.item_type);
}

#[test]
fn test_insert_get_delete() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let item = make_item("ut-1", "text", "hello insert");
    // insert
    let insert_json = insert_received_db_data(item.clone()).expect("insert failed");
    let inserted: ClipboardItem = serde_json::from_str(&insert_json).expect("parse inserted");
    assert_eq!(inserted.id, item.id);
    assert_eq!(inserted.content, item.content);

    // get by id
    let json = get_data_by_id(&item.id).expect("get failed");
    assert_ne!(json, "null");
    let fetched: ClipboardItem = serde_json::from_str(&json).expect("parse fetched");
    assert_eq!(fetched.id, item.id);
    assert_eq!(fetched.content, item.content);

    // delete by id
    let rows = delete_data_by_id(&item.id).expect("delete failed");
    assert!(rows >= 1);

    // ensure deleted
    let json2 = get_data_by_id(&item.id).expect("get after delete");
    assert_eq!(json2, "null");
}

#[test]
fn test_not_text_data_insert_and_delete() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // insert image, file, folder types
    let img_item = make_item("img-1", "image", ".\\files\\image.png");
    let file_item = make_item("file-1", "file", ".\\files\\document.pdf");
    let folder_item = make_item("folder-1", "folder", ".\\files\\myfolder");

    // insert
    insert_received_db_data(img_item.clone()).expect("insert image failed");
    insert_received_db_data(file_item.clone()).expect("insert file failed");
    insert_received_db_data(folder_item.clone()).expect("insert folder failed");

    // get and verify
    let img_json = get_data_by_id(&img_item.id).expect("get image failed");
    let fetched_img: ClipboardItem = serde_json::from_str(&img_json).expect("parse image");
    assert_eq!(fetched_img.id, img_item.id);

    let file_json = get_data_by_id(&file_item.id).expect("get file failed");
    let fetched_file: ClipboardItem = serde_json::from_str(&file_json).expect("parse file");
    assert_eq!(fetched_file.id, file_item.id);

    let folder_json = get_data_by_id(&folder_item.id).expect("get folder failed");
    let fetched_folder: ClipboardItem = serde_json::from_str(&folder_json).expect("parse folder");
    assert_eq!(fetched_folder.id, folder_item.id);

    // delete
    delete_data_by_id(&img_item.id).expect("delete image failed");
    delete_data_by_id(&file_item.id).expect("delete file failed");
    delete_data_by_id(&folder_item.id).expect("delete folder failed");

    // ensure deleted
    let img_json2 = get_data_by_id(&img_item.id).expect("get image after delete");
    assert_eq!(img_json2, "null");
    let file_json2 = get_data_by_id(&file_item.id).expect("get file after delete");
    assert_eq!(file_json2, "null");
    let folder_json2 = get_data_by_id(&folder_item.id).expect("get folder after delete");
    assert_eq!(folder_json2, "null");
}

#[test]
fn test_get_latest_data() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // initially should be null
    let initial = get_latest_data().expect("get latest failed");
    assert_eq!(initial, "null");

    let item = make_item("latest-1", "text", "latest content");
    insert_received_db_data(item.clone()).expect("insert latest failed");

    let latest_json = get_latest_data().expect("get latest after insert failed");
    let latest: ClipboardItem = serde_json::from_str(&latest_json).expect("parse latest");
    assert_eq!(latest.id, item.id);
    assert_eq!(latest.content, item.content);
}

#[test]
fn test_get_all_data() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let a = make_item("all-1", "text", "one");
    let b = make_item("all-2", "image", "/tmp/img.png");

    insert_received_db_data(a.clone()).unwrap();
    insert_received_db_data(b.clone()).unwrap();

    let all_json = get_all_data().expect("get_all failed");
    let vec: Vec<ClipboardItem> = serde_json::from_str(&all_json).expect("parse array");
    let ids: Vec<String> = vec.into_iter().map(|it| it.id).collect();
    assert!(ids.contains(&a.id));
    assert!(ids.contains(&b.id));
}

#[test]
fn update_data() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();
    let item = make_item("update-1", "text", "original content");
    insert_received_db_data(item.clone()).expect("insert for update");

    // update content: 函数返回更新后的记录 JSON，解析后断言 content 字段
    let new_content = "updated content";
    let updated_json =
        update_data_content_by_id(&item.id, new_content).expect("update content failed");
    let updated_item: ClipboardItem = serde_json::from_str(&updated_json).expect("parse updated");
    assert_eq!(updated_item.content, new_content);

    // update notes: 同理解析并断言 notes 字段
    let new_notes = "these are notes";
    let updated_notes_json = add_notes_by_id(&item.id, new_notes).expect("update notes failed");
    let updated_item2: ClipboardItem =
        serde_json::from_str(&updated_notes_json).expect("parse notes updated");
    assert_eq!(updated_item2.notes, new_notes);
}

#[test]
fn test_delete_data() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();
    // 插入 4 个 item
    let a = make_item("del-1", "text", "one");
    let b = make_item("del-2", "text", "two");
    let c = make_item("del-3", "text", "three");
    let d = make_item("del-4", "text", "four");

    insert_received_db_data(a.clone()).expect("insert a");
    insert_received_db_data(b.clone()).expect("insert b");
    insert_received_db_data(c.clone()).expect("insert c");
    insert_received_db_data(d.clone()).expect("insert d");

    // 按 id 删除 del-1
    let rows = delete_data_by_id(&a.id).expect("delete by id failed");
    assert!(rows >= 1, "expected >=1 row deleted for delete_data_by_id");

    let got = get_data_by_id(&a.id).expect("get after delete");
    assert_eq!(got, "null", "deleted item should return null");

    // 使用 delete_data（传入 ClipboardItem）删除 del-2
    let rows2 = delete_data(b.clone()).expect("delete_data failed");
    assert!(rows2 >= 1, "expected >=1 row deleted for delete_data");

    let got2 = get_data_by_id(&b.id).expect("get after delete b");
    assert_eq!(got2, "null", "deleted item b should return null");

    // 收藏 c（使其不会被删除）
    let _ = set_favorite_status_by_id(&c.id).expect("set favorite for c");

    // 删除所有非收藏 item
    let rows3 = delete_unfavorited_data().expect("delete_non_favorite_data failed");
    assert!(
        rows3 >= 1,
        "expected >=1 row deleted for delete_non_favorite_data"
    );
    let got3 = get_data_by_id(&d.id).expect("get after delete d");
    assert_eq!(got3, "null", "deleted item d should return null");
    let got4 = get_data_by_id(&c.id).expect("get favorite c after delete non-fav");
    assert_ne!(got4, "null", "favorite item c should not be deleted");

    // 最后删除所有数据，确保数据库为空
    let rows4 = delete_all_data().expect("delete_all_data failed");
    assert!(rows4 >= 1, "expected >=1 row deleted for delete_all_data");
    let all_after = get_all_data().expect("get_all after delete_all");
    let vec_after: Vec<ClipboardItem> = serde_json::from_str(&all_after).expect("parse all after");
    assert!(
        vec_after.is_empty(),
        "database should be empty after delete_all"
    );
}

