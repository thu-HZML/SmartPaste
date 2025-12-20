use rusqlite::{params, Connection, Result as SqlResult};
use uuid::Uuid;
use std::path::{Path, PathBuf};
use std::fs;
use crate::clipboard::{ClipboardItem, clipboard_items_to_json, clipboard_item_to_json};
use super::{get_db_path, init_db, notify_cleanup};

/// 将接收到的数据插入数据库。
/// Param:
/// data: ClipboardItem - 要插入的数据项
/// Returns:
/// String - 插入的数据的 JSON 字符串。如果失败则返回错误信息
pub fn insert_received_db_data(data: ClipboardItem) -> Result<String, String> {
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

    // 通知后台清理线程进行实时裁剪
    notify_cleanup();

    clipboard_item_to_json(data)
}

/// 将接收到的文本数据插入数据库。作为 Tauri command 暴露给前端调用。
/// Param:
/// text: &str - 要插入的文本数据
/// Returns:
/// String - 插入的数据的 JSON 字符串。如果失败则返回错误信息
#[tauri::command]
pub fn insert_received_text_data(text: &str) -> Result<String, String> {
    let clipboard_item = ClipboardItem {
        id: Uuid::new_v4().to_string(),
        item_type: "text".to_string(),
        content: text.to_string(),
        size: Some(text.len() as u64),
        is_favorite: false,
        notes: "".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
    };
    insert_received_db_data(clipboard_item)
}

