#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, PhysicalPosition,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

// ✅ 新增命令：动态设置窗口鼠标穿透
#[tauri::command]
fn set_mouse_passthrough(window: tauri::Window, passthrough: bool) {
    if let Err(e) = window.set_ignore_cursor_events(passthrough) {
        eprintln!("⚠️ 设置鼠标穿透失败: {:?}", e);
    } else {
        println!(
            "🎯 已设置窗口鼠标穿透状态为: {}",
            if passthrough { "开启" } else { "关闭" }
        );
    }
}

fn main() {
    // 防抖控制点击频率
    let last_click_time = Arc::new(Mutex::new(Instant::now()));

    let result = tauri::Builder::default()
        .setup(move |app| {
            let click_time_clone = Arc::clone(&last_click_time);

            // 创建托盘菜单
            let menu = Menu::new(app)?;
            let show_hide = MenuItem::with_id(app, "show_hide", "显示/隐藏", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            menu.append(&show_hide)?;
            menu.append(&quit)?;

            // 托盘图标
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("桌面宠物")
                .on_menu_event(move |app, event| {
                    println!("🖱️ 菜单项点击: {}", event.id().as_ref());
                    if let Some(window) = app.get_webview_window("main") {
                        match event.id().as_ref() {
                            "show_hide" => toggle_window_visibility(&window),
                            "quit" => {
                                println!("🚪 退出应用");
                                std::process::exit(0);
                            }
                            _ => {}
                        }
                    }
                })
                .on_tray_icon_event(move |tray, event| {
                    match event {
                        TrayIconEvent::Click { button, .. } => {
                            let now = Instant::now();
                            let mut last_time = click_time_clone.lock().unwrap();

                            // 防抖：200ms 内的重复点击忽略
                            if now.duration_since(*last_time) < Duration::from_millis(200) {
                                println!("⏰ 忽略重复点击");
                                return;
                            }
                            *last_time = now;

                            println!("🎯 托盘点击事件: {:?}", button);
                            match button {
                                tauri::tray::MouseButton::Left => {
                                    if let Some(window) = tray.app_handle().get_webview_window("main")
                                    {
                                        toggle_window_visibility(&window);
                                    }
                                }
                                tauri::tray::MouseButton::Right => {
                                    println!("📋 右键点击，显示菜单");
                                }
                                _ => {}
                            }
                        }
                        TrayIconEvent::DoubleClick { .. } => {
                            println!("🖱️ 托盘双击事件");
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                toggle_window_visibility(&window);
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            println!("✅ 托盘图标创建成功");

            // 设置主窗口为透明 + 穿透
            if let Some(window) = app.get_webview_window("main") {
                
                window.show()?;
            }

            // 全局快捷键 Alt+Shift+V 显示/隐藏窗口
            let show_hide_shortcut =
                Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyV);
            let shortcut_for_handler = show_hide_shortcut.clone();
            let handle = app.handle().clone();

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if shortcut == &shortcut_for_handler {
                            if event.state() == ShortcutState::Pressed {
                                println!("⌨️ Alt+Shift+V 被按下，切换窗口可见性");
                                if let Some(window) = handle.get_webview_window("main") {
                                    toggle_window_visibility(&window);
                                }
                            }
                        }
                    })
                    .build(),
            )?;

            app.global_shortcut().register(show_hide_shortcut)?;
            println!("✅ 已注册全局快捷键 Alt+Shift+V");

            Ok(())
        })
        // ✅ 注册前端命令
        .invoke_handler(tauri::generate_handler![set_mouse_passthrough])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("❌ 启动 Tauri 应用失败: {:?}", e);
    }
}

// 辅助函数：切换窗口显示/隐藏
fn toggle_window_visibility(window: &tauri::WebviewWindow) {
    match window.is_visible() {
        Ok(visible) => {
            if visible {
                if let Err(e) = window.hide() {
                    eprintln!("❌ 隐藏窗口失败: {:?}", e);
                } else {
                    println!("👻 隐藏桌宠窗口");
                }
            } else {
                if let Err(e) = window.show() {
                    eprintln!("❌ 显示窗口失败: {:?}", e);
                } else {
                    println!("👀 显示桌宠窗口");
                }
            }
        }
        Err(e) => eprintln!("❌ 获取窗口可见性失败: {:?}", e),
    }
}
