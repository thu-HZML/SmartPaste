use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

fn main() {
    let result = tauri::Builder::default()
        .setup(|app| {
            let show_hide_shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyV);
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
                                // 可以选择在这里打印释放事件，用于调试
                                // println!("⭕️ 按键被释放，不执行任何操作");
                            }
                        }
                    })
                    .build()
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