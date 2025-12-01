use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;
static CONFIG_PATH_GLOBAL: RwLock<Option<PathBuf>> = RwLock::new(None);
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
    /// 主界面快捷键
    #[serde(default = "default_shortcut")]
    pub global_shortcut: String,
    /// 第二界面快捷键
    #[serde(default = "default_shortcut_2")]
    pub global_shortcut_2: String,
    /// 第三快捷键 (新增)
    #[serde(default = "default_shortcut_3")]
    pub global_shortcut_3: String,
    /// 第四快捷键 (新增)
    #[serde(default = "default_shortcut_4")]
    pub global_shortcut_4: String,
    /// 第五快捷键 (新增)
    #[serde(default = "default_shortcut_5")]
    pub global_shortcut_5: String,

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

    // --- OCR 设置 ---
    /// OCR 提供商标识（例如 "tesseract"、"google" 等）
    pub ocr_provider: Option<String>,
    /// OCR 语言列表（例如 ["eng", "chi"]）
    pub ocr_languages: Option<Vec<String>>,
    /// OCR 置信度阈值（0.0 - 1.0）
    pub ocr_confidence_threshold: Option<f32>,
    /// OCR 超时时间（秒）
    pub ocr_timeout_secs: Option<u64>,
}
/// 辅助枚举，表示配置项的名称。
/// # Variants
/// ConfigKey - 枚举变体，表示不同的配置项名称
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ConfigKey {
    // 通用设置
    /// 开机自启
    Autostart,
    /// 托盘图标可见性
    TrayIconVisible,
    /// 启动最小化到托盘
    MinimizeToTray,
    /// 自动保存剪贴板历史
    AutoSave,
    /// 历史记录保留天数
    RetentionDays,
    /// 主界面快捷键
    GlobalShortcut,
    /// 第二界面快捷键
    GlobalShortcut2,
    /// 第三快捷键
    GlobalShortcut3,
    /// 第四快捷键
    GlobalShortcut4,
    /// 第五快捷键
    GlobalShortcut5,

    // 剪贴板参数
    /// 最大历史记录数量
    MaxHistoryItems,
    /// 忽略短文本的最短字符数
    IgnoreShortTextLen,
    /// 忽略大文件的大小阈值
    IgnoreBigFileMb,
    /// 被忽略的应用列表
    IgnoredApps,
    /// 是否自动分类
    AutoClassify,
    /// 是否启用 OCR 自动识别
    OcrAutoRecognition,
    /// 删除时是否弹出确认对话框
    DeleteConfirmation,
    /// 删除时是否保留收藏内容
    KeepFavoritesOnDelete,
    /// 是否启用自动排序
    AutoSort,

    // AI Agent 相关
    /// 是否启用 AI 助手
    AiEnabled,
    /// AI 服务提供商标识
    AiService,
    /// AI API Key
    AiApiKey,
    /// 是否启用 AI 自动打标签
    AiAutoTag,
    /// 是否启用 AI 自动摘要
    AiAutoSummary,
    /// 是否启用 AI 翻译功能
    AiTranslation,
    /// 是否启用 AI 联网搜索功能
    AiWebSearch,

    // 安全与隐私
    /// 是否启用敏感词过滤总开关
    SensitiveFilter,
    /// 是否过滤密码类型内容
    FilterPasswords,
    /// 是否过滤银行卡号
    FilterBankCards,
    /// 是否过滤身份证号
    FilterIdCards,
    /// 是否过滤手机号
    FilterPhoneNumbers,
    /// 隐私记录自动清理天数
    PrivacyRetentionDays,
    /// 标记为隐私的记录 ID 列表
    PrivacyRecords,

    // 数据备份
    /// 数据存储路径
    StoragePath,
    /// 是否启用自动备份
    AutoBackup,
    /// 备份频率
    BackupFrequency,
    /// 最近一次备份文件路径
    LastBackupPath,

    // 云端同步
    /// 是否启用云端同步
    CloudSyncEnabled,
    /// 同步频率
    SyncFrequency,
    /// 同步内容类型
    SyncContentType,
    /// 是否对云端数据进行加密
    EncryptCloudData,
    /// 是否仅在 WiFi 下进行同步
    SyncOnlyWifi,

    // 用户信息
    /// 用户名
    Username,
    /// 邮箱
    Email,
    /// 用户简介
    Bio,
    /// 头像文件路径
    AvatarPath,

    // OCR 设置
    /// OCR 提供商标识
    OcrProvider,
    /// OCR 语言列表
    OcrLanguages,
    /// OCR 置信度阈值
    OcrConfidenceThreshold,
    /// OCR 超时时间
    OcrTimeoutSecs,
}

