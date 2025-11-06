use rusqlite::{params, Connection, Result};
use uuid::Uuid;
// use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{path::Path, sync::OnceLock};

use crate::clipboard::clipboard_item_to_json;
use crate::clipboard::clipboard_items_to_json;
use crate::clipboard::ClipboardItem;

// const DB_PATH: &str = "smartpaste.db";

static DB_PATH_GLOBAL: OnceLock<PathBuf> = OnceLock::new();

/// 设置数据库路径
/// # Param
/// path: PathBuf - 数据库文件路径
pub fn set_db_path(path: PathBuf) {
    let _ = DB_PATH_GLOBAL.set(path);
}

/// 获取数据库路径
/// # Returns
/// PathBuf - 数据库文件路径
fn get_db_path() -> PathBuf {
    DB_PATH_GLOBAL
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("smartpaste.db"))
}

/// 初始化数据库（合并了 CREATE TABLE IF NOT EXISTS 的逻辑）
/// path: &Path - 数据库文件路径
pub fn init_db(path: &Path) -> Result<()> {
    let conn = Connection::open(path)?;

    // 元数据表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS data (
            id TEXT PRIMARY KEY NOT NULL, 
            item_type TEXT NOT NULL,
            content TEXT NOT NULL,
            size INTEGER NOT NULL,
            is_favorite INTEGER NOT NULL,
            notes TEXT,
            timestamp INTEGER NOT NULL
        )",
        [],
    )?;

    // 收藏夹表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS folders (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL
            )",
        [],
    )?;

    // 收藏夹与数据关联表，用于多对多关系
    conn.execute(
        "CREATE TABLE IF NOT EXISTS folder_items (
            folder_id TEXT NOT NULL,
            item_id TEXT NOT NULL,
            PRIMARY KEY (folder_id, item_id),
            FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE,
            FOREIGN KEY (item_id) REFERENCES data(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(())
}

/// 将接收到的数据插入数据库。作为 Tauri command 暴露给前端调用。
/// Param:
/// data: ClipboardItem - 要插入的数据项
/// Returns:
/// String - 插入的数据的 JSON 字符串。如果失败则返回错误信息
#[tauri::command]
pub fn insert_received_data(data: ClipboardItem) -> Result<String, String> {
    // NOTE: 这里我们把数据库文件放在工作目录下的 smartpaste.db 中。
    // 更稳妥的做法是在运行时从 `tauri::api::path::app_dir` 或 `app.path_resolver()` 获取应用本地数据目录。
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    conn.execute("INSERT OR REPLACE INTO data (id, item_type, content, size, is_favorite, notes, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            data.id,
            data.item_type,
            data.content,
            data.size.unwrap_or(0) as i64,
            data.is_favorite as i32, // SQLite 使用整数表示布尔值
            data.notes,
            data.timestamp,
        ],
    ) 
        .map_err(|e| e.to_string())?;

    // 插入成功后，更新全局最后插入项
    crate::clipboard::set_last_inserted(data.clone());

    clipboard_item_to_json(data)
}

/// 获取上一条数据。作为 Tauri command 暴露给前端调用。
/// # Returns
/// String - 包含上一条数据的 JSON 字符串，若无则返回 null
#[tauri::command]
pub fn get_latest_data() -> Result<String, String> {
    if let Some(item) = crate::clipboard::get_last_inserted() {
        clipboard_item_to_json(item)
    } else {
        Ok("null".to_string())
    }
}

/// 获取所有数据。作为 Tauri command 暴露给前端调用。
/// # Returns
/// String - 包含所有数据记录的 JSON 字符串
#[tauri::command]
pub fn get_all_data() -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, item_type, content, size, is_favorite, notes, timestamp FROM data")
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map([], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                item_type: row.get(1)?,
                content: row.get(2)?,
                size: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                is_favorite: row.get::<_, i32>(4)? != 0,
                notes: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for item in clipboard_iter {
        results.push(item.map_err(|e| e.to_string())?);
    }

    clipboard_items_to_json(results)
}