/// 将接收到的数据插入数据库。作为 Tauri command 暴露给前端调用。
/// Param:
/// data: String - 包含要插入数据的 JSON 字符串
/// Returns:
/// String - 插入的数据的 JSON 字符串。如果失败则返回错误信息
#[tauri::command]
pub fn insert_received_data(data: String) -> Result<String, String> {
    let clipboard_item: ClipboardItem = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    insert_received_db_data(clipboard_item)
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
        .prepare("SELECT id, item_type, content, size, is_favorite, notes, timestamp FROM data ORDER BY timestamp DESC") // 添加 ORDER BY
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

/// 删除所有数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// item_type: Option<&str> - 可选的数据类型过滤（如 "text", "image" 等），其他内容则视为folders的ID进行过滤
/// keep_favorites: bool - 是否保留已收藏记录
/// # Returns
/// usize - 受影响的行数
#[tauri::command]
pub fn delete_all_data(item_type: Option<&str>, keep_favorites: bool) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut sql = String::from("DELETE FROM data WHERE id IN (SELECT data.id FROM data");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let param_idx = 1;

    // 处理 item_type 逻辑
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
    sql.push_str(" WHERE 1=1");

    if let Some(folder_id) = folder_id_opt {
        sql.push_str(&format!(" AND folder_items.folder_id = ?{}", param_idx));
        params.push(Box::new(folder_id));
    } else if let Some(t) = type_filter_opt {
        sql.push_str(&format!(" AND data.item_type = ?{}", param_idx));
        params.push(Box::new(t));
    }

    if keep_favorites {
        sql.push_str(" AND data.is_favorite = 0");
    }

    sql.push_str(")");

    let rows_affected = conn
        .execute(
            &sql,
            rusqlite::params_from_iter(params.iter().map(|p| &**p)),
        )
        .map_err(|e| e.to_string())?;

    // 重新计算所有收藏夹的 item 数量
    conn.execute(
        "UPDATE folders SET num_items = (SELECT COUNT(*) FROM folder_items WHERE folder_items.folder_id = folders.id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(rows_affected)
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

    // ---------------------------------------------------------
    // 1. 在删除记录前，先查询该记录的文件路径
    // ---------------------------------------------------------
    let query_result: SqlResult<(String, String)> = conn.query_row(
        "SELECT item_type, content FROM data WHERE id = ?1",
        params![id],
        |row| Ok((row.get(0)?, row.get(1)?)), // 获取 item_type 和 content
    );

    if let Ok((item_type, content)) = query_result {
        // 获取当前配置的存储路径
        let storage_path = crate::config::get_current_storage_path();
        
        // 处理相对路径：如果是以 ".\files\" 或 "./files/" 开头的相对路径
        let file_path = if content.starts_with(r".\files\") || content.starts_with("./files/") || content.starts_with("files/") {
            // 从相对路径中提取文件名部分
            let file_name = if let Some(name) = content.split(r"\files\").last() {
                name.to_string()
            } else if let Some(name) = content.split(r"./files/").last() {
                name.to_string()
            } else if let Some(name) = content.split("files/").last() {
                name.to_string()
            } else {
                content.to_string()
            };
            
            // 构建完整路径：storage_path + "files" + 文件名
            storage_path.join("files").join(file_name)
        } else if content.starts_with(r"files\") {
            // 处理 files\xxx 格式
            let file_name = content.split(r"files\").last().unwrap_or(&content);
            storage_path.join("files").join(file_name)
        } else {
            // 如果不是相对路径，直接使用
            PathBuf::from(&content)
        };

        println!("🗑️ 尝试删除文件: {:?}", file_path);
        println!("🗑️ 存储根目录: {:?}", storage_path);

        // 检查路径是否存在
        if file_path.exists() {
            // ✅ 情况 A: 如果是文件夹类型 (或者物理路径确实是个文件夹)
            if item_type == "folder" || file_path.is_dir() {
                // 使用 remove_dir_all 递归删除文件夹及其内容
                if let Err(e) = fs::remove_dir_all(&file_path) {
                    eprintln!("⚠️ 删除本地文件夹失败 (ID: {}): {:?} - {}", id, file_path, e);
                } else {
                    println!("🗑️ 已删除关联的本地文件夹: {:?}", file_path);
                }
            }
            // ✅ 情况 B: 如果是图片或普通文件
            else if item_type == "image" || item_type == "file" || file_path.is_file() {
                // 使用 remove_file 删除单个文件
                if let Err(e) = fs::remove_file(&file_path) {
                    eprintln!("⚠️ 删除本地文件失败 (ID: {}): {:?} - {}", id, file_path, e);
                } else {
                    println!("🗑️ 已删除关联的本地文件: {:?}", file_path);
                }
            }
        } else {
            println!("ℹ️ 本地路径不存在，跳过物理删除: {:?}", file_path);
            // 尝试调试：打印可能的其他路径
            let alt_path = Path::new(&content);
            println!("ℹ️ 原始路径: {:?}", alt_path);
            if alt_path.exists() {
                println!("ℹ️ 原始路径存在，尝试删除");
                // 尝试删除原始路径
                if alt_path.is_dir() {
                    let _ = fs::remove_dir_all(alt_path);
                } else {
                    let _ = fs::remove_file(alt_path);
                }
            }
        }
    }

    // ---------------------------------------------------------
    // 2. 执行数据库删除
    // ---------------------------------------------------------
    let rows_affected = conn
        .execute("DELETE FROM data WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    // 重新计算所有收藏夹的 item 数量
    conn.execute(
        "UPDATE folders SET num_items = (SELECT COUNT(*) FROM folder_items WHERE folder_items.folder_id = folders.id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(rows_affected)
}

/// 根据 ID 修改数据内容。作为 Tauri command 暴露给前端调用。
/// # Param
/// id: &str - 要修改数据的 ID
/// new_content: &str - 新的内容
/// # Returns
/// String - 更新后的记录的 JSON 字符串
#[tauri::command]
pub fn update_data_content_by_id(id: &str, new_content: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE data SET content = ?1 WHERE id = ?2",
        params![new_content, id],
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

/// 更新file/folder/image数据的本地路径。作为 Tauri command 暴露给前端调用。
/// # Param
/// old_path: &str - 旧的本地路径
/// new_path: &str - 新的本地路径
/// # Returns
/// Result<usize, String> - 受影响的行数，如果失败则返回错误信息
#[tauri::command]
pub fn update_data_path(old_path: &str, new_path: &str) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let mut conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    println!("🔧 更新数据库中的文件路径...");
    println!("  旧路径: {}", old_path);
    println!("  新路径: {}", new_path);

    // 开启事务以确保数据一致性
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. 获取所有相关类型的记录
    let mut stmt = tx.prepare(
        "SELECT id, content FROM data WHERE item_type IN ('file', 'image', 'folder')"
    ).map_err(|e| e.to_string())?;

    let rows: Vec<(String, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?
    .filter_map(Result::ok)
    .collect();

    // 释放 statement 借用，以便后续使用 tx
    drop(stmt);

    let mut count = 0;

    // 2. 遍历并更新匹配的路径
    for (id, content) in rows {
        let mut updated = false;
        let mut new_content = content.clone();
        
        // 检查是否需要更新
        // 处理 Windows 路径分隔符问题
        let normalized_content = content.replace('\\', "/");
        let normalized_old_path = old_path.replace('\\', "/");
        
        // 检查是否以旧路径开头（处理绝对路径）
        if normalized_content.starts_with(&normalized_old_path) {
            // 替换前缀
            new_content = content.replacen(old_path, new_path, 1);
            updated = true;
        } 
        // 检查是否是相对路径（以 files/ 开头）
        else if normalized_content.starts_with("files/") || normalized_content.starts_with("./files/") || normalized_content.starts_with(r".\files\") {
            // 对于相对路径，我们需要更新存储路径，但相对路径保持不变
            // 这里不需要修改，因为相对路径相对于新的存储路径仍然有效
            println!("ℹ️ 记录 {} 使用相对路径，无需修改: {}", id, content);
        }
        // 检查是否是绝对路径但包含旧存储路径的其他形式
        else if let Some(relative_path) = normalized_content.split("/files/").last() {
            // 如果路径包含 "/files/"，尝试将其转换为新路径
            if relative_path != normalized_content {
                new_content = format!("{}/files/{}", new_path, relative_path);
                updated = true;
            }
        }
        
        if updated {
            println!("🔄 更新记录 {} 的路径:", id);
            println!("  旧路径: {}", content);
            println!("  新路径: {}", new_content);
            
            tx.execute(
                "UPDATE data SET content = ?1 WHERE id = ?2",
                params![new_content, id],
            ).map_err(|e| e.to_string())?;
            
            count += 1;
        }
    }

    // 提交事务
    tx.commit().map_err(|e| e.to_string())?;

    println!("✅ 数据库路径更新完成，共更新 {} 条记录", count);
    Ok(count)
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

/// 根据 ID 取消收藏数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// id: &str - 要取消收藏数据的 ID
/// # Returns
/// usize - 受影响的行数
#[tauri::command]
pub fn unfavorite_data_by_id(id: &str) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let rows_affected = conn
        .execute("UPDATE data SET is_favorite = 0 WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(rows_affected)
}

/// 按收藏状态筛选数据。作为 Tauri command 暴露给前端调用。
/// # Param
/// is_favorite: bool - 是否筛选收藏的数据
/// # Returns
/// String - 包含筛选后数据记录的 JSON 字符串
#[tauri::command]
pub fn filter_data_by_favorite(is_favorite: bool) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let fav_value = if is_favorite { 1 } else { 0 };

    let mut stmt = conn
        .prepare(
            "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
             FROM data 
             WHERE is_favorite = ?1
             ORDER BY timestamp DESC",
        )
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map(params![fav_value], |row| {
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

/// 获取 favorite 数据数量。作为 Tauri command 暴露给前端调用。
/// # Returns
/// usize - 收藏的数据数量
#[tauri::command]
pub fn get_favorite_data_count() -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let count: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM data WHERE is_favorite = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(count)
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
/// item_type: &str - 数据类型（如 "text", "image" 等）。
/// *(当输入 "folder" 或 "file" 时，会同时返回 folder 和 file 类型的数据)*
/// # Returns
/// String - 包含筛选后数据记录的 JSON 字符串
#[tauri::command]
pub fn filter_data_by_type(item_type: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let (sql, params) = if item_type == "folder" || item_type == "file" {
        // 当类型为 folder 或 file 时，同时返回两种类型的数据
        (
            "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
             FROM data 
             WHERE item_type IN ('folder', 'file')
             ORDER BY timestamp DESC",
            vec![],
        )
    } else {
        // 其他类型按原来的逻辑处理
        (
            "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
             FROM data 
             WHERE item_type = ?1
             ORDER BY timestamp DESC",
            vec![item_type],
        )
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

    let row_to_clipboard_item = |row: &rusqlite::Row| -> rusqlite::Result<ClipboardItem> {
        Ok(ClipboardItem {
            id: row.get(0)?,
            item_type: row.get(1)?,
            content: row.get(2)?,
            size: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
            is_favorite: row.get::<_, i32>(4)? != 0,
            notes: row.get(5)?,
            timestamp: row.get(6)?,
        })
    };

    let clipboard_iter = if params.is_empty() {
        stmt.query_map([], row_to_clipboard_item)
    } else {
        stmt.query_map(rusqlite::params![params[0]], row_to_clipboard_item)
    }
    .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for item in clipboard_iter {
        results.push(item.map_err(|e| e.to_string())?);
    }

    clipboard_items_to_json(results)
}

/// 根据ID将数据置顶。作为 Tauri command 暴露给前端调用。
/// # Param
/// id: &str - 要置顶数据的 ID
/// # Returns
/// String - 该修改后的数据记录的 JSON 字符串，若报错则返回错误信息
#[tauri::command]
pub fn top_data_by_id(id: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let current_timestamp = chrono::Utc::now().timestamp_millis();

    conn.execute(
        "UPDATE data SET timestamp = ?1 WHERE id = ?2",
        params![current_timestamp, id],
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