/// 辅助函数。解析字符串到 ConfigKey 枚举
/// # Param
/// key: &str - 配置项名称
/// # Returns
/// Option<ConfigKey> - 解析成功返回 Some(ConfigKey)，否则返回 None
pub fn parse_config_key(key: &str) -> Option<ConfigKey> {
    match key {
        // 通用设置
        "autostart" => Some(ConfigKey::Autostart),
        "tray_icon_visible" => Some(ConfigKey::TrayIconVisible),
        "minimize_to_tray" => Some(ConfigKey::MinimizeToTray),
        "auto_save" => Some(ConfigKey::AutoSave),
        "retention_days" => Some(ConfigKey::RetentionDays),
        "global_shortcut" => Some(ConfigKey::GlobalShortcut),
        "global_shortcut_2" => Some(ConfigKey::GlobalShortcut2),
        "global_shortcut_3" => Some(ConfigKey::GlobalShortcut3),
        "global_shortcut_4" => Some(ConfigKey::GlobalShortcut4),
        "global_shortcut_5" => Some(ConfigKey::GlobalShortcut5),

        // 剪贴板参数
        "max_history_items" => Some(ConfigKey::MaxHistoryItems),
        "ignore_short_text_len" => Some(ConfigKey::IgnoreShortTextLen),
        "ignore_big_file_mb" => Some(ConfigKey::IgnoreBigFileMb),
        "ignored_apps" => Some(ConfigKey::IgnoredApps),
        "auto_classify" => Some(ConfigKey::AutoClassify),
        "ocr_auto_recognition" => Some(ConfigKey::OcrAutoRecognition),
        "delete_confirmation" => Some(ConfigKey::DeleteConfirmation),
        "keep_favorites_on_delete" => Some(ConfigKey::KeepFavoritesOnDelete),
        "auto_sort" => Some(ConfigKey::AutoSort),

        // AI Agent 相关
        "ai_enabled" => Some(ConfigKey::AiEnabled),
        "ai_service" => Some(ConfigKey::AiService),
        "ai_api_key" => Some(ConfigKey::AiApiKey),
        "ai_auto_tag" => Some(ConfigKey::AiAutoTag),
        "ai_auto_summary" => Some(ConfigKey::AiAutoSummary),
        "ai_translation" => Some(ConfigKey::AiTranslation),
        "ai_web_search" => Some(ConfigKey::AiWebSearch),
        // 安全与隐私
        "sensitive_filter" => Some(ConfigKey::SensitiveFilter),
        "filter_passwords" => Some(ConfigKey::FilterPasswords),
        "filter_bank_cards" => Some(ConfigKey::FilterBankCards),
        "filter_id_cards" => Some(ConfigKey::FilterIdCards),
        "filter_phone_numbers" => Some(ConfigKey::FilterPhoneNumbers),
        "privacy_retention_days" => Some(ConfigKey::PrivacyRetentionDays),
        "privacy_records" => Some(ConfigKey::PrivacyRecords),
        // 数据备份
        "storage_path" => Some(ConfigKey::StoragePath),
        "auto_backup" => Some(ConfigKey::AutoBackup),
        "backup_frequency" => Some(ConfigKey::BackupFrequency),
        "last_backup_path" => Some(ConfigKey::LastBackupPath),
        // 云端同步
        "cloud_sync_enabled" => Some(ConfigKey::CloudSyncEnabled),
        "sync_frequency" => Some(ConfigKey::SyncFrequency),
        "sync_content_type" => Some(ConfigKey::SyncContentType),
        "encrypt_cloud_data" => Some(ConfigKey::EncryptCloudData),
        "sync_only_wifi" => Some(ConfigKey::SyncOnlyWifi),
        // 用户信息
        "username" => Some(ConfigKey::Username),
        "email" => Some(ConfigKey::Email),
        "bio" => Some(ConfigKey::Bio),
        "avatar_path" => Some(ConfigKey::AvatarPath),
        // OCR 设置
        "ocr_provider" => Some(ConfigKey::OcrProvider),
        "ocr_languages" => Some(ConfigKey::OcrLanguages),
        "ocr_confidence_threshold" => Some(ConfigKey::OcrConfidenceThreshold),
        "ocr_timeout_secs" => Some(ConfigKey::OcrTimeoutSecs),
        _ => None,
    }
}

/// 辅助函数，将Config结构体转化为JSON字符串
/// # Param
/// config: &Config - 配置结构体引用
/// # Returns
/// String - 配置的JSON字符串表示
pub fn config_to_json(config: &Config) -> String {
    serde_json::to_string_pretty(config).unwrap_or_default()
}

// 辅助函数，防止旧 config.json 缺少字段导致解析失败
fn default_shortcut() -> String {
    "Alt+Shift+V".to_string()
}
fn default_shortcut_2() -> String {
    "Alt+Shift+C".to_string()
}
fn default_shortcut_3() -> String {
    "Alt+Shift+A".to_string()
} // 新增
fn default_shortcut_4() -> String {
    "Ctrl+Shift+V".to_string()
} // 新增
fn default_shortcut_5() -> String {
    "Ctrl+Shift+Delete".to_string()
} // 新增

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
            global_shortcut: default_shortcut(),
            global_shortcut_2: default_shortcut_2(),
            global_shortcut_3: default_shortcut_3(), // 新增
            global_shortcut_4: default_shortcut_4(), // 新增
            global_shortcut_5: default_shortcut_5(), // 新增
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

            // OCR
            ocr_provider: None,             // OCR 提供商：无（使用默认值）
            ocr_languages: None,            // OCR 语言列表：无（使用默认值）
            ocr_confidence_threshold: None, // OCR 置信度阈值：无（使用默认值）
            ocr_timeout_secs: None,         // OCR 超时时间：无（使用默认值）
        }
    }
}

pub static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

/// 设置配置 JSON 文件路径
/// # Param
/// path: PathBuf - 配置文件路径
pub fn set_config_path(path: PathBuf) {
    println!("🔄 设置配置路径: {}", path.display());
    let mut global_path = CONFIG_PATH_GLOBAL.write().unwrap();
    *global_path = Some(path);
}
/// 获取配置 JSON 文件路径
/// # Returns
/// PathBuf - 配置文件路径
pub fn get_config_path() -> PathBuf {
    let global_path = CONFIG_PATH_GLOBAL.read().unwrap();
    global_path.clone().unwrap_or_else(|| {
        println!("⚠️ 使用默认配置路径");
        PathBuf::from("config.json")
    })
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

/// 获取配置信息的 JSON 字符串表示。作为 Tauri Command 暴露给前端调用。
/// # Returns
/// String - 配置的 JSON 字符串。若未初始化则返回空字符串。
#[tauri::command]
pub fn get_config_json() -> String {
    if let Some(lock) = CONFIG.get() {
        let cfg = lock.read().unwrap();
        config_to_json(&cfg)
    } else {
        "".to_string()
    }
}

// --------------- 配置信息修改函数 ---------------

// 优化：统合所有配置信息修改函数逻辑为以下通用模式，避免重复代码

/// 保存配置到文件
pub fn save_config(config: Config) -> Result<(), String> {
    let config_path = get_config_path();
    println!("💾 正在保存配置到: {}", config_path.display());
    
    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("创建配置目录失败: {}", e));
        }
    }
    
    let data = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    match fs::write(&config_path, &data) {
        Ok(_) => {
            println!("✅ 配置保存成功: {}", config_path.display());
            
            // 验证文件确实被创建
            if config_path.exists() {
                println!("✅ 配置文件确认存在");
                if let Ok(metadata) = fs::metadata(&config_path) {
                    println!("📊 配置文件大小: {} 字节", metadata.len());
                }
            } else {
                println!("❌ 配置文件不存在，保存可能失败");
            }
            
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("保存配置到 {} 失败: {}", config_path.display(), e);
            println!("❌ {}", error_msg);
            Err(error_msg)
        }
    }
}

