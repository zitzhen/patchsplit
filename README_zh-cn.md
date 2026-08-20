# patchsplit

语言：[English](README.md) | 简体中文

`patchsplit` 是一个 Rust CLI，用来从 GitHub 下载 Pull Request 的 `.patch`
文件，并按 commit 拆分成多个独立 patch 文件。

## 用法

```sh
patchsplit <owner/repo> <pr-number> [--out <dir>] [--force]
patchsplit <owner> <repo> <pr-number> [--out <dir>] [--force]
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

## 参数

- `-o, --out <dir>`：指定拆分后 patch 文件的输出目录。
- `-f, --force`：允许覆盖已存在的 patch 文件。
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

release 默认是草稿，需要在 GitHub Releases 页面检查后手动发布。

## 安装

### Linux

```sh
tar -xzf patchsplit-linux-x86_64.tar.gz
chmod +x patchsplit
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