/// 返回数据。作为 Tauri command 暴露给前端调用。
/// 根据数据 ID 返回对应的数据记录。
/// # Param
/// id: &str - 数据 ID
/// # Returns
/// String - 包含数据记录的 JSON 字符串，若未找到则返回 null
#[tauri::command]
pub fn get_data_by_id(id: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
             FROM data 
             WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                item_type: row.get(1)?,
                content: row.get(2)?,
                size: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                is_favorite: row.get::<_, i32>(4)? != 0,
                notes: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    if let Some(item) = rows.next() {
        let clipboard_item = item.map_err(|e| e.to_string())?;
        clipboard_item_to_json(clipboard_item)
    } else {
        Ok("null".to_string())
    }
}
/// 删除数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// data: ClipboardDBItem - 包含要删除数据的 ID 字段
#[tauri::command]
pub fn delete_data(data: ClipboardItem) -> Result<usize, String> {
    delete_data_by_id(&data.id)
}

/// 根据 ID 删除数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// id: &str - 要删除数据的 ID
#[tauri::command]
pub fn delete_data_by_id(id: &str) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let rows_affected = conn
        .execute("DELETE FROM data WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(rows_affected)
}

/// 根据 ID 修改收藏状态。作为 Tauri command 暴露给前端调用。
/// 如果 is_favorite 为 true，则收藏数据；否则取消收藏数据。
/// # Param
/// id: &str - 要修改收藏状态的数据 ID
/// # Returns
/// String - 信息。若收藏成功返回 "favorited"，取消收藏成功返回 "unfavorited"，否则返回错误信息
#[tauri::command]
pub fn set_favorite_status_by_id(id: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // 先查询当前的收藏状态
    let mut stmt = conn
        .prepare("SELECT is_favorite FROM data WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let current_status: Option<i32> = stmt
        .query_row(params![id], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    match current_status {
        Some(status) => {
            if status == 0 {
                // 当前未收藏，执行收藏操作
                favorite_data_by_id(id)?;
                Ok("favorited".to_string())
            } else {
                // 当前已收藏，执行取消收藏操作
                unfavorite_data_by_id(id)?;
                Ok("unfavorited".to_string())
            }
        }
        None => Err("Item not found".to_string()),
    }
}

/// 根据 ID 收藏数据。
/// # Param
/// id: &str - 要收藏数据的 ID
/// # Returns
/// usize - 受影响的行数
pub fn favorite_data_by_id(id: &str) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let rows_affected = conn
        .execute("UPDATE data SET is_favorite = 1 WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(rows_affected)
}

/// 根据 ID 取消收藏数据。
/// # Param
/// id: &str - 要取消收藏数据的 ID
/// # Returns
/// usize - 受影响的行数
pub fn unfavorite_data_by_id(id: &str) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let rows_affected = conn
        .execute("UPDATE data SET is_favorite = 0 WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(rows_affected)
}

/// 文本搜索。作为 Tauri command 暴露给前端调用。
/// 根据传入的字符串，对所有属于 text 类的 content 字段进行模糊搜索，返回匹配的记录列表。
/// # Param
/// query: &str - 搜索关键词
/// # Returns
/// String - 包含匹配数据记录的 JSON 字符串
#[tauri::command]
pub fn search_text_content(query: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let like_pattern = format!("%{}%", query);

    let mut stmt = conn
        .prepare(
            "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
             FROM data 
             WHERE item_type = 'text' AND content LIKE ?1",
        )
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map(params![like_pattern], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                item_type: row.get(1)?,
                content: row.get(2)?,
                size: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                is_favorite: row.get::<_, i32>(4)? != 0,
                notes: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for item in clipboard_iter {
        results.push(item.map_err(|e| e.to_string())?);
    }

    clipboard_items_to_json(results)
}

/// 增加备注。作为 Tauri command 暴露给前端调用。
/// # Param
/// id: &str - 数据 ID
/// notes: &str - 备注内容
/// # Returns
/// String - 更新后的记录的 JSON 字符串
#[tauri::command]
pub fn add_notes_by_id(id: &str, notes: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE data SET notes = ?1 WHERE id = ?2",
        params![notes, id],
    )
    .map_err(|e| e.to_string())?;

    // 返回更新后的记录（以 JSON 字符串形式）
    let json = get_data_by_id(id)?;
    if json == "null" {
        Err("Item not found after update".to_string())
    } else {
        Ok(json)
    }
}

/// 按类型筛选数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// item_type: &str - 数据类型（如 "text", "image" 等）
/// # Returns
/// String - 包含筛选后数据记录的 JSON 字符串
#[tauri::command]
pub fn filter_data_by_type(item_type: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
             FROM data 
             WHERE item_type = ?1",
        )
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map(params![item_type], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                item_type: row.get(1)?,
                content: row.get(2)?,
                size: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                is_favorite: row.get::<_, i32>(4)? != 0,
                notes: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for item in clipboard_iter {
        results.push(item.map_err(|e| e.to_string())?);
    }

    clipboard_items_to_json(results)
}

/// 新建收藏夹。作为 Tauri command 暴露给前端调用。
/// # Param
/// name: &str - 收藏夹名称
/// # Returns
/// String - 新建收藏夹的 ID，若失败则返回错误信息
#[tauri::command]
pub fn create_new_folder(name: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO folders (id, name) VALUES (?1, ?2)",
        params![id, name],
    )
    .map_err(|e| e.to_string())?;

    Ok(id)
}

/// 重命名收藏夹。作为 Tauri command 暴露给前端调用。
/// # Param
/// folder_id: &str - 收藏夹 ID
/// new_name: &str - 新名称
/// # Returns
/// String - 信息。若重命名成功返回 "renamed"，否则返回错误信息
#[tauri::command]
pub fn rename_folder(folder_id: &str, new_name: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE folders SET name = ?1 WHERE id = ?2",
        params![new_name, folder_id],
    )
    .map_err(|e| e.to_string())?;

    Ok("renamed".to_string())
}

