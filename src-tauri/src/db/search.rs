use super::{get_db_path, init_db};
use crate::clipboard::{clipboard_items_to_json, ClipboardItem};
use rusqlite::Connection;

/// # Param
/// query: &str - 搜索关键词，可以在 content/notes/ocr_text 字段中进行模糊匹配
/// item_type: Option<&str> - 可选的数据类型过滤（如 "text", "image" 等），其他内容则视为folders的ID进行过滤
/// start_timestamp: Option<i64> - 可选的起始时间戳过滤
/// end_timestamp: Option<i64> - 可选的结束时间戳过滤
/// # Returns
/// String - 包含匹配数据记录的 JSON 字符串，或者错误信息
#[tauri::command]
pub fn comprehensive_search(
    query: &str,
    item_type: Option<&str>,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut sql = String::from(
        "SELECT data.id, data.item_type, data.content, data.size, data.is_favorite, data.notes, data.timestamp 
         FROM data 
         LEFT JOIN extended_data ON data.id = extended_data.item_id",
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(format!("%{}%", query))];
    let mut param_idx = 2;

    // 处理 item_type 逻辑：标准类型 vs 收藏夹ID
    let mut folder_id_opt = None;
    let mut type_filter_opt = None;

    if let Some(t) = item_type {
        match t {
            "text" | "image" | "file" | "folder" => {
                type_filter_opt = Some(t);
            }
            "private" => {
                sql.push_str(" JOIN private_data ON data.id = private_data.item_id");
            }
            _ => {
                // 视为 Folder ID
                folder_id_opt = Some(t);
                sql.push_str(" JOIN folder_items ON data.id = folder_items.item_id");
            }
        }
    }

    // WHERE 子句
    sql.push_str(
        " WHERE (data.content LIKE ?1 OR data.notes LIKE ?1 OR extended_data.ocr_text LIKE ?1)",
    );

    if let Some(folder_id) = folder_id_opt {
        sql.push_str(&format!(" AND folder_items.folder_id = ?{}", param_idx));
        params.push(Box::new(folder_id));
        param_idx += 1;
    } else if let Some(t) = type_filter_opt {
        sql.push_str(&format!(" AND data.item_type = ?{}", param_idx));
        params.push(Box::new(t));
        param_idx += 1;
    }

    if let (Some(start), Some(end)) = (start_timestamp, end_timestamp) {
        sql.push_str(&format!(
            " AND data.timestamp BETWEEN ?{} AND ?{}",
            param_idx,
            param_idx + 1
        ));
        params.push(Box::new(start));
        params.push(Box::new(end));
    }

    sql.push_str(" ORDER BY data.timestamp DESC");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map(
            rusqlite::params_from_iter(params.iter().map(|p| &**p)),
            |row| {
                Ok(ClipboardItem {
                    id: row.get(0)?,
                    item_type: row.get(1)?,
                    content: row.get(2)?,
                    size: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
                    is_favorite: row.get::<_, i32>(4)? != 0,
                    notes: row.get(5)?,
                    timestamp: row.get(6)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for item in clipboard_iter {
        results.push(item.map_err(|e| e.to_string())?);
    }

    clipboard_items_to_json(results)
}
