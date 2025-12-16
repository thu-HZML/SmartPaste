use rusqlite::{params, Connection, OptionalExtension};
use crate::clipboard::{ClipboardItem, clipboard_items_to_json};
use super::{get_db_path, init_db};

/// 插入 OCR 文本数据。
/// # Param
/// item_id: &str - 数据项 ID
/// ocr_text: &str - OCR 识别的文本内容
/// # Returns
/// String - 信息。若插入成功返回 "ocr inserted"，否则返回错误信息
pub fn insert_ocr_text(item_id: &str, ocr_text: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO extended_data (item_id, ocr_text) VALUES (?1, ?2)",
        params![item_id, ocr_text],
    )
    .map_err(|e| e.to_string())?;

    Ok("ocr inserted".to_string())
}

/// 返回对应数据项的 OCR 文本。作为 Tauri command 暴露给前端调用。
/// # Param
/// item_id: &str - 数据项 ID
/// # Returns
/// String - 包含 OCR 文本的字符串，若无则返回空字符串
#[tauri::command]
pub fn get_ocr_text_by_item_id(item_id: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT ocr_text FROM extended_data WHERE item_id = ?1")
        .map_err(|e| e.to_string())?;

    let ocr_text: Option<String> = stmt
        .query_row(params![item_id], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(ocr_text.unwrap_or_default())
}

/// 按 OCR 文本搜索数据项。作为 Tauri command 暴露给前端调用。
/// # Param
/// query: &str - 搜索关键词
/// # Returns
/// String - 包含匹配数据记录的 JSON 字符串
#[tauri::command]
pub fn search_data_by_ocr_text(query: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let like_pattern = format!("%{}%", query);

    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.item_type, d.content, d.size, d.is_favorite, d.notes, d.timestamp
             FROM data d
             JOIN extended_data ed ON d.id = ed.item_id
             WHERE ed.ocr_text LIKE ?1",
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

/// 插入 icon_data 数据。
/// # Param
/// item_id: &str - 数据项 ID
/// icon_data: &str - 图标数据（Base64 编码字符串）
/// # Returns
/// String - 信息。若插入成功返回 "icon_data inserted"，否则返回错误信息
pub fn insert_icon_data(item_id: &str, icon_data: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO extended_data (item_id, icon_data) VALUES (?1, ?2)",
        params![item_id, icon_data],
    )
    .map_err(|e| e.to_string())?;

    Ok("icon_data inserted".to_string())
}

/// 根据 item ID 获取 icon_data 数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// item_id: &str - 数据项 ID
/// # Returns
/// String - 包含 icon_data 的字符串，若无则返回空字符串
#[tauri::command]
pub fn get_icon_data_by_item_id(item_id: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT icon_data FROM extended_data WHERE item_id = ?1")
        .map_err(|e| e.to_string())?;

    let icon_data: Option<String> = stmt
        .query_row(params![item_id], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(icon_data.unwrap_or_default())
}
