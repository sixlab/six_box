# 参与 SixBox / Contributing to SixBox

## 问题反馈与开发

欢迎通过 [GitHub Issues](https://github.com/sixlab/six_box/issues) 报告问题或讨论需求。请提供系统版本、芯片架构、复现步骤和必要日志；移除令牌、账号信息和其他敏感数据。

- 当前阶段为 M1。较大变更先讨论范围，不在修复中混入下一阶段功能。
- 开发基准见 [PRD](docs/PRD.md)、[架构设计](docs/ARCHITECTURE.md) 和 [AGENTS.md](AGENTS.md)。
- 优先简单实现，修改行为时同步调用方和文档。
- 提交主题和正文使用中文，采用 Conventional Commits，例如 `fix(runtime): 修复服务退出清理`。
- 代码变更执行 `pnpm check`；影响构建、资源或进程行为时，还需执行 `pnpm build` 和 `pnpm verify:bundle`。验收前退出已有 SixBox。
- 不提交依赖目录、内置运行时、构建产物、日志、凭据或用户数据。保留两个锁文件。

## 贡献的授权

本仓库使用 [AGPL-3.0-only](LICENSE)。提交者须有权提供贡献，并披露引入的第三方代码、来源和许可条件；涉及雇主或其他权利人的代码，应先取得相应许可。

SixLab 计划维护官方闭源服务和商业产品。**合并外部代码贡献前，维护者须另行取得允许 SixLab 商业使用、闭源发行及重新许可该贡献的明确书面授权。** 如涉及专利，也应处理相应授权。贡献者保留自己的版权。

当前没有自动 CLA 签署服务，也未在此提供完整的贡献者许可协议。请先通过 Issue 或 PR 与维护者确认授权安排；完成所需书面授权并保存记录前，不合并外部代码贡献。该要求适用于贡献合并流程，不限制任何人依据 AGPL 使用或 fork 本项目。

**提交 PR、普通签名提交或 `Signed-off-by` 不视为已授予上述额外权利。** 不得把 AGPL 本身当作任意闭源重新许可的授权，也不得把额外授权解释为撤销已发布版本的 AGPL 权利。一般问题反馈不视为版权转让。

## English

Report bugs and discuss proposals through [GitHub Issues](https://github.com/sixlab/six_box/issues). Include your OS version, architecture, reproduction steps, and relevant logs with sensitive information removed. The current scope is M1; discuss substantial changes before implementation.

Follow the [requirements](docs/PRD.md), [architecture](docs/ARCHITECTURE.md), and [repository instructions](AGENTS.md). Use Chinese Conventional Commit subjects and bodies. Run `pnpm check` for code changes, plus `pnpm build` and `pnpm verify:bundle` when builds, resources, or process behavior are affected. Quit existing SixBox instances before bundle verification. Keep lockfiles; exclude generated artifacts, credentials, and user data.

You must have the right to submit your contribution and disclose third-party code and its license. Obtain employer or other rights-holder permission where necessary.

The repository is licensed under AGPL-3.0-only. Before merging external code contributions, maintainers must obtain and retain a separate, explicit written grant allowing SixLab to use those contributions commercially, distribute them in proprietary products, and relicense them, including relevant patent permissions where needed. Contributors retain their copyright.

No automated CLA service or complete contributor agreement is provided here. Contact the maintainers through an Issue or PR to arrange the required written permission before merge. A PR, a signed commit, or a `Signed-off-by` trailer does not by itself grant those additional rights. This contribution policy does not restrict anyone's AGPL rights, transfer copyright through ordinary feedback, or revoke AGPL rights already granted for published versions.
