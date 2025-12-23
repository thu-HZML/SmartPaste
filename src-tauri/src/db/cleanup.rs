use super::{get_db_path, init_db};
use rusqlite::{params, Connection};
use std::sync::mpsc::Sender;
use std::sync::RwLock;

static CLEANUP_SENDER: RwLock<Option<Sender<()>>> = RwLock::new(None);

/// 按配置中的天数清理过期数据，自动屏蔽未收藏的数据。
/// # Param
/// days: u32 - 过期天数
/// # Returns
/// Result<usize, String> - 被删除的记录数量，若失败则返回错误信息
pub fn clear_data_expired(days: u32) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let cutoff_timestamp = chrono::Utc::now().timestamp() - (days as i64 * 86400);

    let rows_deleted = conn
        .execute(
            "DELETE FROM data WHERE timestamp < ?1 AND is_favorite = 0",
            params![cutoff_timestamp],
        )
        .map_err(|e| e.to_string())?;

    Ok(rows_deleted)
}

/// 按设定的最大历史记录数量删除多余的数据，自动屏蔽未收藏的数据。
/// 删除优先级：按照时间戳从旧到新排序，删除最旧的数据。
/// # Param
/// max_items: usize - 最大历史记录数量
/// # Returns
/// Result<usize, String> - 被删除的记录数量，若失败则返回错误信息
pub fn enforce_max_history_items(max_items: u32) -> Result<usize, String> {
    // 如果配置为 0，表示不启用自动清理（保留默认行为）
    if max_items == 0 {
        return Ok(0);
    }

    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // 计算需要删除的记录数量
    let total_count: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM data WHERE is_favorite = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if total_count <= max_items {
        return Ok(0); // 不需要删除任何记录
    }

    let to_delete_count = total_count - max_items;

    // 删除最旧的记录
    let rows_deleted = conn
        .execute(
            "DELETE FROM data 
             WHERE id IN (
                 SELECT id FROM data 
                 WHERE is_favorite = 0 
                 ORDER BY timestamp ASC 
                 LIMIT ?1
             )",
            params![to_delete_count],
        )
        .map_err(|e| e.to_string())?;

    Ok(rows_deleted)
}

/// 设置清理通知 Sender（由 app_setup 调用）
/// # Param
/// sender: Sender<()> - 清理通知的 Sender
pub fn set_cleanup_sender(sender: Sender<()>) {
    let mut s = CLEANUP_SENDER.write().unwrap();
    *s = Some(sender);
}

/// 通知清理线程执行清理（内部使用）
pub fn notify_cleanup() {
    if let Some(sender) = CLEANUP_SENDER.read().unwrap().as_ref() {
        let _ = sender.send(()); // 忽略发送错误
    }
}

/// 手动触发清理操作。作为 Tauri command 暴露给前端调用。
/// # Returns
/// String - 信息。若触发成功返回 "cleanup triggered"，否则返回错误信息
#[tauri::command]
pub fn trigger_cleanup() -> Result<String, String> {
    if let Some(sender) = CLEANUP_SENDER.read().unwrap().as_ref() {
        sender.send(()).map_err(|e| e.to_string())?;
        Ok("cleanup triggered".to_string())
    } else {
        Err("cleanup worker not started".to_string())
    }
}

#[cfg(test)]
pub fn reset_cleanup_sender() {
    let mut s = CLEANUP_SENDER.write().unwrap();
    *s = None;
}
