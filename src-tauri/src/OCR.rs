//! 对OCR相关功能的封装，实现图像文字识别。
//! 依赖 uniocr 库进行 OCR 处理。
// #[cfg(feature = "with_uniocr")]
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;
use uni_ocr::{Language, OcrEngine, OcrOptions, OcrProvider};

/// 辅助函数，解析字符串到 Language 枚举。
fn parse_language(code: &str) -> Result<Language, String> {
    let s = code.trim().to_lowercase();
    match s.as_str() {
        "eng" | "en" | "english" => Ok(Language::English),
        "zh" | "chi" | "chinese" | "zh-cn" | "chi_sim" | "zh_cn" => Ok(Language::Chinese),
        "ja" | "jpn" | "japanese" => Ok(Language::Japanese),
        other => Err(format!("Unsupported language code: {}", other)),
    }
}

/// 辅助函数，解析字符串到 OcrProvider 枚举。
fn parse_provider(provider: &str) -> Result<OcrProvider, String> {
    let s = provider.trim().to_lowercase();
    match s.as_str() {
        "auto" => Ok(OcrProvider::Auto),
        "tesseract" => Ok(OcrProvider::Tesseract),
        "windows" => Ok(OcrProvider::Windows),
        "macos" => Ok(OcrProvider::MacOS),
        other => Err(format!("Unsupported OCR provider: {}", other)),
    }
}

/// 全局 OCR 引擎实例，使用 OnceLock 确保线程安全的单例模式。
static OCR_ENGINE: OnceLock<Mutex<Option<Arc<OcrEngine>>>> = OnceLock::new();

/// 配置 OCR 选项。作为 Tauri Command 暴露给前端调用。
/// # Param
/// provider: Option<String> - 可选的 OCR 提供者名称，若为 None 则使用自动选择。
/// language: Option<Vec<&str>> - 可选的语言列表，若为 None 则使用默认语言。
/// confidence_threshold: Option<f32> - 可选的置信度阈值，若为 None 则使用默认值。
/// timeout_secs: Option<u64> - 可选的超时时间（秒），若为 None 则使用默认值。
/// # Return
/// String - 配置结果的描述信息，若配置失败则返回错误信息。
#[tauri::command]
pub fn configure_ocr(
    provider: Option<String>,
    language: Option<Vec<&str>>,
    confidence_threshold: Option<f32>,
    timeout_secs: Option<u64>,
) -> Result<String, String> {
    let ocr_provider = provider.unwrap_or("auto".to_string());
    let langs = language.unwrap_or_else(|| vec!["eng", "chi"]);
    let confidence = confidence_threshold.unwrap_or(0.8);
    let timeout = timeout_secs.unwrap_or(30);

    // 解析提供者字符串为对应的枚举
    let ocr_provider_parsed = parse_provider(&ocr_provider)
        .map_err(|e| format!("Failed to parse OCR provider: {}", e))?;

    // 解析语言字符串为对应的结构体
    let langs_parsed: Vec<Language> = langs
        .into_iter()
        .map(|code| parse_language(code).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Failed to parse languages: {}", e))?;

    let options = OcrOptions::default()
        .languages(langs_parsed.clone())
        .confidence_threshold(confidence)
        .timeout(Duration::from_secs(timeout));

    let engine = OcrEngine::new(ocr_provider_parsed.clone())
        .map_err(|e| format!("Failed to create OCR engine: {}", e))?
        .with_options(options);

    // 存入 Arc 并替换（允许重复配置）
    let arc_engine = Arc::new(engine);
    let slot = OCR_ENGINE.get_or_init(|| Mutex::new(None));
    let mut guard = slot
        .lock()
        .map_err(|e| format!("lock error: {}", e.to_string()))?;
    *guard = Some(arc_engine.clone());
    drop(guard);

    Ok(format!(
        "OCR engine configured with provider: {:?}, languages: {:?}, confidence_threshold: {}, timeout: {}s",
        ocr_provider_parsed, langs_parsed, confidence, timeout
    ))
}

/// OCR 识别函数，作为 Tauri Command 暴露给前端调用。
/// # Param
/// String - 图像文件路径
/// # Return
/// String - 识别到的文本内容。格式为json，包含两个字段：confidence（置信度）和text（识别文本）。
/// 若识别失败，返回错误信息。
#[tauri::command]
pub async fn ocr_image(file_path: String) -> Result<String, String> {
    // 先从全局取出 Arc<OcrEngine> 的克隆，避免持锁跨 await
    let maybe_engine = {
        let slot = OCR_ENGINE.get_or_init(|| Mutex::new(None));
        let guard = slot
            .lock()
            .map_err(|e| format!("lock error: {}", e.to_string()))?;
        guard.clone()
        // guard 在此处被 drop
    };

    let engine = maybe_engine.ok_or_else(|| "OCR engine is not configured.".to_string())?;

    // 使用克隆的 Arc 引擎调用异步识别
    let (_provider, text, _confidence) = engine
        .recognize_file(&file_path)
        .await
        .map_err(|e| format!("OCR recognition failed: {}", e))?;

    Ok(text)
}

/// 可选：提供一个重置函数（测试时方便清理）
pub fn reset_ocr_engine() -> Result<(), String> {
    let slot = OCR_ENGINE.get_or_init(|| Mutex::new(None));
    let mut guard = slot
        .lock()
        .map_err(|e| format!("lock error: {}", e.to_string()))?;
    *guard = None;
    Ok(())
}

#[cfg(test)]
#[path = "test_unit/test_ocr.rs"]
mod test_ocr;
