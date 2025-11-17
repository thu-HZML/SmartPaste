use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{self, Path, PathBuf},
    sync::{OnceLock, RwLock},
};

/// 系统配置结构体，包含通用设置、剪贴板参数、AI、隐私、备份、云同步和用户信息等配置项。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    // --- 通用设置 ---
    /// 是否启用开机自启动
    pub autostart: bool,
    /// 系统托盘图标是否可见
    pub tray_icon_visible: bool,
    /// 启动时是否最小化到托盘
    pub minimize_to_tray: bool,
    /// 是否自动保存剪贴板历史
    pub auto_save: bool,
    /// 历史记录保留天数（天）
    pub retention_days: u32,

    // --- 剪贴板参数 ---
    /// 最大历史记录数量
    pub max_history_items: u32,
    /// 忽略短文本的最短字符数（少于该值将被忽略）
    pub ignore_short_text_len: u32,
    /// 忽略大文件的大小阈值（单位：MB）
    pub ignore_big_file_mb: u32,
    /// 被忽略的应用列表（按应用名匹配）
    pub ignored_apps: Vec<String>,
    /// 是否自动分类
    pub auto_classify: bool,
    /// 是否启用 OCR 自动识别
    pub ocr_auto_recognition: bool,
    /// 删除时是否弹出确认对话框
    pub delete_confirmation: bool,
    /// 删除时是否保留收藏内容
    pub keep_favorites_on_delete: bool,
    /// 是否启用自动排序
    pub auto_sort: bool,

    // --- AI Agent 相关 ---
    /// 是否启用 AI 助手
    pub ai_enabled: bool,
    /// AI 服务提供商标识（例如 "openai"、"azure" 等）
    pub ai_service: Option<String>,
    /// AI API Key（如有则存储）
    pub ai_api_key: Option<String>,
    /// 是否启用 AI 自动打标签
    pub ai_auto_tag: bool,
    /// 是否启用 AI 自动摘要
    pub ai_auto_summary: bool,
    /// 是否启用 AI 翻译功能
    pub ai_translation: bool,
    /// 是否启用 AI 联网搜索功能
    pub ai_web_search: bool,

    // --- 安全与隐私 ---
    /// 是否启用敏感词过滤总开关
    pub sensitive_filter: bool,
    /// 是否过滤密码类型内容
    pub filter_passwords: bool,
    /// 是否过滤银行卡号
    pub filter_bank_cards: bool,
    /// 是否过滤身份证号
    pub filter_id_cards: bool,
    /// 是否过滤手机号
    pub filter_phone_numbers: bool,
    /// 隐私记录自动清理天数（天）
    pub privacy_retention_days: u32,
    /// 标记为隐私的记录 ID 列表（可用于快速查询/导出）
    pub privacy_records: Vec<String>,

    // --- 数据备份 ---
    /// 数据存储路径（若为空使用应用默认路径）
    pub storage_path: Option<String>,
    /// 是否启用自动备份
    pub auto_backup: bool,
    /// 备份频率（"daily"/"weekly"/"monthly"）
    pub backup_frequency: String,
    /// 最近一次备份文件路径（可选）
    pub last_backup_path: Option<String>,

    // --- 云端同步 ---
    /// 是否启用云端同步
    pub cloud_sync_enabled: bool,
    /// 同步频率（例如 "realtime"/"5min"/"15min"/"1hour"）
    pub sync_frequency: String,
    /// 同步内容类型（例如 "onlytxt"/"containphoto"/"containfile"）
    pub sync_content_type: String,
    /// 是否对云端数据进行加密
    pub encrypt_cloud_data: bool,
    /// 是否仅在 WiFi 下进行同步
    pub sync_only_wifi: bool,

    // --- 用户信息 ---
    /// 用户名（如果有登录/配置）
    pub username: Option<String>,
    /// 邮箱
    pub email: Option<String>,
    /// 用户简介
    pub bio: Option<String>,
    /// 头像文件路径
    pub avatar_path: Option<String>,
}

