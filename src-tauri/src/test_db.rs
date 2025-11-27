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

#[test]
fn test_set_favorite_status_by_id() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let item = make_item("fav-1", "text", "to be favorited");
    insert_received_db_data(item.clone()).expect("insert for favorite");

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
fn test_search_data_comprehensive() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 准备测试数据：不同类型的数据项
    let text1 = make_item("search-1", "text", "hello world");
    let text2 = make_item("search-2", "text", "rust programming");
    let image1 = make_item("search-3", "image", "/path/to/image.png");
    let file1 = make_item("search-4", "file", "/path/to/document.pdf");

    // 插入数据并等待一小段时间以确保时间戳不同
    insert_received_db_data(text1.clone()).expect("insert text1 failed");
    std::thread::sleep(std::time::Duration::from_millis(10));
    insert_received_db_data(text2.clone()).expect("insert text2 failed");
    std::thread::sleep(std::time::Duration::from_millis(10));
    insert_received_db_data(image1.clone()).expect("insert image1 failed");
    std::thread::sleep(std::time::Duration::from_millis(10));
    insert_received_db_data(file1.clone()).expect("insert file1 failed");

    // ==================== 测试文本内容搜索 ====================

    // 搜索 "world" - 应该只匹配 text1
    let result_world = search_data("text", "world").expect("search world failed");
    let items_world: Vec<ClipboardItem> =
        serde_json::from_str(&result_world).expect("parse world results");
    let ids_world: Vec<String> = items_world.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_world.len(), 1, "expected 1 item for 'world'");
    assert!(ids_world.contains(&text1.id), "should contain text1");
    assert!(!ids_world.contains(&text2.id), "should not contain text2");

    // 搜索 "path" - 应该匹配 image1 和 file1 (它们的 content 包含 path)
    let result_path = search_data("text", "path").expect("search path failed");
    let items_path: Vec<ClipboardItem> =
        serde_json::from_str(&result_path).expect("parse path results");
    let ids_path: Vec<String> = items_path.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_path.len(), 2, "expected 2 items for 'path'");
    assert!(ids_path.contains(&image1.id), "should contain image1");
    assert!(ids_path.contains(&file1.id), "should contain file1");
    assert!(!ids_path.contains(&text1.id), "should not contain text1");
    assert!(!ids_path.contains(&text2.id), "should not contain text2");

    // 搜索不存在的关键词 - 应该返回空数组
    let result_empty = search_data("text", "nonexistent").expect("search nonexistent failed");
    let items_empty: Vec<ClipboardItem> =
        serde_json::from_str(&result_empty).expect("parse empty results");
    assert!(
        items_empty.is_empty(),
        "expected empty results for nonexistent keyword"
    );

    // ==================== 测试 OCR 搜索 ====================

    let result_ocr = search_data("ocr", "hello").expect("search ocr failed");
    let items_ocr: Vec<ClipboardItem> =
        serde_json::from_str(&result_ocr).expect("parse ocr results");
    // OCR 搜索应该通过 content 字段进行（因为实现中使用的是相同的 LIKE 查询）
    let ids_ocr: Vec<String> = items_ocr.iter().map(|it| it.id.clone()).collect();
    assert!(
        ids_ocr.contains(&text1.id),
        "ocr search should find text1 with 'hello'"
    );

    // ==================== 测试路径搜索 ====================

    let result_path_search = search_data("path", "document").expect("search path failed");
    let items_path_search: Vec<ClipboardItem> =
        serde_json::from_str(&result_path_search).expect("parse path search");
    let ids_path_search: Vec<String> = items_path_search.iter().map(|it| it.id.clone()).collect();
    assert!(
        ids_path_search.contains(&file1.id),
        "path search should find file1"
    );
    assert_eq!(
        ids_path_search.len(),
        1,
        "path search should only find 1 item"
    );

    // ==================== 测试时间戳范围搜索 ====================

    // 获取所有数据以确定时间戳范围
    let all_data_json = get_all_data().expect("get all data failed");
    let all_data: Vec<ClipboardItem> =
        serde_json::from_str(&all_data_json).expect("parse all data");

    let min_timestamp = all_data.iter().map(|it| it.timestamp).min().unwrap();
    let max_timestamp = all_data.iter().map(|it| it.timestamp).max().unwrap();
    let mid_timestamp = (min_timestamp + max_timestamp) / 2;

    // 搜索全部时间范围 - 应该返回所有 4 个项目
    let timestamp_query_all = format!("{},{}", min_timestamp - 1000, max_timestamp + 1000);
    let result_timestamp_all =
        search_data("timestamp", &timestamp_query_all).expect("search timestamp all failed");
    let items_timestamp_all: Vec<ClipboardItem> =
        serde_json::from_str(&result_timestamp_all).expect("parse timestamp all results");
    assert_eq!(
        items_timestamp_all.len(),
        4,
        "expected all 4 items in full time range"
    );

    // 搜索部分时间范围 - 应该返回部分项目
    let timestamp_query_partial = format!("{},{}", min_timestamp - 1000, mid_timestamp);
    let result_timestamp_partial = search_data("timestamp", &timestamp_query_partial)
        .expect("search timestamp partial failed");
    let items_timestamp_partial: Vec<ClipboardItem> =
        serde_json::from_str(&result_timestamp_partial).expect("parse timestamp partial results");
    assert!(
        items_timestamp_partial.len() >= 1 && items_timestamp_partial.len() <= 4,
        "expected some items in partial time range"
    );

    // 测试无效的时间戳格式 - 应该返回错误
    let result_invalid_format = search_data("timestamp", "invalid-format");
    assert!(
        result_invalid_format.is_err(),
        "expected error for invalid timestamp format"
    );
    assert!(
        result_invalid_format
            .unwrap_err()
            .contains("Invalid timestamp range format"),
        "expected specific error message"
    );

    // 测试单个时间戳（缺少逗号） - 应该返回错误
    let result_single_timestamp = search_data("timestamp", "12345");
    assert!(
        result_single_timestamp.is_err(),
        "expected error for single timestamp"
    );

    // 测试非数字时间戳 - 应该返回错误
    let result_non_numeric = search_data("timestamp", "abc,def");
    assert!(
        result_non_numeric.is_err(),
        "expected error for non-numeric timestamps"
    );
    assert!(
        result_non_numeric
            .unwrap_err()
            .contains("Invalid start timestamp"),
        "expected specific error message for non-numeric"
    );

    // ==================== 测试边界情况 ====================

    // 空搜索关键词 - 应该返回所有包含空字符串的项（实际上是所有项）
    let result_empty_query = search_data("text", "").expect("search empty query failed");
    let items_empty_query: Vec<ClipboardItem> =
        serde_json::from_str(&result_empty_query).expect("parse empty query results");
    assert_eq!(
        items_empty_query.len(),
        4,
        "empty query should match all items"
    );

    // 特殊字符搜索 - 测试 SQL 注入防护
    let result_special =
        search_data("text", "'; DROP TABLE data; --").expect("search special chars failed");
    let items_special: Vec<ClipboardItem> =
        serde_json::from_str(&result_special).expect("parse special chars results");
    assert!(
        items_special.is_empty(),
        "SQL injection attempt should return empty results"
    );

    // 区分大小写测试（SQLite LIKE 默认不区分大小写）
    let result_case = search_data("text", "HELLO").expect("search uppercase failed");
    let items_case: Vec<ClipboardItem> =
        serde_json::from_str(&result_case).expect("parse case results");
    let ids_case: Vec<String> = items_case.iter().map(|it| it.id.clone()).collect();
    assert!(
        ids_case.contains(&text1.id),
        "case-insensitive search should find 'hello'"
    );

    // 模糊匹配测试 - 部分关键词
    let result_partial = search_data("text", "prog").expect("search partial failed");
    let items_partial: Vec<ClipboardItem> =
        serde_json::from_str(&result_partial).expect("parse partial results");
    let ids_partial: Vec<String> = items_partial.iter().map(|it| it.id.clone()).collect();
    assert!(
        ids_partial.contains(&text2.id),
        "partial match should find 'programming'"
    );
}

