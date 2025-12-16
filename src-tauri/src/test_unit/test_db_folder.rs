/// 单元测试数据库收藏夹相关操作
/// 此文件提供收藏夹功能点测试，包括创建、重命名、删除、添加/移除项等
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
fn test_delete_all_and_count_folder_and_item() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 插入 5 个 item
    let items = vec![
        make_item("d-1", "text", "one"),
        make_item("d-2", "text", "two"),
        make_item("d-3", "image", "/img/1"),
        make_item("d-4", "text", "four"),
        make_item("d-5", "image", "/img/2"),
    ];
    for it in &items {
        insert_received_db_data(it.clone()).expect("insert failed");
    }

    // 创建两个收藏夹，并将部分 item 添加进去
    let folder_a = create_new_folder("DelFolderA").expect("create folder A");
    let folder_b = create_new_folder("DelFolderB").expect("create folder B");

    add_item_to_folder(&folder_a, "d-1").expect("add d-1 to DelFolderA failed");
    add_item_to_folder(&folder_a, "d-2").expect("add d-2 to DelFolderA failed");
    add_item_to_folder(&folder_b, "d-3").expect("add d-3 to DelFolderB failed");
    add_item_to_folder(&folder_b, "d-4").expect("add d-4 to DelFolderB failed");

    // 获取folders列表并验证
    let all_folders_json = get_all_folders().expect("get all folders before delete failed");
    let all_folders: Vec<FolderItem> =
        serde_json::from_str(&all_folders_json).expect("parse all folders before delete");
    assert_eq!(all_folders.len(), 2, "should have 2 folders before delete");
    for f in &all_folders {
        if f.id == folder_a {
            assert_eq!(f.num_items, 2, "DelFolderA should have 2 items");
        } else if f.id == folder_b {
            assert_eq!(f.num_items, 2, "DelFolderB should have 2 items");
        } else {
            panic!("unexpected folder id");
        }
    }

    // 删除单个数据
    let deleted_single = delete_data_by_id("d-1").expect("delete d-1 failed");
    assert_eq!(1, deleted_single, "should delete 1 item");

    // 验证 DelFolderA 的 num_items 减少
    let all_folders_json = get_all_folders().expect("get all folders after single delete failed");
    let all_folders: Vec<FolderItem> =
        serde_json::from_str(&all_folders_json).expect("parse all folders after single delete");
    let fa = all_folders
        .iter()
        .find(|f| f.id == folder_a)
        .expect("DelFolderA missing after single delete");
    assert_eq!(
        fa.num_items, 1,
        "DelFolderA should have 1 item after deleting d-1"
    );

    // 删除所有数据
    let deleted_count = delete_all_data(None, false).expect("delete all data failed");
    assert_eq!(deleted_count, 4, "should delete 4 items");

    // 再次获取 folders 列表，验证 num_items 都为 0
    let all_folders_json = get_all_folders().expect("get all folders after delete failed");
    let all_folders: Vec<FolderItem> =
        serde_json::from_str(&all_folders_json).expect("parse all folders after delete");
    assert_eq!(
        all_folders.len(),
        2,
        "should still have 2 folders after delete"
    );
    for f in &all_folders {
        assert_eq!(
            f.num_items, 0,
            "all folders should have 0 items after delete"
        );
    }
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
