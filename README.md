# patchsplit

Language: English | [Simplified Chinese](README_zh-cn.md)

`patchsplit` is a Rust CLI that downloads a GitHub Pull Request `.patch` file
and splits it into one patch file per commit.

## Usage

```sh
patchsplit <owner/repo> <pr-number> [--out <dir>] [--force]
patchsplit <owner> <repo> <pr-number> [--out <dir>] [--force]
```

Examples:

```sh
patchsplit rust-lang/rust 12345
patchsplit openai codex 42 -o pr-42-patches
```

The default output directory is `patches/`. Output files are named with a
four-digit index and the commit subject:

```text
patches/
  0001-add-parser.patch
  0002-wire-cli.patch
```

Existing output files are not overwritten by default. Pass `--force` to replace
them.

## Options

- `-o, --out <dir>`: Output directory for split patch files.
- `-f, --force`: Overwrite existing patch files.
- `-h, --help`: Show help.
- `-V, --version`: Show version.

## Dependencies

The CLI uses `thiserror` for internal error types. It calls the system `curl`
command to download GitHub `.patch` files, so `curl` must be available in
`PATH` at runtime.

## Localization

User-facing CLI text is wired through a PO-based i18n layer. At runtime,
`patchsplit` picks the first locale from `PATCHSPLIT_LANGUAGE`, `LANGUAGE`,
`LC_ALL`, `LC_MESSAGES`, or `LANG`, and reads UTF-8 `.po` catalogs from
`PATCHSPLIT_LOCALEDIR` or installation-relative locations next to the
executable.

Refresh the translation template with GNU gettext tools:

```sh
scripts/update-pot.sh
```

The template is generated at `po/patchsplit.pot`. Source files used for
extraction are listed in `po/POTFILES.in`.

## Build

```sh
cargo build --release
```

The release binary is generated at:

```text
target/release/patchsplit
```

## Release

Pushing a `v*` tag triggers GitHub Actions to build release packages for three
platforms and automatically create a GitHub draft release:

```sh
git tag v1.x.x
git push origin v1.x.x
```

You can also run the `Release` workflow manually from GitHub Actions. Select the
branch or commit to package, then enter the release tag. If the tag does not
exist, it will point to the workflow commit. The workflow creates these files:

- `patchsplit-linux-x86_64.tar.gz`
- `patchsplit-macos-x86_64.tar.gz`
- `patchsplit-windows-x86_64.zip`

Releases are created as drafts, so they should be reviewed and published from
the GitHub Releases page.

## Install

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

If macOS blocks the downloaded binary, remove the quarantine attribute:

```sh
xattr -d com.apple.quarantine /usr/local/bin/patchsplit
```

### Windows

Extract the archive in PowerShell:

```powershell
Expand-Archive .\patchsplit-windows-x86_64.zip -DestinationPath .\patchsplit
.\patchsplit\patchsplit.exe --version
```

To use it globally, add the extracted `patchsplit` directory to your user `Path`
environment variable.
