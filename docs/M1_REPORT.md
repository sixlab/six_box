# SixBox M1 验收报告

日期：2026-11-10。范围：M1 工程与运行时基础。**M1 已完成；没有开始 M2。**

## 实际交付

| M1 要求 | 对应实现 |
|---|---|
| 正式 Monorepo | pnpm Workspace；desktop、core、plugin-protocol 三个实际使用的包 |
| Tauri 2 + React + TypeScript | 本地打包首屏，不依赖 Node HTTP；严格类型检查 |
| 应用身份 | SixBox / cn.sixlab.box / SixLab；本地 origin 指向 sixlab/six_box |
| 数据、缓存、日志目录 | Rust 解析系统目录；分离程序/用户数据；日志轮转 |
| Rust Runtime Manager | 固定内置来源、执行检测、状态、Ready 握手、健康检查、有限恢复和手动重试 |
| 内置 Node | 官方 24.22.2 arm64；归档和二进制 SHA-256、动态库、许可证检查 |
| Node 缺失降级 | 基础首屏、错误说明、存储诊断、重试和退出仍可用 |
| Rust / TS 通信 | v1 逐行 JSON；唯一请求 ID、错误码、事件、超时、64 KiB 限长 |
| 单实例与故障处理 | 文件锁 + 有界聚焦通知；崩溃/挂起恢复；正常及原生退出清理 |
| 脚本与测试 | dev、build、check、verify:bundle；锁文件；macOS arm64 CI 配置 |

原始 PRD 和技术设计已归档，`cmp` 确认与 PoC 中的两份源文件完全相同。差异记录见 [POC_REVIEW.md](POC_REVIEW.md)。正式代码没有迁入 Electron、业务插件、PoC 管理控制接口或业务输入采集。

## 环境与结果

实测环境：Apple Silicon arm64、macOS 26.6.2（25G83）、Rust 1.87.0、开发 Node 24.14.1、pnpm 12.15.0。产品内置 Node 24.22.2。Tauri Rust 2.11.6、JS API 2.11.0、CLI 2.12.4。

| 命令 / 验证 | 结果 |
|---|---|
| `pnpm typecheck` | 通过 |
| `pnpm test:ts` | 6/6 通过 |
| `pnpm test:rust` | 16/16 通过，包含真实 Node 子进程 |
| `pnpm check` | 全部通过，clippy 无警告、rustfmt 通过 |
| `pnpm build` | macOS arm64 release `.app` 构建及 ad-hoc 签名成功 |
| `pnpm verify:bundle` | 9/9 通过 |
| `pnpm dev` | esbuild 服务返回页面，Tauri debug 启动并进入 Ready；随后正常退出 |
| 原生 UI 操作 | 正常首屏、诊断展开、Node 缺失首屏、重试、退出、窗口隐藏与恢复通过 |
| 挂起服务后原生 Cmd+Q | 日志记录 native-exit → force-stop → stopped，服务 PID 已回收 |

构建产物：`.cache/target/aarch64-apple-darwin/release/bundle/macos/SixBox.app`，约 123.84 MiB（解包应用逻辑大小，非压缩包体积）。

M1 开发结束时的自动验收记录：`artifacts/verify-1794314591178/results.json`。检查、构建和开发启动日志分别在 `.cache/check.log`、`.cache/release-build.log`、`.cache/dev-smoke.log`。这些本地证据和运行时二进制不提交 Git。

### 9 项正式包自动验收

1. 应用标识和完整 ad-hoc 签名验证。
2. PATH 无用户 Node，设置 Node 注入变量后，仍使用包内 Node 正常 Ready。
3. 重复启动退出第二实例，已有服务 PID 不变。
4. 管理服务 SIGKILL 后恢复，宿主保持运行。
5. 管理服务 SIGSTOP 后通过健康检查恢复，旧进程被回收。
6. 服务挂起时连续 SIGTERM 退出请求，宿主及服务均停止。
7. 同时启动 6 份正式可执行文件，仅一个宿主与管理服务运行。
8. 复制正式包并删除 Node 后，宿主存活且报告 NODE_MISSING；真实界面另行检查通过。
9. 宿主 SIGKILL 后，正常响应中的服务在 stdin EOF 后退出；重新启动能取得文件锁。

