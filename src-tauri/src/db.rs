use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

const DB_PATH: &str = "smartpaste.db";

// 传入的数据结构（根据前端实际字段调整）
// 派生 Clone 以便在测试中可以 clone 实例；同时派生 Debug 有助于断言失败时打印信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClipboardDBItem {
    id: String,
    item_type: String, // 数据类型：text/image/file
    content: String,   // 对text类型，存储文本内容；对image/file类型，存储文件路径
    is_favorite: bool, // 是否收藏
    notes: String,     // 备注
    timestamp: i64,
}

/// 初始化数据库，在此处设计数据库的表结构
/// TODO: 根据实际需求调整表结构
/// # Param
/// path: str - 数据库文件路径
///
/// # Example
/// init_db("smartpaste.db")?;
///
// pub fn init_db(path: &str) -> Result<()> {
//     let conn = Connection::open(path)?;
//     conn.execute(
//         "CREATE TABLE IF NOT EXISTS SmartPaste (
//             id TEXT PRIMARY KEY NOT NULL,
//             value TEXT NOT NULL
//         )",
//         [],
//     )?;
//     Ok(())
// }

/// 将接收到的数据插入数据库。作为 Tauri command 暴露给前端调用。
/// TODO: 根据实际需求调整插入逻辑
///
#[tauri::command]
pub fn insert_received_data(data: ClipboardDBItem) -> Result<String, String> {
    // NOTE: 这里我们把数据库文件放在工作目录下的 smartpaste.db 中。
    // 更稳妥的做法是在运行时从 `tauri::api::path::app_dir` 或 `app.path_resolver()` 获取应用本地数据目录。
    // let db_path = "smartpaste.db";

    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;

    // 确保表存在（幂等）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS data (
            id TEXT PRIMARY KEY NOT NULL, 
            item_type TEXT NOT NULL,
            content TEXT NOT NULL,
            is_favorite INTEGER NOT NULL,
            notes TEXT,
            timestamp INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    // let ts = data.timestamp.unwrap_or(0);

    conn.execute("INSERT OR REPLACE INTO data (id, item_type, content, is_favorite, notes, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            data.id,
            data.item_type,
            data.content,
            data.is_favorite as i32, // SQLite 使用整数表示布尔值
            data.notes,
            data.timestamp,
        ],
    )
        .map_err(|e| e.to_string())?;
    Ok("inserted".into())
}