/// 内部辅助函数：更新简单配置项
/// 返回 Ok(true) 表示已处理并更新内存
/// 返回 Ok(false) 表示该 key (如 Autostart) 需要特殊处理，未更新
/// 返回 Err 表示类型错误或其他错误
fn update_simple_config_item(key: &ConfigKey, value: serde_json::Value) -> Result<bool, String> {
    macro_rules! update_cfg {
        ($field:ident, $type:ty) => {{
            match serde_json::from_value::<$type>(value) {
                Ok(v) => {
                    if let Some(lock) = CONFIG.get() {
                        let mut cfg = lock.write().unwrap();
                        cfg.$field = v;
                    }
                    Ok(true)
                }
                Err(_) => Err(format!("Invalid type for config key")),
            }
        }};
    }

    match key {
        ConfigKey::Autostart => Ok(false),
        ConfigKey::TrayIconVisible => update_cfg!(tray_icon_visible, bool),
        ConfigKey::MinimizeToTray => update_cfg!(minimize_to_tray, bool),
        ConfigKey::AutoSave => update_cfg!(auto_save, bool),
        ConfigKey::RetentionDays => update_cfg!(retention_days, u32),
        ConfigKey::GlobalShortcut => update_cfg!(global_shortcut, String),
        ConfigKey::GlobalShortcut2 => update_cfg!(global_shortcut_2, String),
        ConfigKey::GlobalShortcut3 => update_cfg!(global_shortcut_3, String),
        ConfigKey::GlobalShortcut4 => update_cfg!(global_shortcut_4, String),
        ConfigKey::GlobalShortcut5 => update_cfg!(global_shortcut_5, String),
        ConfigKey::MaxHistoryItems => update_cfg!(max_history_items, u32),
        ConfigKey::IgnoreShortTextLen => update_cfg!(ignore_short_text_len, u32),
        ConfigKey::IgnoreBigFileMb => update_cfg!(ignore_big_file_mb, u32),
        ConfigKey::IgnoredApps => update_cfg!(ignored_apps, Vec<String>),
        ConfigKey::AutoClassify => update_cfg!(auto_classify, bool),
        ConfigKey::OcrAutoRecognition => update_cfg!(ocr_auto_recognition, bool),
        ConfigKey::DeleteConfirmation => update_cfg!(delete_confirmation, bool),
        ConfigKey::KeepFavoritesOnDelete => update_cfg!(keep_favorites_on_delete, bool),
        ConfigKey::AutoSort => update_cfg!(auto_sort, bool),
        ConfigKey::AiEnabled => update_cfg!(ai_enabled, bool),
        ConfigKey::AiService => update_cfg!(ai_service, Option<String>),
        ConfigKey::AiApiKey => update_cfg!(ai_api_key, Option<String>),
        ConfigKey::AiAutoTag => update_cfg!(ai_auto_tag, bool),
        ConfigKey::AiAutoSummary => update_cfg!(ai_auto_summary, bool),
        ConfigKey::AiTranslation => update_cfg!(ai_translation, bool),
        ConfigKey::AiWebSearch => update_cfg!(ai_web_search, bool),
        ConfigKey::SensitiveFilter => update_cfg!(sensitive_filter, bool),
        ConfigKey::FilterPasswords => update_cfg!(filter_passwords, bool),
        ConfigKey::FilterBankCards => update_cfg!(filter_bank_cards, bool),
        ConfigKey::FilterIdCards => update_cfg!(filter_id_cards, bool),
        ConfigKey::FilterPhoneNumbers => update_cfg!(filter_phone_numbers, bool),
        ConfigKey::PrivacyRetentionDays => update_cfg!(privacy_retention_days, u32),
        ConfigKey::PrivacyRecords => update_cfg!(privacy_records, Vec<String>),
        ConfigKey::StoragePath => update_cfg!(storage_path, Option<String>),
        ConfigKey::AutoBackup => update_cfg!(auto_backup, bool),
        ConfigKey::BackupFrequency => update_cfg!(backup_frequency, String),
        ConfigKey::LastBackupPath => update_cfg!(last_backup_path, Option<String>),
        ConfigKey::CloudSyncEnabled => update_cfg!(cloud_sync_enabled, bool),
        ConfigKey::SyncFrequency => update_cfg!(sync_frequency, String),
        ConfigKey::SyncContentType => update_cfg!(sync_content_type, String),
        ConfigKey::EncryptCloudData => update_cfg!(encrypt_cloud_data, bool),
        ConfigKey::SyncOnlyWifi => update_cfg!(sync_only_wifi, bool),
        ConfigKey::Username => update_cfg!(username, Option<String>),
        ConfigKey::Email => update_cfg!(email, Option<String>),
        ConfigKey::Bio => update_cfg!(bio, Option<String>),
        ConfigKey::AvatarPath => update_cfg!(avatar_path, Option<String>),
        ConfigKey::OcrProvider => update_cfg!(ocr_provider, Option<String>),
        ConfigKey::OcrLanguages => update_cfg!(ocr_languages, Option<Vec<String>>),
        ConfigKey::OcrConfidenceThreshold => update_cfg!(ocr_confidence_threshold, Option<f32>),
        ConfigKey::OcrTimeoutSecs => update_cfg!(ocr_timeout_secs, Option<u64>),
    }
}

