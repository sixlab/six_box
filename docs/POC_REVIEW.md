# PoC 与正式设计核对

核对日期：2026-11-10。来源：`/Users/panic/codespace/project_idea/toolbox-architecture-poc`。

`PRD.md`、`ARCHITECTURE.md` 从该目录的 `docs/` 原样归档，分别为《SixBox PRD V1.0》和《SixBox 技术设计 V1.0》。PRD 的“需求基线草案”是原文状态；本次用户明确指定两份文档为开发基准。本文单独记录实现差异，不把 PoC 行为改写进产品要求。

| 项目 | PoC 实际行为 | 正式设计 / M1 处理 |
|---|---|---|
| 首屏 | Node HTTP 服务启动后请求 Rust 创建主窗口 | 本地打包 React 首屏，独立于 Node；必须重做 |
| 管理服务故障 | stdout 结束后 Rust 退出 | 宿主保留，有限退避重启、状态提示、手动重试 |
| Node 检测 | 直接 spawn，错误向 setup 传播 | 检测固定包内路径、版本、平台、架构、执行超时；失败进入降级界面 |
| 数据目录 | 默认应用旁边 runtime-data | Rust 统一系统数据、缓存、日志目录；Node 仅接收初始化路径 |
| 单实例 | 未实现 | 在运行时启动前注册单实例，第二次启动聚焦已有主窗口 |
| stdio | 逐行 JSON，部分写入错误被忽略 | 复用传输方式，重新定义版本、ID、请求/响应/错误/事件；限长、超时、断连、错误可见 |
| 退出 | 有等待及强杀，但 Worker 停止回调按同步处理 | M1 只管理服务；Shutdown 确认、限时退出和强杀回收，父进程断连退出 |
| 插件 / SDK | 类型耦合 NetworkConfig；测试管理员接口和业务输入日志 | 不迁入；M2 重新定义实际插件需要的 SDK，无 PoC 兼容层 |
| 卸载 / 更新 | 卸载删除配置；更新先卸载 | 与正式设计默认保留数据冲突，M2/M6 按正式设计实现 |
| 窗口 / 托盘 | API 已验证，托盘点击未验收 | 复用 show/focus/hide 思路；M1 只有主窗口与最小打开/退出托盘，不实现 M3 完整菜单 |
| Node 分发 | 官方 24.22.2，externalBin，许可证、归档和文件哈希 | 复用固定版本与验证做法；不复制全部构建系统，不维护 Electron |
| 最低系统 | 声明 13.5，只实测 26.6.2 | 延续构建下限 13.5，不宣称已在 13.5 验收 |

结论：正式分层、TypeScript、Rust↔Node stdio、未来插件 UI 回环 HTTP 均与验证结果一致。差异主要是正式设计明确要求补齐 PoC 未实现的可靠性，以及产品行为有意替换原型行为；没有需要改动总体架构的阻断项。

M1 不启动 HTTP 服务、不加载插件、不创建 SDK 空包和 UI 组件空包。只有 `desktop`、`core`、`plugin-protocol` 三个实际使用的包。协议包当前仅定义宿主与管理服务契约，不视为完整插件协议。

参考实现：`hosts/tauri/src/main.rs`、`hosts/electron/main.ts`、`packages/runtime/{tauri-service,supervisor,server,manager}.ts`、`scripts/prepare-tauri.mjs`。参考报告：`docs/poc-report.md`，原有 18/18 结果仅属于 PoC，不能作为 SixBox 测试结果。

官方实现参考：[Tauri 单实例](https://v2.tauri.app/zh-cn/plugin/single-instance/)、[Tauri Sidecar](https://v2.tauri.app/develop/sidecar/)。
