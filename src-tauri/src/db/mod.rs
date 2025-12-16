use rusqlite::{Connection, Result};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

pub mod cleanup;
pub mod core;
pub mod extended;
pub mod folders;
pub mod privacy;
pub mod search;

pub use self::cleanup::*;
pub use self::core::*;
pub use self::extended::*;
pub use self::folders::*;
pub use self::privacy::*;
pub use self::search::*;

static DB_PATH_GLOBAL: RwLock<Option<PathBuf>> = RwLock::new(None);

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
pub(crate) fn get_db_path() -> PathBuf {
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

#[cfg(test)]
#[path = "../test_unit/test_db_adv.rs"]
mod test_db_adv;
/// # 单元测试
#[cfg(test)]
#[path = "../test_unit/test_db_base.rs"]
mod test_db_base;
#[cfg(test)]
#[path = "../test_unit/test_db_extend.rs"]
mod test_db_extend;
#[cfg(test)]
#[path = "../test_unit/test_db_folder.rs"]
mod test_db_folder;
#[cfg(test)]
#[path = "../test_unit/test_db_private.rs"]
mod test_db_private;

#[cfg(test)]
pub static TEST_RUN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
