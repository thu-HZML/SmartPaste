use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};
use tauri_plugin_autostart::ManagerExt;
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
// 辅助函数，防止旧 config.json 缺少字段导致解析失败
fn default_shortcut() -> String {
    "Alt+Shift+V".to_string()
}
fn default_shortcut_2() -> String {
    "Alt+Shift+C".to_string()
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
            global_shortcut: default_shortcut(),
            global_shortcut_2: default_shortcut_2(),
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

static CONFIG_PATH_GLOBAL: OnceLock<PathBuf> = OnceLock::new();
pub static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

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

/// 获取配置信息的 JSON 字符串表示。作为 Tauri Command 暴露给前端调用。
/// # Returns
/// String - 配置的 JSON 字符串。若未初始化则返回空字符串。
#[tauri::command]
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
pub fn set_db_storage_path(path: PathBuf) -> String {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.storage_path = Some(path.to_string_lossy().to_string());
        "storage path updated".to_string()
    } else {
        "config not initialized".to_string()
    }
}
/// 设置主快捷键 (修复了死锁问题)
pub fn set_global_shortcut_internal(shortcut: String) {
    // 第一步：先获取写锁，更新内存中的配置
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.global_shortcut = shortcut;
    }
    // 写锁在这里自动释放

    // 第二步：先获取读锁拿到配置副本，然后释放读锁
    let cfg_clone = if let Some(lock) = CONFIG.get() {
        lock.read().unwrap().clone()
    } else {
        return;
    };
    // 读锁在这里自动释放

    // 第三步：调用 save_config (它内部会再次获取写锁，但现在是安全的)
    if let Err(e) = save_config(cfg_clone) {
        eprintln!("❌ 保存配置文件失败: {}", e);
    }
}

/// 设置第二快捷键 (修复了死锁问题)
pub fn set_global_shortcut_2_internal(shortcut: String) {
    // 第一步：更新内存
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.global_shortcut_2 = shortcut;
    }

    // 第二步：获取副本
    let cfg_clone = if let Some(lock) = CONFIG.get() {
        lock.read().unwrap().clone()
    } else {
        return;
    };

    // 第三步：保存
    if let Err(e) = save_config(cfg_clone) {
        eprintln!("❌ 保存配置文件失败: {}", e);
    }
}
/// 设置开机自启配置 (包含持久化保存，已处理死锁问题)
/// # Param
/// enable: bool - 是否启用
pub fn set_autostart_config(enable: bool) -> Result<(), String> {
    // 1. 更新内存中的配置
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.autostart = enable;
    } else {
        return Err("Config not initialized".to_string());
    }

    // 2. 获取配置副本 (此时已释放写锁)
    let cfg_clone = if let Some(lock) = CONFIG.get() {
        lock.read().unwrap().clone()
    } else {
        return Err("Config not initialized".to_string());
    };

    // 3. 保存到文件 (save_config 内部会再次获取锁，但现在是安全的)
    save_config(cfg_clone)
}
// --------------- 1. 通用设置 ---------------

/// 设置或取消应用的开机自启。作为 Tauri command 暴露给前端调用。
/// # Param
/// app: tauri::AppHandle - Tauri 的应用句柄，用于访问应用相关功能。
/// enable: bool - true表示启用开机自启，false表示禁用。
/// # Returns
/// Result<(), String> - 操作成功则返回 Ok(())，失败则返回包含错误信息的 Err。
#[tauri::command]
pub fn set_autostart(app: tauri::AppHandle, enable: bool) -> Result<(), String> {
    let autolaunch = app.autolaunch();

    if enable {
        autolaunch
            .enable()
            .map_err(|e| format!("启用开机自启失败: {}", e))?;
    } else {
        autolaunch
            .disable()
            .map_err(|e| format!("禁用开机自启失败: {}", e))?;
    }
    crate::config::set_autostart_config(enable)?;
    Ok(())
}

