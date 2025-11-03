#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{menu::{Menu, MenuItem}, tray::{TrayIconBuilder, TrayIconEvent}, Manager, PhysicalPosition};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

fn main() {
    // 使用共享状态来防止重复点击
    let last_click_time = Arc::new(Mutex::new(Instant::now()));
    
    let result = tauri::Builder::default()
        .setup(move |app| {
            let click_time_clone = Arc::clone(&last_click_time);
            
            // 创建托盘菜单
            let menu = Menu::new(app)?;
            
            // 创建菜单项
            let show_hide = MenuItem::with_id(app, "show_hide", "显示/隐藏", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            
            // 添加菜单项到菜单
            menu.append(&show_hide)?;
            menu.append(&quit)?;

            // 创建托盘图标
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("桌面宠物")
                .on_menu_event(move |app, event| {
                    println!("🖱️ 菜单项点击: {}", event.id().as_ref());
                    
                    if let Some(window) = app.get_webview_window("main") {
                        match event.id().as_ref() {
                            "show_hide" => {
                                toggle_window_visibility(&window);
                            }
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
                            
                            // 防抖处理：如果距离上次点击太近（小于200毫秒），忽略这次点击
                            if now.duration_since(*last_time) < Duration::from_millis(200) {
                                println!("⏰ 忽略重复点击");
                                return;
                            }
                            
                            *last_time = now;
                            
                            println!("🎯 托盘点击事件: {:?}", button);
                            
                            match button {
                                tauri::tray::MouseButton::Left => {
                                    // 左键点击：只切换显示/隐藏，不显示菜单
                                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                                        toggle_window_visibility(&window);
                                    }
                                    // 重要：左键点击后不显示菜单
                                }
                                tauri::tray::MouseButton::Right => {
                                    // 右键点击：显示菜单
                                    println!("📋 右键点击，显示菜单");
                                    // 右键菜单由系统自动处理
                                }
                                _ => {}
                            }
                        }
                        TrayIconEvent::DoubleClick { .. } => {
                            println!("🖱️ 托盘双击事件");
                            // 双击也可以用来切换显示/隐藏
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                toggle_window_visibility(&window);
                            }
                        }
                        _ => {
                            // 移除了其他事件的日志输出
                        }
                    }
                })
                .build(app)?;

            println!("✅ 托盘图标创建成功");
            /*
            // 设置窗口初始位置到右下角
            if let Some(window) = app.get_webview_window("main") {
                window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: 150, height: 150 }))?;
                
                if let Ok(monitor) = window.current_monitor() {
                    if let Some(monitor) = monitor {
                        let screen_size = monitor.size();
                        let x = screen_size.width as i32 - 150 - 20;
                        let y = screen_size.height as i32 - 150 - 20;
                        window.set_position(tauri::Position::Physical(PhysicalPosition { x, y }))?;
                        println!("📍 设置窗口位置: x={}, y={}", x, y);
                    }
                } else {
                    let x = 100;
                    let y = 100;
                    window.set_position(tauri::Position::Physical(PhysicalPosition { x, y }))?;
                    println!("⚠️ 使用默认窗口位置: x={}, y={}", x, y);
                }

                window.show()?;
                println!("🪟 窗口初始显示状态设置完成");
            }
            */
            // 全局快捷键设置
            let show_hide_shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyV);
            let shortcut_for_handler = show_hide_shortcut.clone();
            let handle = app.handle().clone();

            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, shortcut, event| {
                        if shortcut == &shortcut_for_handler {
                            if event.state() == ShortcutState::Pressed {
                                println!("⌨️ 全局快捷键被按下，执行窗口切换逻辑");
                                if let Some(window) = handle.get_webview_window("main") {
                                    toggle_window_visibility(&window);
                                }
                            }
                        }
                    })
                    .build()
            )?;

            app.global_shortcut().register(show_hide_shortcut)?;
            println!("✅ 已注册全局快捷键 Alt+Shift+V");

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!());

    if let Err(e) = result {
        eprintln!("❌ 启动 Tauri 应用失败: {:?}", e);
    }
}

// 切换窗口显示/隐藏的辅助函数
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
                }
                if let Err(e) = window.set_focus() {
                    eprintln!("⚠️ 设置窗口焦点失败: {:?}", e);
                } else {
                    println!("👀 显示桌宠窗口");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 获取窗口可见性失败: {:?}", e);
        }
    }
}