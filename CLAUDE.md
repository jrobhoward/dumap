# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Project Overview

diskmap is a cross-platform disk usage visualizer that scans directories and generates interactive treemap visualizations. Phase 1 outputs self-contained HTML files using ECharts; Phase 2 will add a native GUI via egui.

**Rust Requirements**: Edition 2024, rust-version 1.88.0

## Workspace Structure

```
diskmap/
├── crates/
│   ├── diskmap-core/     # Tree construction, filesystem scanning, HTML generation
│   └── diskmap-cli/      # CLI binary (scan + top subcommands)
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
cargo run -- scan /path [-o output.html] [--depth N] [--open]
cargo run -- top /path [-n COUNT] [--files]
```

## Key Architecture

### Crate Responsibilities

- **diskmap-core**: `DirNode` tree (HashMap-based), `EChartsNode` JSON conversion, `scan_directory()` using `ignore::WalkBuilder`, `generate_html()` ECharts template, `format_size()`, `find_largest()`
- **diskmap-cli**: `clap` CLI with `scan` (HTML treemap output) and `top` (table of largest entries) subcommands

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

- **Tree data structures**: `crates/diskmap-core/src/tree/node.rs`
- **Filesystem scanning**: `crates/diskmap-core/src/scan/walker.rs`
- **HTML generation**: `crates/diskmap-core/src/html.rs`
- **Error types**: `crates/diskmap-core/src/error.rs`
- **CLI**: `crates/diskmap-cli/src/main.rs`
