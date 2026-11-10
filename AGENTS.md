# SixBox 协作约定

## 沟通与提交

- 默认使用简体中文，结论前置，采用中国团队评审的直白短句。
- Git commit 主题和正文用中文；Conventional Commits type/scope、标识符、路径、专有名词和 trailer 保留英文。
- 本规则同样适用于分支、worktree、子代理、文档与修复。

## 实现原则

- 优先简单直接实现，优先修改现有代码并同步调用方，清理废弃逻辑。
- 不为临时 PoC 接口添加兼容层，不为假想需求创建空模块或扩展抽象。
- 默认不向后兼容；涉及存量外部接口、数据或独立组件时，先确认兼容要求。
- `docs/PRD.md` 与 `docs/ARCHITECTURE.md` 是开发基准。重大架构变更先说明。
- 当前阶段是 M1；完成阶段任务后不得自动开始下一阶段。

## 验证

- `pnpm check`：类型检查、TypeScript/Rust 测试、clippy、rustfmt。
- `pnpm build`：macOS Apple Silicon 正式应用。
- `pnpm verify:bundle`：真实应用进程验收，运行前退出已有 SixBox。
- 不以开发模式或 PoC 的测试结果替代 SixBox 正式包验收。
- 运行方法与验证边界见 `docs/DEVELOPMENT.md` 和 `docs/M1_REPORT.md`。