/// 返回数据。作为 Tauri command 暴露给前端调用。
/// 根据数据 ID 返回对应的数据记录。
/// # Param
/// id: &str - 数据 ID
/// # Returns
/// Option<ClipboardDBItem> - 如果找到对应记录则返回 Some(记录)，否则返回 None
#[tauri::command]
pub fn get_data_by_id(id: &str) -> Result<Option<ClipboardDBItem>, String> {
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, item_type, content, is_favorite, notes, timestamp FROM data WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(ClipboardDBItem {
                id: row.get(0)?,
                item_type: row.get(1)?,
                content: row.get(2)?,
                is_favorite: row.get::<_, i32>(3)? != 0,
                notes: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    if let Some(result) = rows.next() {
        return Ok(Some(result.map_err(|e| e.to_string())?));
    }

    Ok(None)
}

/// 删除数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// data: ClipboardDBItem - 包含要删除数据的 ID 字段
#[tauri::command]
pub fn delete_data(data: ClipboardDBItem) -> Result<usize, String> {
    delete_data_by_id(&data.id)
}

/// 根据 ID 删除数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// id: &str - 要删除数据的 ID
#[tauri::command]
pub fn delete_data_by_id(id: &str) -> Result<usize, String> {
    // let db_path = "smartpaste.db";
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;

    let rows_affected = conn
        .execute("DELETE FROM data WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(rows_affected)
}

/// 根据 ID 收藏数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// id: &str - 要收藏数据的 ID
#[tauri::command]
pub fn favorite_data_by_id(id: &str) -> Result<usize, String> {
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;

    let rows_affected = conn
        .execute("UPDATE data SET is_favorite = 1 WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(rows_affected)
}

/// 文本搜索。作为 Tauri command 暴露给前端调用。
/// 根据传入的字符串，对所有属于 text 类的 content 字段进行模糊搜索，返回匹配的记录列表。
/// # Param
/// query: &str - 搜索关键词
/// # Returns
/// Vec<ClipboardDBItem> - 匹配的记录列表
#[tauri::command]
pub fn search_text_content(query: &str) -> Result<Vec<ClipboardDBItem>, String> {
    let conn = Connection::open(DB_PATH).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, item_type, content, is_favorite, notes, timestamp FROM data WHERE item_type = 'text' AND content LIKE ?1")
        .map_err(|e| e.to_string())?;

    let like_query = format!("%{}%", query);
    let clipboard_iter = stmt
        .query_map(params![like_query], |row| {
            Ok(ClipboardDBItem {
                id: row.get(0)?,
                item_type: row.get(1)?,
                content: row.get(2)?,
                is_favorite: row.get::<_, i32>(3)? != 0,
                notes: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for item in clipboard_iter {
        results.push(item.map_err(|e| e.to_string())?);
    }

    Ok(results)
}

/// # 单元测试
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn clear_db() {
        let _ = fs::remove_file("smartpaste.db");
    }

    #[test]
    fn test_insert_and_delete_and_get() {
        clear_db();
        let item = ClipboardDBItem {
            id: "ut-1".to_string(),
            item_type: "text".to_string(),
            content: "ut content".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };

        // 调用 insert（直接调用 Rust 函数）
        let res = insert_received_data(item.clone());
        assert!(res.is_ok());

        // 调用 get_data_by_id，验证插入结果
        let get_res = get_data_by_id(&item.id);
        assert!(get_res.is_ok());
        let fetched_item = get_res.unwrap();
        assert!(fetched_item.is_some());
        let fetched_item = fetched_item.unwrap();
        assert_eq!(fetched_item.id, item.id);
        assert_eq!(fetched_item.content, item.content);
        assert_eq!(fetched_item.item_type, item.item_type);
        assert_eq!(fetched_item.is_favorite, item.is_favorite);
        assert_eq!(fetched_item.notes, item.notes);
        assert_eq!(fetched_item.timestamp, item.timestamp);

        // 调用 delete
        let del_res = delete_data(item);
        assert!(del_res.is_ok());
        // 确认删除
        {
            let conn = rusqlite::Connection::open("smartpaste.db").unwrap();
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM data WHERE id = ?1", [&"ut-1"], |r| {
                    r.get(0)
                })
                .unwrap();
            assert_eq!(count, 0);
        }
    }

    #[test]
    fn test_favorite() {
        clear_db();
        let item1 = ClipboardDBItem {
            id: "ut-2".to_string(),
            item_type: "text".to_string(),
            content: "some content 2".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };
        let item2 = ClipboardDBItem {
            id: "ut-3".to_string(),
            item_type: "text".to_string(),
            content: "other content 3".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };

        // 插入两条数据
        let res1 = insert_received_data(item1.clone());
        assert!(res1.is_ok());
        let res2 = insert_received_data(item2.clone());
        assert!(res2.is_ok());

        // 收藏第二条数据
        let fav_res = favorite_data_by_id(&item1.id);
        assert!(fav_res.is_ok());
        let fav_res = favorite_data_by_id(&item2.id);
        assert!(fav_res.is_ok());

        // 验证收藏状态
        let get_res1 = get_data_by_id(&item1.id);
        assert!(get_res1.is_ok());
        let fetched_item1 = get_res1.unwrap().unwrap();
        assert!(fetched_item1.is_favorite);
        let get_res2 = get_data_by_id(&item2.id);
        assert!(get_res2.is_ok());
        let fetched_item2 = get_res2.unwrap().unwrap();
        assert!(fetched_item2.is_favorite);
    }

    #[test]
    fn test_search_text_content() {
        clear_db();
        let item1 = ClipboardDBItem {
            id: "ut-4".to_string(),
            item_type: "text".to_string(),
            content: "hello world".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };
        let item2 = ClipboardDBItem {
            id: "ut-5".to_string(),
            item_type: "text".to_string(),
            content: "goodbye world".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };
        let item3 = ClipboardDBItem {
            id: "ut-6".to_string(),
            item_type: "image".to_string(),
            content: "/path/to/image.png".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };

        // 插入三条数据
        assert!(insert_received_data(item1.clone()).is_ok());
        assert!(insert_received_data(item2.clone()).is_ok());
        assert!(insert_received_data(item3.clone()).is_ok());

        // 搜索包含 "world" 的文本内容
        let search_res = search_text_content("world");
        assert!(search_res.is_ok());
        let results = search_res.unwrap();
        assert_eq!(results.len(), 2);

        // 验证返回的记录是正确的
        let ids: Vec<String> = results.into_iter().map(|item| item.id).collect();
        assert!(ids.contains(&item1.id));
        assert!(ids.contains(&item2.id));
    }

    #[test]
    fn test_search_no_results() {
        clear_db();
        let search_res = search_text_content("nonexistent");
        assert!(search_res.is_ok());
        let results = search_res.unwrap();
        assert_eq!(results.len(), 0);
    }
}
// 未来扩展点：查询、批量写入、事务封装、连接池（r2d2 + rusqlite）等。