#[test]
fn test_filter_data_by_type() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let item1 = make_item("filter-1", "text", "some text");
    let item2 = make_item("filter-2", "image", "/tmp/img.png");
    let item3 = make_item("filter-3", "text", "more text");

    insert_received_db_data(item1.clone()).unwrap();
    insert_received_db_data(item2.clone()).unwrap();
    insert_received_db_data(item3.clone()).unwrap();

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
        insert_received_db_data(it.clone()).expect("insert failed");
    }

    // 新建两个收藏夹
    let folder_a = create_new_folder("FolderA").expect("create folder A");
    let folder_b = create_new_folder("FolderB").expect("create folder B");

    // 测试获取所有收藏夹（初始 num_items 应为 0）
    let all_folders_json = get_all_folders().expect("get all folders failed");
    let all_folders: Vec<FolderItem> =
        serde_json::from_str(&all_folders_json).expect("parse all folders");
    let folder_names: Vec<String> = all_folders.iter().map(|f| f.name.clone()).collect();
    assert!(folder_names.contains(&"FolderA".to_string()));
    assert!(folder_names.contains(&"FolderB".to_string()));

    // 初始数量检查
    let fa = all_folders
        .iter()
        .find(|f| f.id == folder_a)
        .expect("folder_a missing");
    let fb = all_folders
        .iter()
        .find(|f| f.id == folder_b)
        .expect("folder_b missing");
    assert_eq!(fa.num_items, 0);
    assert_eq!(fb.num_items, 0);

    // 向 FolderA 添加 5 个 item，向 FolderB 添加 2 个
    for id in &["f-1", "f-2", "f-3", "f-4", "f-5"] {
        add_item_to_folder(&folder_a, id).expect("add to FolderA failed");
    }
    for id in &["f-6", "f-7"] {
        add_item_to_folder(&folder_b, id).expect("add to FolderB failed");
    }

    // 重复添加同一项（应被 IGNORE，不会出现重复）
    add_item_to_folder(&folder_a, "f-1").expect("duplicate add failed");

    // 额外：把 f-2 也加入 FolderB，测试单个 item 属于多个收藏夹的情况
    add_item_to_folder(&folder_b, "f-2").expect("add f-2 to FolderB failed");

    // 验证 num_items 值（FolderA 应为 5，FolderB 应为 3）
    let all_folders_json = get_all_folders().expect("get all folders after add failed");
    let all_folders: Vec<FolderItem> =
        serde_json::from_str(&all_folders_json).expect("parse all folders after add");
    let fa = all_folders
        .iter()
        .find(|f| f.id == folder_a)
        .expect("folder_a missing after add");
    let fb = all_folders
        .iter()
        .find(|f| f.id == folder_b)
        .expect("folder_b missing after add");
    assert_eq!(fa.num_items, 5, "FolderA should have 5 items");
    assert_eq!(fb.num_items, 3, "FolderB should have 3 items");

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

    // 测试 get_folders_by_item_id：f-2 应该属于 FolderA 和 FolderB，且返回的 FolderItem.num_items 要正确
    let folders_for_f2_json = get_folders_by_item_id("f-2").expect("get_folders_by_item_id failed");
    let folders_for_f2: Vec<FolderItem> =
        serde_json::from_str(&folders_for_f2_json).expect("parse folders for f-2");
    let folder_ids: Vec<String> = folders_for_f2.iter().map(|f| f.id.clone()).collect();
    assert!(folder_ids.contains(&folder_a));
    assert!(folder_ids.contains(&folder_b));
    let fa_entry = folders_for_f2
        .iter()
        .find(|f| f.id == folder_a)
        .expect("folder_a missing in f-2 list");
    let fb_entry = folders_for_f2
        .iter()
        .find(|f| f.id == folder_b)
        .expect("folder_b missing in f-2 list");
    assert_eq!(
        fa_entry.num_items, 5,
        "FolderA.num_items should be 5 in get_folders_by_item_id"
    );
    assert_eq!(
        fb_entry.num_items, 3,
        "FolderB.num_items should be 3 in get_folders_by_item_id"
    );

    // 从 FolderA 移除 f-3 并验证数量和内容
    remove_item_from_folder(&folder_a, "f-3").expect("remove from FolderA failed");

    let all_folders_json = get_all_folders().expect("get all folders after remove failed");
    let all_folders: Vec<FolderItem> =
        serde_json::from_str(&all_folders_json).expect("parse all folders after remove");
    let fa = all_folders
        .iter()
        .find(|f| f.id == folder_a)
        .expect("folder_a missing after remove");
    assert_eq!(fa.num_items, 4, "FolderA should have 4 items after removal");

    let res_after_remove = filter_data_by_folder("FolderA").expect("filter after remove failed");
    let vec_after_remove: Vec<ClipboardItem> =
        serde_json::from_str(&res_after_remove).expect("parse after remove");
    let ids_after_remove: Vec<String> = vec_after_remove.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_after_remove.len(), 4);
    assert!(!ids_after_remove.contains(&"f-3".to_string()));

    // 再次测试 get_folders_by_item_id，确保移除操作不会影响其它文件夹的计数（f-2 仍在 FolderA 和 FolderB）
    let folders_for_f2_json =
        get_folders_by_item_id("f-2").expect("get_folders_by_item_id after remove failed");
    let folders_for_f2: Vec<FolderItem> =
        serde_json::from_str(&folders_for_f2_json).expect("parse folders for f-2 after remove");
    let fa_entry = folders_for_f2
        .iter()
        .find(|f| f.id == folder_a)
        .expect("folder_a missing in f-2 list after remove");
    let fb_entry = folders_for_f2
        .iter()
        .find(|f| f.id == folder_b)
        .expect("folder_b missing in f-2 list after remove");
    assert_eq!(
        fa_entry.num_items, 4,
        "FolderA.num_items should be 4 after removal"
    );
    assert_eq!(
        fb_entry.num_items, 3,
        "FolderB.num_items should remain 3 after removal of unrelated item"
    );

    // 重命名 FolderA 并通过新名称查询，数量应保持不变
    let renamed = rename_folder(&folder_a, "FolderA_Renamed").expect("rename failed");
    assert_eq!(renamed, "renamed");

    let all_folders_json = get_all_folders().expect("get all folders after rename failed");
    let all_folders: Vec<FolderItem> =
        serde_json::from_str(&all_folders_json).expect("parse all folders after rename");
    let fa = all_folders
        .iter()
        .find(|f| f.id == folder_a)
        .expect("folder_a missing after rename");
    assert_eq!(fa.name, "FolderA_Renamed");
    assert_eq!(
        fa.num_items, 4,
        "FolderA should still have 4 items after rename"
    );

    let res_renamed = filter_data_by_folder("FolderA_Renamed").expect("filter renamed failed");
    let vec_renamed: Vec<ClipboardItem> =
        serde_json::from_str(&res_renamed).expect("parse renamed");
    let ids_renamed: Vec<String> = vec_renamed.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_renamed.len(), 4);
    for id in &["f-1", "f-2", "f-4", "f-5"] {
        assert!(ids_renamed.contains(&id.to_string()));
    }

    // 删除 FolderB 并确认通过名称查询为空，且在所有收藏夹列表中不存在
    let deleted = delete_folder(&folder_b).expect("delete FolderB failed");
    assert_eq!(deleted, "deleted");
    let res_b = filter_data_by_folder("FolderB").expect("filter FolderB after delete failed");
    let vec_b: Vec<ClipboardItem> =
        serde_json::from_str(&res_b).expect("parse FolderB after delete");
    assert!(vec_b.is_empty());

    let all_folders_json = get_all_folders().expect("get all folders after delete folder_b failed");
    let all_folders: Vec<FolderItem> =
        serde_json::from_str(&all_folders_json).expect("parse all folders after delete folder_b");
    assert!(
        all_folders.iter().find(|f| f.id == folder_b).is_none(),
        "FolderB should be removed from folders list"
    );
}