/// 供 Rust 内部调用的配置更新函数（不支持 Autostart）
pub fn set_config_item_internal(key: &str, value: serde_json::Value) -> Result<(), String> {
    let config_key = match parse_config_key(key) {
        Some(k) => k,
        None => return Err(format!("Invalid config key: {}", key)),
    };

    match update_simple_config_item(&config_key, value) {
        Ok(true) => {
            let cfg_clone = CONFIG.get().unwrap().read().unwrap().clone();
            save_config(cfg_clone)
        }
        Ok(false) => Err(format!("Config key '{}' requires AppHandle context", key)),
        Err(e) => Err(e),
    }
}
/// 迁移数据到新的存储路径
fn migrate_data_to_new_path(old_path: &PathBuf, new_path: &PathBuf) -> Result<(), String> {
    println!("🚚 开始迁移数据文件从 {} 到 {}", old_path.display(), new_path.display());
    
    // 确保新路径存在
    if let Err(e) = fs::create_dir_all(new_path) {
        return Err(format!("创建新存储路径失败: {}", e));
    }

    // 🔥 关键修复：在迁移前先清理新路径下的现有文件
    println!("🧹 检查并清理新路径下的现有文件...");
    let files_to_clean = vec![
        ("smartpaste.db", "数据库文件"),
        ("files", "文件目录")
    ];

    for (file_name, desc) in files_to_clean {
        let target_path = new_path.join(file_name);
        if target_path.exists() {
            println!("🗑️ 删除现有的 {}: {}", desc, file_name);
            if file_name == "files" && target_path.is_dir() {
                // 删除整个 files 文件夹
                if let Err(e) = fs::remove_dir_all(&target_path) {
                    return Err(format!("删除现有 {} 失败: {}", desc, e));
                }
            } else {
                // 删除文件
                if let Err(e) = fs::remove_file(&target_path) {
                    return Err(format!("删除现有 {} 失败: {}", desc, e));
                }
            }
            println!("✅ 已删除现有的 {}: {}", desc, file_name);
        } else {
            println!("ℹ️ 新路径下没有现有的 {}: {}", desc, file_name);
        }
    }

    let files_to_migrate = vec![
        ("smartpaste.db", "数据库文件"),
        ("files", "文件目录")
    ];

    for (file_name, desc) in files_to_migrate {
        let old_file_path = old_path.join(file_name);
        let new_file_path = new_path.join(file_name);
        
        if old_file_path.exists() {
            if file_name == "files" && old_file_path.is_dir() {
                // 处理文件夹迁移 - 现在目标文件夹已经被清理，直接复制
                match copy_dir_all(&old_file_path, &new_file_path) {
                    Ok(_) => println!("✅ 已迁移 {}: {}", desc, file_name),
                    Err(e) => return Err(format!("迁移 {} 失败: {}", desc, e)),
                }
            } else {
                // 处理文件迁移
                match fs::copy(&old_file_path, &new_file_path) {
                    Ok(_) => println!("✅ 已迁移 {}: {}", desc, file_name),
                    Err(e) => return Err(format!("迁移 {} 失败: {}", desc, e)),
                }
            }
        } else {
            println!("ℹ️ {} 不存在，跳过迁移: {}", desc, file_name);
        }
    }
    
    println!("🎉 数据文件迁移完成");
    Ok(())
}

