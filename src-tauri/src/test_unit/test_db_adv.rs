/// 单元测试数据库高级操作
/// 此文件提供高级功能点测试，包括筛选、搜索、收藏状态切换等
/// 测试使用临时数据库文件，避免污染真实数据
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
fn test_filter_data_by_type_comprehensive() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 准备多种类型的数据项
    let text1 = make_item("type-text-1", "text", "hello text");
    let text2 = make_item("type-text-2", "text", "another text");
    let image1 = make_item("type-img-1", "image", "/tmp/img1.png");
    let image2 = make_item("type-img-2", "image", "/tmp/img2.jpg");
    let folder1 = make_item("type-folder-1", "folder", "/path/to/folder1");
    let folder2 = make_item("type-folder-2", "folder", "/path/to/folder2");
    let file1 = make_item("type-file-1", "file", "/path/to/file1.txt");
    let file2 = make_item("type-file-2", "file", "/path/to/file2.doc");

    // 插入所有数据项
    for item in &[
        &text1, &text2, &image1, &image2, &folder1, &folder2, &file1, &file2,
    ] {
        insert_received_db_data((*item).clone()).expect("insert failed");
    }

    // 测试筛选 text 类型
    let text_json = filter_data_by_type("text").expect("filter text failed");
    let text_results: Vec<ClipboardItem> = serde_json::from_str(&text_json).expect("parse text");
    let text_ids: Vec<String> = text_results.iter().map(|it| it.id.clone()).collect();
    assert_eq!(text_ids.len(), 2, "expected 2 text items");
    assert!(text_ids.contains(&text1.id));
    assert!(text_ids.contains(&text2.id));
    assert!(!text_ids.contains(&image1.id), "should not contain image");
    assert!(!text_ids.contains(&folder1.id), "should not contain folder");
    assert!(!text_ids.contains(&file1.id), "should not contain file");

    // 测试筛选 image 类型
    let image_json = filter_data_by_type("image").expect("filter image failed");
    let image_results: Vec<ClipboardItem> = serde_json::from_str(&image_json).expect("parse image");
    let image_ids: Vec<String> = image_results.iter().map(|it| it.id.clone()).collect();
    assert_eq!(image_ids.len(), 2, "expected 2 image items");
    assert!(image_ids.contains(&image1.id));
    assert!(image_ids.contains(&image2.id));
    assert!(!image_ids.contains(&text1.id), "should not contain text");
    assert!(
        !image_ids.contains(&folder1.id),
        "should not contain folder"
    );
    assert!(!image_ids.contains(&file1.id), "should not contain file");

    // 测试筛选 folder 类型（应同时返回 folder 和 file）
    let folder_json = filter_data_by_type("folder").expect("filter folder failed");
    let folder_results: Vec<ClipboardItem> =
        serde_json::from_str(&folder_json).expect("parse folder");
    let folder_ids: Vec<String> = folder_results.iter().map(|it| it.id.clone()).collect();
    assert_eq!(
        folder_ids.len(),
        4,
        "expected 4 items (2 folders + 2 files)"
    );
    assert!(folder_ids.contains(&folder1.id));
    assert!(folder_ids.contains(&folder2.id));
    assert!(
        folder_ids.contains(&file1.id),
        "folder filter should include files"
    );
    assert!(
        folder_ids.contains(&file2.id),
        "folder filter should include files"
    );
    assert!(!folder_ids.contains(&text1.id), "should not contain text");
    assert!(!folder_ids.contains(&image1.id), "should not contain image");

    // 测试筛选 file 类型（应同时返回 file 和 folder）
    let file_json = filter_data_by_type("file").expect("filter file failed");
    let file_results: Vec<ClipboardItem> = serde_json::from_str(&file_json).expect("parse file");
    let file_ids: Vec<String> = file_results.iter().map(|it| it.id.clone()).collect();
    assert_eq!(file_ids.len(), 4, "expected 4 items (2 files + 2 folders)");
    assert!(file_ids.contains(&file1.id));
    assert!(file_ids.contains(&file2.id));
    assert!(
        file_ids.contains(&folder1.id),
        "file filter should include folders"
    );
    assert!(
        file_ids.contains(&folder2.id),
        "file filter should include folders"
    );
    assert!(!file_ids.contains(&text1.id), "should not contain text");
    assert!(!file_ids.contains(&image1.id), "should not contain image");

    // 测试筛选不存在的类型
    let empty_json = filter_data_by_type("nonexistent").expect("filter nonexistent failed");
    let empty_results: Vec<ClipboardItem> = serde_json::from_str(&empty_json).expect("parse empty");
    assert!(
        empty_results.is_empty(),
        "expected no results for nonexistent type"
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
    let folder1 = make_item("search-5", "folder", "/path/to/myfolder");

    // 插入数据并等待一小段时间以确保时间戳不同
    insert_received_db_data(text1.clone()).expect("insert text1 failed");
    std::thread::sleep(std::time::Duration::from_millis(10));
    insert_received_db_data(text2.clone()).expect("insert text2 failed");
    std::thread::sleep(std::time::Duration::from_millis(10));
    insert_received_db_data(image1.clone()).expect("insert image1 failed");
    std::thread::sleep(std::time::Duration::from_millis(10));
    insert_received_db_data(file1.clone()).expect("insert file1 failed");
    std::thread::sleep(std::time::Duration::from_millis(10));
    insert_received_db_data(folder1.clone()).expect("insert folder1 failed");

    // ==================== 测试 "text" 类型搜索 ====================
    // 只搜索 item_type = 'text' 的数据

    // 搜索 "world" - 应该只匹配 text1
    let result_world = search_data("text", "world").expect("search world failed");
    let items_world: Vec<ClipboardItem> =
        serde_json::from_str(&result_world).expect("parse world results");
    let ids_world: Vec<String> = items_world.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_world.len(), 1, "expected 1 item for 'world'");
    assert!(ids_world.contains(&text1.id), "should contain text1");
    assert!(!ids_world.contains(&text2.id), "should not contain text2");
    assert!(!ids_world.contains(&image1.id), "should not contain image1");

    // 搜索 "programming" - 应该只匹配 text2
    let result_prog = search_data("text", "programming").expect("search programming failed");
    let items_prog: Vec<ClipboardItem> =
        serde_json::from_str(&result_prog).expect("parse programming results");
    let ids_prog: Vec<String> = items_prog.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_prog.len(), 1, "expected 1 item for 'programming'");
    assert!(ids_prog.contains(&text2.id), "should contain text2");
    assert!(!ids_prog.contains(&text1.id), "should not contain text1");

    // 搜索不存在的关键词 - 应该返回空数组
    let result_empty = search_data("text", "nonexistent").expect("search nonexistent failed");
    let items_empty: Vec<ClipboardItem> =
        serde_json::from_str(&result_empty).expect("parse empty results");
    assert!(
        items_empty.is_empty(),
        "expected empty results for nonexistent keyword"
    );

    // ==================== 测试 "ocr" 类型搜索 ====================
    // 只搜索 item_type = 'image' 的数据的 content 字段

    let result_ocr = search_data("ocr", "image").expect("search ocr failed");
    let items_ocr: Vec<ClipboardItem> =
        serde_json::from_str(&result_ocr).expect("parse ocr results");
    let ids_ocr: Vec<String> = items_ocr.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_ocr.len(), 1, "expected 1 image item for 'image'");
    assert!(
        ids_ocr.contains(&image1.id),
        "ocr search should find image1"
    );
    assert!(
        !ids_ocr.contains(&text1.id),
        "ocr search should not find text items"
    );
    assert!(
        !ids_ocr.contains(&file1.id),
        "ocr search should not find file items"
    );

    // ==================== 测试 "path" 类型搜索 ====================
    // 搜索 item_type IN ('file', 'folder', 'image') 的数据

    let result_path_search = search_data("path", "document").expect("search path failed");
    let items_path_search: Vec<ClipboardItem> =
        serde_json::from_str(&result_path_search).expect("parse path search");
    let ids_path_search: Vec<String> = items_path_search.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_path_search.len(), 1, "path search should find 1 item");
    assert!(
        ids_path_search.contains(&file1.id),
        "path search should find file1"
    );
    assert!(
        !ids_path_search.contains(&text1.id),
        "path search should not find text items"
    );

    // 搜索 "path" - 应该匹配 image1, file1, folder1
    let result_path_all = search_data("path", "path").expect("search path keyword failed");
    let items_path_all: Vec<ClipboardItem> =
        serde_json::from_str(&result_path_all).expect("parse path all results");
    let ids_path_all: Vec<String> = items_path_all.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_path_all.len(), 3, "expected 3 items containing 'path'");
    assert!(ids_path_all.contains(&image1.id), "should contain image1");
    assert!(ids_path_all.contains(&file1.id), "should contain file1");
    assert!(ids_path_all.contains(&folder1.id), "should contain folder1");
    assert!(
        !ids_path_all.contains(&text1.id),
        "should not contain text1"
    );
    assert!(
        !ids_path_all.contains(&text2.id),
        "should not contain text2"
    );

    // ==================== 测试时间戳范围搜索 ====================

    // 获取所有数据以确定时间戳范围
    let all_data_json = get_all_data().expect("get all data failed");
    let all_data: Vec<ClipboardItem> =
        serde_json::from_str(&all_data_json).expect("parse all data");

    let min_timestamp = all_data.iter().map(|it| it.timestamp).min().unwrap();
    let max_timestamp = all_data.iter().map(|it| it.timestamp).max().unwrap();
    let mid_timestamp = (min_timestamp + max_timestamp) / 2;

    // 搜索全部时间范围 - 应该返回所有 5 个项目
    let timestamp_query_all = format!("{},{}", min_timestamp - 1000, max_timestamp + 1000);
    let result_timestamp_all =
        search_data("timestamp", &timestamp_query_all).expect("search timestamp all failed");
    let items_timestamp_all: Vec<ClipboardItem> =
        serde_json::from_str(&result_timestamp_all).expect("parse timestamp all results");
    assert_eq!(
        items_timestamp_all.len(),
        5,
        "expected all 5 items in full time range"
    );

    // 搜索部分时间范围 - 应该返回部分项目
    let timestamp_query_partial = format!("{},{}", min_timestamp - 1000, mid_timestamp);
    let result_timestamp_partial = search_data("timestamp", &timestamp_query_partial)
        .expect("search timestamp partial failed");
    let items_timestamp_partial: Vec<ClipboardItem> =
        serde_json::from_str(&result_timestamp_partial).expect("parse timestamp partial results");
    assert!(
        items_timestamp_partial.len() >= 1 && items_timestamp_partial.len() <= 5,
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

    // 空搜索关键词 (text 类型) - 应该返回所有 text 类型的项
    let result_empty_query = search_data("text", "").expect("search empty query failed");
    let items_empty_query: Vec<ClipboardItem> =
        serde_json::from_str(&result_empty_query).expect("parse empty query results");
    assert_eq!(
        items_empty_query.len(),
        2,
        "empty query should match all text items"
    );
    let ids_empty: Vec<String> = items_empty_query.iter().map(|it| it.id.clone()).collect();
    assert!(ids_empty.contains(&text1.id), "should contain text1");
    assert!(ids_empty.contains(&text2.id), "should contain text2");

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
    assert_eq!(ids_case.len(), 1, "expected 1 match for case-insensitive");
    assert!(
        ids_case.contains(&text1.id),
        "case-insensitive search should find 'hello'"
    );

    // 模糊匹配测试 - 部分关键词
    let result_partial = search_data("text", "prog").expect("search partial failed");
    let items_partial: Vec<ClipboardItem> =
        serde_json::from_str(&result_partial).expect("parse partial results");
    let ids_partial: Vec<String> = items_partial.iter().map(|it| it.id.clone()).collect();
    assert_eq!(ids_partial.len(), 1, "expected 1 match for partial");
    assert!(
        ids_partial.contains(&text2.id),
        "partial match should find 'programming'"
    );

    // ==================== 测试默认类型（未知搜索类型）====================
    // 应该搜索所有类型的 content 字段

    let result_default = search_data("unknown_type", "path").expect("search default type failed");
    let items_default: Vec<ClipboardItem> =
        serde_json::from_str(&result_default).expect("parse default results");
    let ids_default: Vec<String> = items_default.iter().map(|it| it.id.clone()).collect();
    // 应该匹配所有包含 "path" 的项（image1, file1, folder1）
    assert_eq!(ids_default.len(), 3, "default search should find 3 items");
    assert!(ids_default.contains(&image1.id), "should contain image1");
    assert!(ids_default.contains(&file1.id), "should contain file1");
    assert!(ids_default.contains(&folder1.id), "should contain folder1");
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

#[test]
fn test_update_data_path() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. 准备数据
    // 需要更新的项 (file, folder, image)
    let file_match = make_item("udp-1", "file", "/old/root/doc.pdf");
    let folder_match = make_item("udp-2", "folder", "/old/root/subfolder");
    let image_match = make_item("udp-3", "image", "/old/root/img.png");

    // 不应更新的项 - 类型不对 (text)
    let text_match_prefix = make_item("udp-4", "text", "/old/root/note.txt");

    // 不应更新的项 - 前缀不匹配
    let file_no_match = make_item("udp-5", "file", "/other/root/doc.pdf");

    // 边界情况：前缀匹配但不是目录边界 (starts_with 逻辑)
    // "/old/root" 匹配 "/old/root_suffix/file"
    let file_partial_match = make_item("udp-6", "file", "/old/root_suffix/file");

    insert_received_db_data(file_match.clone()).expect("insert file_match");
    insert_received_db_data(folder_match.clone()).expect("insert folder_match");
    insert_received_db_data(image_match.clone()).expect("insert image_match");
    insert_received_db_data(text_match_prefix.clone()).expect("insert text_match");
    insert_received_db_data(file_no_match.clone()).expect("insert file_no_match");
    insert_received_db_data(file_partial_match.clone()).expect("insert file_partial");

    // 2. 执行更新
    let old_path = "/old/root";
    let new_path = "/new/root";
    let count = update_data_path(old_path, new_path).expect("update_data_path failed");

    // 3. 验证受影响行数
    // file_match, folder_match, image_match, file_partial_match 应该被更新 (共4个)
    // text_match_prefix 类型不对
    // file_no_match 前缀不对
    assert_eq!(count, 4, "expected 4 items to be updated");

    // 4. 验证内容更新结果
    let get_content = |id| -> String {
        let json = get_data_by_id(id).unwrap();
        let item: ClipboardItem = serde_json::from_str(&json).unwrap();
        item.content
    };

    assert_eq!(get_content(&file_match.id), "/new/root/doc.pdf");
    assert_eq!(get_content(&folder_match.id), "/new/root/subfolder");
    assert_eq!(get_content(&image_match.id), "/new/root/img.png");

    // 验证部分匹配的替换结果
    assert_eq!(get_content(&file_partial_match.id), "/new/root_suffix/file");

    // 5. 验证未受影响的项目保持原样
    assert_eq!(get_content(&text_match_prefix.id), "/old/root/note.txt");
}

