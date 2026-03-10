# todox

[中文文档 / Chinese README](./README.zh_CN.md)

`todox` is a local-first terminal todo app built with Rust and `ratatui`.
It focuses on a compact task list, a clear focus panel, and runtime language switching without any remote service dependency.

## Highlights

- Compact single-line task list focused on title, status, priority, and due date
- Detail panel for the selected task
- Keyboard-first editing flow
- Built-in runtime i18n with `en_US` and `zh_CN`
- Custom locale packs loaded from disk
- Plain JSON storage under your local data directory

## Quick Start

```bash
cargo run
```

Use a custom data file if you want to isolate a workspace or test dataset:

```bash
cargo run -- --data-file /custom/tasks.json
```

## Install from crates.io

```bash
cargo install todox-tui
todox
```

The package name on crates.io is `todox-tui`, while the installed command remains `todox`.

## Build Release

Build an optimized release binary:

```bash
cargo build --release
```

The executable will be generated at:

```text
./target/release/todox
```

You can then run it directly without `cargo`:

```bash
./target/release/todox
```

Or point it at a dedicated task file:

```bash
./target/release/todox --data-file /custom/tasks.json
```

If you want to use it as a standalone command, copy the binary into a directory on your `PATH`, for example:

```bash
install -m 755 ./target/release/todox /usr/local/bin/todox
```

After that, you can launch it anywhere with:

```bash
todox
```

## Download Prebuilt Releases

You can download ready-to-use binaries and Linux `.deb` packages from the
[GitHub Releases](https://github.com/gcnyin/todox/releases) page.

Current release assets:

- Linux x86_64: `todox-x86_64-unknown-linux-gnu.tar.gz`, `todox_<version>_amd64.deb`
- Linux arm64: `todox-aarch64-unknown-linux-gnu.tar.gz`, `todox_<version>_arm64.deb`
- macOS Apple Silicon: `todox-aarch64-apple-darwin.tar.gz`

Pick the asset that matches your system, then use one of the methods below.

### Download with `curl` or `wget`

For example, to download the Linux x86_64 binary archive:

```bash
VERSION=v0.1.0
BASE_URL="https://github.com/gcnyin/todox/releases/download/${VERSION}"

curl -LO "${BASE_URL}/todox-x86_64-unknown-linux-gnu.tar.gz"
```

Or with `wget`:

```bash
VERSION=v0.1.0
BASE_URL="https://github.com/gcnyin/todox/releases/download/${VERSION}"

wget "${BASE_URL}/todox-x86_64-unknown-linux-gnu.tar.gz"
```

To download the Debian package instead:

```bash
VERSION=v0.1.0
BASE_URL="https://github.com/gcnyin/todox/releases/download/${VERSION}"

curl -LO "${BASE_URL}/todox_0.1.0_amd64.deb"
```

Replace the filename with the asset you need for your platform.

## Use a Downloaded Binary Archive

### Linux / macOS

Extract the archive and run the binary directly:

```bash
tar -xzf todox-<target>.tar.gz
./todox
```

To install it as a regular command:

```bash
install -m 755 ./todox /usr/local/bin/todox
todox
```

## Install the Debian Package

For Debian, Ubuntu, and compatible distributions, you can install the `.deb`
package directly:

```bash
sudo apt install ./todox_<version>_amd64.deb
```

Or on Linux arm64:

```bash
sudo apt install ./todox_<version>_arm64.deb
```

After installation, launch it with:

```bash
todox
```

Remove it later if needed:

```bash
sudo apt remove todox
```

## Keyboard Basics

Main list:

- `Up` / `Down` or `j` / `k`: move between tasks
- `n`: new task
- `e`: edit selected task
- `Space`: toggle done
- `f`: change filter
- `/`: search
- `1` / `2` / `3`: set priority on the selected task
- `Left` / `Right` or `h` / `l`: cycle priority on the selected task
- `g`: open language picker
- `?`: help
- `q` or `Esc`: quit

Edit modal:

- `Tab`, `Shift+Tab`, `Up`, `Down`: move between fields
- `Left` / `Right` or `h` / `l`: adjust priority when the priority field is focused
- `1` / `2` / `3`: pick a priority when the priority field is focused
- `Enter`: save
- `Esc`: cancel

## Data Files

When the default data path is used, `todox` stores files under `~/.todox/`:

- `tasks.json`
- `config.json`
- `i18n/`

If you pass `--data-file /custom/tasks.json`, the app uses:

- `/custom/tasks.json`
- `/custom/config.json`
- `/custom/i18n/`

## Config Format

```json
{
  "i18n": {
    "locale": "en_US"
  }
}
```

## i18n

- Default locale: `en_US`
- Built-in locales: `en_US`, `zh_CN`
- Locale source: `<data-dir>/config.json`
- TUI shortcut: press `g` to open the language picker

## Custom Locale Packs

Place flat JSON files in `<data-dir>/i18n/` using the filename `<locale>.json`.
Missing keys fall back to built-in `en_US`.

Example `ja_JP.json`:

```json
{
  "top.filter": "フィルター",
  "top.search": "検索",
  "modal.locale.title": "言語"
}
```

The built-in `en_US` catalog in [`src/i18n.rs`](./src/i18n.rs) is the reference key list for custom translations.

## Development

```bash
cargo fmt
cargo test
```

## Release Checklist

```bash
# 1) bump the version in Cargo.toml
cargo fmt
cargo test
cargo package --list
cargo publish --dry-run --registry crates-io
cargo publish --registry crates-io
```

After publishing, verify the package page on crates.io and update the GitHub release assets if you are shipping prebuilt binaries.

## License

Licensed under the [GNU GPL v3.0 or later](./LICENSE).
