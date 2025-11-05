use rusqlite::{params, Connection, Result};
// use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use std::{path::Path, sync::OnceLock};

use crate::clipboard::ClipboardItem;

// const DB_PATH: &str = "smartpaste.db";

static DB_PATH_GLOBAL: OnceLock<PathBuf> = OnceLock::new();

/// 将 ClipboardItem 转换为 JSON 字符串。
/// # Param
/// item: ClipboardItem - 要转换的剪贴板项
pub fn clipboard_item_to_json(item: ClipboardItem) -> Result<String, String> {
    serde_json::to_string(&item).map_err(|e| e.to_string())
}
/// 将 ClipboardItem 列表转换为 JSON 字符串。
/// # Param
/// items: Vec<ClipboardItem> - 要转换的剪贴板项列表
pub fn clipboard_items_to_json(items: Vec<ClipboardItem>) -> Result<String, String> {
    serde_json::to_string(&items).map_err(|e| e.to_string())
}

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
    Ok(())
}

/// 将接收到的数据插入数据库。作为 Tauri command 暴露给前端调用。
/// Param:
/// data: ClipboardItem - 要插入的数据项
/// Returns:
/// String - 插入结果信息
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
    crate::clipboard::set_last_inserted(data);

    Ok("inserted".to_string())
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

/// 新建收藏夹。作为 Tauri command 暴露给前端调用。
/// # Param
/// name: &str - 收藏夹名称
/// # Returns
/// String - 成功信息
/// TODO: 尚未完成
#[tauri::command]
pub fn create_new_folder(name: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    if name.is_empty() {
        return Err("folder name is empty".to_string());
    }
    // 仅允许字母、数字和下划线，避免 SQL 注入或非法列名
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(
            "folder name contains invalid characters; only letters, digits and underscore allowed"
                .to_string(),
        );
    }

    let folder_name_in_database = format!("folder_{}", name);

    // 检查列是否已存在：使用 PRAGMA table_info(data) 获取列名
    let mut stmt = conn
        .prepare("PRAGMA table_info(data)")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?;

    while let Some(col_res) = rows.next() {
        let col = col_res.map_err(|e| e.to_string())?;
        if col == folder_name_in_database {
            return Ok(format!("收藏夹 '{}' 已存在", name));
        }
    }

    // 添加新列，类型为 INTEGER，NOT NULL，默认 0（代表 false）
    let alter_sql = format!(
        "ALTER TABLE data ADD COLUMN \"{}\" INTEGER NOT NULL DEFAULT 0",
        folder_name_in_database
    );
    conn.execute(&alter_sql, []).map_err(|e| e.to_string())?;

    Ok(format!("收藏夹 '{}' 已创建", name))
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
        TEST_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn set_test_db_path() {
        // 在临时目录下使用独立数据库文件，避免污染真实数据
        let mut p = std::env::temp_dir();
        p.push("smartpaste_test.db");
        // 覆盖全局 OnceLock（只会在第一次调用设置）
        set_db_path(p);
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
        let res = insert_received_data(item.clone()).expect("insert failed");
        assert_eq!(res, "inserted");

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
    fn test_create_new_folder_valid_and_invalid() {
        let _g = test_lock();
        set_test_db_path();
        clear_db_file();

        // valid
        let ok = create_new_folder("testfolder").expect("create folder failed");
        assert!(ok.contains("已创建") || ok.contains("已存在"));

        // creating again should return exists
        let ok2 = create_new_folder("testfolder").expect("create folder second failed");
        assert!(ok2.contains("已存在"));

        // invalid name
        let err = create_new_folder("bad name!").err();
        assert!(err.is_some());
    }
}
