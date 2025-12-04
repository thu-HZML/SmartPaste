/// OCR 单元测试
/// 测试 OCR 引擎的配置和图像识别功能
/// 依赖于 src-tauri/src/ocr/mod.rs 中的 OCR 功能
/// 需确保测试环境中有适当的 OCR 测试图片
use super::*;
use std::path::Path;

// 重置并配置 OCR 引擎，确保每个测试有确定的初始状态
fn reset_and_configure() -> Result<(), String> {
    // 忽略 reset 的错误（如果尚未初始化也没关系）
    let _ = reset_ocr_engine();
    configure_ocr(
        Some("windows".to_string()),
        Some(vec!["eng", "chi", "jpn"]),
        Some(0.9),
        Some(60),
    )
    .map(|_| ())
}

#[tokio::test]
/// 测试 OCR 配置：在干净状态下能配置并允许重复配置（替换）
async fn test_configure_ocr() -> Result<(), String> {
    // 保证干净状态并第一次配置
    reset_and_configure()?;

    // 再次配置（应当成功，允许替换）
    let res = configure_ocr(
        Some("windows".to_string()),
        Some(vec!["eng", "chi", "jpn"]),
        Some(0.9),
        Some(60),
    );
    assert!(res.is_ok(), "reconfigure failed: {:?}", res.err());
    println!("{}", res.unwrap());

    Ok(())
}

#[tokio::test]
/// 测试 OCR 识别：在识别前确保已配置，若测试图片不存在则跳过具体断言
async fn test_ocr_image() -> Result<(), String> {
    reset_and_configure()?;

    let images = [
        "./src/OCR_test_images/OCR_zh_en.png",
        "./src/OCR_test_images/OCR_ja.png",
    ];

    for p in images {
        if !Path::new(p).exists() {
            println!("test image missing, skip: {}", p);
            continue;
        }
        let text = ocr_image(p.to_string()).await?;
        println!("Recognized text from {}: {}", p, text);
        assert!(!text.trim().is_empty(), "OCR returned empty text for {}", p);
    }

    Ok(())
}
