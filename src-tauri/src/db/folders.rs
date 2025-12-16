use super::{get_db_path, init_db};
use crate::clipboard::{clipboard_items_to_json, folder_items_to_json, ClipboardItem, FolderItem};
use rusqlite::{params, Connection};
use uuid::Uuid;

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
        "INSERT INTO folders (id, name, num_items) VALUES (?1, ?2, ?3)",
        params![id, name, 0],
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

    conn.execute("DELETE FROM folders WHERE id = ?1", params![folder_id])
        .map_err(|e| e.to_string())?;

    Ok("deleted".to_string())
}

/// 返回所有收藏夹的列表。作为 Tauri command 暴露给前端调用。
/// # Returns
/// String - 包含所有收藏夹项的 JSON 字符串
#[tauri::command]
pub fn get_all_folders() -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name , num_items FROM folders")
        .map_err(|e| e.to_string())?;

    let folder_iter = stmt
        .query_map([], |row| {
            Ok(FolderItem {
                id: row.get(0)?,
                name: row.get(1)?,
                num_items: row.get::<_, i64>(2)? as u32,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for item in folder_iter {
        results.push(item.map_err(|e| e.to_string())?);
    }

    folder_items_to_json(results)
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

    let rows = conn
        .execute(
            "INSERT OR IGNORE INTO folder_items (folder_id, item_id) VALUES (?1, ?2)",
            params![folder_id, item_id],
        )
        .map_err(|e| e.to_string())?;

    if rows > 0 {
        conn.execute(
            "UPDATE folders SET num_items = num_items + 1 WHERE id = ?1",
            params![folder_id],
        )
        .map_err(|e| e.to_string())?;
    }

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

    let rows = conn
        .execute(
            "DELETE FROM folder_items WHERE folder_id = ?1 AND item_id = ?2",
            params![folder_id, item_id],
        )
        .map_err(|e| e.to_string())?;

    if rows > 0 {
        conn.execute(
            "UPDATE folders SET num_items = num_items - 1 WHERE id = ?1 AND num_items > 0",
            params![folder_id],
        )
        .map_err(|e| e.to_string())?;
    }

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

/// 根据 item ID 查阅数据所属的所有收藏夹。作为 Tauri command 暴露给前端调用。
/// # Param
/// item_id: &str - 数据项 ID
/// # Returns
/// String - 包含所属收藏夹列表的 JSON 字符串，若失败则返回错误信息
#[tauri::command]
pub fn get_folders_by_item_id(item_id: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.name, f.num_items
             FROM folders f
             JOIN folder_items fi ON f.id = fi.folder_id
             WHERE fi.item_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let folder_iter = stmt
        .query_map(params![item_id], |row| {
            Ok(FolderItem {
                id: row.get(0)?,
                name: row.get(1)?,
                num_items: row.get::<_, i64>(2)? as u32,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for item in folder_iter {
        results.push(item.map_err(|e| e.to_string())?);
    }

    folder_items_to_json(results)
}
