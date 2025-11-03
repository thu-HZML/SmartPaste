use rusqlite::{params, Connection, Result};
// use serde::{Deserialize, Serialize};
use serde_json;
use std::path::PathBuf;
use std::{path::Path, sync::OnceLock};

use crate::ClipboardItem;

// const DB_PATH: &str = "smartpaste.db";

static DB_PATH_GLOBAL: OnceLock<PathBuf> = OnceLock::new();

/// 将 ClipboardItem 转换为 JSON 字符串。作为 Tauri command 暴露给前端调用。
/// # Param
/// item: ClipboardItem - 要转换的剪贴板项
#[tauri::command]
pub fn clipboard_item_to_json(item: ClipboardItem) -> Result<String, String> {
    serde_json::to_string(&item).map_err(|e| e.to_string())
}
/// 将 ClipboardItem 列表转换为 JSON 字符串。作为 Tauri command 暴露给前端调用。
/// # Param
/// items: Vec<ClipboardItem> - 要转换的剪贴板项列表
#[tauri::command]
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
            is_favorite INTEGER NOT NULL,
            notes TEXT,
            timestamp INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

/// 将接收到的数据插入数据库。作为 Tauri command 暴露给前端调用。
/// TODO: 根据实际需求调整插入逻辑
///
#[tauri::command]
pub fn insert_received_data(data: ClipboardItem) -> Result<String, String> {
    // NOTE: 这里我们把数据库文件放在工作目录下的 smartpaste.db 中。
    // 更稳妥的做法是在运行时从 `tauri::api::path::app_dir` 或 `app.path_resolver()` 获取应用本地数据目录。
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

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

/// 获取所有数据。作为 Tauri command 暴露给前端调用。
/// # Returns
/// Vec<ClipboardDBItem> - 所有数据记录列表
#[tauri::command]
pub fn get_all_data() -> Result<Vec<ClipboardItem>, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, item_type, content, is_favorite, notes, timestamp FROM data")
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map([], |row| {
            Ok(ClipboardItem {
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

/// 返回数据。作为 Tauri command 暴露给前端调用。
/// 根据数据 ID 返回对应的数据记录。
/// # Param
/// id: &str - 数据 ID
/// # Returns
/// Option<ClipboardDBItem> - 如果找到对应记录则返回 Some(记录)，否则返回 None
#[tauri::command]
pub fn get_data_by_id(id: &str) -> Result<Option<ClipboardItem>, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, item_type, content, is_favorite, notes, timestamp FROM data WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            Ok(ClipboardItem {
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

/// 根据 ID 收藏数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// id: &str - 要收藏数据的 ID
/// # Returns
/// usize - 受影响的行数
#[tauri::command]
pub fn favorite_data_by_id(id: &str) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

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
pub fn search_text_content(query: &str) -> Result<Vec<ClipboardItem>, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, item_type, content, is_favorite, notes, timestamp FROM data WHERE item_type = 'text' AND content LIKE ?1")
        .map_err(|e| e.to_string())?;

    let like_query = format!("%{}%", query);
    let clipboard_iter = stmt
        .query_map(params![like_query], |row| {
            Ok(ClipboardItem {
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

/// 增加备注。作为 Tauri command 暴露给前端调用。
/// # Param
/// id: &str - 数据 ID
/// notes: &str - 备注内容
/// # Returns
/// ClipboardDBItem - 更新后的数据记录
#[tauri::command]
pub fn add_notes_by_id(id: &str, notes: &str) -> Result<ClipboardItem, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE data SET notes = ?1 WHERE id = ?2",
        params![notes, id],
    )
    .map_err(|e| e.to_string())?;

    // 返回更新后的记录
    get_data_by_id(id)?.ok_or_else(|| "Record not found after adding notes".to_string())
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
    use std::fs;

    fn clear_db() {
        let _ = fs::remove_file("smartpaste.db");
    }

    #[test]
    fn test_insert_and_delete_and_get() {
        clear_db();
        let item = ClipboardItem {
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
        let item1 = ClipboardItem {
            id: "ut-2".to_string(),
            item_type: "text".to_string(),
            content: "some content 2".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };
        let item2 = ClipboardItem {
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
        let item1 = ClipboardItem {
            id: "ut-4".to_string(),
            item_type: "text".to_string(),
            content: "hello world".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };
        let item2 = ClipboardItem {
            id: "ut-5".to_string(),
            item_type: "text".to_string(),
            content: "goodbye world".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };
        let item3 = ClipboardItem {
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
    fn test_get_all_data() {
        clear_db();
        let item1 = ClipboardItem {
            id: "ut-7".to_string(),
            item_type: "text".to_string(),
            content: "data one".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };
        let item2 = ClipboardItem {
            id: "ut-8".to_string(),
            item_type: "image".to_string(),
            content: "/path/to/image2.png".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };

        // 插入两条数据
        assert!(insert_received_data(item1.clone()).is_ok());
        assert!(insert_received_data(item2.clone()).is_ok());

        // 获取所有数据
        let all_data_res = get_all_data();
        assert!(all_data_res.is_ok());
        let all_data = all_data_res.unwrap();
        assert_eq!(all_data.len(), 2);

        let ids: Vec<String> = all_data.into_iter().map(|item| item.id).collect();
        assert!(ids.contains(&item1.id));
        assert!(ids.contains(&item2.id));
    }

    #[test]
    fn test_add_notes() {
        clear_db();
        let item = ClipboardItem {
            id: "ut-9".to_string(),
            item_type: "text".to_string(),
            content: "note test".to_string(),
            is_favorite: false,
            notes: "".to_string(),
            timestamp: 0,
        };

        // 插入数据
        assert!(insert_received_data(item.clone()).is_ok());

        // 添加备注
        let notes_content = "This is a test note.";
        let add_notes_res = add_notes_by_id(&item.id, notes_content);
        assert!(add_notes_res.is_ok());
        let updated_item = add_notes_res.unwrap();
        assert_eq!(updated_item.notes, notes_content);

        // 验证通过 get_data_by_id 获取的记录也包含备注
        let get_res = get_data_by_id(&item.id);
        assert!(get_res.is_ok());
        let fetched_item = get_res.unwrap().unwrap();
        assert_eq!(fetched_item.notes, notes_content);
    }

    #[test]
    fn test_create_new_folder() {
        clear_db();
        let folder_name = "testfolder";

        // 创建新收藏夹
        let create_res = create_new_folder(folder_name);
        assert!(create_res.is_ok());
        let msg = create_res.unwrap();
        assert_eq!(msg, format!("收藏夹 '{}' 已创建", folder_name));

        // 尝试创建同名收藏夹，应该提示已存在
        let create_res2 = create_new_folder(folder_name);
        assert!(create_res2.is_ok());
        let msg2 = create_res2.unwrap();
        assert_eq!(msg2, format!("收藏夹 '{}' 已存在", folder_name));
    }

    // #[test]
    // fn test_search_no_results() {
    //     clear_db();
    //     let search_res = search_text_content("nonexistent");
    //     assert!(search_res.is_ok());
    //     let results = search_res.unwrap();
    //     assert_eq!(results.len(), 0);
    // }
}

mod test_to_json {
    use super::*;
    #[test]
    fn test_clipboard_item_to_json() {
        let item = ClipboardItem {
            id: "ut-10".to_string(),
            item_type: "text".to_string(),
            content: "json test".to_string(),
            is_favorite: true,
            notes: "some notes".to_string(),
            timestamp: 1234567890,
        };

        let json_res = clipboard_item_to_json(item.clone());
        assert!(json_res.is_ok());
        let json_str = json_res.unwrap();

        let expected_json = r#"{"id":"ut-10","item_type":"text","content":"json test","is_favorite":true,"notes":"some notes","timestamp":1234567890}"#;
        assert_eq!(json_str, expected_json);
    }

    #[test]
    fn test_clipboard_items_to_json() {
        let items = vec![
            ClipboardItem {
                id: "ut-11".to_string(),
                item_type: "text".to_string(),
                content: "first item".to_string(),
                is_favorite: false,
                notes: "".to_string(),
                timestamp: 1111111111,
            },
            ClipboardItem {
                id: "ut-12".to_string(),
                item_type: "image".to_string(),
                content: "/path/to/image.png".to_string(),
                is_favorite: true,
                notes: "image note".to_string(),
                timestamp: 2222222222,
            },
        ];

        let json_res = clipboard_items_to_json(items.clone());
        assert!(json_res.is_ok());
        let json_str = json_res.unwrap();

        let expected_json = r#"[{"id":"ut-11","item_type":"text","content":"first item","is_favorite":false,"notes":"","timestamp":1111111111},{"id":"ut-12","item_type":"image","content":"/path/to/image.png","is_favorite":true,"notes":"image note","timestamp":2222222222}]"#;
        assert_eq!(json_str, expected_json);
    }
}
// 未来扩展点：查询、批量写入、事务封装、连接池（r2d2 + rusqlite）等。