/// 递归复制目录
/// 递归复制目录
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    // 确保目标目录存在
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    } else {
        // 如果目标目录已存在，确保它是目录
        if !dst.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "目标路径不是目录"
            ));
        }
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            // 复制文件，如果目标文件已存在则覆盖
            fs::copy(&entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// 获取当前的数据存储路径
fn get_current_storage_path() -> PathBuf {
    // 首先检查配置中的存储路径
    if let Some(lock) = CONFIG.get() {
        let cfg = lock.read().unwrap();
        if let Some(ref path_str) = cfg.storage_path {
            let custom_path = PathBuf::from(path_str);
            if !path_str.trim().is_empty() {
                return custom_path;
            }
        }
    }
    
    // 回退到配置文件的父目录
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        return parent.to_path_buf();
    }
    
    // 最后回退到当前目录
    PathBuf::from(".")
}
/// 按传入参数修改配置信息。作为 Tauri Command 暴露给前端调用。
///
/// 该函数是前端修改配置的统一入口。根据传入的 `key` 找到对应的配置项，并将 `value` 转换为相应的类型进行更新。
/// 更新成功后会自动保存到本地配置文件。
///
/// # Param
/// * `key`: &str - 配置项名称。支持的键名及其对应的值类型如下：
///
/// **通用设置**
/// * `"autostart"`: `bool` - 是否开机自启 (特殊处理：会调用系统 API)
/// * `"tray_icon_visible"`: `bool` - 托盘图标是否可见
/// * `"minimize_to_tray"`: `bool` - 启动时是否最小化到托盘
/// * `"auto_save"`: `bool` - 是否自动保存剪贴板历史
/// * `"retention_days"`: `u32` - 历史记录保留天数
/// * `"global_shortcut"`: `String` - 主界面快捷键 (如 "Alt+Shift+V")
/// * `"global_shortcut_2"`: `String` - 第二界面快捷键
/// * `"global_shortcut_3"`: `String` - 第三快捷键
/// * `"global_shortcut_4"`: `String` - 第四快捷键
/// * `"global_shortcut_5"`: `String` - 第五快捷键
///
/// **剪贴板参数**
/// * `"max_history_items"`: `u32` - 最大历史记录数量
/// * `"ignore_short_text_len"`: `u32` - 忽略短文本的最短字符数
/// * `"ignore_big_file_mb"`: `u32` - 忽略大文件的大小阈值 (MB)
/// * `"ignored_apps"`: `Vec<String>` - 被忽略的应用列表
/// * `"auto_classify"`: `bool` - 是否自动分类
/// * `"ocr_auto_recognition"`: `bool` - 是否启用 OCR 自动识别
/// * `"delete_confirmation"`: `bool` - 删除时是否弹出确认对话框
/// * `"keep_favorites_on_delete"`: `bool` - 删除时是否保留收藏内容
/// * `"auto_sort"`: `bool` - 是否启用自动排序
///
/// **AI Agent 相关**
/// * `"ai_enabled"`: `bool` - 是否启用 AI 助手
/// * `"ai_service"`: `Option<String>` - AI 服务提供商标识 (如 "openai")
/// * `"ai_api_key"`: `Option<String>` - AI API Key
/// * `"ai_auto_tag"`: `bool` - 是否启用 AI 自动打标签
/// * `"ai_auto_summary"`: `bool` - 是否启用 AI 自动摘要
/// * `"ai_translation"`: `bool` - 是否启用 AI 翻译功能
/// * `"ai_web_search"`: `bool` - 是否启用 AI 联网搜索功能
///
/// **安全与隐私**
/// * `"sensitive_filter"`: `bool` - 是否启用敏感词过滤总开关
/// * `"filter_passwords"`: `bool` - 是否过滤密码类型内容
/// * `"filter_bank_cards"`: `bool` - 是否过滤银行卡号
/// * `"filter_id_cards"`: `bool` - 是否过滤身份证号
/// * `"filter_phone_numbers"`: `bool` - 是否过滤手机号
/// * `"privacy_retention_days"`: `u32` - 隐私记录自动清理天数
/// * `"privacy_records"`: `Vec<String>` - 标记为隐私的记录 ID 列表
///
/// **数据备份**
/// * `"storage_path"`: `Option<String>` - 数据存储路径
/// * `"auto_backup"`: `bool` - 是否启用自动备份
/// * `"backup_frequency"`: `String` - 备份频率 ("daily"/"weekly"/"monthly")
/// * `"last_backup_path"`: `Option<String>` - 最近一次备份文件路径
///
/// **云端同步**
/// * `"cloud_sync_enabled"`: `bool` - 是否启用云端同步
/// * `"sync_frequency"`: `String` - 同步频率
/// * `"sync_content_type"`: `String` - 同步内容类型
/// * `"encrypt_cloud_data"`: `bool` - 是否对云端数据进行加密
/// * `"sync_only_wifi"`: `bool` - 是否仅在 WiFi 下进行同步
///
/// **用户信息**
/// * `"username"`: `Option<String>` - 用户名
/// * `"email"`: `Option<String>` - 邮箱
/// * `"bio"`: `Option<String>` - 用户简介
/// * `"avatar_path"`: `Option<String>` - 头像文件路径
///
/// **OCR 设置**
/// * `"ocr_provider"`: `Option<String>` - OCR 提供商标识
/// * `"ocr_languages"`: `Option<Vec<String>>` - OCR 语言列表
/// * `"ocr_confidence_threshold"`: `Option<f32>` - OCR 置信度阈值
/// * `"ocr_timeout_secs"`: `Option<u64>` - OCR 超时时间
///
/// * `value`: serde_json::Value - 新的配置值，类型必须与上述列表一致。
///
/// # Returns
/// String - 修改结果信息，若成功返回 "config updated"，否则返回错误信息（类型不匹配等）
#[tauri::command]
pub fn set_config_item(app: tauri::AppHandle, key: &str, value: serde_json::Value) -> String {
    let config_key = match parse_config_key(key) {
        Some(k) => k,
        None => return format!("Invalid config key: {}", key),
    };

    // 特殊处理存储路径修改
    if config_key == ConfigKey::StoragePath {
        let new_path_str = match value.as_str() {
            Some(s) => s.to_string(),
            None => return "Invalid storage path value".to_string(),
        };

        // 获取当前存储路径
        let current_path = get_current_storage_path();
        let new_path = PathBuf::from(&new_path_str);

        println!("🔄 开始修改存储路径: {} -> {}", current_path.display(), new_path.display());

        // 验证新路径
        if new_path_str.trim().is_empty() {
            return "Storage path cannot be empty".to_string();
        }

        // 如果新旧路径相同，直接返回
        if current_path == new_path {
            return "Storage path unchanged".to_string();
        }

        // 创建新路径
        if let Err(e) = fs::create_dir_all(&new_path) {
            return format!("Failed to create storage path: {}", e);
        }

        // 保存当前配置到旧路径，确保所有更改已持久化
        if let Some(lock) = CONFIG.get() {
            let current_config = lock.read().unwrap().clone();
            if let Err(e) = save_config(current_config) {
                return format!("Failed to save current config before migration: {}", e);
            }
        }

        // 执行数据迁移（不包括 config.json）
        if let Err(e) = migrate_data_to_new_path(&current_path, &new_path) {
            return format!("Data migration failed: {}", e);
        }

        // 更新内存中的配置
        if let Some(lock) = CONFIG.get() {
            let mut cfg = lock.write().unwrap();
            cfg.storage_path = Some(new_path_str.clone());
        }

        // 保存配置到新路径
        let new_config_path = new_path.join("config.json");
        let old_config_path = get_config_path();
        
        println!("💾 准备保存配置到新路径: {}", new_config_path.display());
        
        // 切换到新路径保存配置
        set_config_path(new_config_path.clone());
        
        // 验证路径是否真的改变了
        let current_path_after_set = get_config_path();
        println!("🔍 设置配置路径后，当前配置路径: {}", current_path_after_set.display());
        
        if current_path_after_set != new_config_path {
            println!("❌ 配置路径设置失败，期望: {}，实际: {}", 
                new_config_path.display(), current_path_after_set.display());
            set_config_path(old_config_path);
            return "Failed to set config path".to_string();
        }

        let cfg_clone = CONFIG.get().unwrap().read().unwrap().clone();
        match save_config(cfg_clone.clone()) {
            Ok(_) => {
                // 更新数据库路径
                let new_db_path = new_path.join("smartpaste.db");
                crate::db::set_db_path(new_db_path);
                
                println!("✅ 存储路径修改完成，配置已保存到新路径: {}", new_config_path.display());
                
                // 验证新配置文件确实存在
                if new_config_path.exists() {
                    println!("✅ 新配置文件确认存在: {}", new_config_path.display());
                    if let Ok(metadata) = fs::metadata(&new_config_path) {
                        println!("📊 新配置文件大小: {} 字节", metadata.len());
                    }
                } else {
                    println!("❌ 新配置文件不存在，保存可能失败");
                }
                
                // 🔥 关键修复：同时更新默认路径的配置文件
                // 这样应用重启后能从默认路径读取到正确的存储路径
                let app_default_dir = app.path().app_data_dir().unwrap();
                let default_config_path = app_default_dir.join("config.json");
                
                if default_config_path != new_config_path {
                    println!("📝 同时更新默认路径的配置文件: {}", default_config_path.display());
                    
                    // 创建默认路径的配置副本
                    let mut default_config = cfg_clone.clone();
                    // 确保存储路径字段正确
                    default_config.storage_path = Some(new_path_str.clone());
                    
                    // 保存到默认路径
                    let old_path_for_default = get_config_path();
                    set_config_path(default_config_path.clone());
                    
                    if let Err(e) = save_config(default_config) {
                        println!("⚠️ 更新默认路径配置文件失败: {}", e);
                        // 恢复配置路径
                        set_config_path(old_path_for_default);
                    } else {
                        println!("✅ 默认路径配置文件更新成功");
                        // 恢复配置路径到新路径
                        set_config_path(new_config_path);
                    }
                }
                
                "config updated and data migrated".to_string()
            }
            Err(e) => {
                // 如果保存失败，恢复旧的配置路径
                set_config_path(old_config_path);
                format!("failed to save config: {}", e)
            }
        }
    } else {
        // 其他配置项的原有逻辑保持不变
        match update_simple_config_item(&config_key, value.clone()) {
            Ok(true) => {
                let cfg_clone = CONFIG.get().unwrap().read().unwrap().clone();
                match save_config(cfg_clone) {
                    Ok(_) => "config updated".to_string(),
                    Err(e) => format!("failed to save config: {}", e),
                }
            }
            Ok(false) => {
                if config_key == ConfigKey::Autostart {
                    match serde_json::from_value::<bool>(value) {
                        Ok(enable) => {
                            let autolaunch = app.autolaunch();
                            let res = if enable {
                                autolaunch.enable()
                            } else {
                                autolaunch.disable()
                            };
                            match res {
                                Ok(_) => {
                                    if let Some(lock) = CONFIG.get() {
                                        let mut cfg = lock.write().unwrap();
                                        cfg.autostart = enable;
                                    }
                                    let cfg_clone = CONFIG.get().unwrap().read().unwrap().clone();
                                    match save_config(cfg_clone) {
                                        Ok(_) => "config updated".to_string(),
                                        Err(e) => format!("failed to save config: {}", e),
                                    }
                                }
                                Err(e) => format!("Failed to change autostart: {}", e),
                            }
                        }
                        Err(_) => format!("Invalid type for key '{}'", key),
                    }
                } else {
                    format!("Unhandled config key: {}", key)
                }
            }
            Err(e) => e,
        }
    }
}
// /// 设置数据存储路径
// /// # Param
// /// path: PathBuf - 新的数据存储路径
// /// # Returns
// /// String - 设置结果信息
// pub fn set_db_storage_path(path: PathBuf) -> String {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.storage_path = Some(path.to_string_lossy().to_string());
//         "storage path updated".to_string()
//     } else {
//         "config not initialized".to_string()
//     }
// }
// /// 设置主快捷键 (修复了死锁问题)
// pub fn set_global_shortcut_internal(shortcut: String) {
//     // 第一步：先获取写锁，更新内存中的配置
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.global_shortcut = shortcut;
//     }
//     // 写锁在这里自动释放

