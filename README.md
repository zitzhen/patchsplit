# patchsplit

Language: English | [Simplified Chinese](README_zh-cn.md)

Download GitHub pull requests or individual commits as patches, one file per
commit or one combined diff.

`patchsplit` is a command-line tool written in Rust. It fetches patches directly
from GitHub without cloning the repository, making it useful for reviewing,
sharing, and applying changes locally.

- **Per-commit patches:** Keep the original patch content, commit messages, and
  authorship, with numbered filenames in commit order.
- **Single commit:** Use `--commit <hash>` to download one commit's `.patch`
  with either a short or full hash.
- **Combined diff:** Use `--squash` to export the PR's net changes as a single patch.
- **Predictable output:** Choose an output directory; existing files are only
  overwritten when you pass `--force`.
- **Localized CLI:** English and built-in Simplified Chinese messages.

[Installation](#installation) | [Usage](#usage) | [Localization](#localization) |
[Development](#development) | [Contributing](#contributing) | [License](#license)

## Installation

`patchsplit` calls the system `curl` command to download patches. Make sure
`curl` is available in your `PATH`; the Debian/Ubuntu package declares it as a
dependency. Rust is only needed when building from source.

### Ubuntu (PPA)

Add the project PPA and refresh the package index:

```sh
sudo add-apt-repository ppa:zitzhen/patchsplit
sudo apt update
```

Install `patchsplit`:

```sh
sudo apt install patchsplit
```

### Debian / Ubuntu (.deb)

Download the `.deb` package from [GitHub Releases](https://github.com/zitzhen/patchsplit/releases),
then install it with the command below. Replace `<version>` with the version in
the downloaded filename.

```sh
sudo apt install ./patchsplit_<version>_amd64.deb
```

### Fedora / RHEL and compatible distributions (.rpm)

Download the main `.rpm` package from [GitHub Releases](https://github.com/zitzhen/patchsplit/releases).
Use its exact filename in place of `<package-file>`:

```sh
sudo dnf install ./<package-file>.rpm
```

Choose `patchsplit-<version>-<release>.*.x86_64.rpm`, excluding any package with
`debuginfo` or `debugsource` in its name.

### Arch Linux (AUR)

The community-maintained [`patchsplit-bin`](https://aur.archlinux.org/packages/patchsplit-bin)
package is provided by [lingbopro](https://github.com/lingbopro).
Install it with either AUR helper:

```sh
paru -S patchsplit-bin
# Or:
yay -S patchsplit-bin
```

### Linux / macOS (prebuilt binaries)

Download the archive for your platform from
[GitHub Releases](https://github.com/zitzhen/patchsplit/releases).
The release workflow builds x86_64 binaries for Linux, macOS, and Windows.

Extract the Linux archive:

```sh
tar -xzf patchsplit-linux-x86_64.tar.gz
```

Or extract the macOS archive:

```sh
tar -xzf patchsplit-macos-x86_64.tar.gz
```

Then install the extracted binary:

```sh
sudo install -m 755 patchsplit /usr/local/bin/patchsplit
patchsplit --version
```

If macOS blocks a binary you have verified and trust, remove its quarantine
attribute:

```sh
xattr -d com.apple.quarantine /usr/local/bin/patchsplit
```

### Windows

Download `patchsplit-windows-x86_64.zip` from
[GitHub Releases](https://github.com/zitzhen/patchsplit/releases), then extract
and run it in PowerShell:

```powershell
Expand-Archive .\patchsplit-windows-x86_64.zip -DestinationPath .\patchsplit
.\patchsplit\patchsplit.exe --version
```

To use it globally, add the extracted `patchsplit` directory to your user `Path`
environment variable. The CLI needs `curl.exe` available in `Path`.

### From source

With Rust, Cargo, and Git installed:

```sh
git clone https://github.com/zitzhen/patchsplit.git
cd patchsplit
cargo install --path . --locked
patchsplit --version
```

Ensure Cargo's binary directory (usually `~/.cargo/bin`) is in your `PATH`.

## Usage

```sh
patchsplit <owner/repo> <pr-number> [--out <dir>] [--force] [--squash]
patchsplit <owner> <repo> <pr-number> [--out <dir>] [--force] [--squash]
patchsplit <owner/repo> --commit <hash> [--out <dir>] [--force]
patchsplit <owner> <repo> --commit <hash> [--out <dir>] [--force]
```

### Split a pull request by commit

Replace the repository and PR number with the pull request you want to download:

```sh
patchsplit rust-lang/rust 12345
patchsplit openai codex 42 -o pr-42-patches
```

The default output directory is `patches/`, created automatically if needed.
Filenames use a zero-padded index and a sanitized commit subject, for example:

```text
patches/
  0001-add-parser.patch
  0002-wire-cli.patch
```

Existing output files are not overwritten by default. Pass `--force` to replace
them.

The per-commit files preserve GitHub's mail-formatted patches. To apply them
with commit messages and authorship, run the following inside the target Git
repository, checked out at a compatible base:

```sh
git am /path/to/patches/*.patch
```

### Export a combined diff

```sh
patchsplit openai/codex 42 --squash -o pr-42-patches
```

`-s, --squash` downloads GitHub's aggregate PR `.diff` and writes
`pr-<pr-number>.patch`. It represents the net change from the PR's merge base
to its head: repeated edits are combined and reverted changes disappear,
rather than concatenating the per-commit patches.

Apply it from inside the target repository on the corresponding base:

```sh
git apply --check /path/to/pr-42-patches/pr-42.patch
git apply /path/to/pr-42-patches/pr-42.patch
```

| Mode | Output | Commit messages and authorship | Apply with |
| --- | --- | --- | --- |
| Default | One numbered patch per commit | Preserved | `git am` |
| `--squash` | One `pr-<pr-number>.patch` | Not included | `git apply` |
| `--commit <hash>` | One `<hash>.patch` | Preserved | `git am` |

The combined output is a raw diff, not a `git am` mailbox. An empty net diff
is reported as an error and no file is written. Binary changes are limited to
the data GitHub includes in its diff; binary file contents may not be included.

### Download a single commit

Pass `--commit <hash>` (or `-commit <hash>`) with a short or full commit hash
instead of a pull request number:

```sh
patchsplit zitzhen patchsplit -commit b430113
patchsplit zitzhen/patchsplit --commit b4301133226e5c3a464cff9649de0b321c0b0a2e
```

This fetches `https://github.com/<owner>/<repo>/commit/<hash>.patch` and writes
the commit's mail-formatted patch verbatim to `<hash>.patch` in the output
directory, for example `patches/b430113.patch`. Apply it with `git am` just
like a per-commit PR patch. `--commit` cannot be combined with `--squash`.

### Options

| Option | Description |
| --- | --- |
| `-o, --out <dir>` | Output directory (default: `patches/`). |
| `-f, --force` | Overwrite existing patch files. |
| `-s, --squash` | Write the PR's net diff as one patch. |
| `--commit <hash>` | Download one commit's `.patch`; accepts a short or full hash. |
| `-h, --help` | Show help. |
| `-V, --version` | Show version. |

## Localization

`patchsplit` includes Simplified Chinese translations and uses English when
no matching translation is available. Language selection checks these
environment variables in order: `PATCHSPLIT_LANGUAGE`, `LANGUAGE`, `LC_ALL`,
`LC_MESSAGES`, then `LANG`.

Override the language for a single command on Linux or macOS:

```sh
PATCHSPLIT_LANGUAGE=zh_CN patchsplit --help
PATCHSPLIT_LANGUAGE=C patchsplit --help
```

Custom UTF-8 `.po` catalogs can be loaded from `PATCHSPLIT_LOCALEDIR` or
installation-relative locations next to the executable. External catalogs
take precedence over the built-in catalog for the same locale.

## Development

Build a release binary from the repository root:

```sh
cargo build --release --locked
```

The binary is written to `target/release/patchsplit` (`patchsplit.exe` on Windows).
Run the test suite with:

```sh
cargo test --locked
```

Tests on Unix also require Git to verify that aggregate patches apply to the
expected file tree.

### Updating translations

Refresh the translation template with GNU gettext tools:

```sh
scripts/update-pot.sh
```

The template is generated at [po/patchsplit.pot](po/patchsplit.pot). Source
files used for extraction are listed in [po/POTFILES.in](po/POTFILES.in).

## Contributing

Bug reports, feature requests, documentation improvements, and translations
are welcome. [Open an issue](https://github.com/zitzhen/patchsplit/issues) with
the command you ran, your OS, the `patchsplit` version, and the expected and
actual behavior when reporting a bug.

For code changes, add or update tests where relevant and run `cargo test --locked`
before submitting a pull request. Keep CLI documentation and translations in
sync with user-facing changes.

## Release maintenance

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
- `patchsplit_<version>_amd64.deb`
- `patchsplit-<version>-<release>.*.x86_64.rpm`

Releases are created as drafts, so they should be reviewed and published from
the GitHub Releases page.

### Launchpad PPA

Publishing a GitHub release triggers the `Notify PPA` workflow. It builds and
signs a Debian source package, then uploads it to Launchpad so the configured
PPA starts building the new version. Configure these repository variables:

- `PPA_OWNER`: Launchpad account name.
- `PPA_NAME`: PPA name, without the `ppa:` prefix.
- `PPA_GPG_KEY_ID`: full fingerprint of the primary signing key (not a signing subkey ID).
- `PPA_MAINTAINER_NAME` and `PPA_MAINTAINER_EMAIL`: optional source package metadata.

Store the ASCII-armored private key as the `PPA_GPG_PRIVATE_KEY` repository
secret. The workflow can also be run manually with a tag, branch, or commit in
the `ref` input.

## License

`patchsplit` is open source under the [MIT License](LICENSE).
