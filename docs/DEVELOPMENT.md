# SixBox M1 开发说明

## 工程结构

```text
apps/desktop/              React 主界面与 Tauri 工程
  src/                    基础界面、运行状态和诊断
  src-tauri/src/
    main.rs               原生生命周期、受限 commands、最小托盘
    paths.rs              系统目录初始化
    single_instance.rs    文件锁及聚焦通知
    lifecycle.rs          退出清理完成前阻止重复退出
    runtime.rs            管理服务状态、重试与监督
    runtime/session.rs    Node 检测、握手、请求与退出
    protocol.rs           有界 JSON 帧和响应校验
    logging.rs            有界 JSONL 日志
packages/core/            TypeScript 管理服务（尚不加载插件）
packages/plugin-protocol/ 宿主/管理服务类型（不是完整插件 SDK）
scripts/                  开发、构建、运行时准备与正式包验收
docs/                     PRD、技术设计、差异核对与验收
```

没有为了后续阶段建立 SDK、UI Kit、MCP、AI 或云服务空模块。

## 安装与运行

环境：macOS arm64、Rust ≥1.87、Xcode Command Line Tools、Node.js 24、pnpm 12.15.0。

```bash
pnpm install --frozen-lockfile
pnpm dev
```

开发脚本构建 TS 服务、准备内置 Node、启动 esbuild 监听和 Tauri dev。界面地址 `http://127.0.0.1:1420`；修改 React/CSS 后刷新窗口，修改 core 后重新启动 `pnpm dev`。开发服务器只绑定回环地址，不是产品插件 API。

若已有官方归档，可离线准备；路径必须指向 `node-v24.22.2-darwin-arm64.tar.gz`：

```bash
SIXBOX_NODE_ARCHIVE=/绝对路径/node-v24.22.2-darwin-arm64.tar.gz pnpm prepare:runtime
```

准备脚本始终校验固定归档 SHA-256 和 Node 文件 SHA-256，检查版本、平台、架构和系统动态库依赖，并包含 Node LICENSE。它是开发构建步骤；应用本身不在线下载运行时、不读取系统 Node 或用户自定义 Node 路径。

## 正式构建

```bash
pnpm build
open .cache/target/aarch64-apple-darwin/release/bundle/macos/SixBox.app
```

正式包中：

```text
SixBox.app/Contents/
  MacOS/sixbox
  MacOS/node
  Resources/LICENSE
  Resources/COPYRIGHT
  Resources/THIRD_PARTY_NOTICES.md
  Resources/runtime/service.cjs
  Resources/runtime/NODE-LICENSE
  Resources/runtime/node-runtime.json
```

React 静态资源由 Tauri 编入宿主，不由 Node 提供。缺失 Node 时仍可显示界面、诊断和退出入口。开发和正式包使用同一套服务协议。

项目使用 `AGPL-3.0-only`，正式包附带许可和版权文件。第三方组件保留原许可；第三方索引不等于完整随包声明。对外分发前须补齐实际依赖的声明及该版本对应源码入口，见 [许可说明](LICENSING.md) 和 [第三方组件说明](../THIRD_PARTY_NOTICES.md)。

固定 Tauri Rust 2.11.6 / JS API 2.11.0 / CLI 2.12.4，具体依赖以两个 lockfile 为准。JS API 与 Rust 保持相同 minor，避免 CLI 的版本一致性检查失败。

当前仅生成本机 ad-hoc 签名 `.app`，不包含公证、自动发布、DMG 安装器或自动更新。`minimumSystemVersion=13.5` 为构建下限，实测范围另见 M1 报告。

## 数据与日志

路径由 Rust 的 Tauri PathResolver 解析并传给管理服务。

| 类型 | macOS 路径 |
|---|---|
| 应用数据 | `~/Library/Application Support/cn.sixlab.box/` |
| 缓存 | `~/Library/Caches/cn.sixlab.box/` |
| 日志 | `~/Library/Logs/cn.sixlab.box/` |

应用数据下创建 `plugins/`、`plugin-data/`、`settings/`、`data/`、`runtimes/node/`。M1 只创建目录，没有插件业务或数据库。已有文件不覆盖，初始化失败会留在基础界面显示错误；修复目录后重启应用。

