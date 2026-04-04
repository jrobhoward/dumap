# dumap

A cross-platform disk usage visualizer that scans directories and generates interactive treemap visualizations.

dumap provides two ways to explore disk usage:

- **`view`** (default) -- opens a native GUI window with a real-time treemap powered by [egui](https://github.com/emilk/egui), featuring click-to-zoom, breadcrumb navigation, and category-colored file types
- **`export`** -- generates a self-contained HTML file with an interactive [ECharts](https://echarts.apache.org/) treemap you can open in any browser

### Native GUI (`dumap` / `dumap view`)

![Native GUI viewer](https://raw.githubusercontent.com/jrobhoward/dumap/main/assets/dumap_screencast.gif)

### HTML export (`dumap export`)

![HTML export](https://raw.githubusercontent.com/jrobhoward/dumap/main/assets/export.png)

## Install

### From crates.io

```sh
cargo install dumap
```

This installs the `dumap` binary with the native GUI enabled by default.

To install without the GUI (HTML export only):

```sh
cargo install dumap-cli --no-default-features
```

### From source

```sh
git clone https://github.com/jrobhoward/dumap.git
cd dumap
cargo build --release
```

The binary will be at `target/release/dumap`.

## Usage

Running `dumap` with no arguments launches the GUI viewer on your home directory:

```sh
dumap
```

### Interactive GUI viewer

```sh
dumap view ~/projects
```

Scans the directory and opens a native window with a zoomable treemap. Click a directory to zoom in, right-click or scroll down to zoom out. A depth slider and breadcrumb bar are provided for navigation.

### HTML export

```sh
dumap export ~/projects -o disk-usage.html --open
```

Scans the directory and writes a self-contained HTML treemap. The `--open` flag opens it in your default browser.

### Options

#### Common options (both `export` and `view`)

| Option | Description |
|--------|-------------|
| `[PATH]` | Directory to scan. Defaults to your home directory if omitted. |
| `--exclude-hidden` | Exclude hidden files and directories from the scan. By default, all files are included -- hidden directories like `.git/`, `.cache/`, and `.config/` are scanned along with everything else. |
| `--apparent-size` | Report logical file sizes instead of actual disk usage. By default, dumap uses disk usage (block-allocated size on Unix), which reflects how much space files actually consume on disk. Apparent size is the byte count written to the file, which can be smaller (sparse files) or simply different due to filesystem block alignment. |

#### `export` only

| Option | Default | Description |
|--------|---------|-------------|
| `-o`, `--output <FILE>` | `dumap.html` | Path for the output HTML file. The file is self-contained with no external dependencies. |
| `-d`, `--depth <N>` | `3` | Number of visible depth levels in the treemap before requiring a click to drill deeper. Higher values show more of the tree at once but can be visually dense. |
| `--max-scan-depth <N>` | unlimited | Maximum directory depth to traverse during scanning. A value of `1` scans only the immediate children of the target directory. Useful for limiting scan time on very deep trees. |
| `--open` | off | Open the generated HTML file in your default browser after writing it. |

### Examples

```sh
# Launch GUI on home directory (default)
dumap

# Launch GUI on a specific directory
dumap view ~/projects/myapp

# Export your home directory with default settings
dumap export

# Export /usr with 5 depth levels, open in browser
dumap export /usr -d 5 --open

# View a project directory, excluding hidden files
dumap view ~/projects/myapp --exclude-hidden

# Export with apparent sizes instead of disk usage
dumap export /var/log --apparent-size -o logs.html
```

## File categories

Files are color-coded by type in both the GUI and HTML views:

| Category   | Extensions (sample)                        |
|------------|--------------------------------------------|
| Code       | rs, py, js, ts, go, c, cpp, java, sh, html |
| Image      | png, jpg, svg, webp, heic, psd             |
| Video      | mp4, mkv, avi, mov, webm                   |
| Audio      | mp3, flac, wav, aac, ogg, opus             |
| Archive    | zip, tar, gz, 7z, rar, iso, deb            |
| Document   | pdf, doc, docx, txt, md, csv, epub         |
| Database   | db, sqlite, sqlite3, mdb                   |
| Executable | exe, dll, so, dylib, class, pyc            |
| Font       | ttf, otf, woff, woff2                      |
| Config     | json, yaml, toml, ini, xml, plist          |
| Data       | bin, dat, parquet, arrow, hdf5             |
| Other      | everything else                            |

## Requirements

- Rust 1.88.0 or later (edition 2024)
- For the GUI: platform support for [eframe/egui](https://github.com/emilk/egui) (Linux/macOS/Windows)

## License

MIT OR Apache-2.0
