use clap::{Parser, Subcommand};
use diskmap_core::scan::{ScanConfig, ScanProgress, find_largest, format_size, scan_directory};
use diskmap_core::tree::EChartsNode;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Instant;

#[derive(Debug, Parser)]
#[command(
    name = "diskmap",
    version,
    about = "Visualize disk usage with interactive treemaps"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Scan a directory and generate an interactive HTML treemap
    Scan {
        /// Directory to scan (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Output HTML file (default: diskmap.html)
        #[arg(short, long, default_value = "diskmap.html")]
        output: PathBuf,

        /// Visible depth levels in treemap (default: 3)
        #[arg(short, long, default_value = "3")]
        depth: u16,

        /// Use apparent file sizes instead of disk usage
        #[arg(long)]
        apparent_size: bool,

        /// Include hidden files and directories
        #[arg(long)]
        include_hidden: bool,

        /// Maximum scan depth (unlimited if not specified)
        #[arg(long)]
        max_scan_depth: Option<usize>,

        /// Open the generated HTML in the default browser
        #[arg(long)]
        open: bool,
    },

    /// Open interactive GUI treemap viewer
    #[cfg(feature = "gui")]
    View {
        /// Directory to scan (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Use apparent file sizes instead of disk usage
        #[arg(long)]
        apparent_size: bool,

        /// Include hidden files and directories
        #[arg(long)]
        include_hidden: bool,
    },

    /// Show the largest files or directories (no GUI)
    Top {
        /// Directory to scan (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Number of entries to show
        #[arg(short = 'n', long, default_value = "20")]
        count: usize,

        /// Show files instead of directories
        #[arg(long)]
        files: bool,

        /// Use apparent file sizes instead of disk usage
        #[arg(long)]
        apparent_size: bool,

        /// Include hidden files and directories
        #[arg(long)]
        include_hidden: bool,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    let result = match args.command {
        Command::Scan {
            path,
            output,
            depth,
            apparent_size,
            include_hidden,
            max_scan_depth,
            open,
        } => run_scan(
            path,
            output,
            depth,
            apparent_size,
            include_hidden,
            max_scan_depth,
            open,
        ),
        #[cfg(feature = "gui")]
        Command::View {
            path,
            apparent_size,
            include_hidden,
        } => run_view(path, apparent_size, include_hidden),
        Command::Top {
            path,
            count,
            files,
            apparent_size,
            include_hidden,
        } => run_top(path, count, files, apparent_size, include_hidden),
    };

    if let Err(err) = result {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run_scan(
    path: PathBuf,
    output: PathBuf,
    depth: u16,
    apparent_size: bool,
    include_hidden: bool,
    max_scan_depth: Option<usize>,
    open_browser: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = ScanConfig {
        root: path.clone(),
        follow_links: false,
        include_hidden,
        max_depth: max_scan_depth,
        apparent_size,
    };
    let progress = ScanProgress::new();

    eprintln!("Scanning {}...", path.display());
    let start = Instant::now();

    let tree = scan_directory(&config, &progress)?;

    let elapsed = start.elapsed();
    let file_count = progress.files_found.load(Ordering::Relaxed);
    let total_size = tree.total_size();

    eprintln!(
        "Scanned {} files ({}) in {:.1}s",
        file_count,
        format_size(total_size),
        elapsed.as_secs_f64(),
    );

    // Convert to ECharts JSON — root children become top-level array
    let echarts_children: Vec<EChartsNode> = tree
        .children
        .iter()
        .map(|(name, node)| node.to_echarts(name))
        .collect();

    let tree_json = serde_json::to_string(&echarts_children)?;

    let scan_path = path.canonicalize().unwrap_or(path).display().to_string();
    let html = diskmap_core::generate_html(
        &tree_json,
        total_size,
        file_count as usize,
        &scan_path,
        depth,
    );

    std::fs::write(&output, &html)?;
    eprintln!("Treemap written to {}", output.display());

    if open_browser {
        let abs_output = std::fs::canonicalize(&output)?;
        let url = format!("file://{}", abs_output.display());
        if let Err(err) = open::that(&url) {
            eprintln!("Failed to open browser: {err}");
        }
    }

    Ok(())
}

fn run_top(
    path: PathBuf,
    count: usize,
    files_only: bool,
    apparent_size: bool,
    include_hidden: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = ScanConfig {
        root: path.clone(),
        follow_links: false,
        include_hidden,
        max_depth: None,
        apparent_size,
    };
    let progress = ScanProgress::new();

    eprintln!("Scanning {}...", path.display());
    let start = Instant::now();

    let tree = scan_directory(&config, &progress)?;

    let elapsed = start.elapsed();
    let total_files = progress.files_found.load(Ordering::Relaxed);
    let total_size = tree.total_size();

    eprintln!(
        "Scanned {} files ({}) in {:.1}s\n",
        total_files,
        format_size(total_size),
        elapsed.as_secs_f64(),
    );

    let kind = if files_only { "files" } else { "directories" };
    println!("Top {} {} by size:\n", count, kind);

    let largest = find_largest(&tree, &path, count, files_only);

    // Column widths
    let max_size_width = largest
        .first()
        .map(|(_, size)| format_size(*size).len())
        .unwrap_or(4);

    for (i, (entry_path, size)) in largest.iter().enumerate() {
        let size_str = format_size(*size);
        println!(
            "{:>3}. {:>width$}  {}",
            i + 1,
            size_str,
            entry_path.display(),
            width = max_size_width,
        );
    }

    Ok(())
}

#[cfg(feature = "gui")]
fn run_view(
    path: PathBuf,
    apparent_size: bool,
    include_hidden: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let scan_config = ScanConfig {
        root: path.clone(),
        follow_links: false,
        include_hidden,
        max_depth: None,
        apparent_size,
    };

    let title = format!("diskmap — {}", path.display());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title(&title),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(move |_cc| Ok(Box::new(diskmap_gui::DiskmapApp::new(scan_config)))),
    )
    .map_err(|e| format!("GUI error: {e}").into())
}
