# 第三方组件 / Third-party components

SixBox 的 `AGPL-3.0-only` 声明适用于本项目原创内容。第三方组件保留各自的版权、许可证和归属声明，不因被 SixBox 使用而改成由 SixLab 单独持有版权。

下表是主要依赖的许可索引，不是全部传递依赖的随包法律声明。准确版本以 `pnpm-lock.yaml`、`apps/desktop/src-tauri/Cargo.lock` 及运行时准备脚本为准；应保留各组件实际分发版本附带的完整许可证。

| 组件 | 许可证声明 | 来源 |
|---|---|---|
| Tauri（Rust、JS API、CLI） | MIT OR Apache-2.0 | [tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| React、React DOM、Scheduler | MIT | [facebook/react](https://github.com/facebook/react) |
| Node.js | MIT 及其内含组件的许可，详见原始 LICENSE | [nodejs/node](https://github.com/nodejs/node/blob/main/LICENSE) |
| serde、serde_json | MIT OR Apache-2.0 | [serde](https://github.com/serde-rs/serde)、[serde_json](https://github.com/serde-rs/json) |
| Tokio | MIT | [tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| fs2 | MIT OR Apache-2.0 | [danburkert/fs2-rs](https://github.com/danburkert/fs2-rs) |
| TypeScript | Apache-2.0 | [microsoft/TypeScript](https://github.com/microsoft/TypeScript) |
| esbuild | MIT | [evanw/esbuild](https://github.com/evanw/esbuild) |

`scripts/prepare-runtime.mjs` 从经过哈希校验的 Node.js 官方归档中原样复制 `LICENSE`，构建后位于 `SixBox.app/Contents/Resources/runtime/NODE-LICENSE`，包含 Node.js 内含组件的声明。不要替换成仅含 MIT 摘要的文件。

当前 Cargo 锁定依赖中还包含 MPL-2.0 组件，例如 `cssparser`、`cssparser-macros`、`dtoa-short`、`option-ext` 和 `selectors`，以及使用 Unicode-3.0、BSD、ISC、Zlib 等许可的组件。锁文件包含构建和跨平台依赖，不能据此认定所有条目均进入 macOS 应用。发布二进制前，需根据实际打包内容保留完整归属和许可文本，并履行适用的 MPL 对应源码提供要求；参考 [Mozilla MPL FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/)。

## English

Third-party components retain their own copyrights, licenses, and notices. SixBox's AGPL-3.0-only declaration covers the project's original material and does not replace those terms.

The table is an index of major dependencies, not a complete set of redistribution notices for all transitive dependencies. Consult both lockfiles and the runtime preparation script for exact versions, and preserve the full license files supplied with the actual distributions.

The original Node.js LICENSE, including its bundled third-party notices, is copied from the verified official archive into `SixBox.app/Contents/Resources/runtime/NODE-LICENSE`. The Cargo lockfile also contains MPL-2.0 and other licensed components, including build-time and cross-platform dependencies. Before distributing binaries, identify what is actually shipped, include the required notices and license texts, and meet any applicable MPL source availability obligations. This index does not claim that the full binary dependency review has been completed.
