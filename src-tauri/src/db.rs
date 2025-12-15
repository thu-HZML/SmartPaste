use rusqlite::{params, Connection, OptionalExtension, Result, Result as SqlResult};
use std::fs;
use uuid::Uuid;
// use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{path::Path, sync::RwLock}; 
use regex::Regex;
// use crate::config;
use std::sync::mpsc::Sender;
// use crate::clipboard::folder_item_to_json;
use crate::clipboard::clipboard_item_to_json;
use crate::clipboard::clipboard_items_to_json;
use crate::clipboard::folder_items_to_json;
use crate::clipboard::ClipboardItem;
use crate::clipboard::FolderItem;

// const DB_PATH: &str = "smartpaste.db";

static DB_PATH_GLOBAL: RwLock<Option<PathBuf>> = RwLock::new(None);
// 用于通知后台清理线程
static CLEANUP_SENDER: RwLock<Option<Sender<()>>> = RwLock::new(None);
/// 设置数据库路径
/// # Param
/// path: PathBuf - 数据库文件路径
pub fn set_db_path(path: PathBuf) {
    // 3. 使用 write() 锁来强制更新路径
    let mut db_path = DB_PATH_GLOBAL.write().unwrap();
    println!("🔄 数据库路径已在内存中更新为: {:?}", path); 
    *db_path = Some(path);
}
/// 获取数据库路径
/// # Returns
/// PathBuf - 数据库文件路径
fn get_db_path() -> PathBuf {
    // 4. 使用 read() 锁来获取当前路径
    let db_path = DB_PATH_GLOBAL.read().unwrap();
    db_path
        .clone()
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
            name TEXT NOT NULL,
            num_items INTEGER NOT NULL DEFAULT 0
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

    // 扩展数据表，存储非元数据的其他信息
    conn.execute(
        "CREATE TABLE IF NOT EXISTS extended_data (
            item_id TEXT PRIMARY KEY NOT NULL,
            ocr_text TEXT,
            icon_data TEXT,
            FOREIGN KEY (item_id) REFERENCES data(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 隐私表，存储标记为隐私的数据 ID 列表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS private_data (
            item_id TEXT PRIMARY KEY NOT NULL,
            FOREIGN KEY (item_id) REFERENCES data(id) ON DELETE CASCADE
        )",
        [],
    )?;
    Ok(())
}

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

/// 删除所有数据。作为 Tauri command 暴露给前端调用。
/// # Returns
/// usize - 受影响的行数
#[tauri::command]
pub fn delete_all_data() -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let rows_affected = conn
        .execute("DELETE FROM data", [])
        .map_err(|e| e.to_string())?;

    // 更新所有收藏夹的 item 数量为 0
    conn.execute("UPDATE folders SET num_items = 0", [])
        .map_err(|e| e.to_string())?;

    Ok(rows_affected)
}

