use std::path::Path;

#[tokio::test]
async fn test_recognize_qrcode_file_not_found() {
    let res = super::recognize_qrcode("this_file_should_not_exist_12345.png".to_string()).await;
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(err.contains("File does not exist"));
}

#[tokio::test]
async fn test_recognize_qrcode_sample_if_exists() {
    // 可选的样例图片路径（工程内如有可放置到此处 src-tauri/src/test_unit/qrcode_sample.png）
    let sample_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src-tauri")
        .join("src")
        .join("test_unit")
        .join("qrcode_sample.png");
    if !sample_path.exists() {
        eprintln!(
            "qrcode sample not found at {:?}, skipping test",
            sample_path
        );
        return;
    }

    let res = super::recognize_qrcode(sample_path.to_string_lossy().into_owned()).await;
    assert!(res.is_ok());
    let text = res.unwrap();
    assert!(!text.trim().is_empty(), "decoded text should not be empty");
}