//     // 第二步：先获取读锁拿到配置副本，然后释放读锁
//     let cfg_clone = if let Some(lock) = CONFIG.get() {
//         lock.read().unwrap().clone()
//     } else {
//         return;
//     };
//     // 读锁在这里自动释放

//     // 第三步：调用 save_config (它内部会再次获取写锁，但现在是安全的)
//     if let Err(e) = save_config(cfg_clone) {
//         eprintln!("❌ 保存配置文件失败: {}", e);
//     }
// }

// /// 设置第二快捷键 (修复了死锁问题)
// pub fn set_global_shortcut_2_internal(shortcut: String) {
//     // 第一步：更新内存
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.global_shortcut_2 = shortcut;
//     }

//     // 第二步：获取副本
//     let cfg_clone = if let Some(lock) = CONFIG.get() {
//         lock.read().unwrap().clone()
//     } else {
//         return;
//     };

//     // 第三步：保存
//     if let Err(e) = save_config(cfg_clone) {
//         eprintln!("❌ 保存配置文件失败: {}", e);
//     }
// }
// /// 设置第三快捷键 (修复了死锁问题)
// pub fn set_global_shortcut_3_internal(shortcut: String) {
//     // 第一步：先获取写锁，更新内存中的配置
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.global_shortcut_3 = shortcut;
//     }
//     // 写锁在这里自动释放

//     // 第二步：先获取读锁拿到配置副本，然后释放读锁
//     let cfg_clone = if let Some(lock) = CONFIG.get() {
//         lock.read().unwrap().clone()
//     } else {
//         return;
//     };
//     // 读锁在这里自动释放

//     // 第三步：调用 save_config (它内部会再次获取写锁，但现在是安全的)
//     if let Err(e) = save_config(cfg_clone) {
//         eprintln!("❌ 保存配置文件失败: {}", e);
//     }
// }

// /// 设置第四快捷键 (修复了死锁问题)
// pub fn set_global_shortcut_4_internal(shortcut: String) {
//     // 第一步：更新内存
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.global_shortcut_4 = shortcut;
//     }

//     // 第二步：获取副本
//     let cfg_clone = if let Some(lock) = CONFIG.get() {
//         lock.read().unwrap().clone()
//     } else {
//         return;
//     };

//     // 第三步：保存
//     if let Err(e) = save_config(cfg_clone) {
//         eprintln!("❌ 保存配置文件失败: {}", e);
//     }
// }

// /// 设置第五快捷键 (修复了死锁问题)
// pub fn set_global_shortcut_5_internal(shortcut: String) {
//     // 第一步：更新内存
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.global_shortcut_5 = shortcut;
//     }

//     // 第二步：获取副本
//     let cfg_clone = if let Some(lock) = CONFIG.get() {
//         lock.read().unwrap().clone()
//     } else {
//         return;
//     };

//     // 第三步：保存
//     if let Err(e) = save_config(cfg_clone) {
//         eprintln!("❌ 保存配置文件失败: {}", e);
//     }
// }

// /// 设置开机自启配置 (包含持久化保存，已处理死锁问题)
// /// # Param
// /// enable: bool - 是否启用
// pub fn set_autostart_config(enable: bool) -> Result<(), String> {
//     // 1. 更新内存中的配置
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.autostart = enable;
//     } else {
//         return Err("Config not initialized".to_string());
//     }

//     // 2. 获取配置副本 (此时已释放写锁)
//     let cfg_clone = if let Some(lock) = CONFIG.get() {
//         lock.read().unwrap().clone()
//     } else {
//         return Err("Config not initialized".to_string());
//     };

//     // 3. 保存到文件 (save_config 内部会再次获取锁，但现在是安全的)
//     save_config(cfg_clone)
// }
// // --------------- 1. 通用设置 ---------------

// /// 设置或取消应用的开机自启。作为 Tauri command 暴露给前端调用。
// /// # Param
// /// app: tauri::AppHandle - Tauri 的应用句柄，用于访问应用相关功能。
// /// enable: bool - true表示启用开机自启，false表示禁用。
// /// # Returns
// /// Result<(), String> - 操作成功则返回 Ok(())，失败则返回包含错误信息的 Err。
// #[tauri::command]
// pub fn set_autostart(app: tauri::AppHandle, enable: bool) -> Result<(), String> {
//     let autolaunch = app.autolaunch();

//     if enable {
//         autolaunch
//             .enable()
//             .map_err(|e| format!("启用开机自启失败: {}", e))?;
//     } else {
//         autolaunch
//             .disable()
//             .map_err(|e| format!("禁用开机自启失败: {}", e))?;
//     }
//     crate::config::set_autostart_config(enable)?;
//     Ok(())
// }

// /// 检查应用是否已设置为开机自启。作为 Tauri command 暴露给前端调用。
// /// # Param
// /// app: tauri::AppHandle - Tauri 的应用句柄，用于访问应用相关功能。
// /// # Returns
// /// Result<bool, String> - 操作成功则返回 Ok(bool)，其中 true 表示已启用自启，false 表示未启用。失败则返回包含错误信息的 Err。
// #[tauri::command]
// pub fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
//     let autolaunch = app.autolaunch();

