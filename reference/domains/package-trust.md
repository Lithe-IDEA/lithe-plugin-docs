# 包信任 / Package trust

[中文](#中文) · [English](#english)

## 中文

这一页说明插件包的目录规则、清单里的签名要求，以及级别 3 的 `packageTrust` 能力。签名要求决定宿主用什么证据接受一个包：发布者签名、宿主自身安装目录的代码签名，或者社区包的用户确认。签名和信任由宿主验证，Core 校验清单里的声明是否合法。

| 项目 | 内容 |
| --- | --- |
| 清单字段 | `vendor.id`、`vendor.displayName`、`vendor.signatureRequirement`；级别 3 可以再加 `vendor.bundledSignatureRequirement` |
| 能力版本 | `packageTrust` 1，只用于 `vendor.bundledSignatureRequirement`，级别 3。见[版本对照](../capabilities-and-versions.md#能力版本requires) |
| 签名文件 | 包根目录的 `lithe-plugin-signature.json`，Ed25519 签名，覆盖包里其他所有文件 |
| API 编号 | 不在 API-01–44 内（安装和打包层面的契约），见 [API 总表](../api-catalog.md#计划中的-api)下方的说明 |
| 权限、激活事件 | 没有 |
| 宿主信任的密钥 | macOS 只信任 Lithe 官方发布者密钥（`lithe-official-plugins-v2`）；社区开发者无法产出宿主信任的发布者签名 |
| Windows | 不验证发布者签名，不声明 `packageTrust`；从磁盘安装的包只靠用户确认安装审查，见下文[宿主现状](#宿主现状) |
| 官方 SDK | 与 SDK 无关；签名属于打包和发布环节 |

### 版本对照

| 版本 | 新增 | Core | macOS | Windows |
| --- | --- | --- | --- | --- |
| 基础规则（无能力版本，级别 1 起） | 包目录规则；`signatureRequirement` 的 `publisherPackage`、`sameTeamAsHost`、`none` | 校验清单和命名空间 | 支持：验证签名、安装封存、内容摘要 | 部分：只校验清单，不验证签名，不记录内容摘要 |
| `packageTrust` 1（级别 3） | `vendor.bundledSignatureRequirement: "sameTeamAsHost"`：随宿主发布的官方插件副本可以不带发布者签名 | 支持 | 支持 | 不支持（不声明，Core 拒绝要求它的包） |

### 包目录规则

| 规则 | 内容 |
| --- | --- |
| 根目录 | `plugin.json`、`lithe-plugin-signature.json`（需要时）和清单引用的资源 |
| 路径 | `plugin.json` 里的每个路径都相对包根目录，用 `/` 分隔，不能含 `..`、开头的 `/`、反斜杠或 NUL |
| 符号链接 | 包里不能有符号链接 |
| 只读 | 宿主把已安装的包当作只读；插件的可写状态放在宿主给的状态目录，不能写回包目录 |

### 三种签名要求

| `signatureRequirement` | 适用于 | 宿主怎样接受 |
| --- | --- | --- |
| `publisherPackage` | 官方插件和其他由宿主信任的发布者签名的包 | 签名文件必须有效，并由宿主信任的密钥签名。macOS 只有 DEBUG 构建允许把未签名的包从磁盘装成本地开发包（`localDevelopment`），发布构建拒绝 |
| `sameTeamAsHost` | 随宿主安装包一起发布的包 | 只认宿主自身已签名安装目录里的副本，由宿主的代码签名封存 |
| `none` | 社区包 | 不要求 Lithe 签名。从第三方市场安装时先用索引里的 SHA-256 校验压缩包；从本地 ZIP 或目录安装时先复制私有快照。两条路都要用户确认安装审查。`dev.lithe` 和 `dev.lithe.*` 不能使用（`plugin.manifest.invalid`）。包里带了签名文件仍要验证，无效签名不会被忽略 |

宿主固定自己官方插件 ID 的签名要求，包不能把自己降级。从磁盘选文件不能绕过这个固定值、命名空间限制、发布者密钥检查，也不能绕过 `sameTeamAsHost` 要求的安装封存。社区 `none` 包在 DEBUG 和发布构建里都走自己的确认流程，不会被当成开发包。不认识 `none` 的旧宿主会把这类清单当作无效清单拒绝，这是有意的。

### `packageTrust` 1：随宿主发布的官方副本

级别 3 的包可以同时声明：

- `requires.packageTrust: 1`
- `vendor.signatureRequirement: "publisherPackage"`
- `vendor.bundledSignatureRequirement: "sameTeamAsHost"`

只有官方命名空间的 ID 可以这样声明，两条要求都必须与宿主固定的官方目录一致。只有实际路径在宿主封存的安装目录内、来源为 `bundled` 的副本可以不带发布者签名；单独安装、导入或回退的副本仍然需要发布者签名。只要带了签名就会验证，封存目录里也一样。这条声明本身不授予任何信任。不支持 `packageTrust` 1 的宿主在下载前就过滤掉这个版本并拒绝它的清单，不需要新增应用版本字段。没有这条声明的旧清单保持原来的规则。

### 签名文件

`lithe-plugin-signature.json` 的字段：

| 字段 | 内容 |
| --- | --- |
| `schemaVersion` | `1` |
| `algorithm` | `"ed25519"` |
| `keyID` | 签名密钥的 ID，只能用小写字母、数字、`.`、`-`、`_` |
| `pluginID`、`pluginVersion` | 必须与 `plugin.json` 的 `id`、`version` 一致 |
| `files` | 包里除签名文件以外每个文件的相对路径到 SHA-256（小写十六进制）的映射；文件集合必须与包里的实际文件完全一致 |
| `signature` | 对规范负载的 Ed25519 签名，base64，64 字节 |

规范负载是 UTF-8 文本，每行以 `\n` 结尾：

```text
schemaVersion=1
algorithm=ed25519
keyID=<keyID>
pluginID=<token(pluginID)>
pluginVersion=<token(pluginVersion)>
file=<token(path)>=<sha256>      # 每个文件一行，按路径排序
```

`token` 是 UTF-8 字节的 base64url 编码，去掉末尾的 `=`。没有发布者签名的包，宿主在安装时记录内容摘要：把每个文件写成 `token(path)=<sha256>\n`，按路径排序后拼接，再取 SHA-256。之后每次扫描、暂存和回退都会重新比对。确切字节以 [`signature-vectors.json`](../../contract/v1/fixtures/package-trust/signature-vectors.json) 为准。

### 宿主现状

| 宿主 | 现状 |
| --- | --- |
| macOS | 完整实现：验证发布者签名（`PluginPackageSignature`），固定官方 ID 的签名要求，`sameTeamAsHost` 只认安装封存内的副本，社区包记录内容摘要并在扫描、暂存和回退时复查，声明 `packageTrust` 1 |
| Windows | 不验证发布者签名，不声明 `packageTrust`。磁盘安装复制私有快照，用 `plugins.marketplace.installReview` 生成审查，用户确认后安装；确认时只比对 `plugin.json` 有没有变，不计算其他文件的摘要，也不记录已批准的程序和来源 |

### 契约章节

| 章节 | 内容 |
| --- | --- |
| [Package layout](../../contract/v1/README.md#package-layout) | 包目录、路径和符号链接规则 |
| [Trust](../../contract/v1/README.md#trust) | 三种签名要求、`packageTrust` 1、官方 ID 固定和开发包规则 |
| [Capability versions (level 3)](../../contract/v1/README.md#capability-versions-level-3) | `packageTrust` 在能力表里的位置 |
| [Local disk snapshots and installation provenance (macOS)](../../contract/v1/README.md#local-disk-snapshots-and-installation-provenance-macos) | 内容摘要和安装来源记录，以及 Windows 的差距 |

### 示例与用例

| 文件 | 内容 |
| --- | --- |
| [`package-trust/publisher-bundled.plugin.json`](../../contract/v1/fixtures/package-trust/publisher-bundled.plugin.json) | 级别 3 的官方语言服务器插件，`requires` 含 `packageTrust: 1`，同时声明 `publisherPackage` 和 `bundledSignatureRequirement: "sameTeamAsHost"` |
| [`package-trust/signature-vectors.json`](../../contract/v1/fixtures/package-trust/signature-vectors.json) | 跨平台签名向量：RFC 8032 测试密钥（不能签真实的包）、含非 ASCII 文件名的固定包、规范负载、确定性签名和内容摘要。CryptoKit 这类带随机数的签名器要用验证代替字节比对。目前只有 macOS 的测试使用这组向量 |

### 相关指南

- [从零构建一个完整生命周期的 LSP 插件](../../guides/lsp-plugin-lifecycle.md)：社区包的信任、签名和发布

---

## English

This page covers the package layout, the signature requirement in the manifest, and the level 3 `packageTrust` capability. The signature requirement decides what evidence a host uses to accept a package: a publisher signature, the code signature of the host's own installation, or the user's confirmation for a community package. Hosts verify signatures and trust; Core validates that the manifest declarations are legal.

| Item | Details |
| --- | --- |
| Manifest fields | `vendor.id`, `vendor.displayName`, `vendor.signatureRequirement`; level 3 may add `vendor.bundledSignatureRequirement` |
| Capability version | `packageTrust` 1, only for `vendor.bundledSignatureRequirement`, level 3. See [versions](../capabilities-and-versions.md#capability-versions-requires) |
| Signature file | `lithe-plugin-signature.json` at the package root, an Ed25519 signature over every other file in the package |
| API number | Outside API-01–44 (an installation and packaging contract); see the note under the [API catalog](../api-catalog.md#planned-apis) |
| Permissions, activation events | None |
| Keys hosts trust | macOS trusts only the Lithe official publisher key (`lithe-official-plugins-v2`); community developers cannot produce a publisher signature a host trusts |
| Windows | Does not verify publisher signatures and does not advertise `packageTrust`; a package installed from disk relies only on the user confirming the install review. See [Host status](#host-status) |
| Official SDKs | Not SDK-related; signing is part of packaging and release |

### Versions compared

| Version | Adds | Core | macOS | Windows |
| --- | --- | --- | --- | --- |
| Baseline rules (no capability version, level 1 and later) | Package layout; `signatureRequirement` values `publisherPackage`, `sameTeamAsHost`, `none` | Validates the manifest and namespace | Supported: signature verification, installation seal, content digest | Partial: validates the manifest only; no signature verification and no content digest |
| `packageTrust` 1 (level 3) | `vendor.bundledSignatureRequirement: "sameTeamAsHost"`: an official plugin copy shipped with the host may omit the publisher signature | Supported | Supported | Not supported (not advertised; Core refuses packages that require it) |

### Package layout

| Rule | Details |
| --- | --- |
| Root | `plugin.json`, `lithe-plugin-signature.json` (when required), and the resources the manifest references |
| Paths | Every path in `plugin.json` is relative to the package root, uses `/`, and may not contain `..`, a leading `/`, a backslash, or NUL |
| Symbolic links | Packages may not contain symbolic links |
| Read-only | Hosts treat an installed package as read-only; a plugin's writable state goes in the state directory the host provides, never back into the package |

### The three signature requirements

| `signatureRequirement` | For | How a host accepts it |
| --- | --- | --- |
| `publisherPackage` | Official plugins and other packages signed by a publisher the host trusts | The signature file must be valid and signed by a key the host trusts. On macOS only DEBUG builds may install an unsigned package from disk as a local development package (`localDevelopment`); release builds refuse it |
| `sameTeamAsHost` | Packages shipped inside the host's installation | Honored only for a copy inside the host's own signed installation, sealed by the host's code signature |
| `none` | Community packages | No Lithe signature required. From a third-party marketplace the archive is first checked against the index SHA-256; from a local ZIP or directory a private snapshot is taken first. Both paths require the user to confirm the install review. `dev.lithe` and `dev.lithe.*` may not use it (`plugin.manifest.invalid`). A signature file that is present is still verified; an invalid signature is never ignored |

A host pins the signature requirement of its official plugin IDs so a package cannot downgrade itself. Selecting a file from disk never overrides that pin, the namespace restrictions, the publisher-key checks, or the installation seal `sameTeamAsHost` requires. Community `none` packages use their own approval path in both DEBUG and release builds and are never treated as development packages. Older hosts that do not know `none` refuse such manifests as invalid, which is intended.

### `packageTrust` 1: official copies shipped with the host

A level 3 package may declare all of:

- `requires.packageTrust: 1`
- `vendor.signatureRequirement: "publisherPackage"`
- `vendor.bundledSignatureRequirement: "sameTeamAsHost"`

Only official-namespace IDs may do so, and both requirements must match the host's pinned official catalog. Only a copy whose actual path is inside the host's sealed installation and whose origin is `bundled` may omit the publisher signature; every independently installed, imported, or rolled-back copy still needs it. A present signature is always verified, inside the seal too. The declaration alone grants no trust. Hosts without `packageTrust` 1 filter the version before download and refuse its manifest; no new application-version field is needed. An older manifest without the declaration keeps its existing rules.

### Signature file

Fields of `lithe-plugin-signature.json`:

| Field | Details |
| --- | --- |
| `schemaVersion` | `1` |
| `algorithm` | `"ed25519"` |
| `keyID` | ID of the signing key, limited to lowercase letters, digits, `.`, `-`, and `_` |
| `pluginID`, `pluginVersion` | Must equal `id` and `version` in `plugin.json` |
| `files` | Map from the relative path of every file except the signature file to its lowercase hex SHA-256; the set must equal the package's actual files exactly |
| `signature` | Base64 Ed25519 signature of the canonical payload, 64 bytes |

The canonical payload is UTF-8 text with every line ending in `\n`:

```text
schemaVersion=1
algorithm=ed25519
keyID=<keyID>
pluginID=<token(pluginID)>
pluginVersion=<token(pluginVersion)>
file=<token(path)>=<sha256>      # one line per file, sorted by path
```

`token` is the base64url encoding of the UTF-8 bytes with trailing `=` removed. For a package without a publisher signature, the host records a content digest at install time: each file becomes `token(path)=<sha256>\n`, the lines are sorted by path and concatenated, and the result is hashed with SHA-256. The digest is rechecked on every scan, staging, and rollback. [`signature-vectors.json`](../../contract/v1/fixtures/package-trust/signature-vectors.json) is authoritative for the exact bytes.

### Host status

| Host | Status |
| --- | --- |
| macOS | Complete: verifies publisher signatures (`PluginPackageSignature`), pins the signature requirement of official IDs, honors `sameTeamAsHost` only inside the installation seal, records a content digest for community packages and rechecks it on scan, staging, and rollback, and advertises `packageTrust` 1 |
| Windows | Does not verify publisher signatures and does not advertise `packageTrust`. Disk installation takes a private snapshot, builds the review with `plugins.marketplace.installReview`, and installs after the user confirms; confirmation only checks that `plugin.json` did not change, computes no digest of the other files, and records no approved programs or provenance |

### Contract sections

| Section | Covers |
| --- | --- |
| [Package layout](../../contract/v1/README.md#package-layout) | Package root, path, and symbolic-link rules |
| [Trust](../../contract/v1/README.md#trust) | The three signature requirements, `packageTrust` 1, official ID pinning, and development packages |
| [Capability versions (level 3)](../../contract/v1/README.md#capability-versions-level-3) | Where `packageTrust` sits in the capability table |
| [Local disk snapshots and installation provenance (macOS)](../../contract/v1/README.md#local-disk-snapshots-and-installation-provenance-macos) | Content digests, installation provenance records, and the Windows gap |

### Examples and cases

| File | Contents |
| --- | --- |
| [`package-trust/publisher-bundled.plugin.json`](../../contract/v1/fixtures/package-trust/publisher-bundled.plugin.json) | A level 3 official language-server plugin whose `requires` includes `packageTrust: 1`, declaring both `publisherPackage` and `bundledSignatureRequirement: "sameTeamAsHost"` |
| [`package-trust/signature-vectors.json`](../../contract/v1/fixtures/package-trust/signature-vectors.json) | Cross-platform signature vectors: the RFC 8032 test key (never for real packages), a fixed package with non-ASCII file names, the canonical payload, a deterministic signature, and the content digest. Signers that add randomness, such as CryptoKit, check by verification instead of byte comparison. Only the macOS tests consume these vectors today |

### Related guides

- [Building a complete-lifecycle LSP plugin](../../guides/lsp-plugin-lifecycle.md): community package trust, signing, and release