日志为 `host.jsonl` 和最多 3 个轮转文件，每文件最多 1 MiB，单条消息截断到 2048 字符。启动时删除文件修改时间超过 7 天的日志；持续运行期间按大小轮转，不承诺逐条记录严格在 7 天后删除。日志不包含插件业务输入，也不保存通信令牌。

## 管理协议 v1

父子进程 stdin/stdout 使用 UTF-8 逐行 JSON。每帧最多 64 KiB；stdout 只承载协议，stderr 经 Rust 写入有界日志。通信不监听 TCP 端口，管道只继承给直接子进程，因此 M1 不需要网络认证令牌；未来插件 HTTP API 的临时认证仍是必需项。

```json
{"version":1,"type":"request","id":1,"method":"initialize","params":{"paths":{}}}
{"version":1,"type":"response","id":1,"result":{"initialized":true}}
{"version":1,"type":"event","name":"ready","payload":{"pid":123,"nodeVersion":"v24.22.2","execPath":".../MacOS/node"}}
{"version":1,"type":"request","id":2,"method":"ping","params":{}}
{"version":1,"type":"response","id":2,"result":{"alive":true}}
{"version":1,"type":"request","id":3,"method":"shutdown","params":{}}
{"version":1,"type":"response","id":3,"result":{"stopping":true}}
```

示例中的 `paths` 需填入协议包 `AppPaths` 的 8 个绝对路径。错误响应使用 `error: { code, message }`，与 `result` 互斥。未知格式无法关联请求时 ID 为 null；Rust 将错误 ID 或非法消息视为通道故障。请求不自动重放，M1 只暴露上述 3 个方法。

## 运行时与应用生命周期

- 检测内置文件的执行能力、精确版本和 macOS arm64 架构，清除 Node 注入环境变量，不回退 PATH 中的 Node。
- 检测 / Ready / 请求上限均为 3 秒；Ready 后每 5 秒 ping。停止 3 秒内未完成则强杀并等待回收。
- 服务异常最多自动重启 3 次，间隔 0.5、1、2 秒。一次应用启动或手动重试为一个恢复周期，成功 Ready 不立即清零次数。达到上限后显示错误，等待用户重试。
- 重试与停止串行处理，运行中拒绝重复重试；服务停止不会关闭基础窗口。
- `.instance.lock` 通过操作系统文件锁保证同一数据目录只有一个宿主。持锁者才初始化服务；Unix socket 只用于聚焦通知，消息只有一个固定字节，具有确认和超时，不开放任何管理能力。
- 原采用的 Tauri single-instance 2.4.5 在 macOS 存在 notify→异步 bind 竞态，因此已移除该依赖；分工仍由 Rust 管理单实例，属于本次局部实现决策。
- 关闭主窗口隐藏到后台；托盘与 Dock 可以重新显示。M1 托盘仅含打开和退出。
- UI 退出、Cmd+Q、SIGTERM/SIGINT 均等待服务停止；清理完成前再次退出不会绕过等待。
- 宿主异常终止时，正常响应中的管理服务会收到 stdin EOF 并退出。SIGKILL 宿主且同时 SIGSTOP 服务无法靠 EOF 回收，不属于当前保证；M1 没有插件 Worker 或任意孙进程。

前端只有 `get_snapshot`、`retry_runtime`、`quit_app` 三个原生命令，ACL 只授权本地主窗口，无 shell/文件系统权限。不在页面注入测试控制接口。

## 测试

```bash
pnpm typecheck
pnpm test:ts
pnpm test:rust
pnpm check
pnpm build
pnpm verify:bundle
```

`test:rust` 会构建资源并准备 Node，随后执行包含真实子进程的 Rust 测试。`check` 额外执行 clippy 与 rustfmt。测试不需要第三方测试框架，TS 使用 Node 自带 test runner。

`verify:bundle` 需要当前 macOS 图形会话，会使用真实系统应用目录，不清除已有数据。先退出已有 SixBox；脚本只停止本轮启动的 PID，并在 `artifacts/verify-*/` 记录结果。复制的缺失 Node 测试包只在 artifacts 内修改。CI 执行类型、核心和完整编译检查；需要真实桌面的正式包验收在本机执行。

运行时二进制、构建输出、缓存和验收产物不提交 Git。新增协议和进程行为必须带有边界测试，后续不继续保留 PoC 接口。
