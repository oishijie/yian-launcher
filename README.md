# 亦安快速启动（yian-launcher）

一款面向 Windows 的轻量级桌面快速启动器：集中管理常用程序、文件夹、文档与网页链接，支持拖放添加、分组管理、全盘搜索、托盘常驻与全局键鼠唤起。

- 技术栈：Rust + [Slint](https://slint.dev) 1.10（winit 后端 / OpenGL + 软件渲染）
- 平台：Windows 7+（x64，静态链接 MSVC 运行库，免装 VC++ Redistributable）
- 形态：便携式——`config.json`、缓存与捆绑的 Everything 运行时均在 exe 同级目录，整目录拷贝即迁移

## 功能特性

- **拖放添加**：把 `.exe` / `.lnk` / `.url` / 文件夹 / 文档 / 图片直接拖入窗口即添加；拖入文本中的 `http(s)` 链接自动识别为网页并抓取站点图标
- **项目启动**：左键单击启动；右键菜单支持管理员运行、打开所在位置、查看属性、编辑信息、删除
- **分组管理**：单击切换、双击重命名、拖拽排序、右键增删；支持**分组密码锁**；一键启动分组全部项目
- **搜索**：顶部关键字实时筛选（名称/路径/关键词/网址，结果高亮）；切到「计算机」经 Everything IPC 全盘秒搜
- **拖出生成快捷方式**：把图标拖到桌面/资源管理器即可导出 `.lnk`
- **托盘与唤起**：关闭窗口缩到托盘；可设键盘组合键或鼠标中键/侧键全局唤起
- **窗口行为**：置顶、开机自启、启动最小化、贴边自动隐藏（QQ 式）、单击运行后隐藏、50–100% 透明度
- **外观**：深色/浅色/暖色/柔和四套皮肤、自定义背景图、图标尺寸滑块

## 构建

### Windows 原生

```sh
cargo build --release
```

要求：

- Rust stable（`Cargo.toml` 声明 `rust-version = "1.70"`）
- Windows SDK 的 `rc.exe`（MSVC）或 MinGW `windres`——`build.rs` 用 `winresource` 嵌入 exe 图标与 PerMonitorV2 清单
- `build.rs` 会把 `everything/` 目录的运行时复制到 `target/{profile}/`，与 exe 同目录

### Linux 交叉编译（仅验证，不产出运行版本）

```sh
rustup target add x86_64-pc-windows-gnu
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
cargo check --target x86_64-pc-windows-gnu
```

## 使用说明

详细功能与常见问题见 [USAGE.txt](USAGE.txt)。

## 目录结构

```
src/
  main.rs                # 入口、全局状态、winit 事件、UI 回调、内存回收
  models.rs              # AppInfo / Category / UserConfig 数据结构
  config.rs              # 便携路径、config.json 读写（原子写入 + 备份恢复）
  app_launcher.rs        # 启动 / 管理员运行 / ShellExecuteExW / 属性
  icon_loader.rs         # Win32 图标提取、LRU 缓存、字母降级
  win_ole_drop.rs        # OLE IDropTarget：文件/文本/HTML 链接拖入
  dnd.rs                 # OLE IDropSource：拖出生成 .lnk
  everything_search.rs   # Everything IPC / es.exe / GUI 三级搜索
  edge_auto_hide.rs      # 贴边自动隐藏状态机
  custom_hotkey.rs       # 键盘/鼠标全局快捷键
  hotkey_mouse_win.rs    # WH_MOUSE_LL 鼠标钩子
  win_extra.rs           # DPI / DWM 圆角 / 透明度 / 工作集回收
  favicon.rs             # 网站图标抓取（多源回退）
  path_convert.rs        # 绝对/相对路径批量转换
  single_instance.rs     # 单实例互斥
ui/
  app.slint              # 主窗口 UI（Slint）
  settings_widgets.slint # 设置弹窗 UI
everything/              # 捆绑的 Everything 便携运行时（构建时复制到 exe 旁）
vendor/winit/            # 本地补丁：修复多屏 DPI 下 WM_DPICHANGED 窗口膨胀（winit #4041/#4600）
```

## 许可

MIT License，见 [LICENSE](LICENSE)。作者：QQ202173661
