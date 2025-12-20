use super::{get_db_path, init_db};
use crate::clipboard::ClipboardItem;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonOsRng, SaltString},
    Argon2,
};
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use regex::Regex;
use rusqlite::{params, Connection};
use std::fs;

/// 利用备注内容匹配内容是否可能为密码，若匹配则标记或删除为隐私数据。作为 Tauri command 暴露给前端调用。
/// **匹配关键词**：
/// - "password"
/// - "密码"
/// - "pwd"
/// - "pass"
/// - "secret"
/// - "key"
/// - "token"
/// - "credential"
/// - "login"
/// - "auth"
/// - "authentication"
/// # Param
/// to_add: bool - 表示是否为增加隐私数据。若为true，则为添加隐私数据；若为false，则为删除隐私数据。
/// # Returns
/// Result<usize, String> - 受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn mark_passwords_as_private(to_add: bool) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let keywords = [
        "password",
        "密码",
        "pwd",
        "pass",
        "secret",
        "key",
        "token",
        "credential",
        "login",
        "auth",
        "authentication",
    ];

    let pattern = keywords
        .iter()
        .map(|kw| {
            if kw.chars().all(|c| c.is_ascii()) {
                format!(r"(?i)(?-u:\b){}(?-u:\b)", regex::escape(kw))
            } else {
                // 对于中文等非 ASCII 关键词，不使用 \b 边界，以便匹配 "登录密码" 等连词情况
                format!(r"(?i){}", regex::escape(kw))
            }
        })
        .collect::<Vec<String>>()
        .join("|");

    let regex = Regex::new(&pattern).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, notes FROM data WHERE item_type = 'text'")
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut count = 0;

    for item in clipboard_iter {
        let (id, notes) = item.map_err(|e| e.to_string())?;

        if regex.is_match(&notes) {
            if to_add {
                conn.execute(
                    "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                    params![id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                conn.execute("DELETE FROM private_data WHERE item_id = ?1", params![id])
                    .map_err(|e| e.to_string())?;
            }
            count += 1;
        }
    }

    Ok(count)
}

/// 辅助函数：实现银行卡号的 Luhn 算法校验
/// # Param
/// card_number: &str - 银行卡号字符串
/// # Returns
/// bool - 是否通过 Luhn 校验
fn is_valid_luhn(card_number: &str) -> bool {
    let card_number = card_number.replace(|c: char| c.is_whitespace() || c == '-', "");

    // Luhn 算法只适用于纯数字串
    if card_number.is_empty() || !card_number.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let sum = card_number
        .chars()
        .rev() // 从右向左遍历（从校验位开始）
        .enumerate()
        .map(|(i, c)| {
            let mut digit = c.to_digit(10).unwrap();

            // 偶数索引（从 0 开始，即第二位、第四位...）执行乘 2
            if i % 2 != 0 {
                digit *= 2;
                if digit > 9 {
                    digit -= 9; // 相当于相加
                }
            }
            digit
        })
        .sum::<u32>();

    sum % 10 == 0
}

/// 利用正则表达式匹配并使用 Luhn 算法校验内容是否可能为银行卡号 (PAN)，
/// 若匹配且校验通过，则标记为隐私数据。
/// # Param
/// to_add: bool - 表示是否为增加隐私数据。若为true，则为添加隐私数据；若为false，则为删除隐私数据。
/// # Returns
/// Result<usize, String> - 受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn mark_bank_cards_as_private(to_add: bool) -> Result<usize, String> {
    // 假设 db_path 和 conn 已经初始化并处理错误
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // 银行卡号的正则表达式 (包含 IIN/BIN 规则，允许空格或连字符作为分隔符)
    // PAN 长度通常在 13-19 位之间，且符合特定 IIN 范围。
    // 使用非捕获分组 `(?:...)` 和 `\b` 边界，并允许分隔符 `[\s-]?`
    let pan_regex = Regex::new(
        r"(?x)
        \b
        (?:
            # Visa (4xxxx): 13, 16, 19位
            4\d{3}[\s-]?\d{4}[\s-]?\d{4}(?:[\s-]?\d{4}(?:[\s-]?\d{3})?)? |
            
            # Mastercard (51-55 或 2221-2720): 16位
            (5[1-5]|222[1-9]|22[3-9]|2[3-6]|27[0-2])\d{2}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4} |
            
            # Amex (34, 37): 15位，分组通常是 4-6-5
            3[47]\d{2}[\s-]?\d{6}[\s-]?\d{5} |

            # Discover/Diners/JCB 等其他主要卡段 (14-19位)
            (3(?:0[0-5]|[689])|6(?:011|5\d{2}|4[4-9]\d{1}))\d{10,15}
        )
        \b
    ",
    )
    .map_err(|e| e.to_string())?;

    // 查询所有文本类型的数据
    let mut stmt = conn
        .prepare("SELECT id, content FROM data WHERE item_type = 'text'")
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut count = 0;

    for item in clipboard_iter {
        let (id, content) = item.map_err(|e| e.to_string())?;

        // 1. 正则初步筛选：查找所有潜在的卡号匹配项
        for capture in pan_regex.captures_iter(&content) {
            let potential_pan = &capture[0]; // 捕获整个匹配串（可能包含分隔符）

            // 2. 移除分隔符并执行 Luhn 校验
            if is_valid_luhn(potential_pan) {
                if to_add {
                    // 标记为隐私数据，即添加到private_data表中
                    conn.execute(
                        "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                        params![id],
                    )
                    .map_err(|e| e.to_string())?;
                } else {
                    // 取消标记为隐私数据
                    conn.execute("DELETE FROM private_data WHERE item_id = ?1", params![id])
                        .map_err(|e| e.to_string())?;
                }

                count += 1;
                // 一旦该记录中找到一个有效的卡号，就可以停止检查并进入下一条记录
                break;
            }
        }
    }

    Ok(count)
}

