# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2026-03-23

### Added

- Interactive native GUI viewer (`dumap view`) with egui — click-to-zoom,
  right-click context menu, breadcrumb navigation, depth slider, scroll-wheel
  zoom, and category-colored file types.
- HTML treemap export (`dumap export`) — self-contained ECharts file with
  drill-down, dark theme, and responsive resize.
- Filesystem scanning via `ignore::WalkBuilder` with symlink safety, hidden
  file filtering, apparent-size mode, and cancellation support.
- Arena-allocated `FileTree` for cache-friendly layout traversal.
- Squarified treemap layout algorithm (Bruls, Huizing, van Wijk 2000) with
  configurable depth, padding, and header height.
- File category classification (Code, Image, Video, Audio, Archive, Document,
  Database, Executable, Font, Config, Data, Other) with shared RGB palette
  across GUI and HTML output.
- Cross-platform support (Linux, macOS, Windows).
- `cargo-deny` configuration for license and advisory auditing.
- GitHub Actions CI (build, test, clippy, fmt, cargo-deny) across
  Linux/macOS/Windows.

### Fixed

- Absolute paths no longer duplicate the scan root component
  (e.g. `C:\Users\Users\...` is now correctly `C:\Users\...`).
- HTML legend now includes all file categories (previously missing Font)
  and is generated dynamically from `FileCategory::ALL` to stay in sync.
