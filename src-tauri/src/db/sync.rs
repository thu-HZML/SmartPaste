use super::{get_db_path, init_db};
use crate::clipboard::{ClipboardItem, FolderItem};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FolderItemRelation {
    pub folder_id: String,
    pub item_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtendedData {
    pub item_id: String,
    pub ocr_text: Option<String>,
    pub icon_data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncData {
    pub data: Vec<ClipboardItem>,
    pub folders: Vec<FolderItem>,
    pub folder_items: Vec<FolderItemRelation>,
    pub extended_data: Vec<ExtendedData>,
}

/// 将云端返回的 JSON 数据同步到本地数据库（仅添加本地不存在的数据）
/// # Param
/// json_data: &str - 云端返回的 JSON 字符串
/// # Returns
/// Result<(), String> - 成功返回 Ok(()), 失败返回错误信息
#[tauri::command]
pub fn sync_cloud_data(json_data: &str) -> Result<(), String> {
    let sync_data: SyncData =
        serde_json::from_str(json_data).map_err(|e| format!("JSON 解析失败: {}", e))?;

    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let mut conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // 1. 插入 data 表
    {
        let mut stmt = tx.prepare(
            "INSERT OR IGNORE INTO data (id, item_type, content, size, is_favorite, notes, timestamp) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        ).map_err(|e| e.to_string())?;

        for item in sync_data.data {
            stmt.execute(params![
                item.id,
                item.item_type,
                item.content,
                item.size.unwrap_or(0) as i64,
                item.is_favorite as i32,
                item.notes,
                item.timestamp,
            ])
            .map_err(|e| e.to_string())?;
        }
    }

    // 2. 插入 folders 表
    {
        let mut stmt = tx
            .prepare(
                "INSERT OR IGNORE INTO folders (id, name, num_items) 
             VALUES (?1, ?2, ?3)",
            )
            .map_err(|e| e.to_string())?;

        for folder in sync_data.folders {
            stmt.execute(params![folder.id, folder.name, folder.num_items,])
                .map_err(|e| e.to_string())?;
        }
    }

    // 3. 插入 folder_items 表
    {
        let mut stmt = tx
            .prepare(
                "INSERT OR IGNORE INTO folder_items (folder_id, item_id) 
             VALUES (?1, ?2)",
            )
            .map_err(|e| e.to_string())?;

        for relation in sync_data.folder_items {
            stmt.execute(params![relation.folder_id, relation.item_id,])
                .map_err(|e| e.to_string())?;
        }
    }

    // 4. 插入 extended_data 表
    {
        let mut stmt = tx
            .prepare(
                "INSERT OR IGNORE INTO extended_data (item_id, ocr_text, icon_data) 
             VALUES (?1, ?2, ?3)",
            )
            .map_err(|e| e.to_string())?;

        for ext in sync_data.extended_data {
            stmt.execute(params![ext.item_id, ext.ocr_text, ext.icon_data,])
                .map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}
