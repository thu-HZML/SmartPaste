use super::*;
use crate::clipboard::ClipboardItem;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

// 复制 test_db_base.rs 中的辅助函数，因为它们不是 public 的

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    crate::db::TEST_RUN_LOCK.lock().unwrap_or_else(|p| p.into_inner())
}

use uuid::Uuid;

fn set_test_db_path() {
    let mut p = std::env::temp_dir();
    let filename = format!("smartpaste_test_extend_{}.db", Uuid::new_v4());
    p.push(filename); // 使用不同的文件名避免冲突
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

fn make_item_with_ts(id: &str, content: &str, is_fav: bool, ts: i64) -> ClipboardItem {
    ClipboardItem {
        id: id.to_string(),
        item_type: "text".to_string(),
        content: content.to_string(),
        size: Some(content.len() as u64),
        is_favorite: is_fav,
        notes: "".to_string(),
        timestamp: ts,
    }
}

#[test]
fn test_clear_data_expired() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    // 准备时间点
    let now = Utc::now().timestamp();
    let one_day_sec = 86400;
    let days_limit = 7;

    // 1. 过期数据 (8天前)
    let item_expired = make_item_with_ts(
        "expired",
        "old data",
        false,
        now - (days_limit + 1) * one_day_sec,
    );
    // 2. 未过期数据 (1天前)
    let item_fresh = make_item_with_ts("fresh", "new data", false, now - 1 * one_day_sec);
    // 3. 过期但收藏 (8天前)
    let item_expired_fav = make_item_with_ts(
        "expired_fav",
        "old fav data",
        true,
        now - (days_limit + 1) * one_day_sec,
    );

    insert_received_db_data(item_expired).unwrap();
    insert_received_db_data(item_fresh).unwrap();
    insert_received_db_data(item_expired_fav).unwrap();

    // 执行清理，保留7天
    let deleted = clear_data_expired(days_limit as u32).unwrap();

    // 验证删除了1条
    assert_eq!(deleted, 1, "Should delete exactly 1 expired item");

    // 验证剩余数据
    let all_data_json = get_all_data().unwrap();
    let all_data: Vec<ClipboardItem> = serde_json::from_str(&all_data_json).unwrap();

    assert_eq!(all_data.len(), 2);

    let ids: Vec<String> = all_data.iter().map(|i| i.id.clone()).collect();
    assert!(
        ids.contains(&"fresh".to_string()),
        "Fresh item should remain"
    );
    assert!(
        ids.contains(&"expired_fav".to_string()),
        "Favorite item should remain even if expired"
    );
    assert!(
        !ids.contains(&"expired".to_string()),
        "Expired item should be deleted"
    );

    clear_db_file();
}

#[test]
fn test_enforce_max_history_items() {
    let _g = test_lock();
    set_test_db_path();
    clear_db_file();

    let now = Utc::now().timestamp();

    // 插入 5 条数据，时间递增
    // item1: 最旧，非收藏 -> 应该被删
    // item2: 次旧，收藏 -> 应该保留
    // item3: 中间，非收藏 -> 应该被删
    // item4: 较新，非收藏 -> 保留
    // item5: 最新，非收藏 -> 保留

    // 我们设置 max_items = 2 (非收藏项限制)
    // 现有 5 条。其中 1 条收藏。
    // 总数 5。非收藏 4。
    // 逻辑是：如果 total_count (非收藏) > max_items，则删除 (total_count - max_items) 条最旧的非收藏。
    // 这里 total_count(非收藏) = 4。 max_items = 2
    // 应该删除 4 - 2 = 2 条最旧的非收藏 (item1, item3)。

    let item1 = make_item_with_ts("1", "1", false, now - 1000);
    let item2 = make_item_with_ts("2", "2", true, now - 900); // 收藏
    let item3 = make_item_with_ts("3", "3", false, now - 800);
    let item4 = make_item_with_ts("4", "4", false, now - 700);
    let item5 = make_item_with_ts("5", "5", false, now - 600);

    insert_received_db_data(item1).unwrap();
    insert_received_db_data(item2).unwrap();
    insert_received_db_data(item3).unwrap();
    insert_received_db_data(item4).unwrap();
    insert_received_db_data(item5).unwrap();

    let deleted = enforce_max_history_items(2).unwrap();
    assert_eq!(deleted, 2);

    let all_data_json = get_all_data().unwrap();
    let all_data: Vec<ClipboardItem> = serde_json::from_str(&all_data_json).unwrap();

    // 剩余应该是 item2(收藏), item4, item5
    assert_eq!(all_data.len(), 3);

    let ids: Vec<String> = all_data.iter().map(|i| i.id.clone()).collect();
    assert!(ids.contains(&"2".to_string()));
    assert!(ids.contains(&"4".to_string()));
    assert!(ids.contains(&"5".to_string()));

    clear_db_file();
}