/// 删除所有未收藏的数据。作为 Tauri command 暴露给前端调用。
/// # Returns
/// usize - 受影响的行数
#[tauri::command]
pub fn delete_unfavorited_data() -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let rows_affected = conn
        .execute("DELETE FROM data WHERE is_favorite = 0", [])
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
             WHERE is_favorite = ?1",
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
/// 搜索。作为 Tauri command 暴露给前端调用。
/// 根据传入的搜索关键词，以及传入的搜索类型，对所有 content 字段进行模糊搜索，返回匹配的记录列表。
/// # Param
/// search_type: &str - 搜索类型 ("text", "ocr", "path", "timestamp")
/// query: &str - 搜索关键词
/// - "text" 类型：待搜索的字符串关键词，在 content 字段中进行模糊匹配，只返回 text 类型数据
/// - "ocr" 类型：待搜索的字符串关键词，在 content 字段中进行模糊匹配，只返回 image 类型数据
/// - "path" 类型：待搜索的字符串关键词，在 content 字段中进行模糊匹配，返回 file、folder、image 类型数据
/// - "timestamp" 类型：待搜索的时间范围，格式为 "start_timestamp,end_timestamp"，在 timestamp 字段中进行范围匹配
/// # Returns
/// String - 包含匹配数据记录的 JSON 字符串，或者错误信息（如格式错误等）
#[tauri::command]
pub fn search_data(search_type: &str, query: &str) -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut results = Vec::new();

    match search_type {
        "timestamp" => {
            let parts: Vec<&str> = query.split(',').collect();
            if parts.len() != 2 {
                return Err("Invalid timestamp range format".to_string());
            }
            let start: i64 = parts[0]
                .parse()
                .map_err(|_| "Invalid start timestamp".to_string())?;
            let end: i64 = parts[1]
                .parse()
                .map_err(|_| "Invalid end timestamp".to_string())?;

            let mut stmt = conn
                .prepare(
                    "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
                     FROM data 
                     WHERE timestamp BETWEEN ?1 AND ?2",
                )
                .map_err(|e| e.to_string())?;

            let clipboard_iter = stmt
                .query_map(params![start, end], |row| {
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

            for item in clipboard_iter {
                results.push(item.map_err(|e| e.to_string())?);
            }
        }
        "text" => {
            let like_pattern = format!("%{}%", query);

            let mut stmt = conn
                .prepare(
                    "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
                     FROM data 
                     WHERE content LIKE ?1 AND item_type = 'text'",
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

            for item in clipboard_iter {
                results.push(item.map_err(|e| e.to_string())?);
            }
        }
        "ocr" => {
            let like_pattern = format!("%{}%", query);

            let mut stmt = conn
                .prepare(
                    "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
                     FROM data 
                     WHERE content LIKE ?1 AND item_type = 'image'",
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

            for item in clipboard_iter {
                results.push(item.map_err(|e| e.to_string())?);
            }
        }
        "path" => {
            let like_pattern = format!("%{}%", query);

            let mut stmt = conn
                .prepare(
                    "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
                     FROM data 
                     WHERE content LIKE ?1 AND item_type IN ('file', 'folder', 'image')",
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

            for item in clipboard_iter {
                results.push(item.map_err(|e| e.to_string())?);
            }
        }
        _ => {
            let like_pattern = format!("%{}%", query);

            let mut stmt = conn
                .prepare(
                    "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
                     FROM data 
                     WHERE content LIKE ?1",
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

            for item in clipboard_iter {
                results.push(item.map_err(|e| e.to_string())?);
            }
        }
    }
    clipboard_items_to_json(results)
}

// 文本搜索。作为 Tauri command 暴露给前端调用。
// 根据传入的字符串，对所有属于 text 类的 content 字段进行模糊搜索，返回匹配的记录列表。
// # Param
// query: &str - 搜索关键词
// # Returns
// String - 包含匹配数据记录的 JSON 字符串
// #[tauri::command]
// pub fn search_text_content(query: &str) -> Result<String, String> {
//     let db_path = get_db_path();
//     init_db(db_path.as_path()).map_err(|e| e.to_string())?;
//     let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

//     let like_pattern = format!("%{}%", query);

//     let mut stmt = conn
//         .prepare(
//             "SELECT id, item_type, content, size, is_favorite, notes, timestamp
//              FROM data
//              WHERE item_type = 'text' AND content LIKE ?1",
//         )
//         .map_err(|e| e.to_string())?;

//     let clipboard_iter = stmt
//         .query_map(params![like_pattern], |row| {
//             Ok(ClipboardItem {
//                 id: row.get(0)?,
//                 item_type: row.get(1)?,
//                 content: row.get(2)?,
//                 size: row.get::<_, Option<i64>>(3)?.map(|v| v as u64),
//                 is_favorite: row.get::<_, i32>(4)? != 0,
//                 notes: row.get(5)?,
//                 timestamp: row.get(6)?,
//             })
//         })
//         .map_err(|e| e.to_string())?;

//     let mut results = Vec::new();
//     for item in clipboard_iter {
//         results.push(item.map_err(|e| e.to_string())?);
//     }

//     clipboard_items_to_json(results)
// }

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
             WHERE item_type IN ('folder', 'file')",
            vec![],
        )
    } else {
        // 其他类型按原来的逻辑处理
        (
            "SELECT id, item_type, content, size, is_favorite, notes, timestamp 
             FROM data 
             WHERE item_type = ?1",
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

// ----------------------- 收藏夹相关操作 ------------------------

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

// ----------------------- 扩展数据相关操作 ------------------------

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

/// 利用备注内容匹配内容是否可能为密码，若匹配则标记或删除为隐私数据。作为 Tauri command 暴露给前端调用。
/// **匹配关键词**：
/// - "password"
/// - "密码"
/// - "pwd"
/// - "pass"
/// - "secret"
/// - "key"
/// - "token"
/// - "credential"
/// - "login"
/// - "auth"
/// - "authentication"
/// # Param
/// to_add: bool - 表示是否为增加隐私数据。若为true，则为添加隐私数据；若为false，则为删除隐私数据。
/// # Returns
/// Result<usize, String> - 受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn mark_passwords_as_private(to_add : bool) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let keywords = [
        "password", "密码", "pwd", "pass", "secret", "key", "token",
        "credential", "login", "auth", "authentication",
    ];

    let pattern = keywords
        .iter()
        .map(|kw| format!(r"(?i)\b{}\b", regex::escape(kw))) // 使用 \b 确保是完整单词匹配
        .collect::<Vec<String>>()
        .join("|");

    let regex = Regex::new(&pattern).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, notes FROM data WHERE item_type = 'text'")
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut count = 0;

    for item in clipboard_iter {
        let (id, notes) = item.map_err(|e| e.to_string())?;
        
        if regex.is_match(&notes) {
            if to_add {
                conn.execute(
                    "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                    params![id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                conn.execute(
                    "DELETE FROM private_data WHERE item_id = ?1",
                    params![id],
                )
                .map_err(|e| e.to_string())?;
            }
            count += 1;
        }
    }

    Ok(count)
}

/// 辅助函数：实现银行卡号的 Luhn 算法校验
/// # Param
/// card_number: &str - 银行卡号字符串
/// # Returns
/// bool - 是否通过 Luhn 校验
fn is_valid_luhn(card_number: &str) -> bool {
    let card_number = card_number.replace(|c: char| c.is_whitespace() || c == '-', "");
    
    // Luhn 算法只适用于纯数字串
    if card_number.is_empty() || !card_number.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let sum = card_number
        .chars()
        .rev() // 从右向左遍历（从校验位开始）
        .enumerate()
        .map(|(i, c)| {
            let mut digit = c.to_digit(10).unwrap();
            
            // 偶数索引（从 0 开始，即第二位、第四位...）执行乘 2
            if i % 2 != 0 {
                digit *= 2;
                if digit > 9 {
                    digit -= 9; // 相当于相加
                }
            }
            digit
        })
        .sum::<u32>();

    sum % 10 == 0
}

/// 利用正则表达式匹配并使用 Luhn 算法校验内容是否可能为银行卡号 (PAN)，
/// 若匹配且校验通过，则标记为隐私数据。
/// # Param
/// to_add: bool - 表示是否为增加隐私数据。若为true，则为添加隐私数据；若为false，则为删除隐私数据。
/// # Returns
/// Result<usize, String> - 受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn mark_bank_cards_as_private(to_add: bool) -> Result<usize, String> {
    // 假设 db_path 和 conn 已经初始化并处理错误
// ----------------------- 扩展功能 ------------------------

/// 按配置中的天数清理过期数据，自动屏蔽未收藏的数据。
/// # Param
/// days: u32 - 过期天数
/// # Returns
/// Result<usize, String> - 被删除的记录数量，若失败则返回错误信息
pub fn clear_data_expired(days: u32) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // 银行卡号的正则表达式 (包含 IIN/BIN 规则，允许空格或连字符作为分隔符)
    // PAN 长度通常在 13-19 位之间，且符合特定 IIN 范围。
    // 使用非捕获分组 `(?:...)` 和 `\b` 边界，并允许分隔符 `[\s-]?`
    let pan_regex = Regex::new(r"(?x)
        \b
        (?:
            # Visa (4xxxx): 13, 16, 19位
            4\d{3}[\s-]?\d{4}[\s-]?\d{4}(?:[\s-]?\d{4}(?:[\s-]?\d{3})?)? |
            
            # Mastercard (51-55 或 2221-2720): 16位
            (5[1-5]|222[1-9]|22[3-9]|2[3-6]|27[0-2])\d{2}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4} |
            
            # Amex (34, 37): 15位，分组通常是 4-6-5
            3[47]\d{2}[\s-]?\d{6}[\s-]?\d{5} |

            # Discover/Diners/JCB 等其他主要卡段 (14-19位)
            (3(?:0[0-5]|[689])|6(?:011|5\d{2}|4[4-9]\d{1}))\d{10,15}
        )
        \b
    ").map_err(|e| e.to_string())?;

    // 查询所有文本类型的数据
    let mut stmt = conn
        .prepare("SELECT id, content FROM data WHERE item_type = 'text'")
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut count = 0;

    for item in clipboard_iter {
        let (id, content) = item.map_err(|e| e.to_string())?;
        
        // 1. 正则初步筛选：查找所有潜在的卡号匹配项
        for capture in pan_regex.captures_iter(&content) {
            let potential_pan = &capture[0]; // 捕获整个匹配串（可能包含分隔符）
            
            // 2. 移除分隔符并执行 Luhn 校验
            if is_valid_luhn(potential_pan) {
                if to_add {
                    // 标记为隐私数据，即添加到private_data表中
                    conn.execute(
                        "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                        params![id],
                    )
                    .map_err(|e| e.to_string())?;
                } else {
                    // 取消标记为隐私数据
                    conn.execute(
                        "DELETE FROM private_data WHERE item_id = ?1",
                        params![id],
                    )
                    .map_err(|e| e.to_string())?;
                }
                
                count += 1;
                // 一旦该记录中找到一个有效的卡号，就可以停止检查并进入下一条记录
                break; 
            }
        }
    }

    Ok(count)
}

/// 利用正则表示匹配内容是否可能为身份证号，若匹配则标记为隐私数据。
/// # Param
/// to_add: bool - 表示是否为增加隐私数据。若为true，则为添加隐私数据；若为false，则为删除隐私数据。
/// # Returns
/// Result<usize, String> - 受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn mark_identity_numbers_as_private(to_add: bool) -> Result<usize, String> {
    let cutoff_timestamp = chrono::Utc::now().timestamp() - (days as i64 * 86400);

    let rows_deleted = conn
        .execute(
            "DELETE FROM data WHERE timestamp < ?1 AND is_favorite = 0",
            params![cutoff_timestamp],
        )
        .map_err(|e| e.to_string())?;

    Ok(rows_deleted)
}

/// 利用正则表达式匹配内容是否可能为手机号，若匹配则标记为隐私数据。
/// # Param
/// to_add: bool - 表示是否为增加隐私数据。若为true，则为添加隐私数据；若为false，则为删除隐私数据。
/// # Returns
/// Result<usize, String> - 受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn mark_phone_numbers_as_private(to_add: bool) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // 手机号的正则表达式（简单版本，适用于中国手机号）
    let phone_regex = Regex::new(r"\b1[3-9]\d{9}\b").map_err(|e| e.to_string())?;

    // 查询所有文本类型的数据
    let mut stmt = conn
        .prepare("SELECT id, content FROM data WHERE item_type = 'text'")
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut count = 0;

    for item in clipboard_iter {
        let (id, content) = item.map_err(|e| e.to_string())?;
        if phone_regex.is_match(&content) {
            if to_add {
                // 标记为隐私数据，也即添加到private_data表中
                conn.execute(
                    "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                    params![id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                // 取消标记为隐私数据
                conn.execute(
                    "DELETE FROM private_data WHERE item_id = ?1",
                    params![id],
                )
                .map_err(|e| e.to_string())?;
            }
            count += 1;
        }
    }

    Ok(count)
}
/// 返回所有被标记为隐私的数据项。作为 Tauri command 暴露给前端调用。
/// # Returns
/// String - 包含隐私数据记录的 JSON 字符串，若失败则返回错误信息
#[tauri::command]
pub fn get_all_private_data() -> Result<String, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.item_type, d.content, d.size, d.is_favorite, d.notes, d.timestamp
             FROM data d
             JOIN private_data pd ON d.id = pd.item_id",
        )
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