#[test]
fn test_filter_data_by_favorite() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 插入 5 个 item
    let items = vec![
        make_item("fav1", "text", "one"),
        make_item("fav2", "text", "two"),
        make_item("fav3", "image", "/img/1"),
        make_item("fav4", "text", "four"),
        make_item("fav5", "image", "/img/2"),
    ];
    for it in &items {
        insert_received_db_data(it.clone()).expect("insert failed");
    }

    // 收藏 fav1 和 fav3
    let r1 = set_favorite_status_by_id("fav1").expect("favorite fav1 failed");
    assert_eq!(r1, "favorited");
    let r2 = set_favorite_status_by_id("fav3").expect("favorite fav3 failed");
    assert_eq!(r2, "favorited");

    // filter true -> 应只包含 fav1 与 fav3
    let res_true = filter_data_by_favorite(true).expect("filter favorite true failed");
    let vec_true: Vec<ClipboardItem> = serde_json::from_str(&res_true).expect("parse fav true");
    let ids_true: Vec<String> = vec_true.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_true.len(), 2);
    assert!(ids_true.contains(&"fav1".to_string()));
    assert!(ids_true.contains(&"fav3".to_string()));
    // 不应包含未收藏项
    for id in &["fav2", "fav4", "fav5"] {
        assert!(!ids_true.contains(&id.to_string()));
    }

    // filter false -> 应包含剩余未收藏的 3 项
    let res_false = filter_data_by_favorite(false).expect("filter favorite false failed");
    let vec_false: Vec<ClipboardItem> = serde_json::from_str(&res_false).expect("parse fav false");
    let ids_false: Vec<String> = vec_false.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_false.len(), 3);
    for id in &["fav2", "fav4", "fav5"] {
        assert!(ids_false.contains(&id.to_string()));
    }
    // 不应包含已收藏项
    assert!(!ids_false.contains(&"fav1".to_string()));
    assert!(!ids_false.contains(&"fav3".to_string()));
}

