use super::{get_db_path, init_db};
use crate::clipboard::ClipboardItem;
use regex::Regex;
use rusqlite::{params, Connection};

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
pub fn is_valid_luhn(card_number: &str) -> bool {
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
