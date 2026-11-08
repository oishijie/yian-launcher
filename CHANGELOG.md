# 更新日志

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循 `Cargo.toml` 的 `version` 字段（`26.8.13` 为源码包基线版本）。

## [Unreleased]

### 修复

- **启动命令注入与 URL 截断**：非 `.exe` 文件与网页从 `cmd /C start` 改为 `ShellExecuteExW`（`open` 动词）。含 `&`、`|`、空格、引号的路径/URL 不再被 `cmd.exe` 解释截断；`.bat`/`.cmd` 现在可携带参数与工作目录启动；管理员运行与属性窗口统一走 Shell API 并关闭进程句柄。
- **Favicon 抓取阻塞 UI**：`add_dropped_http_url` / `add_dropped_path` 的图标下载移入后台线程异步回写，拖入网址不再卡界面（原最坏 8–16 秒阻塞）。
- **加密分组越权删除**：带锁分组删除前必须先输入正确密码确认，密码通过后经待办动作队列执行删除。
- **配置文件写入安全**：`save_config_sync` 改为原子写入（临时文件 + rename）并保留 `config.json.bak`；主配置损坏时自动从备份恢复，坏文件另存为 `config.json.corrupt`。
- **`.lnk` 拖入丢失信息**：拖入快捷方式时一并解析 `GetArguments` 与 `GetWorkingDirectory`，启动参数与工作目录不再丢失。

### 变更

- **Favicon 抓取增强**：增加连接/读取/总超时（3s/4s/5s）与 2MB 大小上限，携带浏览器 UA；新增 DuckDuckGo 图标源回退（站点 `favicon.ico` → DuckDuckGo → Google）。
- **参数解析**：编辑对话框与 `.lnk` 参数改用支持双引号包裹与 `\"`/`\\` 转义的命令行拆分，不再按空白粗暴切分。
- **便携路径转换**：绝对/相对路径批量转换现在同步处理 `icon_path` 与 `work_dir`，整目录迁移后图标与工作目录不失效。
- **网站图标缓存路径**：改为相对路径（`icons/<id>.png`）存储，配合便携迁移。
- **图标缓存清理**：删除分组/应用时同时清理自定义图标与网站图标缓存。

## [26.8.13] - 基线版本

随源码包提供（未发布到 GitHub 之前的版本快照，含全部既有功能：拖放添加、分组与密码锁、Everything 全盘搜索、托盘常驻、全局键鼠唤起、贴边隐藏、主题/背景/透明度等）。