/// 检查应用是否已设置为开机自启。作为 Tauri command 暴露给前端调用。
/// # Param
/// app: tauri::AppHandle - Tauri 的应用句柄，用于访问应用相关功能。
/// # Returns
/// Result<bool, String> - 操作成功则返回 Ok(bool)，其中 true 表示已启用自启，false 表示未启用。失败则返回包含错误信息的 Err。
#[tauri::command]
pub fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    let autolaunch = app.autolaunch();

    let state = autolaunch
        .is_enabled()
        .map_err(|e| format!("检查自启状态失败: {}", e))?;
    if let Err(e) = crate::config::set_autostart_config(state) {
        eprintln!("同步开机自启状态到配置文件失败: {}", e);
    }
    Ok(state)
}

/// 设置系统托盘图标可见性。作为 Tauri Command 暴露给前端调用。
/// # Param
/// visible: bool - 图标是否可见
#[tauri::command]
pub fn set_tray_icon_visible(visible: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.tray_icon_visible = visible;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置启动时最小化到托盘。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用启动时最小化到托盘
#[tauri::command]
pub fn set_minimize_to_tray(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.minimize_to_tray = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置自动保存剪贴板历史。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用自动保存剪贴板历史
#[tauri::command]
pub fn set_auto_save(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.auto_save = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置历史记录保留天数。作为 Tauri Command 暴露给前端调用。
/// # Param
/// days: u32 - 保留天数
#[tauri::command]
pub fn set_retention_days(days: u32) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.retention_days = days;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

// --------------- 2. 剪贴板参数 ---------------

/// 设置最大历史记录数量。作为 Tauri Command 暴露给前端调用。
/// # Param
/// max_items: u32 - 最大历史记录数量
#[tauri::command]
pub fn set_max_history_items(max_items: u32) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.max_history_items = max_items;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置忽略短文本的最小字符数。作为 Tauri Command 暴露给前端调用。
/// # Param
/// min_length: u32 - 小于该长度的文本将被忽略
#[tauri::command]
pub fn set_ignore_short_text(min_length: u32) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ignore_short_text_len = min_length;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置忽略大文件的大小 (MB)。作为 Tauri Command 暴露给前端调用。
/// # Param
/// min_capacity: u32 - 大于等于该值的文件（MB）将被忽略
#[tauri::command]
pub fn set_ignore_big_file(min_capacity: u32) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ignore_big_file_mb = min_capacity;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 添加一个忽略的应用（按应用名匹配）。作为 Tauri Command 暴露给前端调用。
/// # Param
/// app_name: String - 应用名
#[tauri::command]
pub fn add_ignored_app(app_name: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        if !cfg.ignored_apps.contains(&app_name) {
            cfg.ignored_apps.push(app_name);
        }
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 移除一个忽略的应用。作为 Tauri Command 暴露给前端调用。
/// # Param
/// app_name: String - 应用名
#[tauri::command]
pub fn remove_ignored_app(app_name: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ignored_apps.retain(|a| a != &app_name);
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 清空所有忽略的应用。作为 Tauri Command 暴露给前端调用。
#[tauri::command]
pub fn clear_all_ignored_apps() {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ignored_apps.clear();
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置自动分类开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用自动分类
#[tauri::command]
pub fn set_auto_classify(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.auto_classify = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置 OCR 自动识别开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用 OCR 自动识别
#[tauri::command]
pub fn set_ocr_auto_recognition(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ocr_auto_recognition = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置删除确认对话框开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否在删除时显示确认对话框
#[tauri::command]
pub fn set_delete_confirmation(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.delete_confirmation = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置删除时是否保留已收藏的内容。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否在删除时保留收藏内容
#[tauri::command]
pub fn set_keep_favorites(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.keep_favorites_on_delete = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置自动排序开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用自动排序
#[tauri::command]
pub fn set_auto_sort(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.auto_sort = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

// --------------- 4. AI Agent 相关 ---------------

/// 设置 AI 助手启用状态。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用 AI 助手
#[tauri::command]
pub fn set_ai_enabled(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ai_enabled = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置 AI 服务提供商（例如 "openai"）。作为 Tauri Command 暴露给前端调用。
/// # Param
/// service: String - 服务提供商标识
#[tauri::command]
pub fn set_ai_service(service: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ai_service = if service.is_empty() {
            None
        } else {
            Some(service)
        };
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置 AI API Key。作为 Tauri Command 暴露给前端调用。
/// # Param
/// api_key: String - API Key
#[tauri::command]
pub fn set_ai_api_key(api_key: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ai_api_key = if api_key.is_empty() {
            None
        } else {
            Some(api_key)
        };
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置 AI 自动打 Tag。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用自动打标签
#[tauri::command]
pub fn set_ai_auto_tag(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ai_auto_tag = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置 AI 自动摘要。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用自动摘要
#[tauri::command]
pub fn set_ai_auto_summary(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ai_auto_summary = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置 AI 翻译功能。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用翻译功能
#[tauri::command]
pub fn set_ai_translation(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ai_translation = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置 AI 联网搜索功能。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用联网搜索
#[tauri::command]
pub fn set_ai_web_search(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ai_web_search = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

// --------------- 5. 安全与隐私 ---------------

/// 设置敏感词过滤总开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用敏感词过滤
#[tauri::command]
pub fn set_sensitive_filter(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.sensitive_filter = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置密码过滤开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用密码过滤
#[tauri::command]
pub fn set_filter_passwords(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.filter_passwords = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置银行卡号过滤开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用银行卡号过滤
#[tauri::command]
pub fn set_filter_bank_cards(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.filter_bank_cards = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置身份证号过滤开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用身份证号过滤
#[tauri::command]
pub fn set_filter_id_cards(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.filter_id_cards = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置手机号过滤开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用手机号过滤
#[tauri::command]
pub fn set_filter_phone_numbers(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.filter_phone_numbers = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置隐私记录自动清理天数。作为 Tauri Command 暴露给前端调用。
/// # Param
/// days: u32 - 保留天数
#[tauri::command]
pub fn set_privacy_retention_days(days: u32) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.privacy_retention_days = days;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 获取所有被标记为隐私的记录 ID 列表（JSON 字符串）。作为 Tauri Command 暴露给前端调用。
/// # Returns
/// String - 隐私记录 ID 列表的 JSON 字符串表示
#[tauri::command]
pub fn get_privacy_records() -> String {
    if let Some(lock) = CONFIG.get() {
        let cfg = lock.read().unwrap();
        serde_json::to_string_pretty(&cfg.privacy_records).unwrap_or_default()
    } else {
        "".to_string()
    }
}

/// 删除所有隐私记录。作为 Tauri Command 暴露给前端调用。
#[tauri::command]
pub fn delete_all_privacy_records() {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.privacy_records.clear();
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

// --------------- 6. 数据备份 ---------------

/// 设置数据存储路径，作为 Tauri Command 暴露给前端调用。
/// # Param
/// String - 新的数据存储路径
/// # Returns
/// String - 设置结果信息
#[tauri::command]
pub fn set_storage_path(path: String) -> String {
    if path.is_empty() {
        // 清空存储路径
        if let Some(lock) = CONFIG.get() {
            let mut cfg = lock.write().unwrap();
            cfg.storage_path = None;
            drop(cfg);
            return if save_config(lock.read().unwrap().clone()).is_ok() {
                "storage path cleared".to_string()
            } else {
                "failed to save config".to_string()
            };
        }
        "config not initialized".to_string()
    } else {
        // 转换 String → PathBuf 并调用内部函数
        set_db_storage_path(PathBuf::from(path))
    }
}

/// 设置自动备份开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用自动备份
#[tauri::command]
pub fn set_auto_backup(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.auto_backup = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置备份频率。作为 Tauri Command 暴露给前端调用。
/// # Param
/// frequency: String - 备份频率（"daily"/"weekly"/"monthly"）
#[tauri::command]
pub fn set_backup_frequency(frequency: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.backup_frequency = frequency;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置最近一次备份文件路径。作为 Tauri Command 暴露给前端调用。
/// # Param
/// path: String - 备份文件路径
#[tauri::command]
pub fn set_last_backup_path(path: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.last_backup_path = if path.is_empty() { None } else { Some(path) };
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

// --------------- 7. 云端同步 ---------------

/// 设置云端同步启用状态。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否启用云端同步
#[tauri::command]
pub fn set_cloud_sync_enabled(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.cloud_sync_enabled = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置同步频率。作为 Tauri Command 暴露给前端调用。
/// # Param
/// frequency: String - 同步频率（例如 "realtime"/"5min"/"15min"/"1hour"）
#[tauri::command]
pub fn set_sync_frequency(frequency: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.sync_frequency = frequency;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置同步内容类型。作为 Tauri Command 暴露给前端调用。
/// # Param
/// content_type: String - 同步内容类型（例如 "onlytxt"/"containphoto"/"containfile"）
#[tauri::command]
pub fn set_sync_content_type(content_type: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.sync_content_type = content_type;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置云端数据加密开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否对云端数据进行加密
#[tauri::command]
pub fn set_encrypt_cloud_data(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.encrypt_cloud_data = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置仅在 WiFi 下进行同步开关。作为 Tauri Command 暴露给前端调用。
/// # Param
/// enabled: bool - 是否仅在 WiFi 下进行同步
#[tauri::command]
pub fn set_sync_only_wifi(enabled: bool) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.sync_only_wifi = enabled;
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

// --------------- 8. 用户信息 ---------------
/// 设置用户名。作为 Tauri Command 暴露给前端调用。
/// # Param
/// username: String - 用户名
#[tauri::command]
pub fn set_username(username: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.username = if username.is_empty() {
            None
        } else {
            Some(username)
        };
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置邮箱。作为 Tauri Command 暴露给前端调用。
/// # Param
/// email: String - 邮箱地址
#[tauri::command]
pub fn set_email(email: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.email = if email.is_empty() { None } else { Some(email) };
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置用户简介。作为 Tauri Command 暴露给前端调用。
/// # Param
/// bio: String - 用户简介
#[tauri::command]
pub fn set_bio(bio: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.bio = if bio.is_empty() { None } else { Some(bio) };
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置头像文件路径。作为 Tauri Command 暴露给前端调用。
/// # Param
/// avatar_path: String - 头像文件路径
#[tauri::command]
pub fn set_avatar_path(avatar_path: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.avatar_path = if avatar_path.is_empty() {
            None
        } else {
            Some(avatar_path)
        };
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

// --------------- 9. OCR 设置 ---------------
/// 设置 OCR 提供商。作为 Tauri Command 暴露给前端调用。
/// # Param
/// provider: String - OCR 提供商标识
#[tauri::command]
pub fn set_ocr_provider(provider: String) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ocr_provider = if provider.is_empty() {
            None
        } else {
            Some(provider)
        };
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置 OCR 语言列表。作为 Tauri Command 暴露给前端调用。
/// # Param
/// languages: Vec<String> - OCR 语言列表
#[tauri::command]
pub fn set_ocr_languages(languages: Vec<String>) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        if languages.is_empty() {
            cfg.ocr_languages = None;
        } else {
            cfg.ocr_languages = Some(languages);
        }
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置 OCR 置信度阈值。作为 Tauri Command 暴露给前端调用。
/// # Param
/// threshold: f32 - 置信度阈值（0.0 - 1.0）
#[tauri::command]
pub fn set_ocr_confidence_threshold(threshold: f32) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ocr_confidence_threshold = Some(threshold);
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}

/// 设置 OCR 超时时间。作为 Tauri Command 暴露给前端调用。
/// # Param
/// timeout_secs: u64 - 超时时间（秒）
#[tauri::command]
pub fn set_ocr_timeout_secs(timeout_secs: u64) {
    if let Some(lock) = CONFIG.get() {
        let mut cfg = lock.write().unwrap();
        cfg.ocr_timeout_secs = Some(timeout_secs);
    }
    save_config(CONFIG.get().unwrap().read().unwrap().clone()).ok();
}
