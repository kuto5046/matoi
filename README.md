# matoi

`matoi` is a terminal UI for browsing tmux sessions/windows and jumping to a window quickly.

## Prerequisites

- `tmux` must be installed
- Run inside a tmux client when using focus/switch features

## Install (for users)

### 1. curl (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/kuto5046/matoi/main/install.sh | sh
```

By default, this installs `matoi` to `~/.local/bin`.

Custom install directory example:

```bash
curl -fsSL https://raw.githubusercontent.com/kuto5046/matoi/main/install.sh | MATOI_INSTALL_DIR="$HOME/.cargo/bin" sh
```

### 2. GitHub Releases (manual install)

Download the archive for your OS from GitHub Releases, extract `matoi`, and place it in a directory on your `PATH` (for example `~/.local/bin`).

### 3. crates.io (for Rust users)

```bash
cargo install matoi
```

### 4. From source (current repo)

```bash
cargo install --path .
```

## Usage

```bash
matoi
```

## Keybindings

- `j` / `Down`: next window
- `k` / `Up`: previous window
- `f` / `Enter`: focus selected window
- `r`: refresh
- `q` / `Ctrl-c`: quit

## Distribution (for maintainer)

### Publish to crates.io

1. Update `version` in `Cargo.toml`
2. Run `cargo publish`

After that, users can install with `cargo install matoi`.

### Publish prebuilt binaries via GitHub Releases

This repository includes a GitHub Actions workflow that builds release binaries when you push a version tag.

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow uploads release assets for Linux/macOS to the GitHub Release for that tag.

### curl installer

`install.sh` downloads the latest GitHub Release binary for the current OS/arch and installs it to `~/.local/bin` by default.
