# patchsplit

语言：[English](README.md) | 简体中文

`patchsplit` 是一个 Rust CLI，用来从 GitHub 下载 Pull Request 或单个 commit 的
`.patch` 文件，PR 补丁可按 commit 拆分成多个独立 patch 文件。

## 用法

```sh
patchsplit <owner/repo> <pr-number> [--out <dir>] [--force] [--squash]
patchsplit <owner> <repo> <pr-number> [--out <dir>] [--force] [--squash]
patchsplit <owner/repo> --commit <hash> [--out <dir>] [--force]
patchsplit <owner> <repo> --commit <hash> [--out <dir>] [--force]
```

示例：

```sh
patchsplit rust-lang/rust 12345
patchsplit openai codex 42 -o pr-42-patches
```

默认输出目录是 `patches/`。输出文件会使用四位序号和 commit subject 命名：

```text
patches/
  0001-add-parser.patch
  0002-wire-cli.patch
```

如果输出文件已存在，命令默认拒绝覆盖。需要覆盖时传入 `--force`。

### 聚合所有 commit

```sh
patchsplit openai/codex 42 --squash -o pr-42-patches
git apply pr-42-patches/pr-42.patch
```

`-s, --squash` 下载 GitHub 提供的 PR 整体 `.diff`，输出一个
`pr-<pr-number>.patch`。它表示 PR 从共同祖先（merge base）到最终 head 的净变化：
同一文件的多次修改会合并，已撤销的修改会消失，不是把各 commit 的补丁拼接起来。
输出为可在对应基线上用 `git apply` 应用的原始 diff，不包含各 commit 的提交说明和
作者信息，不能作为 `git am` 邮件补丁使用。净变化为空时会报补丁为空，不生成文件。
二进制变更受 GitHub diff 返回内容限制，可能不包含二进制文件内容。

### 下载单个 commit

不传 PR 编号，改用 `--commit <hash>`（也可写作 `-commit <hash>`）传入短哈希或
完整哈希：

```sh
patchsplit zitzhen patchsplit -commit b430113
patchsplit zitzhen/patchsplit --commit b4301133226e5c3a464cff9649de0b321c0b0a2e
```

对应下载 `https://github.com/<owner>/<repo>/commit/<hash>.patch`，并把该 commit
的邮件格式补丁原样写入输出目录下的 `<hash>.patch`，例如
`patches/b430113.patch`。它和按 commit 拆分的 PR 补丁一样可用 `git am` 应用。
`--commit` 不能与 `--squash` 同时使用。

## 参数

- `-o, --out <dir>`：指定 patch 文件的输出目录。
- `-f, --force`：允许覆盖已存在的 patch 文件。
- `-s, --squash`：将 PR 的最终净变化输出为一个补丁。
- `--commit <hash>`：下载单个 commit 的 `.patch`，接受短哈希或完整哈希。
- `-h, --help`：显示帮助。
- `-V, --version`：显示版本。

## 依赖

CLI 使用 `thiserror` 处理内部错误类型。下载 GitHub `.patch` 时会调用系统里的
`curl`，因此运行环境需要能在 `PATH` 中找到 `curl`。

## 本地化

CLI 的用户可见文本已经接入基于 PO 的 i18n 层。运行时，`patchsplit` 会从
`PATCHSPLIT_LANGUAGE`、`LANGUAGE`、`LC_ALL`、`LC_MESSAGES` 或 `LANG` 中选择
第一个 locale，并从 `PATCHSPLIT_LOCALEDIR` 或可执行文件旁边的安装目录读取
UTF-8 `.po` catalog。

使用 GNU gettext 工具刷新翻译模板：

```sh
scripts/update-pot.sh
```

生成的模板位于 `po/patchsplit.pot`。参与提取的源码文件列在
`po/POTFILES.in`。

## 构建

在 Unix 上运行测试（`cargo test`）还需要 Git，用于验证聚合补丁应用后的文件树。

```sh
cargo build --release
```

生成的二进制在：

```text
target/release/patchsplit
```

## 发版

推送 `v*` tag 会触发 GitHub Actions 打包三个平台的 release 产物，并自动创建
GitHub draft release：

```sh
git tag v1.x.x
git push origin v1.x.x
```

也可以在 GitHub Actions 的 `Release` workflow 里手动运行，选择需要打包的分支或
提交，并输入 release tag。tag 不存在时会指向本次 workflow 的提交。workflow 会
生成这些文件：