/// 删除收藏夹。作为 Tauri command 暴露给前端调用。
/// # Param
/// folder_id: &str - 收藏夹 ID
/// # Returns
/// String - 信息。若删除成功返回 "deleted"，否则返回错误信息
#[tauri::command]
pub fn delete_folder(folder_id: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM folders WHERE id = ?1",
        params![folder_id],
    )
    .map_err(|e| e.to_string())?;

    Ok("deleted".to_string())
}

/// 向收藏夹添加数据项。作为 Tauri command 暴露给前端调用。
/// # Param
/// folder_id: &str - 收藏夹 ID
/// item_id: &str - 数据项 ID
/// # Returns
/// String - 信息。若添加成功返回 "added to folder"，否则返回错误信息
#[tauri::command]
pub fn add_item_to_folder(folder_id: &str, item_id: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR IGNORE INTO folder_items (folder_id, item_id) VALUES (?1, ?2)",
        params![folder_id, item_id],
    )
    .map_err(|e| e.to_string())?;

    Ok("added to folder".to_string())
}

/// 从收藏夹移除数据项。作为 Tauri command 暴露给前端调用。
/// # Param
/// folder_id: &str - 收藏夹 ID
/// item_id: &str - 数据项 ID
/// # Returns
/// String - 信息。若移除成功返回 "removed from folder"，否则返回错误信息
#[tauri::command]
pub fn remove_item_from_folder(folder_id: &str, item_id: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM folder_items WHERE folder_id = ?1 AND item_id = ?2",
        params![folder_id, item_id],
    )
    .map_err(|e| e.to_string())?;

    Ok("removed from folder".to_string())
}

