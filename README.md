# SixBox 工具箱

**常用工具，一盒搞定。**

简体中文 · [English](README.en.md)

SixBox 是由 **SixLab（六楼实验室）** 开发的本地优先桌面工具箱，目标是把常用工具、插件和快捷操作收纳到一个应用中。

[GitHub](https://github.com/sixlab/six_box) · [问题反馈](https://github.com/sixlab/six_box/issues) · [AGPL-3.0-only](LICENSE)

## 当前进展

项目处于早期开发阶段，**已完成 M1：工程与运行时基础**。当前可构建和验证的平台为 **macOS Apple Silicon**。

已实现：

- Tauri 2 + React + TypeScript 桌面基础界面、运行状态和诊断。
- Rust 管理系统数据目录、单实例、窗口与应用退出。
- 随应用附带 Node.js，无需使用者另装 Node.js。
- 管理服务启动、健康检查、异常恢复、手动重试和退出清理。
- 内置 Node.js 缺失时，基础界面和诊断仍可使用。

插件系统、完整管理页面、全局快捷面板和具体工具尚未实现。当前版本适合开发与验证，尚未提供面向普通用户的正式安装包。

## 开发环境

| 依赖 | 要求 |
|---|---|
| 系统 | macOS，Apple Silicon / arm64 |
| 原生工具链 | Xcode Command Line Tools |
| Rust | 1.87 或以上，包含 `clippy`、`rustfmt` |
| Node.js | 24.14.0 或以上的 24.x 版本 |
| pnpm | 12.15.0 |

```bash
git clone https://github.com/sixlab/six_box.git
cd six_box
pnpm install --frozen-lockfile
pnpm dev
```

首次运行会下载并校验官方 Node.js **24.22.2**，供最终应用内置使用。开发者的 Node.js / pnpm 仅用于构建；应用运行时使用自己的 Node.js。

离线准备运行时、工程结构及通信协议见[开发说明](docs/DEVELOPMENT.md)。

## 检查与构建

```bash
pnpm check          # 类型检查、TypeScript / Rust 测试、clippy、rustfmt
pnpm build          # 构建 macOS arm64 正式应用
pnpm verify:bundle  # 验收真实应用进程，执行前退出已有 SixBox
```

构建产物：

```text
.cache/target/aarch64-apple-darwin/release/bundle/macos/SixBox.app
```

当前使用本机 ad-hoc 签名，尚未做 Developer ID 签名、公证或 DMG 安装器。构建声明最低 macOS 13.5；目前实测 macOS 26.6.2，其他系统版本尚未验证。Windows、Linux 和 macOS Intel 尚未适配。

`verify:bundle` 需要 macOS 图形会话，会使用系统标准应用目录，只终止本轮启动的进程，不清除已有用户数据。结果保存在 `artifacts/verify-*/results.json`。

## 后续计划

| 阶段 | 内容 | 状态 |
|---|---|---|
| M1 | 工程、桌面壳、内置运行时和系统目录 | 已完成 |
| M2 | 插件 SDK、安装与生命周期管理 | 规划中 |
| M3 | 独立窗口、管理页面与统一托盘 | 规划中 |
| M4 | 命令注册与全局快捷面板 | 规划中 |
| M5 | 网络监控与 JSON 工具 | 规划中 |
| M6 | 手动安全更新、长期运行和发布验收 | 规划中 |

账号、云同步和 Pro 功能属于后续规划，不在当前 M1 范围内。官方服务端计划独立维护并保持闭源。

## 文档与参与

- [产品需求基线](docs/PRD.md)
- [技术设计基线](docs/ARCHITECTURE.md)
- [开发、协议与构建说明](docs/DEVELOPMENT.md)
- [M1 验收报告与验证边界](docs/M1_REPORT.md)
- [PoC 差异与复用核对](docs/POC_REVIEW.md)
- [贡献指南](CONTRIBUTING.md)
- [许可与商业使用说明](docs/LICENSING.md)
- [第三方组件说明](THIRD_PARTY_NOTICES.md)

欢迎通过 GitHub Issues 反馈问题、讨论需求。提交代码前请阅读贡献指南；当前开发范围以 M1 为准。

## 许可证

Copyright © 2026 SixLab（六楼实验室）。

除另有声明的第三方组件外，本仓库采用 **GNU Affero General Public License v3.0 only**，SPDX 标识为 `AGPL-3.0-only`。完整条款见 [LICENSE](LICENSE)，项目授权声明见 [COPYRIGHT](COPYRIGHT)。

**AGPL 允许商业使用。** 分发软件及其衍生版本、或修改后通过网络与用户交互时，需要按协议履行相应的源码提供义务。本项目不附加“禁止商业使用”条款。官方独立服务端和另行授权的产品不因本仓库公开而自动成为开源软件；具体边界见[许可说明](docs/LICENSING.md)。