//     let state = autolaunch
//         .is_enabled()
//         .map_err(|e| format!("检查自启状态失败: {}", e))?;
//     if let Err(e) = crate::config::set_autostart_config(state) {
//         eprintln!("同步开机自启状态到配置文件失败: {}", e);
//     }
//     Ok(state)
// }

// /// 设置系统托盘图标可见性。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// visible: bool - 图标是否可见
// #[tauri::command]
// pub fn set_tray_icon_visible(visible: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.tray_icon_visible = visible;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置启动时最小化到托盘。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用启动时最小化到托盘
// #[tauri::command]
// pub fn set_minimize_to_tray(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.minimize_to_tray = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置自动保存剪贴板历史。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用自动保存剪贴板历史
// #[tauri::command]
// pub fn set_auto_save(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.auto_save = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置历史记录保留天数。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// days: u32 - 保留天数
// #[tauri::command]
// pub fn set_retention_days(days: u32) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.retention_days = days;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// // --------------- 2. 剪贴板参数 ---------------

// /// 设置最大历史记录数量。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// max_items: u32 - 最大历史记录数量
// #[tauri::command]
// pub fn set_max_history_items(max_items: u32) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.max_history_items = max_items;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置忽略短文本的最小字符数。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// min_length: u32 - 小于该长度的文本将被忽略
// #[tauri::command]
// pub fn set_ignore_short_text(min_length: u32) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ignore_short_text_len = min_length;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置忽略大文件的大小 (MB)。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// min_capacity: u32 - 大于等于该值的文件（MB）将被忽略
// #[tauri::command]
// pub fn set_ignore_big_file(min_capacity: u32) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ignore_big_file_mb = min_capacity;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 添加一个忽略的应用（按应用名匹配）。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// app_name: String - 应用名
// #[tauri::command]
// pub fn add_ignored_app(app_name: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         if !cfg.ignored_apps.contains(&app_name) {
//             cfg.ignored_apps.push(app_name);
//         }
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 移除一个忽略的应用。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// app_name: String - 应用名
// #[tauri::command]
// pub fn remove_ignored_app(app_name: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ignored_apps.retain(|a| a != &app_name);
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 清空所有忽略的应用。作为 Tauri Command 暴露给前端调用。
// #[tauri::command]
// pub fn clear_all_ignored_apps() {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ignored_apps.clear();
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置自动分类开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用自动分类
// #[tauri::command]
// pub fn set_auto_classify(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.auto_classify = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置 OCR 自动识别开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用 OCR 自动识别
// #[tauri::command]
// pub fn set_ocr_auto_recognition(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ocr_auto_recognition = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置删除确认对话框开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否在删除时显示确认对话框
// #[tauri::command]
// pub fn set_delete_confirmation(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.delete_confirmation = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置删除时是否保留已收藏的内容。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否在删除时保留收藏内容
// #[tauri::command]
// pub fn set_keep_favorites(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.keep_favorites_on_delete = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置自动排序开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用自动排序
// #[tauri::command]
// pub fn set_auto_sort(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.auto_sort = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// // --------------- 4. AI Agent 相关 ---------------

// /// 设置 AI 助手启用状态。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用 AI 助手
// #[tauri::command]
// pub fn set_ai_enabled(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ai_enabled = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置 AI 服务提供商（例如 "openai"）。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// service: String - 服务提供商标识
// #[tauri::command]
// pub fn set_ai_service(service: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ai_service = if service.is_empty() {
//             None
//         } else {
//             Some(service)
//         };
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置 AI API Key。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// api_key: String - API Key
// #[tauri::command]
// pub fn set_ai_api_key(api_key: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ai_api_key = if api_key.is_empty() {
//             None
//         } else {
//             Some(api_key)
//         };
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置 AI 自动打 Tag。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用自动打标签
// #[tauri::command]
// pub fn set_ai_auto_tag(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ai_auto_tag = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置 AI 自动摘要。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用自动摘要
// #[tauri::command]
// pub fn set_ai_auto_summary(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ai_auto_summary = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置 AI 翻译功能。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用翻译功能
// #[tauri::command]
// pub fn set_ai_translation(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ai_translation = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置 AI 联网搜索功能。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用联网搜索
// #[tauri::command]
// pub fn set_ai_web_search(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ai_web_search = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// // --------------- 5. 安全与隐私 ---------------

// /// 设置敏感词过滤总开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用敏感词过滤
// #[tauri::command]
// pub fn set_sensitive_filter(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.sensitive_filter = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置密码过滤开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用密码过滤
// #[tauri::command]
// pub fn set_filter_passwords(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.filter_passwords = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置银行卡号过滤开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用银行卡号过滤
// #[tauri::command]
// pub fn set_filter_bank_cards(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.filter_bank_cards = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置身份证号过滤开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用身份证号过滤
// #[tauri::command]
// pub fn set_filter_id_cards(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.filter_id_cards = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置手机号过滤开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用手机号过滤
// #[tauri::command]
// pub fn set_filter_phone_numbers(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.filter_phone_numbers = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置隐私记录自动清理天数。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// days: u32 - 保留天数
// #[tauri::command]
// pub fn set_privacy_retention_days(days: u32) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.privacy_retention_days = days;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 获取所有被标记为隐私的记录 ID 列表（JSON 字符串）。作为 Tauri Command 暴露给前端调用。
// /// # Returns
// /// String - 隐私记录 ID 列表的 JSON 字符串表示
// #[tauri::command]
// pub fn get_privacy_records() -> String {
//     if let Some(lock) = CONFIG.get() {
//         let cfg = lock.read().unwrap();
//         serde_json::to_string_pretty(&cfg.privacy_records).unwrap_or_default()
//     } else {
//         "".to_string()
//     }
// }

// /// 删除所有隐私记录。作为 Tauri Command 暴露给前端调用。
// #[tauri::command]
// pub fn delete_all_privacy_records() {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.privacy_records.clear();
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// // --------------- 6. 数据备份 ---------------

// /// 设置数据存储路径，作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// String - 新的数据存储路径
// /// # Returns
// /// String - 设置结果信息
// #[tauri::command]
// pub fn set_storage_path(path: String) -> String {
//     if path.is_empty() {
//         // 清空存储路径
//         if let Some(lock) = CONFIG.get() {
//             let mut cfg = lock.write().unwrap();
//             cfg.storage_path = None;
//             drop(cfg);
//             return if save_config(lock.read().unwrap().clone()).is_ok() {
//                 "storage path cleared".to_string()
//             } else {
//                 "failed to save config".to_string()
//             };
//         }
//         "config not initialized".to_string()
//     } else {
//         // 转换 String → PathBuf 并调用内部函数
//         set_db_storage_path(PathBuf::from(path))
//     }
// }

