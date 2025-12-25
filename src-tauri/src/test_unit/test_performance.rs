/// 性能测试模块
/// 测试数据库插入操作的性能，确保在合理时间内完成
/// 依赖于 src-tauri/src/db/mod.rs 中的数据库操作功能
#[cfg(test)]
mod tests {
    use crate::clipboard::ClipboardItem;
    use crate::db;
    use chrono::Utc;
    use std::path::PathBuf;
    use std::time::Instant;
    use uuid::Uuid;

    #[test]
    fn test_db_insert_performance() {
        // Acquire global test lock to prevent interference with other tests
        let _g = crate::db::TEST_RUN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        
        println!("🚀 开始数据库插入性能测试...");

        // 设置临时的测试数据库路径
        let mut p = std::env::temp_dir();
        let filename = format!("smartpaste_test_perf_{}.db", Uuid::new_v4());
        p.push(filename);
        
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
        db::set_db_path(p.clone());

        // 1. 测试主数据插入
        let item_id = Uuid::new_v4().to_string();
        let item = ClipboardItem {
            id: item_id.clone(),
            item_type: "file".to_string(),
            content: "C:\\Fake\\Path\\For\\Performance\\Test.txt".to_string(),
            size: Some(1024),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: Utc::now().timestamp_millis(),
        };

        let start_main = Instant::now();
        match db::insert_received_db_data(item) {
            Ok(_) => println!("✅ 主数据插入成功"),
            Err(e) => panic!("❌ 主数据插入失败: {}", e),
        }
        let duration_main = start_main.elapsed();
        println!(
            "⏱️ [Test] insert_received_db_data 耗时: {:?}",
            duration_main
        );

        // 2. 测试图标数据插入 (模拟 5KB 的 Base64 数据)
        let icon_data = "data:image/png;base64,".to_string() + &"A".repeat(5120);

        let start_icon = Instant::now();
        match db::insert_icon_data(&item_id, &icon_data) {
            Ok(_) => println!("✅ 图标数据插入成功"),
            Err(e) => panic!("❌ 图标数据插入失败: {}", e),
        }
        let duration_icon = start_icon.elapsed();
        println!("⏱️ [Test] insert_icon_data 耗时: {:?}", duration_icon);

        // 清理
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }
}