#[test]
fn test_comprehensive_search_with_ocr() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 1. 准备数据
    let item_text = make_item("search-text-1", "text", "hello world content");
    let item_img_ocr = make_item("search-img-ocr", "image", "/tmp/image_ocr.png");
    let item_note = make_item("search-note-1", "text", "some content");

    insert_received_db_data(item_text.clone()).expect("insert text failed");
    insert_received_db_data(item_img_ocr.clone()).expect("insert img failed");
    insert_received_db_data(item_note.clone()).expect("insert note failed");

    // 2. 添加 Note 和 OCR
    add_notes_by_id(&item_note.id, "important note keyword").expect("add note failed");
    insert_ocr_text(&item_img_ocr.id, "detected ocr keyword inside image")
        .expect("insert ocr failed");

    // 3. 测试搜索 Content
    let res_content =
        comprehensive_search("world", None, None, None).expect("search content failed");
    let items_content: Vec<ClipboardItem> =
        serde_json::from_str(&res_content).expect("parse content res");
    assert!(items_content.iter().any(|i| i.id == item_text.id));

    // 4. 测试搜索 Note
    let res_note = comprehensive_search("important", None, None, None).expect("search note failed");
    let items_note: Vec<ClipboardItem> = serde_json::from_str(&res_note).expect("parse note res");
    assert!(items_note.iter().any(|i| i.id == item_note.id));

    // 5. 测试搜索 OCR (新功能验证)
    let res_ocr = comprehensive_search("detected", None, None, None).expect("search ocr failed");
    let items_ocr: Vec<ClipboardItem> = serde_json::from_str(&res_ocr).expect("parse ocr res");
    assert_eq!(items_ocr.len(), 1, "should find exactly one item by ocr");
    assert_eq!(
        items_ocr[0].id, item_img_ocr.id,
        "should find the image with ocr"
    );

    // 6. 测试组合搜索 (OCR + Type)
    let res_combo =
        comprehensive_search("keyword", Some("image"), None, None).expect("search combo failed");
    let items_combo: Vec<ClipboardItem> =
        serde_json::from_str(&res_combo).expect("parse combo res");
    assert_eq!(items_combo.len(), 1);
    assert_eq!(items_combo[0].id, item_img_ocr.id);

    // 7. 测试组合搜索 (OCR + Wrong Type)
    let res_wrong_type = comprehensive_search("detected", Some("text"), None, None)
        .expect("search wrong type failed");
    let items_wrong: Vec<ClipboardItem> =
        serde_json::from_str(&res_wrong_type).expect("parse wrong type res");
    assert_eq!(
        items_wrong.len(),
        0,
        "should not find image when filtering by text"
    );
}