#[test]
fn test_search_data_by_ocr_text() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 准备三个 item：两个 image（会插入 OCR），一个 text（不应被 OCR 搜索命中）
    let a = make_item("ocr-1", "image", "/tmp/img1.png");
    let b = make_item("ocr-2", "image", "/tmp/img2.png");
    let c = make_item("ocr-3", "text", "plain text not ocr");

    insert_received_db_data(a.clone()).expect("insert a");
    insert_received_db_data(b.clone()).expect("insert b");
    insert_received_db_data(c.clone()).expect("insert c");

    // 插入 OCR 文本：a 包含 "． 今天 你 好"，b 包含 "今天天气很好"
    insert_ocr_text(&a.id, "． 今天 你 好").expect("insert ocr for a");
    insert_ocr_text(&b.id, "今天天气很好").expect("insert ocr for b");

    // 额外校验：直接通过 get_ocr_text_by_item_id 读取并比对
    let got_a = get_ocr_text_by_item_id(&a.id).expect("get ocr for a failed");
    assert_eq!(got_a, "． 今天 你 好");
    let got_b = get_ocr_text_by_item_id(&b.id).expect("get ocr for b failed");
    assert_eq!(got_b, "今天天气很好");
    // c 应该无 OCR 文本
    let got_c = get_ocr_text_by_item_id(&c.id).expect("get ocr for c failed");
    assert_eq!(
        got_c, "",
        "text item without OCR should return empty string"
    );

    // 搜索只匹配 "天气" -> 仅应返回 b
    let res_json = search_data_by_ocr_text("天气").expect("search ocr failed");
    let vec: Vec<ClipboardItem> = serde_json::from_str(&res_json).expect("parse results");
    let ids: Vec<String> = vec.iter().map(|it| it.id.clone()).collect();
    assert!(ids.contains(&b.id), "expected b to be matched by '天气'");
    assert!(!ids.contains(&a.id), "a should not match '天气'");
    assert!(
        !ids.contains(&c.id),
        "text item c should not be matched (no extended ocr)"
    );

    // 搜索匹配 "今天" -> 应返回 a 和 b（模糊匹配）
    let res_json2 = search_data_by_ocr_text("今天").expect("search ocr failed for 今天");
    let vec2: Vec<ClipboardItem> = serde_json::from_str(&res_json2).expect("parse results 2");
    let ids2: Vec<String> = vec2.iter().map(|it| it.id.clone()).collect();
    assert!(ids2.contains(&a.id), "expected a to be matched by '今天'");
    assert!(ids2.contains(&b.id), "expected b to be matched by '今天'");
    assert!(!ids2.contains(&c.id), "text item c should not be matched");

    // 搜索一个不存在的词，应该返回空数组
    let res_json3 = search_data_by_ocr_text("不存在的关键词").expect("search unknown failed");
    let vec3: Vec<ClipboardItem> = serde_json::from_str(&res_json3).expect("parse results 3");
    assert!(vec3.is_empty(), "expected no results for unknown keyword");
}

#[test]
fn test_insert_get_icon_data() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let item = make_item("icon-1", "file", "/tmp/somefile.bin");
    insert_received_db_data(item.clone()).expect("insert item for icon");

    // 插入 icon_data（这里用简单 base64 字符串作为示例）
    let sample_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAUA";
    insert_icon_data(&item.id, sample_b64).expect("insert_icon_data failed");

    // 通过 get_icon_data_by_item_id 读取并断言一致
    let got_icon = get_icon_data_by_item_id(&item.id).expect("get_icon_data_by_item_id failed");
    assert_eq!(got_icon, sample_b64);

    // 对不存在的 id 应返回空字符串
    let got_none = get_icon_data_by_item_id("non-existent-id").expect("get icon none failed");
    assert_eq!(got_none, "");
}