/// 清除所有隐私数据。作为 Tauri command 暴露给前端调用。
/// # Returns
/// Result<usize, String> - 受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn clear_all_private_data() -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let rows = conn
        .execute("DELETE FROM private_data", [])
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

/// 根据配置文件的选项，自动设置隐私数据标记。作为 Tauri command 暴露给前端调用。
/// # Param
/// password_flag: bool - 是否标记密码
/// bank_card_flag: bool - 是否标记银行卡号
/// id_number_flag: bool - 是否标记身份证号
/// phone_number_flag: bool - 是否标记手机号
/// # Returns
/// Result<usize, String> - 最终受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn auto_mark_private_data(
    password_flag: bool,
    bank_card_flag: bool,
    id_number_flag: bool,
    phone_number_flag: bool,
) -> Result<usize, String> {
    let mut total_count = 0;

    total_count += mark_passwords_as_private(password_flag)?;
    total_count += mark_bank_cards_as_private(bank_card_flag)?;
    total_count += mark_identity_numbers_as_private(id_number_flag)?;
    total_count += mark_phone_numbers_as_private(phone_number_flag)?;
    Ok(total_count)
    // 计算需要删除的记录数量
    let total_count: u32 = conn
        .query_row("SELECT COUNT(*) FROM data WHERE is_favorite = 0", [], |row| {
            row.get(0)
        })
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
  
/// 按设定的最大历史记录数量删除多余的数据，自动屏蔽未收藏的数据。
/// 删除优先级：按照时间戳从旧到新排序，删除最旧的数据。
/// # Param
/// max_items: usize - 最大历史记录数量
/// # Returns
/// Result<usize, String> - 被删除的记录数量，若失败则返回错误信息
pub fn enforce_max_history_items(max_items: u32) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // 身份证号的正则表达式（简单版本）
    let id_regex = Regex::new(r"\b\d{15}\b|\b\d{18}\b|\b\d{17}X\b").map_err(|e| e.to_string())?;

    // 查询所有文本类型的数据
    let mut stmt = conn
        .prepare("SELECT id, content FROM data WHERE item_type = 'text'")
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut count = 0;

    for item in clipboard_iter {
        let (id, content) = item.map_err(|e| e.to_string())?;
        if id_regex.is_match(&content) {
            if to_add {
                // 标记为隐私数据，也即添加到private_data表中
                conn.execute(
                    "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                    params![id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                // 取消标记为隐私数据
                conn.execute(
                    "DELETE FROM private_data WHERE item_id = ?1",
                    params![id],
                )
                .map_err(|e| e.to_string())?;
            }
            count += 1;
        }
    }

    Ok(count)
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

/// # 单元测试
#[cfg(test)]
#[path = "test_unit/test_db_base.rs"]
mod test_db_base;
#[path = "test_unit/test_db_adv.rs"]
mod test_db_adv;
#[path = "test_unit/test_db_folder.rs"]
mod test_db_folder;
#[path = "test_unit/test_db_private.rs"]
mod test_db_private;
#[path = "test_unit/test_db_extend.rs"]
mod test_db_extend;
