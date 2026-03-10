# todox

[English README / 英文文档](./README.md)

`todox` 是一个基于 Rust 和 `ratatui` 构建的本地优先终端待办应用。
它强调紧凑的任务列表、清晰的焦点详情面板，以及无需依赖远程服务的运行时语言切换能力。

## 特性

- 紧凑的单行任务列表，聚焦标题、状态、优先级和截止日期
- 当前选中任务的详情面板
- 面向键盘操作的编辑流程
- 内置运行时 i18n，默认提供 `en_US` 和 `zh_CN`
- 支持从磁盘加载自定义语言包
- 使用本地 JSON 文件存储数据

## 快速开始

```bash
cargo run
```

如果你想为某个工作区或测试数据集单独使用一份任务文件，可以这样启动：

```bash
cargo run -- --data-file /custom/tasks.json
```

## 从 crates.io 安装

```bash
cargo install todox-tui
todox
```

在 crates.io 上的包名是 `todox-tui`，安装后的命令仍然是 `todox`。

## 构建 Release

构建优化后的 release 可执行文件：

```bash
cargo build --release
```

生成的可执行文件路径为：

```text
./target/release/todox
```

之后你可以不依赖 `cargo`，直接运行这个二进制：

```bash
./target/release/todox
```

也可以指定一份独立的任务数据文件：

```bash
./target/release/todox --data-file /custom/tasks.json
```

如果你希望把它当作系统命令使用，可以把二进制复制到 `PATH` 目录，例如：

```bash
install -m 755 ./target/release/todox /usr/local/bin/todox
```

完成后，就可以在任意位置直接运行：

```bash
todox
```

## 下载预编译版本

你可以直接从 [GitHub Releases](https://github.com/gcnyin/todox/releases)
页面下载已经编译好的二进制文件和 Linux `.deb` 安装包。

当前提供的发布产物：

- Linux x86_64：`todox-x86_64-unknown-linux-gnu.tar.gz`、`todox_<version>_amd64.deb`
- Linux arm64：`todox-aarch64-unknown-linux-gnu.tar.gz`、`todox_<version>_arm64.deb`
- macOS Apple Silicon：`todox-aarch64-apple-darwin.tar.gz`

请根据你的系统选择对应文件，然后按下面的方法使用。

### 使用 `curl` 或 `wget` 下载

例如，下载 Linux x86_64 的二进制压缩包：

```bash
VERSION=v0.1.0
BASE_URL="https://github.com/gcnyin/todox/releases/download/${VERSION}"

curl -LO "${BASE_URL}/todox-x86_64-unknown-linux-gnu.tar.gz"
```

或者使用 `wget`：

```bash
VERSION=v0.1.0
BASE_URL="https://github.com/gcnyin/todox/releases/download/${VERSION}"

wget "${BASE_URL}/todox-x86_64-unknown-linux-gnu.tar.gz"
```

如果想下载 `.deb` 安装包：

```bash
VERSION=v0.1.0
BASE_URL="https://github.com/gcnyin/todox/releases/download/${VERSION}"

curl -LO "${BASE_URL}/todox_0.1.0_amd64.deb"
```

把文件名替换成你所需平台对应的发布产物即可。

## 使用下载好的二进制压缩包

### Linux / macOS

先解压，再直接运行二进制文件：

```bash
tar -xzf todox-<target>.tar.gz
./todox
```

如果你希望把它安装成全局命令，可以执行：

```bash
install -m 755 ./todox /usr/local/bin/todox
todox
```

## 安装 Debian 包

对于 Debian、Ubuntu 及兼容发行版，你可以直接安装 `.deb` 包：

```bash
sudo apt install ./todox_<version>_amd64.deb
```

如果你使用的是 Linux arm64：

```bash
sudo apt install ./todox_<version>_arm64.deb
```

安装完成后，可以直接运行：

```bash
todox
```

如果后续想卸载：

```bash
sudo apt remove todox
```

## 键位说明

主列表：

- `Up` / `Down` 或 `j` / `k`：在任务之间移动
- `n`：新建任务
- `e`：编辑当前任务
- `Space`：切换完成状态
- `f`：切换过滤器
- `/`：搜索
- `1` / `2` / `3`：直接设置当前任务优先级
- `Left` / `Right` 或 `h` / `l`：循环调整当前任务优先级
- `g`：打开语言选择器
- `?`：打开帮助
- `q` 或 `Esc`：退出

编辑弹窗：

- `Tab`、`Shift+Tab`、`Up`、`Down`：在字段之间移动
- `Left` / `Right` 或 `h` / `l`：当焦点在优先级字段时调整优先级
- `1` / `2` / `3`：当焦点在优先级字段时直接选择优先级
- `Enter`：保存
- `Esc`：取消

## 数据文件

使用默认数据路径时，`todox` 会把文件存储在 `~/.todox/` 下：

- `tasks.json`
- `config.json`
- `i18n/`

如果你传入 `--data-file /custom/tasks.json`，应用会改用：

- `/custom/tasks.json`
- `/custom/config.json`
- `/custom/i18n/`

## 配置格式

```json
{
  "i18n": {
    "locale": "en_US"
  }
}
```

## 国际化

- 默认语言：`en_US`
- 内置语言：`en_US`、`zh_CN`
- 语言配置来源：`<data-dir>/config.json`
- TUI 内快捷键：按 `g` 打开语言选择器

## 自定义语言包

把扁平 JSON 文件放到 `<data-dir>/i18n/` 目录下，文件名使用 `<locale>.json`。
缺失的 key 会自动回退到内置 `en_US`。

示例 `ja_JP.json`：

```json
{
  "top.filter": "フィルター",
  "top.search": "検索",
  "modal.locale.title": "言語"
}
```

内置 `en_US` 词条位于 [`src/i18n.rs`](./src/i18n.rs)，可作为自定义翻译的参考 key 列表。

## 开发

```bash
cargo fmt
cargo test
```

## 发布检查清单

```bash
# 1) 先在 Cargo.toml 中更新版本号
cargo fmt
cargo test
cargo package --list
cargo publish --dry-run --registry crates-io
cargo publish --registry crates-io
```

发布后，记得到 crates.io 检查包页面是否正常；如果你也提供预编译二进制，请同步更新 GitHub Releases 里的产物。

## 许可证

本项目基于 [GNU GPL v3.0 或更高版本](./LICENSE) 发布。