### 本轮发现并修复

- Tauri JS API 范围版本升级到 2.12，触发 CLI 与 Rust 2.11 的版本检查：固定 API 为 2.11.0。
- pnpm 12 使用 `allowBuilds`：显式允许 esbuild 构建脚本。
- 官方单实例插件 macOS 实现存在 notify→异步 bind 竞态：改为操作系统文件锁，通知仅用于聚焦；并发单元测试和 6 进程正式包测试通过。
- 重复退出可绕过异步清理：分离请求与清理完成状态，增加回归测试。
- macOS 原生 Quit 可能直接触发最终 Exit：加入最终退出清理兜底，并使并发 shutdown 调用等待同一次清理；真实 Cmd+Q 验证通过。
- macOS socket 通知写入后立即关闭会使接收端设置失败：加入固定字节确认与超时，保证接收完成；没有加入通用远程控制能力。

执行过一次独立只读审查。两项 Important 已修复并通过回归；日志期限这一 Minor 采用明确文档行为处理：只承诺启动时清理过期文件，以及持续运行时的大小上限。

## 未解决问题与验证边界

- 未做 Developer ID 签名、公证、DMG、联网下载分发或自动更新；当前产物用于本机开发和验收。
- 构建下限为 macOS 13.5，实测只有 26.6.2；不承诺其他系统版本、Intel、Windows 或 Linux 已验证。
- 没有数日持续运行、睡眠唤醒、系统关机、磁盘写满的完整验收。普通目录失败、日志上限和基本进程故障已有测试。
- 托盘菜单存在，但本轮未单独做菜单栏图标点击验收；完整托盘菜单属于 M3。
- 同时强杀宿主并挂起服务时，服务无法处理 EOF；不承诺任意后代进程回收。M1 尚无插件 Worker。
- 日志 7 天清理在启动时执行，持续运行期间按大小轮转；不提供逐条记录的严格时间清理。
- CI 配置已加入，尚未推送到 GitHub 执行远端流水线。
- M1 使用临时几何图标和基础界面，不代表最终品牌视觉。

## M2 建议（未实施）

1. 固定最小 manifest Schema 与通用 TypeScript SDK，从实际 JSON 工具和网络监控需求反推必要接口。
2. 实现可信本地插件安装、发现、启停与卸载；程序目录和用户数据分离，卸载默认保留数据。
3. 建立独立 Worker 生命周期，包含异步停止、心跳、有限恢复；Worker 一律使用当前包内 Node。
4. 引入插件 HTTP UI/API 时实现临时会话认证、来源检查和插件范围限制；不得直接暴露 M1 宿主管理命令。
5. 保留 M1 的无 Node 首屏、单实例、服务恢复和退出回归；M2 不抢做 M3 页面、M4 快捷面板或完整市场。

M1 开发验收结束时，代码保留在 `feat/m1-foundation` 分支，尚未提交。随后按维护者要求补齐中英文 README、忽略规则和 AGPL-3.0-only 许可文件，纳入项目首次提交；提交记录以 Git 历史为准。未推送、创建 PR 或发布。

## 首次提交前复核（2026-11-10）

- `pnpm check`：类型检查、6 项 TypeScript 测试、16 项 Rust 测试、clippy 和 rustfmt 全部通过。最初受限沙箱拒绝 Unix socket 监听，改在正常本机权限下完成整套验证，未改动业务代码。
- `pnpm build`：macOS arm64 正式包构建及 ad-hoc 签名通过，新增随包 `LICENSE`、`COPYRIGHT`、`THIRD_PARTY_NOTICES.md` 与仓库文件逐字节一致；Node.js 原始许可文件保留。
- `pnpm verify:bundle`：9/9 通过，记录为 `artifacts/verify-1794324277928/results.json`。本轮复核范围为自动进程验收，前述人工 UI 记录仍来自 M1 开发验收。
- 许可证全文与 SPDX 官方文件 blob `0c97efd25b5974b974ed9a8a18207bc4f55bb338` 一致；元数据、文档相对链接、28 个忽略路径和 19 个保留路径均检查通过。
- 只读审查未发现阻断项。第三方组件文件为许可索引，完整二进制依赖合规审计和对外发布仍不在本轮范围。
