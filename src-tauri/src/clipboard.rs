// src/clipboard.rs

use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::{OnceLock, RwLock};

/// 剪贴板数据项结构体
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClipboardItem {
    pub id: String,
    pub item_type: String, // 数据类型：text/image/file
    pub content: String,   // 对text类型，存储文本内容；对其他类型，存储文件路径
    pub size: Option<u64>, // 文件大小。对text类型，为文本长度（字符数）；对file/image类型，为文件字节大小
    pub is_favorite: bool,
    pub notes: String,
    pub timestamp: i64,
}

/// 收藏夹结构体
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FolderItem {
    pub id: String,
    pub name: String,
    pub num_items: u32,
}

/// 将 FolderItem 转换为 JSON 字符串。
/// # Param
/// item: FolderItem - 要转换的收藏夹项
/// # Returns
/// String - 转换后的 JSON 字符串
pub fn folder_item_to_json(item: FolderItem) -> Result<String, String> {
    serde_json::to_string(&item).map_err(|e| e.to_string())
}

/// 将 FolderItem 列表转换为 JSON 字符串。
/// # Param
/// items: Vec<FolderItem> - 要转换的收藏夹项列表
/// # Returns
/// String - 转换后的 JSON 字符串
pub fn folder_items_to_json(items: Vec<FolderItem>) -> Result<String, String> {
    serde_json::to_string(&items).map_err(|e| e.to_string())
}

/// 将 ClipboardItem 转换为 JSON 字符串。
/// # Param
/// item: ClipboardItem - 要转换的剪贴板项
/// # Returns
/// String - 转换后的 JSON 字符串
pub fn clipboard_item_to_json(item: ClipboardItem) -> Result<String, String> {
    serde_json::to_string(&item).map_err(|e| e.to_string())
}
/// 将 ClipboardItem 列表转换为 JSON 字符串。
/// # Param
/// items: Vec<ClipboardItem> - 要转换的剪贴板项列表
/// # Returns
/// String - 转换后的 JSON 字符串
pub fn clipboard_items_to_json(items: Vec<ClipboardItem>) -> Result<String, String> {
    serde_json::to_string(&items).map_err(|e| e.to_string())
}

// 全局保存最后一次插入的数据（线程安全，可克隆取出）
static LAST_INSERTED: OnceLock<RwLock<Option<ClipboardItem>>> = OnceLock::new();

/// 设置最后一次插入的数据（覆盖）
/// # Param:
/// item: ClipboardItem - 要设置的数据项
pub fn set_last_inserted(item: ClipboardItem) {
    let lock = LAST_INSERTED.get_or_init(|| RwLock::new(None));
    let mut w = lock.write().unwrap_or_else(|p| p.into_inner());
    *w = Some(item);
}

/// 以克隆方式读取最后一次插入的数据（不移除）
/// # Returns:
/// Option<ClipboardItem> - 克隆的最后插入数据项，若无则返回 None
pub fn get_last_inserted() -> Option<ClipboardItem> {
    let lock = LAST_INSERTED.get_or_init(|| RwLock::new(None));
    let r = lock.read().unwrap_or_else(|p| p.into_inner());
    r.clone()
}

/// 取出并清空最后一次插入的数据（移动语义）
/// # Returns:
/// Option<ClipboardItem> - 取出的最后插入数据项，若无则返回 None
pub fn take_last_inserted() -> Option<ClipboardItem> {
    let lock = LAST_INSERTED.get_or_init(|| RwLock::new(None));
    let mut w = lock.write().unwrap_or_else(|p| p.into_inner());
    w.take()
}