// /// 设置自动备份开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用自动备份
// #[tauri::command]
// pub fn set_auto_backup(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.auto_backup = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置备份频率。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// frequency: String - 备份频率（"daily"/"weekly"/"monthly"）
// #[tauri::command]
// pub fn set_backup_frequency(frequency: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.backup_frequency = frequency;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置最近一次备份文件路径。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// path: String - 备份文件路径
// #[tauri::command]
// pub fn set_last_backup_path(path: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.last_backup_path = if path.is_empty() { None } else { Some(path) };
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// // --------------- 7. 云端同步 ---------------

// /// 设置云端同步启用状态。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否启用云端同步
// #[tauri::command]
// pub fn set_cloud_sync_enabled(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.cloud_sync_enabled = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置同步频率。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// frequency: String - 同步频率（例如 "realtime"/"5min"/"15min"/"1hour"）
// #[tauri::command]
// pub fn set_sync_frequency(frequency: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.sync_frequency = frequency;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置同步内容类型。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// content_type: String - 同步内容类型（例如 "onlytxt"/"containphoto"/"containfile"）
// #[tauri::command]
// pub fn set_sync_content_type(content_type: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.sync_content_type = content_type;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置云端数据加密开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否对云端数据进行加密
// #[tauri::command]
// pub fn set_encrypt_cloud_data(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.encrypt_cloud_data = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置仅在 WiFi 下进行同步开关。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// enabled: bool - 是否仅在 WiFi 下进行同步
// #[tauri::command]
// pub fn set_sync_only_wifi(enabled: bool) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.sync_only_wifi = enabled;
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// // --------------- 8. 用户信息 ---------------
// /// 设置用户名。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// username: String - 用户名
// #[tauri::command]
// pub fn set_username(username: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.username = if username.is_empty() {
//             None
//         } else {
//             Some(username)
//         };
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置邮箱。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// email: String - 邮箱地址
// #[tauri::command]
// pub fn set_email(email: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.email = if email.is_empty() { None } else { Some(email) };
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置用户简介。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// bio: String - 用户简介
// #[tauri::command]
// pub fn set_bio(bio: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.bio = if bio.is_empty() { None } else { Some(bio) };
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置头像文件路径。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// avatar_path: String - 头像文件路径
// #[tauri::command]
// pub fn set_avatar_path(avatar_path: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.avatar_path = if avatar_path.is_empty() {
//             None
//         } else {
//             Some(avatar_path)
//         };
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// // --------------- 9. OCR 设置 ---------------
// /// 设置 OCR 提供商。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// provider: String - OCR 提供商标识
// #[tauri::command]
// pub fn set_ocr_provider(provider: String) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ocr_provider = if provider.is_empty() {
//             None
//         } else {
//             Some(provider)
//         };
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置 OCR 语言列表。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// languages: Vec<String> - OCR 语言列表
// #[tauri::command]
// pub fn set_ocr_languages(languages: Vec<String>) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         if languages.is_empty() {
//             cfg.ocr_languages = None;
//         } else {
//             cfg.ocr_languages = Some(languages);
//         }
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置 OCR 置信度阈值。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// threshold: f32 - 置信度阈值（0.0 - 1.0）
// #[tauri::command]
// pub fn set_ocr_confidence_threshold(threshold: f32) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ocr_confidence_threshold = Some(threshold);
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

// /// 设置 OCR 超时时间。作为 Tauri Command 暴露给前端调用。
// /// # Param
// /// timeout_secs: u64 - 超时时间（秒）
// #[tauri::command]
// pub fn set_ocr_timeout_secs(timeout_secs: u64) {
//     if let Some(lock) = CONFIG.get() {
//         let mut cfg = lock.write().unwrap();
//         cfg.ocr_timeout_secs = Some(timeout_secs);
//     }
//     save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
// }

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_update_simple_config_item() {
        // 确保配置已初始化
        let _ = init_config();

        // 1. 测试布尔值更新 (TrayIconVisible)
        let key = ConfigKey::TrayIconVisible;
        // 先设置为 true
        update_simple_config_item(&key, json!(true)).unwrap();
        assert_eq!(
            CONFIG.get().unwrap().read().unwrap().tray_icon_visible,
            true
        );

        // 设置为 false
        let res = update_simple_config_item(&key, json!(false));
        assert_eq!(res, Ok(true));
        assert_eq!(
            CONFIG.get().unwrap().read().unwrap().tray_icon_visible,
            false
        );

        // 2. 测试数值更新 (MaxHistoryItems)
        let key = ConfigKey::MaxHistoryItems;
        let res = update_simple_config_item(&key, json!(999));
        assert_eq!(res, Ok(true));
        assert_eq!(CONFIG.get().unwrap().read().unwrap().max_history_items, 999);

        // 3. 测试字符串更新 (GlobalShortcut)
        let key = ConfigKey::GlobalShortcut;
        let res = update_simple_config_item(&key, json!("Ctrl+Alt+K"));
        assert_eq!(res, Ok(true));
        assert_eq!(
            CONFIG.get().unwrap().read().unwrap().global_shortcut,
            "Ctrl+Alt+K"
        );

        // 4. 测试 Option 类型更新 (AiApiKey)
        let key = ConfigKey::AiApiKey;
        let res = update_simple_config_item(&key, json!("sk-123456"));
        assert_eq!(res, Ok(true));
        assert_eq!(
            CONFIG.get().unwrap().read().unwrap().ai_api_key,
            Some("sk-123456".to_string())
        );

        let res = update_simple_config_item(&key, json!(null));
        assert_eq!(res, Ok(true));
        assert_eq!(CONFIG.get().unwrap().read().unwrap().ai_api_key, None);

        // 5. 测试类型错误
        let key = ConfigKey::MaxHistoryItems;
        let res = update_simple_config_item(&key, json!("not a number"));
        assert!(res.is_err());
        // 确保值未改变
        assert_eq!(CONFIG.get().unwrap().read().unwrap().max_history_items, 999);

        // 6. 测试 Autostart (应返回 Ok(false))
        let key = ConfigKey::Autostart;
        let res = update_simple_config_item(&key, json!(true));
        assert_eq!(res, Ok(false));
    }
}
