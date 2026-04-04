# dumap 1.1 Release Notes

**dumap** is a cross-platform disk usage visualizer for the terminal. Point it at any directory and get an interactive treemap — either as a native GUI window or a self-contained HTML file.

## What It Does

- **Native GUI** (`dumap view` or just `dumap`) — an [egui](https://github.com/emilk/egui)-powered window with click-to-zoom, breadcrumb navigation, a depth slider, and category-colored file types.
- **HTML export** (`dumap export`) — a self-contained [ECharts](https://echarts.apache.org/) treemap you can open in any browser, share with colleagues, or archive.
- **Fast scanning** — built on [ignore::WalkBuilder](https://docs.rs/ignore) with symlink safety, atomic progress counters, and support for both disk-usage and apparent-size modes.

### Native GUI

![Native GUI viewer](https://raw.githubusercontent.com/jrobhoward/dumap/main/assets/dumap_screencast.gif)

### HTML export

![HTML export](https://raw.githubusercontent.com/jrobhoward/dumap/main/assets/export.png)

## File Categories

Files are color-coded by type in both the GUI and HTML views, making it easy to spot what's consuming space at a glance — large clusters of video files, build artifacts, cached dependencies, etc. Categories include Code, Image, Video, Audio, Archive, Document, Database, Executable, Font, Config, Data, and Other.

## What's New in 1.1

### Facade Crate

You can now install dumap with:

```sh
cargo install dumap
```

Previously, the crate was published as `dumap-cli`, which wasn't discoverable. The `dumap-cli` crate still works and is the underlying implementation — the `dumap` crate is a thin wrapper around it.

### GUI Is the Default

Running `dumap` with no arguments launches the native GUI viewer on your home directory:

```sh
dumap              # GUI on home directory
dumap view ~/src   # GUI on a specific path
dumap export       # HTML export (still available)
```

No subcommand needed for the most common workflow.

### Hidden Files Included by Default

Hidden files and directories (`.git/`, `.cache/`, `.node_modules/`, etc.) are now included in scans by default. These are often the largest consumers of disk space, and excluding them gave an incomplete picture. Use `--exclude-hidden` if you want the old behavior.

## Getting Started

### Install

```sh
cargo install dumap
```

To install without the GUI (HTML export only):

```sh
cargo install dumap --no-default-features
```

### Build from source

```sh
git clone https://github.com/jrobhoward/dumap.git
cd dumap
cargo build --release
# Binary: target/release/dumap
```

### Quick start

```sh
dumap                                          # GUI on home directory
dumap view /usr                                # GUI on /usr
dumap export ~/projects -o usage.html --open   # HTML export, open in browser
```

## Platform Support

| Platform | GUI | HTML export |
|----------|-----|-------------|
| Linux    | Yes | Yes         |
| macOS    | Yes | Yes         |
| Windows  | Yes | Yes         |

## Crate Structure

| Crate | Description |
|-------|-------------|
| [`dumap`](https://crates.io/crates/dumap) | Facade — install this one |
| [`dumap-cli`](https://crates.io/crates/dumap-cli) | CLI binary with `view` and `export` subcommands |
| [`dumap-core`](https://crates.io/crates/dumap-core) | Filesystem scanning, tree construction, HTML generation |
| [`dumap-gui`](https://crates.io/crates/dumap-gui) | Interactive egui treemap viewer |
| [`dumap-layout`](https://crates.io/crates/dumap-layout) | Squarified treemap layout algorithm |

## Links

- [GitHub repository](https://github.com/jrobhoward/dumap)
- [crates.io](https://crates.io/crates/dumap)
- [Changelog](https://github.com/jrobhoward/dumap/blob/main/CHANGELOG.md)

## License

MIT OR Apache-2.0
