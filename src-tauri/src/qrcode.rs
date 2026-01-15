//! 二维码识别模块
//! 依赖 rxing 库进行二维码解码。

use rxing;
use std::path::Path;

/// 二维码识别函数，作为 Tauri Command 暴露给前端调用。
/// # Param
/// file_path: String - 图像文件路径
/// # Return
/// String - 识别到的文本内容。
/// 若识别失败，返回错误信息。
#[tauri::command]
pub async fn recognize_qrcode(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(format!("File does not exist: {}", file_path));
    }

    // 使用 rxing 的 helper 函数直接读取文件并尝试解码
    // 这个函数会自动尝试多种解码器，包括 QR Code
    match rxing::helpers::detect_in_file(&file_path, None) {
        Ok(result) => Ok(result.getText().to_string()),
        Err(e) => Err(format!("Failed to decode QR code: {}", e)),
    }
}