- `patchsplit-linux-x86_64.tar.gz`
- `patchsplit-macos-x86_64.tar.gz`
- `patchsplit-windows-x86_64.zip`
- `patchsplit_<version>_amd64.deb`
- `patchsplit-<version>-<release>.*.x86_64.rpm`

release 默认是草稿，需要在 GitHub Releases 页面检查后手动发布。

### Launchpad PPA

发布 GitHub release 后会触发 `Notify PPA` workflow。它会生成并签名 Debian
源代码包，然后上传到配置的 Launchpad PPA，由 Launchpad 自动构建新版本。请在
仓库设置以下 Variables：

- `PPA_OWNER`：Launchpad 账号名。
- `PPA_NAME`：PPA 名称，不包含 `ppa:` 前缀。
- `PPA_GPG_KEY_ID`：用于签名的主密钥完整指纹，不要填写签名子密钥 ID。
- `PPA_MAINTAINER_NAME` 和 `PPA_MAINTAINER_EMAIL`：可选的源包维护者信息。

将 ASCII-armored 格式的私钥保存为仓库 Secret `PPA_GPG_PRIVATE_KEY`。也可以手动运行
该 workflow，并在 `ref` 中输入 tag、branch 或 commit。

## 安装

`patchsplit` 会调用系统中的 `curl` 下载补丁，请确保 `curl` 位于 `PATH`。Debian/Ubuntu
软件包会自动声明此依赖。只有从源码构建时才需要 Rust。

### Ubuntu（PPA）

```sh
sudo add-apt-repository ppa:zitzhen/patchsplit
sudo apt update
sudo apt install patchsplit
```

> [!NOTE]
> 对于 Arch Linux，使用由 [lingbopro](https://github.com/lingbopro) 维护的
> [`patchsplit-bin`](https://aur.archlinux.org/packages/patchsplit-bin) AUR 包。
>
> - 使用 `paru`: `paru -S patchsplit-bin`
> - 使用 `yay`: `yay -S patchsplit-bin`

### Debian / Ubuntu（`.deb`）

在 Debian 和 Ubuntu 上，下载 release 中的 `.deb` 文件并执行：

```sh
sudo apt install ./patchsplit_<version>_amd64.deb
```

### Fedora / RHEL 及兼容发行版（`.rpm`）

从 release 下载主 `.rpm` 包，选择不含 `debuginfo` 或 `debugsource` 的 `x86_64` 文件：

```sh
sudo dnf install ./patchsplit-*.x86_64.rpm
```

如果 release 中同时提供了 debuginfo 包，请安装主包
`patchsplit-<version>-<release>.x86_64.rpm`，不要安装 debuginfo 包。

### Linux / macOS（预编译二进制）

从 [GitHub Releases](https://github.com/zitzhen/patchsplit/releases) 下载对应平台的压缩包，解压后安装：

```sh
tar -xzf patchsplit-linux-x86_64.tar.gz
sudo install -m 755 patchsplit /usr/local/bin/patchsplit
patchsplit --version
```

### macOS

```sh
tar -xzf patchsplit-macos-x86_64.tar.gz
chmod +x patchsplit
sudo install -m 755 patchsplit /usr/local/bin/patchsplit
patchsplit --version
```

如果 macOS 阻止运行从浏览器下载的二进制，可以移除 quarantine 属性：

```sh
xattr -d com.apple.quarantine /usr/local/bin/patchsplit
```

### Windows

在 PowerShell 中解压：

```powershell
Expand-Archive .\patchsplit-windows-x86_64.zip -DestinationPath .\patchsplit
.\patchsplit\patchsplit.exe --version
```

需要全局使用时，把解压后的 `patchsplit` 目录加入用户 `Path` 环境变量。

### 从源码安装

```sh
git clone https://github.com/zitzhen/patchsplit.git
cd patchsplit
cargo install --path . --locked
patchsplit --version
```

请确保 Cargo 的二进制目录（通常是 `~/.cargo/bin`）位于 `PATH` 中。

## 贡献

欢迎提交 bug、功能建议、文档改进和翻译。请通过 [GitHub Issues](https://github.com/zitzhen/patchsplit/issues)
提交问题，并附上命令、操作系统、版本及预期和实际行为。代码变更请运行 `cargo test --locked`。

## 许可证

`patchsplit` 使用 [MIT License](LICENSE) 开源。