/// 利用正则表示匹配内容是否可能为身份证号，若匹配则标记为隐私数据。
/// # Param
/// to_add: bool - 表示是否为增加隐私数据。若为true，则为添加隐私数据；若为false，则为删除隐私数据。
/// # Returns
/// Result<usize, String> - 受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn mark_identity_numbers_as_private(to_add: bool) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // 身份证号的正则表达式（简单版本）
    let id_regex = Regex::new(r"\b\d{15}\b|\b\d{18}\b|\b\d{17}X\b").map_err(|e| e.to_string())?;

    // 查询所有文本类型的数据
    let mut stmt = conn
        .prepare("SELECT id, content FROM data WHERE item_type = 'text'")
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut count = 0;

    for item in clipboard_iter {
        let (id, content) = item.map_err(|e| e.to_string())?;
        if id_regex.is_match(&content) {
            if to_add {
                // 标记为隐私数据，也即添加到private_data表中
                conn.execute(
                    "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                    params![id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                // 取消标记为隐私数据
                conn.execute("DELETE FROM private_data WHERE item_id = ?1", params![id])
                    .map_err(|e| e.to_string())?;
            }
            count += 1;
        }
    }

    Ok(count)
}

/// 利用正则表达式匹配内容是否可能为手机号，若匹配则标记为隐私数据。
/// # Param
/// to_add: bool - 表示是否为增加隐私数据。若为true，则为添加隐私数据；若为false，则为删除隐私数据。
/// # Returns
/// Result<usize, String> - 受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn mark_phone_numbers_as_private(to_add: bool) -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // 手机号的正则表达式（简单版本，适用于中国手机号）
    let phone_regex = Regex::new(r"\b1[3-9]\d{9}\b").map_err(|e| e.to_string())?;

    // 查询所有文本类型的数据
    let mut stmt = conn
        .prepare("SELECT id, content FROM data WHERE item_type = 'text'")
        .map_err(|e| e.to_string())?;

    let clipboard_iter = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut count = 0;

    for item in clipboard_iter {
        let (id, content) = item.map_err(|e| e.to_string())?;
        if phone_regex.is_match(&content) {
            if to_add {
                // 标记为隐私数据，也即添加到private_data表中
                conn.execute(
                    "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                    params![id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                // 取消标记为隐私数据
                conn.execute("DELETE FROM private_data WHERE item_id = ?1", params![id])
                    .map_err(|e| e.to_string())?;
            }
            count += 1;
        }
    }

    Ok(count)
}

/// 清除所有隐私数据。作为 Tauri command 暴露给前端调用。
/// # Returns
/// Result<usize, String> - 受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn clear_all_private_data() -> Result<usize, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    let rows = conn
        .execute("DELETE FROM private_data", [])
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

