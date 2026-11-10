# SixBox 许可说明 / Licensing

## 适用协议

本仓库原创代码、文档和资源采用 **GNU Affero General Public License v3.0 only**（`AGPL-3.0-only`），另有声明的第三方内容除外。项目权利人署名为 **SixLab（六楼实验室）**。

- [LICENSE](../LICENSE)：完整英文协议原文。
- [COPYRIGHT](../COPYRIGHT)：SixBox 的版权和版本选择声明。
- [第三方组件说明](../THIRD_PARTY_NOTICES.md)：第三方许可索引和发布注意事项。
- [贡献指南](../CONTRIBUTING.md)：代码贡献的授权流程。

`LICENSE` 取自 [SPDX 官方许可证文本仓库](https://github.com/spdx/license-list-data/blob/main/text/AGPL-3.0-only.txt)，文件 blob 为 `0c97efd25b5974b974ed9a8a18207bc4f55bb338`，未修改正文。固定选择第 3 版，不自动接受将来的协议版本；原文末尾的应用示例不改变该选择。

本说明用于帮助理解，不替代或修改协议。发生不一致时，以完整协议和适用的授权声明为准。

## 使用、修改与分发

| 场景 | 说明 |
|---|---|
| 个人使用、企业内部使用 | 允许；原样运行不要求另购商业许可 |
| 学习、修改、fork | 允许；后续分发及网络交互须遵守协议 |
| 收费分发原版或改版 | 允许；保留版权和许可声明，标明修改，并按第 4—6 条提供对应源码 |
| 修改后供用户通过网络交互 | 按第 13 条，向这些用户显著提供免费获取该版本对应源码的机会 |
| 把受 AGPL 覆盖的衍生软件闭源分发 | AGPL 本身不授予这种权利；须另行取得所有必要授权 |

**本项目没有“禁止商用”或“只有官方能收费”的附加条款。** 履行 AGPL 义务的第三方可以收费并参与竞争。源码义务针对协议所覆盖的作品及适用场景，不是要求公开使用者的所有代码、业务数据或密钥。

## 官方服务端、云同步和 Pro

账号、云同步及 Pro 仍是后续规划，当前 M1 没有这些服务。本仓库的许可不承诺公开未包含在仓库中的官方服务端代码，也不授予访问官方付费服务的权利。

独立开发的服务端不会仅因与 AGPL 客户端通信就自动适用 AGPL；是否构成同一个或衍生程序，要结合代码复用、链接和通信关系判断，不能只靠拆仓库或更换进程规避义务。未来客户端中按 AGPL 发布的部分仍受 AGPL 约束，不能因为标记为 Pro 就自动改为闭源。

SixLab 只能对自己持有版权或已获充分授权的部分另行许可；第三方依赖和贡献者的代码不能凭维护者身份自动重新许可。外部贡献的额外商业授权见[贡献指南](../CONTRIBUTING.md)。未来许可调整不撤销已依法取得的旧版本 AGPL 权利。

## 名称与品牌

AGPL 授予的代码权利不等于官方背书或商标授权。可如实说明来源与兼容关系，不得把第三方版本冒充官方产品。本说明不禁止 AGPL 允许的修改、分发或商业使用。

## 发布维护

- `package.json`、Rust 和 Tauri 元数据统一使用 `AGPL-3.0-only`。
- 正式应用附带 `LICENSE`、`COPYRIGHT` 和第三方组件说明；Node.js 原始许可证位于应用资源的 `runtime/NODE-LICENSE`。
- 对外提供二进制时，应同时提供该版本可构建的对应源码、锁文件、构建脚本及必要说明。下载页关联实际版本的 tag 或 commit，不能仅指向持续变化的分支。
- 修改程序并提供远程网络交互时，按第 13 条维护显著、可用的对应源码入口。
- 更新依赖时重新核对许可和归属声明。本文与第三方索引不是完整依赖合规审计。
- 正式商业发行及另行授权前，核实 SixLab 名称对应的实际签约主体和权利链；需要时由法律专业人士复核。

## English

Original code, documentation, and assets in this repository are licensed under **AGPL-3.0-only**, except where separate notices apply. The project copyright notice names **SixLab（六楼实验室）**. The unmodified license text comes from the SPDX license text repository linked above. The project chooses version 3 only, regardless of the illustrative “or later” notice in the license's application examples.

Commercial use is permitted. Distribution, including paid distribution, must satisfy the applicable license, notice, modification, and Corresponding Source requirements. Under section 13, modified versions supporting remote user interaction must prominently offer those users free access to their Corresponding Source. There is no noncommercial or official-only sales restriction. This guide does not replace or amend the license.

Official accounts, cloud synchronization, and Pro features are future plans outside M1. This repository does not license unpublished server code or grant access to paid official services. An independent server is not automatically covered merely because it communicates with the client, but the legal boundary depends on code reuse and how the programs are combined. Separate repositories or processes alone do not decide it. Client code published under the AGPL remains subject to it even if labeled Pro.

SixLab can offer alternative licenses only for code it owns or has sufficient rights to relicense, while respecting third-party licenses. External contributions require the separate authorization described in [CONTRIBUTING.md](../CONTRIBUTING.md) before merge. Future licensing changes do not revoke existing AGPL rights. The code license does not grant official endorsement or trademark rights.

Distributors must provide the applicable notices and the Corresponding Source for the exact binary version, including necessary build files and instructions. A link to a moving branch is insufficient by itself. Modified network services must also satisfy section 13. The application bundles the license, copyright notice, and third-party index; Node.js carries its original license separately. Review dependency notices for each release. These documents are not a complete dependency compliance audit.

## 参考 / References

- [AGPL-3.0 正文（OSI）](https://opensource.org/license/agpl-3.0)
- [SPDX：AGPL-3.0-only](https://spdx.org/licenses/AGPL-3.0-only.html)
- [GNU：独立程序与组合程序](https://www.gnu.org/licenses/gpl-faq.en.html#MereAggregation)
- [GNU：版权持有人采用多种许可](https://www.gnu.org/licenses/gpl-faq.en.html#ReleaseUnderGPLAndNF)
