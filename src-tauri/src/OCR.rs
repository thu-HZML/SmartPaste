//! 对OCR相关功能的封装，实现图像文字识别。
//! 依赖 uniocr 库进行 OCR 处理。
// #[cfg(feature = "with_uniocr")]
use std::sync::OnceLock;
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
static OCR_ENGINE: OnceLock<OcrEngine> = OnceLock::new();

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

    OCR_ENGINE
        .set(engine)
        .map_err(|_| "OCR engine is already configured.".to_string())?;

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
    let engine = OCR_ENGINE
        .get()
        .ok_or_else(|| "OCR engine is not configured.".to_string())?;

    let (_provider, text, _confidence) = engine
        .recognize_file(&file_path)
        .await
        .map_err(|e| format!("OCR recognition failed: {}", e))?;

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    // 如果未配置则进行配置；已配置则直接返回 Ok
    fn ensure_configured() -> Result<(), String> {
        if OCR_ENGINE.get().is_none() {
            configure_ocr(
                Some("windows".to_string()),
                Some(vec!["eng", "chi", "jpn"]),
                Some(0.9),
                Some(60),
            )?;
        }
        Ok(())
    }

    #[tokio::test]
    /// 测试 OCR 配置函数：接受已配置或首次配置两种情况
    async fn test_configure_ocr() -> Result<(), String> {
        match configure_ocr(
            Some("windows".to_string()),
            Some(vec!["eng", "chi", "jpn"]),
            Some(0.9),
            Some(60),
        ) {
            Ok(s) => {
                println!("{}", s);
                Ok(())
            }
            Err(e) => {
                // 如果已经配置，返回的错误应包含提示，视为通过
                assert!(e.contains("already configured"));
                Ok(())
            }
        }
    }

    #[tokio::test]
    /// 测试 OCR 识别函数：在识别前确保已配置
    async fn test_ocr_image() -> Result<(), String> {
        ensure_configured()?;

        // 测试识别图像，断言返回非空纯文本
        let text_en_zh = ocr_image("./src/OCR_test_images/OCR_zh_en.png".to_string()).await?;
        println!("Recognized text: {}", text_en_zh);
        assert!(!text_en_zh.trim().is_empty(), "OCR 返回空文本");

        let text_jp = ocr_image("./src/OCR_test_images/OCR_ja.png".to_string()).await?;
        println!("Recognized text: {}", text_jp);
        assert!(!text_jp.trim().is_empty(), "OCR 返回空文本");

        Ok(())
    }
}