/// 根据配置文件的选项，自动设置隐私数据标记。作为 Tauri command 暴露给前端调用。
/// # Param
/// password_flag: bool - 是否标记密码
/// bank_card_flag: bool - 是否标记银行卡号
/// id_number_flag: bool - 是否标记身份证号
/// phone_number_flag: bool - 是否标记手机号
/// # Returns
/// Result<usize, String> - 最终受影响的行数，若失败则返回错误信息
#[tauri::command]
pub fn auto_mark_private_data(
    password_flag: bool,
    bank_card_flag: bool,
    id_number_flag: bool,
    phone_number_flag: bool,
) -> Result<usize, String> {
    let mut total_count = 0;

    total_count += mark_passwords_as_private(password_flag)?;
    total_count += mark_bank_cards_as_private(bank_card_flag)?;
    total_count += mark_identity_numbers_as_private(id_number_flag)?;
    total_count += mark_phone_numbers_as_private(phone_number_flag)?;
    Ok(total_count)
}

/// 检查单个数据项是否应标记为隐私，并更新数据库。
/// 逻辑与 auto_mark_private_data 一致：
/// - 若匹配且对应 flag 为 true，则标记为隐私（插入 private_data）。
/// - 若匹配且对应 flag 为 false，则取消标记（从 private_data 删除）。
/// - 若不匹配，则不进行操作（保留原有状态）。
#[tauri::command]
pub fn check_and_mark_private_item(
    item: ClipboardItem,
    password_flag: bool,
    bank_card_flag: bool,
    id_number_flag: bool,
    phone_number_flag: bool,
) -> Result<bool, String> {
    let db_path = get_db_path();
    init_db(db_path.as_path()).map_err(|e| e.to_string())?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    // 1. Password Check
    let keywords = [
        "password",
        "密码",
        "pwd",
        "pass",
        "secret",
        "key",
        "token",
        "credential",
        "login",
        "auth",
        "authentication",
    ];
    let pattern = keywords
        .iter()
        .map(|kw| {
            // if kw.chars().all(|c| c.is_ascii()) {
            //     format!(r"(?i)(?-u:\b){}(?-u:\b)", regex::escape(kw))
            // } else {
            // 不使用单词边界，以支持中文关键词
            format!(r"(?i){}", regex::escape(kw))
            // }
        })
        .collect::<Vec<String>>()
        .join("|");
    let pwd_regex = Regex::new(&pattern).map_err(|e| e.to_string())?;

    if pwd_regex.is_match(&item.notes) {
        if password_flag {
            conn.execute(
                "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                params![item.id],
            )
            .map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "DELETE FROM private_data WHERE item_id = ?1",
                params![item.id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 2. Bank Card Check
    let pan_regex = Regex::new(
        r"(?x)
        \b
        (?:
            4\d{3}[\s-]?\d{4}[\s-]?\d{4}(?:[\s-]?\d{4}(?:[\s-]?\d{3})?)? |
            (5[1-5]|222[1-9]|22[3-9]|2[3-6]|27[0-2])\d{2}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4} |
            3[47]\d{2}[\s-]?\d{6}[\s-]?\d{5} |
            (3(?:0[0-5]|[689])|6(?:011|5\d{2}|4[4-9]\d{1}))\d{10,15}
        )
        \b
    ",
    )
    .map_err(|e| e.to_string())?;

    let mut is_bank_card = false;
    for capture in pan_regex.captures_iter(&item.content) {
        if is_valid_luhn(&capture[0]) {
            is_bank_card = true;
            break;
        }
    }

    if is_bank_card {
        if bank_card_flag {
            conn.execute(
                "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                params![item.id],
            )
            .map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "DELETE FROM private_data WHERE item_id = ?1",
                params![item.id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 3. ID Number Check
    let id_regex = Regex::new(r"\b\d{15}\b|\b\d{18}\b|\b\d{17}X\b").map_err(|e| e.to_string())?;
    if id_regex.is_match(&item.content) {
        if id_number_flag {
            conn.execute(
                "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                params![item.id],
            )
            .map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "DELETE FROM private_data WHERE item_id = ?1",
                params![item.id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 4. Phone Number Check
    let phone_regex = Regex::new(r"\b1[3-9]\d{9}\b").map_err(|e| e.to_string())?;
    if phone_regex.is_match(&item.content) {
        if phone_number_flag {
            conn.execute(
                "INSERT OR IGNORE INTO private_data (item_id) VALUES (?1)",
                params![item.id],
            )
            .map_err(|e| e.to_string())?;
        } else {
            conn.execute(
                "DELETE FROM private_data WHERE item_id = ?1",
                params![item.id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Check final status
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM private_data WHERE item_id = ?1",
            params![item.id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(count > 0)
}

/// 准备加密的数据库文件用于上传
/// # Param
/// dek_hex: String - 32字节的数据加密密钥 (Hex 编码)
/// # Returns
/// Result<String, String> - 加密后的数据库文件的 Base64 编码，若失败则返回错误信息
#[tauri::command]
pub fn prepare_encrypted_db_upload(dek_hex: String) -> Result<String, String> {
    // 1. 对 DEK 进行解码
    let key_bytes = hex::decode(&dek_hex).map_err(|e| format!("Invalid DEK hex: {}", e))?;
    if key_bytes.len() != 32 {
        return Err("DEK must be 32 bytes (64 hex chars)".to_string());
    }

    // 2. 将当前 DB 复制到临时文件
    let db_path = get_db_path();
    let temp_path = db_path.with_extension("enc.db");
    fs::copy(&db_path, &temp_path).map_err(|e| e.to_string())?;

    // 3. 打开临时 DB 并加密内容
    // 确保在此作用域内连接有效
    {
        let mut conn = Connection::open(&temp_path).map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // 4. 读取所有数据并加密
        // 使用单独的块来读取数据，确保 stmt 在块结束时被销毁，释放对 tx 的借用
        let all_rows = {
            let mut stmt = tx
                .prepare("SELECT id, content, notes FROM data")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                })
                .map_err(|e| e.to_string())?;

            rows.collect::<Result<Vec<(String, String, Option<String>)>, _>>()
                .map_err(|e| e.to_string())?
        };

        // 使用单独的块来更新数据，确保 update_stmt 在块结束时被销毁
        {
            let mut update_stmt = tx
                .prepare("UPDATE data SET content = ?1, notes = ?2 WHERE id = ?3")
                .map_err(|e| e.to_string())?;

            for (id, content, notes) in all_rows {
                let enc_content = encrypt_string(&key_bytes, &content)?;
                let enc_notes = if let Some(n) = notes {
                    Some(encrypt_string(&key_bytes, &n)?)
                } else {
                    None
                };

                update_stmt
                    .execute(params![enc_content, enc_notes, id])
                    .map_err(|e| e.to_string())?;
            }
        }

        tx.commit().map_err(|e| e.to_string())?;
    }

    // 5. 读取加密后的文件内容并进行 Base64 编码
    let file_content = fs::read(&temp_path).map_err(|e| e.to_string())?;
    let base64_str = general_purpose::STANDARD.encode(file_content);

    // 6. 删除临时文件
    let _ = fs::remove_file(temp_path);

    Ok(base64_str)
}

fn encrypt_string(key: &[u8], plaintext: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut rng = rand::rng();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    let nonce_b64 = general_purpose::STANDARD.encode(nonce_bytes);
    let cipher_b64 = general_purpose::STANDARD.encode(ciphertext);

    Ok(format!("{}:{}", nonce_b64, cipher_b64))
}

/// 从加密的数据库文件恢复
/// # Param
/// dek_hex: String - 32字节的数据加密密钥 (Hex 编码)
/// encrypted_db_base64: String - 加密后的数据库文件的 Base64 编码
/// # Returns
/// Result<(), String> - 成功返回 Ok(())
#[tauri::command]
pub fn restore_from_encrypted_db(
    dek_hex: String,
    encrypted_db_base64: String,
) -> Result<(), String> {
    // 1. 对 DEK 进行解码
    let key_bytes = hex::decode(&dek_hex).map_err(|e| format!("Invalid DEK hex: {}", e))?;
    if key_bytes.len() != 32 {
        return Err("DEK must be 32 bytes (64 hex chars)".to_string());
    }

    // 2. 对加密的 DB 进行 Base64 解码
    let db_bytes = general_purpose::STANDARD
        .decode(encrypted_db_base64)
        .map_err(|e| e.to_string())?;

    // 3. 写入临时文件
    let db_path = get_db_path();
    let temp_path = db_path.with_extension("dec.db");
    fs::write(&temp_path, db_bytes).map_err(|e| e.to_string())?;

    // 4. 打开临时 DB 并解密内容
    {
        let mut conn = Connection::open(&temp_path).map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // 读取所有数据
        let all_rows = {
            let mut stmt = tx
                .prepare("SELECT id, content, notes FROM data")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                })
                .map_err(|e| e.to_string())?;

            rows.collect::<Result<Vec<(String, String, Option<String>)>, _>>()
                .map_err(|e| e.to_string())?
        };

        // 更新解密后的数据
        {
            let mut update_stmt = tx
                .prepare("UPDATE data SET content = ?1, notes = ?2 WHERE id = ?3")
                .map_err(|e| e.to_string())?;

            for (id, content, notes) in all_rows {
                // Try to decrypt. If fail (e.g. not encrypted or bad key), maybe keep as is or error?
                // For now, assume all are encrypted.
                let dec_content = decrypt_string(&key_bytes, &content).unwrap_or(content.clone());
                let dec_notes = if let Some(n) = notes {
                    Some(decrypt_string(&key_bytes, &n).unwrap_or(n))
                } else {
                    None
                };

                update_stmt
                    .execute(params![dec_content, dec_notes, id])
                    .map_err(|e| e.to_string())?;
            }
        }

        tx.commit().map_err(|e| e.to_string())?;
    }

    // 5. 替代原数据库文件
    // 先备份旧文件
    let backup_path = db_path.with_extension("bak");
    let _ = fs::copy(&db_path, &backup_path);

    // 备份后替换，若失败则保留旧文件
    match fs::rename(&temp_path, &db_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            // If rename fails (e.g. cross-device link or locked), try copy and delete
            fs::copy(&temp_path, &db_path)
                .map_err(|e2| format!("Rename failed: {}, Copy failed: {}", e, e2))?;
            let _ = fs::remove_file(temp_path);
            Ok(())
        }
    }
}

/// 辅助函数：解密字符串
/// # Param
/// key: &[u8] - 32字节的密钥
/// ciphertext_combined: &str - 包含 nonce 和密文的字符串，格式为 "nonce:ciphertext"
/// # Returns
/// Result<String, String> - 解密后的明文字符串，若失败则返回错误信息
fn decrypt_string(key: &[u8], ciphertext_combined: &str) -> Result<String, String> {
    let parts: Vec<&str> = ciphertext_combined.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid ciphertext format".to_string());
    }

    let nonce_bytes = general_purpose::STANDARD
        .decode(parts[0])
        .map_err(|e| e.to_string())?;
    let cipher_bytes = general_purpose::STANDARD
        .decode(parts[1])
        .map_err(|e| e.to_string())?;

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, cipher_bytes.as_ref())
        .map_err(|e| e.to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

/// 加密单个文件
/// # Param
/// input_path: String - 源文件路径
/// output_path: String - 输出加密文件的路径
/// dek_hex: String - 32字节的数据加密密钥 (Hex 编码)
/// # Returns
/// Result<(), String> - 成功返回 Ok(())
#[tauri::command]
pub fn encrypt_file(
    input_path: String,
    output_path: String,
    dek_hex: String,
) -> Result<(), String> {
    // 1. Decode DEK
    let key_bytes = hex::decode(&dek_hex).map_err(|e| format!("Invalid DEK hex: {}", e))?;
    if key_bytes.len() != 32 {
        return Err("DEK must be 32 bytes (64 hex chars)".to_string());
    }

    // 2. Read file content
    let plaintext =
        fs::read(&input_path).map_err(|e| format!("Failed to read input file: {}", e))?;

    // 3. Encrypt
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
    let mut rng = rand::rng();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // 4. Write to output file (Format: [Nonce 12 bytes][Ciphertext ...])
    let mut output_file = fs::File::create(&output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    use std::io::Write;
    output_file
        .write_all(&nonce_bytes)
        .map_err(|e| e.to_string())?;
    output_file
        .write_all(&ciphertext)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 解密单个文件
/// # Param
/// input_path: String - 加密文件路径
/// output_path: String - 输出解密文件的路径
/// dek_hex: String - 32字节的数据加密密钥 (Hex 编码)
/// # Returns
/// Result<(), String> - 成功返回 Ok(())
#[tauri::command]
pub fn decrypt_file(
    input_path: String,
    output_path: String,
    dek_hex: String,
) -> Result<(), String> {
    // 1. Decode DEK
    let key_bytes = hex::decode(&dek_hex).map_err(|e| format!("Invalid DEK hex: {}", e))?;
    if key_bytes.len() != 32 {
        return Err("DEK must be 32 bytes (64 hex chars)".to_string());
    }

    // 2. Read encrypted file
    let file_bytes =
        fs::read(&input_path).map_err(|e| format!("Failed to read encrypted file: {}", e))?;

    if file_bytes.len() < 12 {
        return Err("File too short to be a valid encrypted file".to_string());
    }

    // 3. Extract Nonce and Ciphertext
    let (nonce_bytes, ciphertext_bytes) = file_bytes.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // 4. Decrypt
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
    let plaintext = cipher
        .decrypt(nonce, ciphertext_bytes)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    // 5. Write to output file
    fs::write(&output_path, plaintext)
        .map_err(|e| format!("Failed to write output file: {}", e))?;

    Ok(())
}

// --- 密钥管理辅助函数 (Key Management) ---

/// 生成随机 Salt (Hex 编码)，作为 Tauri command 暴露给前端调用。
/// # Returns
/// String - Hex 编码的 Salt
#[tauri::command]
pub fn generate_salt() -> String {
    let salt = SaltString::generate(&mut ArgonOsRng);
    hex::encode(salt.as_str().as_bytes())
}

/// 生成随机 DEK (Hex 编码)，作为 Tauri command 暴露给前端调用。
/// # Returns
/// String - Hex 编码的 DEK (32 bytes)
#[tauri::command]
pub fn generate_dek() -> String {
    let mut key = [0u8; 32];
    rand::rng().fill(&mut key);
    hex::encode(key)
}

/// 使用 Argon2id 从密码和 Salt 派生主密钥 (MK)，作为 Tauri command 暴露给前端调用。
/// # Param
/// password: &str - 用户密码
/// salt_hex: &str - Hex 编码的 Salt
/// # Returns
/// Result<String, String> - Hex 编码的 MK (32 bytes)
#[tauri::command]
pub fn derive_mk(password: &str, salt_hex: &str) -> Result<String, String> {
    // 1. Decode salt hex to bytes, then to string (Argon2 expects string salt format usually,
    // but here we treat the input salt_hex as the raw salt bytes or the salt string itself?
    // To be compatible with standard Argon2 usage, let's assume salt_hex decodes to the raw salt bytes.
    // However, the `argon2` crate's `SaltString` has specific format requirements.
    // For simplicity and robustness, we will use the decoded bytes as the salt directly if possible,
    // or if we generated it using SaltString, we should pass it as string.

    // Let's assume salt_hex is the hex representation of the raw salt bytes.
    let salt_bytes = hex::decode(salt_hex).map_err(|e| format!("Invalid salt hex: {}", e))?;

    // Argon2 configuration
    let argon2 = Argon2::default();

    // We need a buffer to store the output key
    let mut output_key = [0u8; 32]; // 256-bit key

    argon2
        .hash_password_into(password.as_bytes(), &salt_bytes, &mut output_key)
        .map_err(|e| format!("Argon2 derivation failed: {}", e))?;

    Ok(hex::encode(output_key))
}

/// 使用 MK 加密 DEK (Key Wrapping)，作为 Tauri command 暴露给前端调用。
/// # Param
/// dek_hex: &str - 明文 DEK (Hex)
/// mk_hex: &str - 主密钥 MK (Hex)
/// # Returns
/// Result<String, String> - 加密后的 DEK (Hex)
#[tauri::command]
pub fn wrap_dek(dek_hex: &str, mk_hex: &str) -> Result<String, String> {
    let dek_bytes = hex::decode(dek_hex).map_err(|e| format!("Invalid DEK hex: {}", e))?;
    let mk_bytes = hex::decode(mk_hex).map_err(|e| format!("Invalid MK hex: {}", e))?;

    if mk_bytes.len() != 32 {
        return Err("MK must be 32 bytes".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(&mk_bytes).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, dek_bytes.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Format: [Nonce 12][Ciphertext]
    let mut result = Vec::new();
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(hex::encode(result))
}

/// 使用 MK 解密 DEK (Key Unwrapping)，作为 Tauri command 暴露给前端调用。
/// # Param
/// encrypted_dek_hex: &str - 加密后的 DEK (Hex)
/// mk_hex: &str - 主密钥 MK (Hex)
/// # Returns
/// Result<String, String> - 明文 DEK (Hex)
#[tauri::command]
pub fn unwrap_dek(encrypted_dek_hex: &str, mk_hex: &str) -> Result<String, String> {
    let encrypted_bytes =
        hex::decode(encrypted_dek_hex).map_err(|e| format!("Invalid encrypted DEK hex: {}", e))?;
    let mk_bytes = hex::decode(mk_hex).map_err(|e| format!("Invalid MK hex: {}", e))?;

    if mk_bytes.len() != 32 {
        return Err("MK must be 32 bytes".to_string());
    }

    if encrypted_bytes.len() < 12 {
        return Err("Encrypted data too short".to_string());
    }

    let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&mk_bytes).map_err(|e| e.to_string())?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed (Wrong password?): {}", e))?;

    Ok(hex::encode(plaintext))
}
