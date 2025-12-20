use crate::app_setup;
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
    // AI 服务提供商标识（例如 "openai"、"azure" 等）
    // pub ai_service: Option<String>,
    /// AI 提供商 (default | openai | google | custom | ...)
    pub ai_provider: String,
    /// AI 模型名称
    pub ai_model: String,
    /// AI 基础 URL (custom时)
    pub ai_base_url: Option<String>,
    /// AI 采样温度
    pub ai_temperature: f32,
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
    // 隐私记录自动清理天数（天）
    // pub privacy_retention_days: u32,
    // 标记为隐私的记录 ID 列表（可用于快速查询/导出）
    // pub privacy_records: Vec<String>,

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
    // AI 服务提供商标识
    // AiService,
    /// AI 提供商
    AiProvider,
    /// AI 模型名称
    AiModel,
    /// AI 基础 URL
    AiBaseUrl,
    /// AI 采样温度
    AiTemperature,
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
    // 隐私记录自动清理天数
    // PrivacyRetentionDays,
    // 标记为隐私的记录 ID 列表
    // PrivacyRecords,

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
        // "ai_service" => Some(ConfigKey::AiService),
        "ai_provider" => Some(ConfigKey::AiProvider),
        "ai_model" => Some(ConfigKey::AiModel),
        "ai_base_url" => Some(ConfigKey::AiBaseUrl),
        "ai_temperature" => Some(ConfigKey::AiTemperature),
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
        // "privacy_retention_days" => Some(ConfigKey::PrivacyRetentionDays),
        // "privacy_records" => Some(ConfigKey::PrivacyRecords),
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
    "Shift+V".to_string()
}
fn default_shortcut_2() -> String {
    "Shift+Alt+C".to_string()
}
fn default_shortcut_3() -> String {
    "Shift+Alt+A".to_string()
} // 新增
fn default_shortcut_4() -> String {
    "Shift+Ctrl+V".to_string()
} // 新增
fn default_shortcut_5() -> String {
    "Shift+Ctrl+Delete".to_string()
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
            ignore_short_text_len: 0,       // 忽略短文本长度：不忽略(0表示不忽略)
            ignore_big_file_mb: 5,          // 忽略大文件大小：5MB
            ignored_apps: Vec::new(),       // 忽略的应用列表：空
            auto_classify: true,            // 自动分类：是
            ocr_auto_recognition: false,    // OCR 自动识别：否
            delete_confirmation: true,      // 删除确认对话框：是
            keep_favorites_on_delete: true, // 删除时保留收藏：是
            auto_sort: false,               // 自动排序：否

            // AI
            ai_enabled: false, // AI 助手：关
            // ai_service: None,                   // AI 服务提供商：无
            ai_provider: "default".to_string(), // AI 提供商：默认
            ai_model: "".to_string(),           // AI 模型名称：空
            ai_base_url: None,
            ai_temperature: 0.7,    // AI 采样温度：0.7
            ai_api_key: None,       // AI API Key：无
            ai_auto_tag: false,     // AI 自动打标签：否
            ai_auto_summary: false, // AI 自动摘要：否
            ai_translation: false,  // AI 翻译功能：否
            ai_web_search: false,   // AI 联网搜索：否

            // 隐私
            sensitive_filter: true,     // 敏感词过滤：开
            filter_passwords: true,     // 过滤密码：是
            filter_bank_cards: true,    // 过滤银行卡号：是
            filter_id_cards: true,      // 过滤身份证号：是
            filter_phone_numbers: true, // 过滤手机号：是
            // privacy_retention_days: 90,  // 隐私记录保留天数：90天
            // privacy_records: Vec::new(), // 隐私记录列表：空

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
    // 🔥 修复：强制规范化路径分隔符
    let path_str = path.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    let normalized_path_str = path_str.replace("/", "\\");

    #[cfg(not(target_os = "windows"))]
    let normalized_path_str = path_str;

    let normalized_path = PathBuf::from(normalized_path_str);

    println!("🔄 设置配置路径(已规范化): {}", normalized_path.display());
    let mut global_path = CONFIG_PATH_GLOBAL.write().unwrap();
    *global_path = Some(normalized_path);
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
/// 将路径转换为正斜杠格式（跨平台统一）
fn normalize_to_forward_slashes(path: &str) -> String {
    path.replace("\\", "/")
}
/// 初始化全局配置。如果存在配置文件则加载，否则使用默认配置并创建文件。
/// # Returns
/// String - 初始化结果信息
pub fn init_config() -> String {
    let config_path = get_config_path();

    let mut config = if config_path.exists() {
        // 读取现有配置文件
        let data = fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        // 使用默认配置并创建文件
        Config::default()
    };

    if config.storage_path.is_none() || config.storage_path.as_ref().unwrap().trim().is_empty() {
        // 获取配置文件的父目录作为默认存储路径
        let default_storage_path = if let Some(parent) = config_path.parent() {
            parent.to_path_buf()
        } else {
            // 如果无法获取父目录，使用当前目录
            PathBuf::from(".")
        };

        // 统一使用正斜杠
        let default_path_str =
            normalize_to_forward_slashes(&default_storage_path.to_string_lossy());
        println!("🔄 设置默认存储路径: {}", default_path_str);
        config.storage_path = Some(default_path_str);

        // 确保目录存在
        if let Err(e) = fs::create_dir_all(&default_storage_path) {
            eprintln!("⚠️ 创建默认存储目录失败: {}", e);
        }
    }

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    // 创建配置文件
    let mut file = match fs::File::create(&config_path) {
        Ok(file) => file,
        Err(e) => return format!("创建配置文件失败: {}", e),
    };

    let data = match serde_json::to_string_pretty(&config) {
        Ok(data) => data,
        Err(e) => return format!("序列化配置失败: {}", e),
    };

    match file.write_all(data.as_bytes()) {
        Ok(_) => println!("✅ 配置文件已创建/更新: {}", config_path.display()),
        Err(e) => return format!("写入配置文件失败: {}", e),
    }

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
        // ConfigKey::AiService => update_cfg!(ai_service, Option<String>),
        ConfigKey::AiProvider => update_cfg!(ai_provider, String),
        ConfigKey::AiModel => update_cfg!(ai_model, String),
        ConfigKey::AiBaseUrl => update_cfg!(ai_base_url, Option<String>),
        ConfigKey::AiTemperature => update_cfg!(ai_temperature, f32),
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
        // ConfigKey::PrivacyRetentionDays => update_cfg!(privacy_retention_days, u32),
        // ConfigKey::PrivacyRecords => update_cfg!(privacy_records, Vec<String>),
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
    println!(
        "🚚 开始迁移数据文件从 {} 到 {}",
        old_path.display(),
        new_path.display()
    );

    // 确保新路径存在
    if let Err(e) = fs::create_dir_all(new_path) {
        return Err(format!("创建新存储路径失败: {}", e));
    }

    // 🔥 关键修复：在迁移前先清理新路径下的现有文件
    println!("🧹 检查并清理新路径下的现有文件...");
    let files_to_clean = vec![("smartpaste.db", "数据库文件"), ("files", "文件目录")];

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

    let files_to_migrate = vec![("smartpaste.db", "数据库文件"), ("files", "文件目录")];

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

    // 🆕 新增功能：迁移完成后删除原路径下的 files 文件夹
    let old_files_dir = old_path.join("files");
    if old_files_dir.exists() && old_files_dir.is_dir() {
        println!(
            "🗑️ 开始删除原路径下的 files 文件夹: {}",
            old_files_dir.display()
        );
        match fs::remove_dir_all(&old_files_dir) {
            Ok(_) => println!("✅ 已成功删除原路径下的 files 文件夹"),
            Err(e) => {
                // 注意：这里不返回错误，只记录日志，因为迁移已经成功
                println!("⚠️ 删除原路径下的 files 文件夹失败: {}", e);
                println!("ℹ️ 这可能是因为文件正在使用中或权限不足，但迁移已完成");
            }
        }
    } else {
        println!("ℹ️ 原路径下没有 files 文件夹，无需删除");
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
                "目标路径不是目录",
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
pub fn get_current_storage_path() -> PathBuf {
    // 首先检查配置中的存储路径
    if let Some(lock) = CONFIG.get() {
        let cfg = lock.read().unwrap();
        if let Some(ref path_str) = cfg.storage_path {
            if !path_str.trim().is_empty() {
                // 🔥 修复：读取时也进行规范化，防止旧配置污染
                #[cfg(target_os = "windows")]
                let clean_path = path_str.replace("/", "\\");
                #[cfg(not(target_os = "windows"))]
                let clean_path = path_str.clone();

                return PathBuf::from(clean_path);
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
/// * `"ai_api_key"`: `Option<String>` - AI API Key
/// * `"ai_auto_tag"`: `bool` - 是否启用 AI 自动打标签
/// * `"ai_auto_summary"`: `bool` - 是否启用 AI 自动摘要
/// * `"ai_translation"`: `bool` - 是否启用 AI 翻译功能
/// * `"ai_web_search"`: `bool` - 是否启用 AI 联网搜索功能
/// * `"ai_provider"`: `String` - AI 提供商名称
/// * `"ai_model"`: `String` - AI 模型名称
/// * `"ai_base_url"`: `Option<String>` - AI 服务基础 URL
/// * `"ai_temperature"`: `f32` - AI 采样温度
///
/// **安全与隐私**
/// * `"sensitive_filter"`: `bool` - 是否启用敏感词过滤总开关
/// * `"filter_passwords"`: `bool` - 是否过滤密码类型内容
/// * `"filter_bank_cards"`: `bool` - 是否过滤银行卡号
/// * `"filter_id_cards"`: `bool` - 是否过滤身份证号
/// * `"filter_phone_numbers"`: `bool` - 是否过滤手机号
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
        #[cfg(target_os = "windows")]
        let new_path_str = new_path_str.replace("/", "\\");
        // 获取当前存储路径
        let current_path = get_current_storage_path();
        let new_path = PathBuf::from(&new_path_str);

        println!(
            "🔄 开始修改存储路径: {} -> {}",
            current_path.display(),
            new_path.display()
        );

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
        // 我们需要将数据库中的旧路径更新为新路径
        let old_path_str = current_path.to_string_lossy().replace('\\', "/");
        let new_path_str = new_path.to_string_lossy().replace('\\', "/");

        println!("🔄 开始更新数据库中的文件路径...");
        println!("  旧路径: {}", old_path_str);
        println!("  新路径: {}", new_path_str);

        // 更新数据库中的文件路径
        match crate::db::update_data_path(&old_path_str, &new_path_str) {
            Ok(count) => {
                println!("✅ 成功更新了 {} 条记录的路径", count);
                if count == 0 {
                    println!("⚠️ 没有找到需要更新的文件路径记录，这可能是正常的");
                }
            }
            Err(e) => {
                println!("⚠️ 更新数据库路径失败: {}", e);
                // 这里不返回错误，继续执行，因为迁移已经成功
            }
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
        println!(
            "🔍 设置配置路径后，当前配置路径: {}",
            current_path_after_set.display()
        );

        if current_path_after_set != new_config_path {
            println!(
                "❌ 配置路径设置失败，期望: {}，实际: {}",
                new_config_path.display(),
                current_path_after_set.display()
            );
            set_config_path(old_config_path);
            return "Failed to set config path".to_string();
        }

        let cfg_clone = CONFIG.get().unwrap().read().unwrap().clone();
        match save_config(cfg_clone.clone()) {
            Ok(_) => {
                // 更新数据库路径
                let new_db_path = new_path.join("smartpaste.db");
                crate::db::set_db_path(new_db_path);

                println!(
                    "✅ 存储路径修改完成，配置已保存到新路径: {}",
                    new_config_path.display()
                );

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
                    println!(
                        "📝 同时更新默认路径的配置文件: {}",
                        default_config_path.display()
                    );

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
                // --- 修复后的动态更新托盘图标可见性逻辑 ---
                if config_key == ConfigKey::TrayIconVisible {
                    if let Ok(visible) = serde_json::from_value::<bool>(value.clone()) {
                        println!("🔄 动态更新托盘图标可见性为: {}", visible);

                        // 关键修改：通过全局函数获取存储的 TrayIconHandle
                        if let Some(tray) = app_setup::get_tray_icon_handle() {
                            if let Err(e) = tray.set_visible(visible) {
                                println!("❌ 托盘图标设置可见性失败: {:?}", e);
                            } else {
                                println!("✅ 托盘图标可见性设置成功");
                            }
                        } else {
                            // 如果句柄不存在，则说明托盘未创建（在启动时配置为不可见）。
                            if visible {
                                // 启动时托盘未创建，配置现在改为可见，提示用户重启
                                println!("⚠️ 托盘图标未创建。新的可见性设置将在下次启动时生效，请重启应用");
                            } else {
                                // 如果托盘不存在，配置改为不可见，忽略。
                                println!("ℹ️ 托盘图标未创建，忽略设置为不可见的操作");
                            }
                        }
                    } else {
                        return format!("Invalid type for key '{}'", key);
                    }
                }
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
/// 强制从当前设置的路径重新加载配置到内存
/// 用于在运行时切换存储路径后更新全局状态
pub fn reload_config() -> String {
    let config_path = get_config_path();
    println!("🔄 正在重新加载配置: {}", config_path.display());

    // 1. 读取文件内容
    let config: Config = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("❌ 解析配置文件失败: {}", e);
                    return format!("Parse error: {}", e);
                }
            },
            Err(e) => {
                eprintln!("❌ 读取配置文件失败: {}", e);
                return format!("Read error: {}", e);
            }
        }
    } else {
        eprintln!("⚠️ 配置文件不存在: {}", config_path.display());
        return "File not found".to_string();
    };

    // 2. 更新全局 RwLock
    if let Some(lock) = CONFIG.get() {
        let mut global_cfg = lock.write().unwrap();
        *global_cfg = config; // 👈 关键点：直接覆盖内存中的旧配置
        println!("✅ 内存配置已更新");
        "reloaded successfully".to_string()
    } else {
        // 理论上不应该走到这里，除非 init_config 还没被调用过
        // 如果没初始化，尝试初始化
        CONFIG
            .set(RwLock::new(config))
            .map(|_| "initialized successfully".to_string())
            .unwrap_or_else(|_| "Unknown error".to_string())
    }
}

/// 全量同步并应用配置。作为 Tauri Command 暴露给前端调用。
#[tauri::command]
pub async fn sync_and_apply_config(app: tauri::AppHandle, content: String) -> Result<String, String> {
    // 1. 解析 JSON 确保数据格式正确
    let new_config: Config = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置失败: {}", e))?;

    // 2. 调用你提到的 save_config 将配置写入磁盘
    save_config(new_config)?;

    // 3. 调用 reload_config 将磁盘内容加载到内存变量 CONFIG
    let reload_res = reload_config();
    if reload_res != "reloaded successfully" {
        return Err(format!("内存刷新失败: {}", reload_res));
    }

    // 4. 关键步骤：重置快捷键监听
    // 必须调用 app_setup 中的函数，否则系统依然占用旧的快捷键
    if let Err(e) = crate::app_setup::setup_global_shortcuts(app) {
        return Err(format!("配置已保存但快捷键重置失败: {}", e));
    }

    println!("🚀 配置全量同步完成，快捷键已即时刷新");
    Ok("Config synchronized and applied".to_string())
}

/// 按传入参数获取配置信息。作为 Tauri Command 暴露给前端调用。
///
/// 该函数是前端获取配置的统一入口。根据传入的 `key` 找到对应的配置项，并返回其当前值。
///
/// # Param
/// * `key`: &str - 配置项名称。支持的键名与 `set_config_item` 相同：
///
/// **通用设置**
/// * `"autostart"`: 返回 `bool` - 是否开机自启
/// * `"tray_icon_visible"`: 返回 `bool` - 托盘图标是否可见
/// * `"minimize_to_tray"`: 返回 `bool` - 启动时是否最小化到托盘
/// * `"auto_save"`: 返回 `bool` - 是否自动保存剪贴板历史
/// * `"retention_days"`: 返回 `u32` - 历史记录保留天数
/// * `"global_shortcut"`: 返回 `String` - 主界面快捷键
/// * `"global_shortcut_2"`: 返回 `String` - 第二界面快捷键
/// * `"global_shortcut_3"`: 返回 `String` - 第三快捷键
/// * `"global_shortcut_4"`: 返回 `String` - 第四快捷键
/// * `"global_shortcut_5"`: 返回 `String` - 第五快捷键
///
/// **剪贴板参数**
/// * `"max_history_items"`: 返回 `u32` - 最大历史记录数量
/// * `"ignore_short_text_len"`: 返回 `u32` - 忽略短文本的最短字符数
/// * `"ignore_big_file_mb"`: 返回 `u32` - 忽略大文件的大小阈值 (MB)
/// * `"ignored_apps"`: 返回 `Vec<String>` - 被忽略的应用列表
/// * `"auto_classify"`: 返回 `bool` - 是否自动分类
/// * `"ocr_auto_recognition"`: 返回 `bool` - 是否启用 OCR 自动识别
/// * `"delete_confirmation"`: 返回 `bool` - 删除时是否弹出确认对话框
/// * `"keep_favorites_on_delete"`: 返回 `bool` - 删除时是否保留收藏内容
/// * `"auto_sort"`: 返回 `bool` - 是否启用自动排序
///
/// **AI Agent 相关**
/// * `"ai_enabled"`: 返回 `bool` - 是否启用 AI 助手
/// * `"ai_api_key"`: 返回 `Option<String>` - AI API Key
/// * `"ai_auto_tag"`: 返回 `bool` - 是否启用 AI 自动打标签
/// * `"ai_auto_summary"`: 返回 `bool` - 是否启用 AI 自动摘要
/// * `"ai_translation"`: 返回 `bool` - 是否启用 AI 翻译功能
/// * `"ai_web_search"`: 返回 `bool` - 是否启用 AI 联网搜索功能
/// * `"ai_provider"`: 返回 `String` - AI 提供商
/// * `"ai_model"`: 返回 `String` - AI 模型
/// * `"ai_base_url"`: 返回 `Option<String>` - AI 基础 URL
/// * `"ai_temperature"`: 返回 `f32` - AI 温度参数
///
/// **安全与隐私**
/// * `"sensitive_filter"`: 返回 `bool` - 是否启用敏感词过滤总开关
/// * `"filter_passwords"`: 返回 `bool` - 是否过滤密码类型内容
/// * `"filter_bank_cards"`: 返回 `bool` - 是否过滤银行卡号
/// * `"filter_id_cards"`: 返回 `bool` - 是否过滤身份证号
/// * `"filter_phone_numbers"`: 返回 `bool` - 是否过滤手机号
///
/// **数据备份**
/// * `"storage_path"`: 返回 `Option<String>` - 数据存储路径
/// * `"auto_backup"`: 返回 `bool` - 是否启用自动备份
/// * `"backup_frequency"`: 返回 `String` - 备份频率
/// * `"last_backup_path"`: 返回 `Option<String>` - 最近一次备份文件路径
///
/// **云端同步**
/// * `"cloud_sync_enabled"`: 返回 `bool` - 是否启用云端同步
/// * `"sync_frequency"`: 返回 `String` - 同步频率
/// * `"sync_content_type"`: 返回 `String` - 同步内容类型
/// * `"encrypt_cloud_data"`: 返回 `bool` - 是否对云端数据进行加密
/// * `"sync_only_wifi"`: 返回 `bool` - 是否仅在 WiFi 下进行同步
///
/// **用户信息**
/// * `"username"`: 返回 `Option<String>` - 用户名
/// * `"email"`: 返回 `Option<String>` - 邮箱
/// * `"bio"`: 返回 `Option<String>` - 用户简介
/// * `"avatar_path"`: 返回 `Option<String>` - 头像文件路径
///
/// **OCR 设置**
/// * `"ocr_provider"`: 返回 `Option<String>` - OCR 提供商标识
/// * `"ocr_languages"`: 返回 `Option<Vec<String>>` - OCR 语言列表
/// * `"ocr_confidence_threshold"`: 返回 `Option<f32>` - OCR 置信度阈值
/// * `"ocr_timeout_secs"`: 返回 `Option<u64>` - OCR 超时时间
///
/// # Returns
/// Result<serde_json::Value, String> - 成功返回配置值的 JSON 表示，失败返回错误信息
#[tauri::command]
pub fn get_config_item(key: &str) -> Result<serde_json::Value, String> {
    let config_key = match parse_config_key(key) {
        Some(k) => k,
        None => return Err(format!("Invalid config key: {}", key)),
    };

    if let Some(lock) = CONFIG.get() {
        let cfg = lock.read().unwrap();

        let value = match config_key {
            // 通用设置
            ConfigKey::Autostart => serde_json::to_value(&cfg.autostart),
            ConfigKey::TrayIconVisible => serde_json::to_value(&cfg.tray_icon_visible),
            ConfigKey::MinimizeToTray => serde_json::to_value(&cfg.minimize_to_tray),
            ConfigKey::AutoSave => serde_json::to_value(&cfg.auto_save),
            ConfigKey::RetentionDays => serde_json::to_value(&cfg.retention_days),
            ConfigKey::GlobalShortcut => serde_json::to_value(&cfg.global_shortcut),
            ConfigKey::GlobalShortcut2 => serde_json::to_value(&cfg.global_shortcut_2),
            ConfigKey::GlobalShortcut3 => serde_json::to_value(&cfg.global_shortcut_3),
            ConfigKey::GlobalShortcut4 => serde_json::to_value(&cfg.global_shortcut_4),
            ConfigKey::GlobalShortcut5 => serde_json::to_value(&cfg.global_shortcut_5),

            // 剪贴板参数
            ConfigKey::MaxHistoryItems => serde_json::to_value(&cfg.max_history_items),
            ConfigKey::IgnoreShortTextLen => serde_json::to_value(&cfg.ignore_short_text_len),
            ConfigKey::IgnoreBigFileMb => serde_json::to_value(&cfg.ignore_big_file_mb),
            ConfigKey::IgnoredApps => serde_json::to_value(&cfg.ignored_apps),
            ConfigKey::AutoClassify => serde_json::to_value(&cfg.auto_classify),
            ConfigKey::OcrAutoRecognition => serde_json::to_value(&cfg.ocr_auto_recognition),
            ConfigKey::DeleteConfirmation => serde_json::to_value(&cfg.delete_confirmation),
            ConfigKey::KeepFavoritesOnDelete => serde_json::to_value(&cfg.keep_favorites_on_delete),
            ConfigKey::AutoSort => serde_json::to_value(&cfg.auto_sort),

            // AI Agent 相关
            ConfigKey::AiEnabled => serde_json::to_value(&cfg.ai_enabled),
            // ConfigKey::AiService => serde_json::to_value(&cfg.ai_service),
            ConfigKey::AiProvider => serde_json::to_value(&cfg.ai_provider),
            ConfigKey::AiModel => serde_json::to_value(&cfg.ai_model),
            ConfigKey::AiBaseUrl => serde_json::to_value(&cfg.ai_base_url),
            ConfigKey::AiTemperature => serde_json::to_value(&cfg.ai_temperature),
            ConfigKey::AiApiKey => serde_json::to_value(&cfg.ai_api_key),
            ConfigKey::AiAutoTag => serde_json::to_value(&cfg.ai_auto_tag),
            ConfigKey::AiAutoSummary => serde_json::to_value(&cfg.ai_auto_summary),
            ConfigKey::AiTranslation => serde_json::to_value(&cfg.ai_translation),
            ConfigKey::AiWebSearch => serde_json::to_value(&cfg.ai_web_search),

            // 安全与隐私
            ConfigKey::SensitiveFilter => serde_json::to_value(&cfg.sensitive_filter),
            ConfigKey::FilterPasswords => serde_json::to_value(&cfg.filter_passwords),
            ConfigKey::FilterBankCards => serde_json::to_value(&cfg.filter_bank_cards),
            ConfigKey::FilterIdCards => serde_json::to_value(&cfg.filter_id_cards),
            ConfigKey::FilterPhoneNumbers => serde_json::to_value(&cfg.filter_phone_numbers),
            // ConfigKey::PrivacyRetentionDays => serde_json::to_value(&cfg.privacy_retention_days),
            // ConfigKey::PrivacyRecords => serde_json::to_value(&cfg.privacy_records),

            // 数据备份
            ConfigKey::StoragePath => serde_json::to_value(&cfg.storage_path),
            ConfigKey::AutoBackup => serde_json::to_value(&cfg.auto_backup),
            ConfigKey::BackupFrequency => serde_json::to_value(&cfg.backup_frequency),
            ConfigKey::LastBackupPath => serde_json::to_value(&cfg.last_backup_path),

            // 云端同步
            ConfigKey::CloudSyncEnabled => serde_json::to_value(&cfg.cloud_sync_enabled),
            ConfigKey::SyncFrequency => serde_json::to_value(&cfg.sync_frequency),
            ConfigKey::SyncContentType => serde_json::to_value(&cfg.sync_content_type),
            ConfigKey::EncryptCloudData => serde_json::to_value(&cfg.encrypt_cloud_data),
            ConfigKey::SyncOnlyWifi => serde_json::to_value(&cfg.sync_only_wifi),

            // 用户信息
            ConfigKey::Username => serde_json::to_value(&cfg.username),
            ConfigKey::Email => serde_json::to_value(&cfg.email),
            ConfigKey::Bio => serde_json::to_value(&cfg.bio),
            ConfigKey::AvatarPath => serde_json::to_value(&cfg.avatar_path),

            // OCR 设置
            ConfigKey::OcrProvider => serde_json::to_value(&cfg.ocr_provider),
            ConfigKey::OcrLanguages => serde_json::to_value(&cfg.ocr_languages),
            ConfigKey::OcrConfidenceThreshold => {
                serde_json::to_value(&cfg.ocr_confidence_threshold)
            }
            ConfigKey::OcrTimeoutSecs => serde_json::to_value(&cfg.ocr_timeout_secs),
        };

        value.map_err(|e| format!("Failed to serialize config value: {}", e))
    } else {
        Err("Config not initialized".to_string())
    }
}

#[cfg(test)]
#[path = "test_unit/test_config.rs"]
mod test_config;
