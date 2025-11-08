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
    let insert_json = insert_received_data(item.clone()).expect("insert failed");
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
fn test_get_latest_data() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // initially should be null
    let initial = get_latest_data().expect("get latest failed");
    assert_eq!(initial, "null");

    let item = make_item("latest-1", "text", "latest content");
    insert_received_data(item.clone()).expect("insert latest failed");

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

    insert_received_data(a.clone()).unwrap();
    insert_received_data(b.clone()).unwrap();

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
    insert_received_data(item.clone()).expect("insert for update");

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
fn test_set_favorite_status_by_id() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let item = make_item("fav-1", "text", "to be favorited");
    insert_received_data(item.clone()).expect("insert for favorite");

    // initially not favorite
    let fetched_json = get_data_by_id(&item.id).expect("get for favorite");
    let fetched: ClipboardItem = serde_json::from_str(&fetched_json).expect("parse for favorite");
    assert!(!fetched.is_favorite);

    // set favorite
    let res1 = set_favorite_status_by_id(&item.id).expect("set favorite");
    assert_eq!(res1, "favorited");

    let fetched_json2 = get_data_by_id(&item.id).expect("get after favorite");
    let fetched2: ClipboardItem =
        serde_json::from_str(&fetched_json2).expect("parse after favorite");
    assert!(fetched2.is_favorite);

    // unset favorite
    let res2 = set_favorite_status_by_id(&item.id).expect("unset favorite");
    assert_eq!(res2, "unfavorited");

    let fetched_json3 = get_data_by_id(&item.id).expect("get after unfavorite");
    let fetched3: ClipboardItem =
        serde_json::from_str(&fetched_json3).expect("parse after unfavorite");
    assert!(!fetched3.is_favorite);
}

#[test]
fn test_search_text_content() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let item1 = make_item("search-1", "text", "hello world");
    let item2 = make_item("search-2", "text", "goodbye world");
    let item3 = make_item("search-3", "image", "/tmp/img.png");

    insert_received_data(item1.clone()).unwrap();
    insert_received_data(item2.clone()).unwrap();
    insert_received_data(item3.clone()).unwrap();

    let results_json = search_text_content("world").expect("search failed");
    let results: Vec<ClipboardItem> =
        serde_json::from_str(&results_json).expect("parse search results");

    let ids: Vec<String> = results.into_iter().map(|it| it.id).collect();
    assert!(ids.contains(&item1.id));
    assert!(ids.contains(&item2.id));
    assert!(!ids.contains(&item3.id)); // image type should not be included
}

#[test]
fn test_filter_data_by_type() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let item1 = make_item("filter-1", "text", "some text");
    let item2 = make_item("filter-2", "image", "/tmp/img.png");
    let item3 = make_item("filter-3", "text", "more text");

    insert_received_data(item1.clone()).unwrap();
    insert_received_data(item2.clone()).unwrap();
    insert_received_data(item3.clone()).unwrap();

    let results_json = filter_data_by_type("text").expect("filter failed");
    let results: Vec<ClipboardItem> =
        serde_json::from_str(&results_json).expect("parse filter results");

    let ids: Vec<String> = results.into_iter().map(|it| it.id).collect();
    assert!(ids.contains(&item1.id));
    assert!(ids.contains(&item3.id));
    assert!(!ids.contains(&item2.id)); // image type should not be included
}

#[test]
fn test_folder_functions() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 创建至少 7 个 item
    let items = vec![
        make_item("f-1", "text", "one"),
        make_item("f-2", "text", "two"),
        make_item("f-3", "image", "/tmp/img1"),
        make_item("f-4", "text", "four"),
        make_item("f-5", "image", "/tmp/img2"),
        make_item("f-6", "text", "six"),
        make_item("f-7", "image", "/tmp/img3"),
    ];
    for it in &items {
        insert_received_data(it.clone()).expect("insert failed");
    }

    // 新建两个收藏夹
    let folder_a = create_new_folder("FolderA").expect("create folder A");
    let folder_b = create_new_folder("FolderB").expect("create folder B");

    // 测试获取所有收藏夹
    let all_folders_json = get_all_folders().expect("get all folders failed");
    let all_folders: Vec<FolderItem> =
        serde_json::from_str(&all_folders_json).expect("parse all folders");
    let folder_names: Vec<String> = all_folders.iter().map(|f| f.name.clone()).collect();
    assert!(folder_names.contains(&"FolderA".to_string()));
    assert!(folder_names.contains(&"FolderB".to_string()));

    // 向 FolderA 添加 5 个 item，向 FolderB 添加 2 个
    for id in &["f-1", "f-2", "f-3", "f-4", "f-5"] {
        add_item_to_folder(&folder_a, id).expect("add to FolderA failed");
    }
    for id in &["f-6", "f-7"] {
        add_item_to_folder(&folder_b, id).expect("add to FolderB failed");
    }

    // 重复添加同一项（应被 IGNORE，不会出现重复）
    add_item_to_folder(&folder_a, "f-1").expect("duplicate add failed");

    // 验证 FolderA 的内容（应包含 f-1..f-5，不包含 f-6/f-7）
    let res_a = filter_data_by_folder("FolderA").expect("filter FolderA failed");
    let vec_a: Vec<ClipboardItem> = serde_json::from_str(&res_a).expect("parse FolderA");
    let ids_a: Vec<String> = vec_a.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_a.len(), 5);
    for id in &["f-1", "f-2", "f-3", "f-4", "f-5"] {
        assert!(ids_a.contains(&id.to_string()));
    }
    assert!(!ids_a.contains(&"f-6".to_string()));
    assert!(!ids_a.contains(&"f-7".to_string()));

    // 从 FolderA 移除 f-3 并验证
    remove_item_from_folder(&folder_a, "f-3").expect("remove from FolderA failed");
    let res_after_remove = filter_data_by_folder("FolderA").expect("filter after remove failed");
    let vec_after_remove: Vec<ClipboardItem> =
        serde_json::from_str(&res_after_remove).expect("parse after remove");
    let ids_after_remove: Vec<String> = vec_after_remove.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_after_remove.len(), 4);
    assert!(!ids_after_remove.contains(&"f-3".to_string()));

    // 重命名 FolderA 并通过新名称查询
    let renamed = rename_folder(&folder_a, "FolderA_Renamed").expect("rename failed");
    assert_eq!(renamed, "renamed");
    let res_renamed = filter_data_by_folder("FolderA_Renamed").expect("filter renamed failed");
    let vec_renamed: Vec<ClipboardItem> =
        serde_json::from_str(&res_renamed).expect("parse renamed");
    let ids_renamed: Vec<String> = vec_renamed.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_renamed.len(), 4);
    for id in &["f-1", "f-2", "f-4", "f-5"] {
        assert!(ids_renamed.contains(&id.to_string()));
    }

    // 删除 FolderB 并确认通过名称查询为空
    let deleted = delete_folder(&folder_b).expect("delete FolderB failed");
    assert_eq!(deleted, "deleted");
    let res_b = filter_data_by_folder("FolderB").expect("filter FolderB after delete failed");
    let vec_b: Vec<ClipboardItem> =
        serde_json::from_str(&res_b).expect("parse FolderB after delete");
    assert!(vec_b.is_empty());
}
