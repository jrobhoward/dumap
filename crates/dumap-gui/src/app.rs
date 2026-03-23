use crate::color::{category_color, lighten, node_color};
use crate::navigation::NavigationModel;
use dumap_core::category::FileCategory;
use dumap_core::scan::{ScanConfig, ScanProgress, format_size};
use dumap_core::tree::{FileTree, NodeId, NodeKind};
use dumap_layout::{LayoutConfig, TreemapLayout, squarify_layout};
use egui::{Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

/// Height of the directory header label in the treemap (pixels).
const DIR_HEADER_HEIGHT: f64 = 20.0;

/// Main application state.
pub struct DumapApp {
    /// The scanned file tree (None until scan completes).
    tree: Option<Arc<FileTree>>,
    /// Cached layout (recomputed on resize, zoom, or depth change).
    layout: Option<TreemapLayout>,
    /// Last panel size used for layout.
    last_panel_size: Option<Vec2>,
    /// Navigation state.
    navigation: Option<NavigationModel>,
    /// Currently hovered node.
    hovered: Option<NodeId>,
    /// Context menu target node.
    context_menu_node: Option<NodeId>,
    /// Scan progress (shared with background thread).
    scan_progress: Option<Arc<ScanProgress>>,
    /// Receive completed FileTree from background scan.
    scan_receiver: Option<std::sync::mpsc::Receiver<Result<FileTree, String>>>,
    /// Scan configuration.
    scan_config: ScanConfig,
    /// Scan timing.
    scan_start: Option<Instant>,
    scan_duration: Option<f64>,
}

impl DumapApp {
    /// Create a new app that will scan the given path.
    pub fn new(scan_config: ScanConfig) -> Self {
        Self {
            tree: None,
            layout: None,
            last_panel_size: None,
            navigation: None,
            hovered: None,
            context_menu_node: None,
            scan_progress: None,
            scan_receiver: None,
            scan_config,
            scan_start: None,
            scan_duration: None,
        }
    }

    /// Start the scan in a background thread.
    fn start_scan(&mut self) {
        let config = ScanConfig {
            root: self.scan_config.root.clone(),
            follow_links: self.scan_config.follow_links,
            include_hidden: self.scan_config.include_hidden,
            max_depth: self.scan_config.max_depth,
            apparent_size: self.scan_config.apparent_size,
        };

        let progress = Arc::new(ScanProgress::new());
        self.scan_progress = Some(Arc::clone(&progress));
        self.scan_start = Some(Instant::now());

        let (tx, rx) = std::sync::mpsc::channel();
        self.scan_receiver = Some(rx);

        std::thread::spawn(move || {
            let result = dumap_core::scan_directory(&config, &progress);
            let _ = tx.send(result.map_err(|e| e.to_string()).map(|dir_node| {
                let scan_root = config.root.clone();
                dumap_core::build_file_tree(&dir_node, scan_root)
            }));
        });
    }

    /// Check for completed scan.
    fn poll_scan_result(&mut self) {
        let receiver = match &self.scan_receiver {
            Some(rx) => rx,
            None => return,
        };

        if let Ok(result) = receiver.try_recv() {
            self.scan_duration = self.scan_start.map(|s| s.elapsed().as_secs_f64());
            self.scan_receiver = None;

            match result {
                Ok(tree) => {
                    let root = tree.root();
                    self.tree = Some(Arc::new(tree));
                    self.navigation = Some(NavigationModel::new(root));
                    self.invalidate_layout();
                }
                Err(err) => {
                    tracing::error!("Scan failed: {err}");
                }
            }
            self.scan_progress = None;
        }
    }

    /// Invalidate the cached layout (forces recomputation next frame).
    fn invalidate_layout(&mut self) {
        self.layout = None;
        self.last_panel_size = None;
    }

    /// Recompute layout if needed.
    fn ensure_layout(&mut self, panel_size: Vec2) {
        let size_changed = !self
            .last_panel_size
            .is_some_and(|s| (s - panel_size).length() <= 0.5);

        if self.layout.is_some() && !size_changed {
            return;
        }

        let tree = match &self.tree {
            Some(t) => t,
            None => return,
        };
        let nav = match &self.navigation {
            Some(n) => n,
            None => return,
        };

        let bounds =
            dumap_layout::LayoutRect::new(0.0, 0.0, panel_size.x as f64, panel_size.y as f64);
        let config = LayoutConfig {
            max_depth: nav.max_display_depth,
            min_rect_size: 2.0,
            padding: 3.0,
            header_height: DIR_HEADER_HEIGHT,
        };

        self.layout = Some(squarify_layout(tree, nav.visible_root(), bounds, &config));
        self.last_panel_size = Some(panel_size);
    }

    /// Render the toolbar (breadcrumb + depth slider + legend).
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        let mut zoom_to: Option<usize> = None;
        let mut new_depth: Option<u16> = None;

        ui.horizontal(|ui| {
            // Depth slider (left side)
            if let Some(nav) = &self.navigation {
                ui.label("Depth:");
                let mut depth = nav.max_display_depth as i32;
                if ui.add(egui::Slider::new(&mut depth, 1..=8)).changed() {
                    new_depth = Some(depth as u16);
                }
            }

            ui.add_space(20.0);

            // Breadcrumb navigation (right of depth slider)
            if let (Some(tree), Some(nav)) = (&self.tree, &self.navigation) {
                let crumbs: Vec<(usize, NodeId, String)> = nav
                    .breadcrumb()
                    .iter()
                    .enumerate()
                    .map(|(i, &id)| (i, id, tree.node(id).name.clone()))
                    .collect();

                for (i, _id, name) in &crumbs {
                    if *i > 0 {
                        ui.label("›");
                    }
                    if ui.small_button(name).clicked() {
                        zoom_to = Some(*i);
                    }
                }
            }
        });

        // Color legend
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            for &cat in FileCategory::ALL {
                let color = category_color(cat);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), Sense::hover());
                ui.painter().rect_filled(rect, CornerRadius::same(2), color);
                ui.label(
                    egui::RichText::new(cat.label())
                        .size(12.0)
                        .color(Color32::GRAY),
                );
            }
        });

        // Apply deferred mutations
        if let Some(level) = zoom_to {
            if let Some(nav) = &mut self.navigation {
                nav.zoom_to_level(level);
            }
            self.invalidate_layout();
        }
        if let Some(depth) = new_depth {
            if let Some(nav) = &mut self.navigation {
                nav.max_display_depth = depth;
            }
            self.invalidate_layout();
        }
    }

    /// Render the info panel showing details of the hovered item.
    fn render_info_panel(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if let Some(progress) = &self.scan_progress {
                let files = progress.files_found.load(Ordering::Relaxed);
                let bytes = progress.bytes_found.load(Ordering::Relaxed);
                ui.spinner();
                ui.label(format!(
                    "Scanning... {} files ({})",
                    files,
                    format_size(bytes)
                ));
                return;
            }

            let Some(tree) = &self.tree else { return };

            // Show hovered item details, or fall back to root summary
            if let Some(hovered_id) = self.hovered {
                let node = tree.node(hovered_id);
                let path = tree.path(hovered_id);
                let scan_root = tree.scan_root();
                let abs_path = scan_root.join(&path);

                ui.label(
                    egui::RichText::new(abs_path.display().to_string())
                        .strong()
                        .monospace(),
                );
                ui.separator();
                ui.label(format_size(node.total_size));

                match &node.kind {
                    NodeKind::File { category, .. } => {
                        ui.separator();
                        ui.label(category.label());
                    }
                    NodeKind::Directory { children } => {
                        ui.separator();
                        ui.label(format!("{} files", node.file_count));
                        ui.separator();
                        ui.label(format!("{} children", children.len()));
                    }
                }
            } else {
                let root = tree.node(tree.root());
                ui.label(format!(
                    "{} files  ·  {}",
                    root.file_count,
                    format_size(root.total_size),
                ));
                if let Some(dur) = self.scan_duration {
                    ui.label(format!("  ·  scanned in {dur:.1}s"));
                }
            }
        });
    }

    /// Render the treemap.
    fn render_treemap(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let (response, painter) =
            ui.allocate_painter(available, Sense::click_and_drag().union(Sense::hover()));
        let origin = response.rect.min;

        self.ensure_layout(available);

        let tree = match &self.tree {
            Some(t) => Arc::clone(t),
            None => return,
        };
        let layout = match &self.layout {
            Some(l) => l,
            None => return,
        };

        // Detect hover
        let hover_pos = response.hover_pos();
        let hovered_entry = hover_pos.and_then(|pos| {
            let local_x = (pos.x - origin.x) as f64;
            let local_y = (pos.y - origin.y) as f64;
            layout.hit_test(local_x, local_y)
        });
        self.hovered = hovered_entry.map(|e| e.node_id);

        // Render all entries
        for entry in layout.entries() {
            let egui_rect = to_egui_rect(&entry.rect, origin);

            if egui_rect.width() < 1.0 || egui_rect.height() < 1.0 {
                continue;
            }

            let node = tree.node(entry.node_id);
            let mut fill = node_color(node);

            // Highlight hovered
            if Some(entry.node_id) == self.hovered {
                fill = lighten(fill, 0.3);
            }

            let is_dir = matches!(node.kind, NodeKind::Directory { .. });

            // Draw filled rectangle
            painter.rect_filled(egui_rect, CornerRadius::ZERO, fill);

            // Border
            let border_width = if is_dir { 1.5 } else { 0.5 };
            let border_color = Color32::from_rgba_premultiplied(20, 20, 30, 200);
            painter.rect_stroke(
                egui_rect,
                CornerRadius::ZERO,
                Stroke::new(border_width, border_color),
                StrokeKind::Outside,
            );

            // Directory header label (drawn inside the padding area at the top)
            if is_dir && egui_rect.width() > 40.0 && egui_rect.height() > DIR_HEADER_HEIGHT as f32 {
                let header_rect = Rect::from_min_size(
                    egui_rect.left_top(),
                    Vec2::new(egui_rect.width(), DIR_HEADER_HEIGHT as f32),
                );
                // Dark header background
                painter.rect_filled(
                    header_rect,
                    CornerRadius::ZERO,
                    Color32::from_rgba_premultiplied(16, 16, 28, 200),
                );
                // Header text: directory name + size
                let header_text = format!("{}  ({})", node.name, format_size(node.total_size));
                painter.text(
                    header_rect.left_center() + Vec2::new(4.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    header_text,
                    FontId::proportional(12.0),
                    Color32::from_rgb(220, 220, 230),
                );
            } else if !is_dir && egui_rect.width() > 60.0 && egui_rect.height() > 18.0 {
                // File label
                let label = if node.total_size > 0 {
                    format!("{}\n{}", node.name, format_size(node.total_size))
                } else {
                    node.name.clone()
                };
                let text_rect = egui_rect.shrink(4.0);
                painter.text(
                    text_rect.left_top(),
                    egui::Align2::LEFT_TOP,
                    label,
                    FontId::proportional(11.0),
                    Color32::WHITE,
                );
            }
        }

        // Scroll wheel to zoom — one level per scroll event.
        // Use raw scroll_delta (not smooth) and only respond to the first
        // event per frame to avoid jumping multiple levels.
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta > 0.0 {
            // Scroll up = zoom into hovered directory
            if let Some(hovered_id) = self.hovered
                && matches!(tree.node(hovered_id).kind, NodeKind::Directory { .. })
                && let Some(nav) = &mut self.navigation
            {
                nav.zoom_into(hovered_id, &tree);
                self.layout = None;
                self.last_panel_size = None;
            }
        } else if scroll_delta < 0.0 {
            // Scroll down = zoom out
            if let Some(nav) = &mut self.navigation
                && nav.can_zoom_out()
            {
                nav.zoom_out();
                self.layout = None;
                self.last_panel_size = None;
            }
        }

        // Left-click to zoom into directory
        if response.clicked()
            && let Some(hovered_id) = self.hovered
            && matches!(tree.node(hovered_id).kind, NodeKind::Directory { .. })
            && let Some(nav) = &mut self.navigation
        {
            nav.zoom_into(hovered_id, &tree);
            self.layout = None;
            self.last_panel_size = None;
        }

        // Right-click context menu
        if response.secondary_clicked() {
            self.context_menu_node = self.hovered;
        }

        // Draw context menu if active
        if self.context_menu_node.is_some() {
            let mut close_menu = false;
            let ctx_node = self.context_menu_node;

            response.clone().context_menu(|ui| {
                if let Some(node_id) = ctx_node {
                    let node = tree.node(node_id);
                    let path = tree.path(node_id);
                    let scan_root = tree.scan_root();
                    let abs_path = scan_root.join(&path);

                    ui.label(egui::RichText::new(&node.name).strong().size(12.0));
                    ui.separator();

                    if ui.button("Copy absolute path").clicked() {
                        ui.ctx().copy_text(abs_path.display().to_string());
                        close_menu = true;
                    }

                    if let Some(parent_path) = abs_path.parent()
                        && ui.button("Copy parent folder path").clicked()
                    {
                        ui.ctx().copy_text(parent_path.display().to_string());
                        close_menu = true;
                    }
                }
            });

            if close_menu {
                self.context_menu_node = None;
            }
        }
    }
}

impl eframe::App for DumapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Start scan on first frame
        if self.tree.is_none() && self.scan_receiver.is_none() {
            self.start_scan();
        }

        // Check for completed scan
        self.poll_scan_result();

        // Top panel: toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.render_toolbar(ui);
        });

        // Bottom panel: info about hovered item (replaces tooltip)
        egui::TopBottomPanel::bottom("info_panel").show(ctx, |ui| {
            self.render_info_panel(ui);
        });

        // Central panel: treemap
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(26, 26, 46)))
            .show(ctx, |ui| {
                self.render_treemap(ui);
            });

        // Request repaint during scan for progress updates
        if self.scan_progress.is_some() {
            ctx.request_repaint();
        }
    }
}

/// Convert a layout rect to an egui Rect, offset by the panel origin.
fn to_egui_rect(r: &dumap_layout::LayoutRect, origin: Pos2) -> Rect {
    Rect::from_min_size(
        Pos2::new(origin.x + r.x as f32, origin.y + r.y as f32),
        Vec2::new(r.w as f32, r.h as f32),
    )
}