/// 为 Config 实现 Default trait，提供默认配置值。
impl Default for Config {
    /// 返回 Config 的默认实例。
    fn default() -> Self {
        Self {
            // 通用
            autostart: false,        // 开机自启：关
            tray_icon_visible: true, // 托盘图标：显示
            minimize_to_tray: false, // 启动最小化：否
            auto_save: true,         // 自动保存历史：是
            retention_days: 30,      // 历史保留天数：30天

            // 剪贴板
            max_history_items: 500,         // 最大历史记录数：500条
            ignore_short_text_len: 3,       // 忽略短文本长度：3字符
            ignore_big_file_mb: 5,          // 忽略大文件大小：5MB
            ignored_apps: Vec::new(),       // 忽略的应用列表：空
            auto_classify: true,            // 自动分类：是
            ocr_auto_recognition: false,    // OCR 自动识别：否
            delete_confirmation: true,      // 删除确认对话框：是
            keep_favorites_on_delete: true, // 删除时保留收藏：是
            auto_sort: false,               // 自动排序：否

            // AI
            ai_enabled: false,      // AI 助手：关
            ai_service: None,       // AI 服务提供商：无
            ai_api_key: None,       // AI API Key：无
            ai_auto_tag: false,     // AI 自动打标签：否
            ai_auto_summary: false, // AI 自动摘要：否
            ai_translation: false,  // AI 翻译功能：否
            ai_web_search: false,   // AI 联网搜索：否

            // 隐私
            sensitive_filter: true,      // 敏感词过滤：开
            filter_passwords: true,      // 过滤密码：是
            filter_bank_cards: true,     // 过滤银行卡号：是
            filter_id_cards: true,       // 过滤身份证号：是
            filter_phone_numbers: true,  // 过滤手机号：是
            privacy_retention_days: 90,  // 隐私记录保留天数：90天
            privacy_records: Vec::new(), // 隐私记录列表：空

            // 备份
            storage_path: None,                     // 数据存储路径：默认
            auto_backup: false,                     // 自动备份：关
            backup_frequency: "weekly".to_string(), // 备份频率：每周
            last_backup_path: None,                 // 最近备份路径：无

            // 云同步
            cloud_sync_enabled: false,                // 云端同步：关
            sync_frequency: "5min".to_string(),       // 同步频率：每5分钟
            sync_content_type: "onlytxt".to_string(), // 同步内容类型：仅文本
            encrypt_cloud_data: false,                // 云端数据加密：否
            sync_only_wifi: true,                     // 仅 WiFi 同步：是

            // 用户
            username: None,    // 用户名：无
            email: None,       // 邮箱：无
            bio: None,         // 用户简介：无
            avatar_path: None, // 头像路径：无
        }
    }
}

static CONFIG_PATH_GLOBAL: OnceLock<PathBuf> = OnceLock::new();
static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

/// 设置配置 JSON 文件路径
/// # Param
/// path: PathBuf - 配置文件路径
pub fn set_config_path(path: PathBuf) {
    CONFIG_PATH_GLOBAL.set(path).ok();
}

/// 获取配置 JSON 文件路径
/// # Returns
/// PathBuf - 配置文件路径
pub fn get_config_path() -> PathBuf {
    CONFIG_PATH_GLOBAL
        .get()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("config.json"))
}

/// 初始化全局配置。如果存在配置文件则加载，否则使用默认配置并创建文件。
/// # Returns
/// String - 初始化结果信息
pub fn init_config() -> String {
    let config_path = get_config_path();

    let config = if config_path.exists() {
        // 读取现有配置文件
        let data = fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        // 使用默认配置并创建文件
        let default_config = Config::default();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let mut file = fs::File::create(&config_path).unwrap();
        let data = serde_json::to_string_pretty(&default_config).unwrap();
        file.write_all(data.as_bytes()).ok();
        default_config
    };

    CONFIG
        .set(RwLock::new(config))
        .map(|_| "initialized successfully".to_string())
        .map_err(|_| "config json already exists".to_string())
        .unwrap_or_else(|e| e)
}

/// 获取配置信息的 JSON 字符串表示
/// # Returns
/// String - 配置的 JSON 字符串。若未初始化则返回空字符串。
pub fn get_config_json() -> String {
    if let Some(lock) = CONFIG.get() {
        let cfg = lock.read().unwrap();
        serde_json::to_string_pretty(&*cfg).unwrap_or_default()
    } else {
        "".to_string()
    }
}

/// 将配置信息保存到文件
/// # Param
/// new_config: Config - 新的配置信息
/// # Returns
/// Result<(), String> - 成功返回 Ok，失败返回错误信息
pub fn save_config(new_config: Config) -> Result<(), String> {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        *cfg = new_config.clone();
        let config_path = get_config_path();
        let data = serde_json::to_string_pretty(&new_config).map_err(|e| e.to_string())?;
        fs::write(&config_path, data).map_err(|e| e.to_string())
    } else {
        Err("config not initialized".to_string())
    }
}

// --------------- 配置信息修改函数 ---------------

/// 设置数据存储路径
/// # Param
/// path: PathBuf - 新的数据存储路径
/// # Returns
/// String - 设置结果信息
pub fn set_storage_path(path: PathBuf) -> String {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.storage_path = Some(path.to_string_lossy().to_string());
        "storage path updated".to_string()
    } else {
        "config not initialized".to_string()
    }
}
