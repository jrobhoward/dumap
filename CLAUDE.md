# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

dumap is a cross-platform disk usage visualizer that scans directories and generates interactive treemap visualizations. It has two output modes: self-contained HTML files using ECharts (`export`) and a native GUI via egui (`view`).

**Rust Requirements**: Edition 2024, rust-version 1.88.0

## Workspace Structure

```
dumap/
├── crates/
│   ├── dumap/          # Facade crate (cargo install dumap)
│   ├── dumap-core/     # Tree construction, filesystem scanning, HTML generation, FileTree arena
│   ├── dumap-layout/   # Squarified treemap layout algorithm (pure geometry)
│   ├── dumap-gui/      # Interactive egui treemap viewer
│   └── dumap-cli/      # CLI library + binary (export, view subcommands)
└── clippy.toml           # allow-unwrap-in-tests = true
```

## Build Commands

**Linux prerequisites** (needed for the GUI/egui crate):
```bash
sudo apt-get install -y libxkbcommon-dev libgtk-3-dev
```

```bash
# Build
cargo build
cargo build --release

# Run all tests
cargo test --workspace

# Run a single test (by name substring)
cargo test -p dumap-core test_name

# Run tests for a single crate
cargo test -p dumap-layout

# Code quality
cargo clippy --workspace --tests
cargo fmt --all
cargo deny check            # license, advisory, ban, source checks (run in CI)

# Run
cargo run -- [/path]                                   # Interactive GUI treemap (default)
cargo run -- view [/path]                              # Explicit GUI treemap
cargo run -- export [/path] [-o output.html] [--open]  # HTML export
```

## Key Architecture

### Crate Responsibilities

- **dumap-core**: `DirNode` tree (HashMap-based), `FileTree` (arena-allocated), `EChartsNode` JSON conversion, `scan_directory()` using `ignore::WalkBuilder`, `generate_html()` ECharts template, `format_size()`, `FileCategory` for extension-based type classification
- **dumap-layout**: Squarified treemap algorithm (`squarify_layout()`), `LayoutRect`, `LayoutEntry`, `TreemapLayout` with hit testing. Pure geometry, no I/O.
- **dumap-gui**: egui/eframe interactive viewer with background scan, category-colored treemap, click-to-zoom, right-click-to-zoom-out, breadcrumb navigation, tooltips, depth slider
- **dumap**: Facade crate — thin wrapper so `cargo install dumap` works. Calls `dumap_cli::run()`.
- **dumap-cli**: `clap` CLI with `view` (launch GUI, default) and `export` (HTML treemap output) subcommands. Exposes `lib.rs` with `run()` entry point. Path defaults to home directory if omitted.

### Scanning

Uses `ignore::WalkBuilder` with:
- `.follow_links(false)` — never follow symlinks
- `.hidden(false)` — include hidden files when requested
- `.parents(false)` — don't walk parent directories
- `symlink_metadata()` — read metadata without following links
- Atomic progress counters (`ScanProgress`) for UI feedback

### Tree Construction

Files are inserted into a `DirNode` HashMap tree by splitting paths into components. `total_size()` aggregates recursively. `to_echarts()` converts to JSON with:
- Single-child directory chain collapsing (`a/b/c` → one node)
- Children sorted by size descending

### HTML Output

Self-contained HTML with ECharts CDN. Features: squarified treemap, `leafDepth: 3` drill-down, breadcrumb navigation, tooltips, hierarchical borders, dark theme, responsive resize.

## Conventions

### Error Handling
- `thiserror` for all error types
- `?` operator in production code — no `.unwrap()` or `.expect()`
- Exception: test modules may use `.unwrap()`/`.expect()`

### Clippy Lints (workspace-wide)
- `unwrap_used = "warn"`
- `expect_used = "warn"`
- `cognitive_complexity = "warn"`
- CI sets `RUSTFLAGS: -Dwarnings`, so all warnings become errors in CI

### Testing
- Tests in separate `*_tests.rs` files
- Test naming: `function____condition____result` (4 underscores)
- Test file headers: `#![allow(clippy::unwrap_used)]`, `#![allow(clippy::expect_used)]`, `#![allow(non_snake_case)]`
- `TempDir` RAII for filesystem tests
- `rstest` for parameterized tests, `proptest` for property-based tests

### Code Style
- Prefer imports over fully-qualified paths
- `parking_lot::RwLock` over `std::sync::Mutex`
- Platform-specific code via `#[cfg(unix)]` / `#[cfg(not(unix))]`

## Definition of Done

- `cargo test --workspace` passes with zero failures
- `cargo clippy --workspace --tests` is clean (zero warnings)
- `cargo fmt --all -- --check` passes
- No `.unwrap()` or `.expect()` in production code

## Important Files

- **Tree data structures**: `crates/dumap-core/src/tree/node.rs` (DirNode, EChartsNode)
- **Arena tree**: `crates/dumap-core/src/tree/arena.rs` (FileTree, NodeId, TreeNode)
- **File categories**: `crates/dumap-core/src/category.rs` (FileCategory, extension mapping)
- **Filesystem scanning**: `crates/dumap-core/src/scan/walker.rs`
- **HTML generation**: `crates/dumap-core/src/html.rs`
- **Squarified layout**: `crates/dumap-layout/src/squarify.rs`
- **Layout rectangle**: `crates/dumap-layout/src/rect.rs`
- **GUI app**: `crates/dumap-gui/src/app.rs`
- **Navigation model**: `crates/dumap-gui/src/navigation.rs`
- **CLI**: `crates/dumap-cli/src/main.rs`
