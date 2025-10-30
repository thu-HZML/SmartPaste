use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod db;

fn main() {
    let result = tauri::Builder::default()
        // 注册 Tauri commands - 数据库接口
        .invoke_handler(tauri::generate_handler![db::insert_received_data])
        .invoke_handler(tauri::generate_handler![db::get_data_by_id])
        .invoke_handler(tauri::generate_handler![db::delete_data])
        .invoke_handler(tauri::generate_handler![db::delete_data_by_id])
        .invoke_handler(tauri::generate_handler![db::favorite_data_by_id])
        .invoke_handler(tauri::generate_handler![db::search_text_content])
        .setup(|app| {
            let show_hide_shortcut =
                Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyV);
            let shortcut_for_handler = show_hide_shortcut.clone();
            let handle = app.handle().clone();

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if shortcut == &shortcut_for_handler {
                            if event.state() == ShortcutState::Pressed {
                                println!("✅ 按键被按下，执行窗口切换逻辑");
                                let window = handle.get_webview_window("main").unwrap();

                                // 切换窗口状态的逻辑保持不变
                                if let Ok(minimized) = window.is_minimized() {
                                    if minimized {
                                        window.unminimize().unwrap();
                                        window.set_focus().unwrap();
                                        return;
                                    }
                                }

                                if let Ok(visible) = window.is_visible() {
                                    if visible {
                                        window.hide().unwrap();
                                    } else {
                                        window.show().unwrap();
                                        window.set_focus().unwrap();
                                    }
                                }
                            } else {
                                // println!("⭕️ 按键被释放，不执行任何操作");
                            }
                        }
                    })
                    .build(),
            )?;

            app.global_shortcut().register(show_hide_shortcut)?;
            println!("✅ 已注册全局快捷键 Alt+Shift+V");

            Ok(())
        })
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("❌ 启动 Tauri 应用失败: {:?}", e);
    }
}
