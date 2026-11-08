# AGENTS.md — 亦安快速启动（yian-launcher）项目规则

面向 AI 助手与协作者的开发约定。**优先级高于通用提示词**，与本文件冲突时以本文件为准。

## 项目定位与形态

- Windows 桌面轻量拖放启动器（Rust + Slint 1.10），**便携式**：`config.json`、图标缓存、捆绑 Everything 运行时全部位于 exe 同级目录。
- 路径基准一律用 `config::application_root_dir()`（exe 所在目录），不要引入 `%APPDATA%` 等用户目录。

## 构建与验证（必须遵守）

### Windows 原生构建

```sh
cargo build --release
```

- 依赖 Windows SDK `rc.exe`（MSVC）或 MinGW `windres`——`build.rs` 用 `winresource` 嵌入图标与 PerMonitorV2 manifest。
- `build.rs` 自动把 `everything/` 运行时复制到 `target/{profile}/` 与 exe 同目录。

### 交叉编译验证（容器内）

```sh
rustup target add x86_64-pc-windows-gnu
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
cargo check --target x86_64-pc-windows-gnu
```

### 验证纪律

- **改 Windows 专有代码必须跑交叉编译 `cargo check`**：`src/app_launcher.rs`（`win_shell`）、`src/win_ole_drop.rs`、`src/dnd.rs`、`src/win_extra.rs`、`src/hotkey_mouse_win.rs` 里的 `#[cfg(windows)]` 部分，以及 `src/main.rs` 中的 `#[cfg(windows)]` 函数。
- **不要用 Linux 原生 `cargo check` 当作验证通过**：它不编译 `#[cfg(windows)]` 代码，且会卡在 `gobject-sys`（缺 pkg-config/GTK）——那是环境问题不是代码问题。
- 依赖改动（`Cargo.toml`）后必须重新交叉编译验证。

## 模块架构速查

| 模块 | 职责 | 备注 |
|---|---|---|
| `main.rs` | 入口、全局状态 `AppState`、winit 事件、UI 回调、内存回收 | 3600+ 行，回调集中在 `setup_callbacks` |
| `models.rs` | `AppInfo` / `Category` / `UserConfig` / `SkinId` | 配置字段一律 `#[serde(default)]` 保证向后兼容 |
| `config.rs` | 便携路径、config.json 读写 | 已实现原子写入（tmp+rename）与 `.bak` 自动恢复 |
| `app_launcher.rs` | 启动 / 管理员 / ShellExecuteExW | 非 exe 统一走 Shell API，**禁止再用 `cmd /C start`** |
| `icon_loader.rs` | Win32 图标提取、LRU 缓存 | 缓存上限 64 项、统一 48px |
| `win_ole_drop.rs` / `dnd.rs` | OLE 拖入 / 拖出 | 手写 COM 虚表，仅 Windows |
| `everything_search.rs` | Everything IPC / es / GUI 三级搜索 | 含硬编码主密码 `ADMIN_MASTER_PASSWORD` |
| `edge_auto_hide.rs` | 贴边隐藏状态机 | 后台线程 16ms tick |
| `custom_hotkey.rs` | 键盘/鼠标全局快捷键 | 鼠标走 `hotkey_mouse_win.rs` 低级钩子 |
| `win_extra.rs` | DPI / DWM 圆角 / 透明度 / 工作集回收 | |
| `favicon.rs` | 网站图标抓取 | 多源回退（站点 → DuckDuckGo → Google） |
| `path_convert.rs` | 绝对/相对路径批量转换 | 同步处理 `path`/`icon_path`/`work_dir` |
| `single_instance.rs` | 单实例互斥 | |

## 红线（不要触碰或改动前必须理解）

1. **`vendor/winit/` 是本地补丁**：修复混合 DPI 多显示器下 `WM_DPICHANGED` 误判导致窗口尺寸膨胀（winit #4041 / #4600）。`Cargo.toml` 的 `[patch.crates-io]` 指向它，**不要移除补丁或回退到 crates.io 原版**。
2. **`everything/` 是捆绑运行时**：`Everything.exe` / `Everything64.dll` / `es.exe` 等由 `build.rs` 复制到 exe 旁，搜索功能依赖它们，不要删除。
3. **DPI 顺序**：任何 HWND（托盘/热键/主窗）创建前先 `win_extra::win::ensure_per_monitor_dpi_v2()`，否则光标与命中区错位。
4. **`post_wm_exit_sizemove` 必须 `SendMessageW` 同步**——源码注释已注明踩坑历史，**禁止改回 `PostMessageW`**（会导致无边框窗拖拽卡死）。
5. **内存回收纪律**：`K32EmptyWorkingSet` 受 30s 冷却与「每次隐藏仅一次」控制，不要随意放宽。
6. **跨线程传 `AppState`**：`Rc` 非 `Send`，跨线程必须用 `Rc::into_raw` + `Rc::from_raw`（项目既有模式），不要直接把 `Rc` 捕获进 `std::thread::spawn`。
7. **配置写入**：只通过 `ConfigManager::save_config_sync`（原子写入 + 备份），不要直接 `fs::write(config.json)`。

## 代码风格约定

- 注释用中文，`///` 文档注释说明意图，`//` 行内注释解释「为什么」（不解释做了什么）。
- Windows 专有能力提供 `#[cfg(windows)]` 与 `#[cfg(not(windows))]` 双份实现（非 Windows 版本返回错误或空操作）。
- UI 与逻辑分离：Slint 回调在 `main.rs`，业务逻辑在对应模块；UI 线程不阻塞（耗时操作放线程 + `slint::invoke_from_event_loop`）。
- 新增配置项：`models.rs` 的 `UserConfig` 加字段并带 `#[serde(default)]`，`create_default_config` / `Default` 同步补默认值。

## 提交规范

- 提交信息格式：`type: 中文摘要`，正文列关键改动点。
- 类型：`fix` / `feat` / `refactor` / `docs` / `chore`。
- 不提交 `target/`、`config.json*`、图标缓存等运行产物（已在 `.gitignore`）。
- 完成一个阶段任务后提醒用户 commit，不自行推送（除非用户明确要求）。
