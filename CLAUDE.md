# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

dumap is a cross-platform disk usage visualizer that scans directories and generates interactive treemap visualizations. Phase 1 outputs self-contained HTML files using ECharts; Phase 2 will add a native GUI via egui.

**Rust Requirements**: Edition 2024, rust-version 1.88.0

## Workspace Structure

```
dumap/
├── crates/
│   ├── dumap-core/     # Tree construction, filesystem scanning, HTML generation, FileTree arena
│   ├── dumap-layout/   # Squarified treemap layout algorithm (pure geometry)
│   ├── dumap-gui/      # Interactive egui treemap viewer
│   └── dumap-cli/      # CLI binary (scan, view, top subcommands)
└── clippy.toml           # allow-unwrap-in-tests = true
```

## Build Commands

```bash
# Build
cargo build
cargo build --release

# Run all tests
cargo test --workspace

# Code quality
cargo clippy --workspace --tests
cargo fmt --all

# Run
cargo run -- view /path                            # Interactive GUI treemap
cargo run -- scan /path [-o output.html] [--open]  # HTML export
cargo run -- top /path [-n COUNT] [--files]         # CLI table output
```

## Key Architecture

### Crate Responsibilities

- **dumap-core**: `DirNode` tree (HashMap-based), `FileTree` (arena-allocated), `EChartsNode` JSON conversion, `scan_directory()` using `ignore::WalkBuilder`, `generate_html()` ECharts template, `format_size()`, `find_largest()`, `FileCategory` for extension-based type classification
- **dumap-layout**: Squarified treemap algorithm (`squarify_layout()`), `LayoutRect`, `LayoutEntry`, `TreemapLayout` with hit testing. Pure geometry, no I/O.
- **dumap-gui**: egui/eframe interactive viewer with background scan, category-colored treemap, click-to-zoom, right-click-to-zoom-out, breadcrumb navigation, tooltips, depth slider
- **dumap-cli**: `clap` CLI with `view` (launch GUI), `scan` (HTML treemap output), and `top` (table of largest entries) subcommands

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
