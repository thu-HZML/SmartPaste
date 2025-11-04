// src/clipboard.rs

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClipboardItem {
    pub id: String,
    pub item_type: String, // 数据类型：text/image/file
    pub content: String, // 对text类型，存储文本内容；对其他类型，存储文件路径
    pub is_favorite: bool,
    pub notes: String,
    pub timestamp: i64,
}