/// 筛选收藏夹内的数据项。作为 Tauri command 暴露给前端调用。
/// # Param
/// folder_name: &str - 收藏夹名称
/// # Returns
/// String - 包含筛选后数据记录的 JSON 字符串，若失败则返回错误信息
#[tauri::command]
pub fn filter_data_by_folder(folder_name: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.item_type, d.content, d.size, d.is_favorite, d.notes, d.timestamp
             FROM data d
             JOIN folder_items fi ON d.id = fi.item_id
             JOIN folders f ON fi.folder_id = f.id
             WHERE f.name = ?1",
        )
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map(params![folder_name], |row| {
            Ok(ClipboardItem {
                id: row.get(0)?,
                item_type: row.get(1)?,
                content: row.get(2)?,
                size: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                is_favorite: row.get::<_, i32>(4)? != 0,
                notes: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for item in clipboard_iter {
        results.push(item.map_err(|e| e.to_string())?);
    }

    clipboard_items_to_json(results)
}

/// # 单元测试
#[cfg(test)]
mod tests {
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
    fn test_set_favorite_status_by_id() {
        let _g = test_lock();
        set_test_db_path();
        clear_db_file();

        let item = make_item("fav-1", "text", "to be favorited");
        insert_received_data(item.clone()).expect("insert for favorite");

        // initially not favorite
        let fetched_json = get_data_by_id(&item.id).expect("get for favorite");
        let fetched: ClipboardItem =
            serde_json::from_str(&fetched_json).expect("parse for favorite");
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

        // 创建并插入 5 个 item
        let items = vec![
            make_item("f-1", "text", "one"),
            make_item("f-2", "text", "two"),
            make_item("f-3", "image", "/tmp/img1"),
            make_item("f-4", "text", "four"),
            make_item("f-5", "image", "/tmp/img2"),
        ];
        for it in &items {
            insert_received_data(it.clone()).expect("insert failed");
        }

        // 新建两个收藏夹
        let folder_id = create_new_folder("TestFolder").expect("create folder");
        let other_folder_id = create_new_folder("OtherFolder").expect("create other");

        // 向 TestFolder 添加 4 个 item，向 OtherFolder 添加 1 个
        for id in &["f-1", "f-2", "f-3", "f-4"] {
            add_item_to_folder(&folder_id, id).expect("add to folder failed");
        }
        add_item_to_folder(&other_folder_id, "f-5").expect("add other failed");

        // 重复添加同一项（应被忽略，不会重复出现在查询结果中）
        add_item_to_folder(&folder_id, "f-1").expect("duplicate add failed");

        // 通过收藏夹名称筛选 TestFolder 的内容，应该包含 f-1..f-4，不包含 f-5
        let res = filter_data_by_folder("TestFolder").expect("filter failed");
        let vec: Vec<ClipboardItem> = serde_json::from_str(&res).expect("parse");
        let ids: Vec<String> = vec.iter().map(|it| it.id.clone()).collect();
        assert_eq!(ids.len(), 4);
        for id in &["f-1", "f-2", "f-3", "f-4"] {
            assert!(ids.contains(&id.to_string()));
        }
        assert!(!ids.contains(&"f-5".to_string()));

        // 重命名收藏夹并验证通过新名称仍能查询到原有项，旧名称不再返回结果
        let ren = rename_folder(&folder_id, "RenamedFolder").expect("rename failed");
        assert_eq!(ren, "renamed");

        let res2 = filter_data_by_folder("RenamedFolder").expect("filter after rename failed");
        let vec2: Vec<ClipboardItem> = serde_json::from_str(&res2).expect("parse2");
        let ids2: Vec<String> = vec2.iter().map(|it| it.id.clone()).collect();
        assert_eq!(ids2.len(), 4);
        for id in &["f-1", "f-2", "f-3", "f-4"] {
            assert!(ids2.contains(&id.to_string()));
        }

        let res_old = filter_data_by_folder("TestFolder").expect("filter old name failed");
        let vec_old: Vec<ClipboardItem> = serde_json::from_str(&res_old).expect("parse old");
        assert!(vec_old.is_empty());
    }
}
