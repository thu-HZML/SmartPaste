```bash
src/
├── main.rs           # 程序主入口，负责构建和运行Tauri应用
├── lib.rs            # 库文件，用于其他构建目标
├── db.rs             # 数据库模块，现在从 clipboard.rs 导入数据结构
├── test_db.rs        # 数据库模块的单元测试
├── clipboard.rs      # 定义核心数据结构 ClipboardItem
└── app_setup.rs      # 包含所有应用设置逻辑 (托盘、快捷键、监控)
```

