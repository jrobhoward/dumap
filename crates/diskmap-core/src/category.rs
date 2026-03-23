use serde::{Serialize, Serializer};
use std::path::Path;

/// File type categories for color-coding the treemap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileCategory {
    Code,
    Image,
    Video,
    Audio,
    Archive,
    Document,
    Database,
    Executable,
    Font,
    Config,
    Data,
    Other,
}

impl FileCategory {
    /// Determine category from a file path (based on extension).
    pub fn from_path(path: &Path) -> Self {
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_ascii_lowercase(),
            None => return Self::Other,
        };

        match ext.as_str() {
            // Code
            "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "go" | "c" | "cpp" | "cc" | "h" | "hpp"
            | "java" | "kt" | "swift" | "rb" | "php" | "cs" | "scala" | "lua" | "r" | "m"
            | "mm" | "pl" | "pm" | "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd"
            | "zig" | "asm" | "s" | "v" | "sv" | "vhd" | "vhdl" | "elm" | "ex" | "exs" | "erl"
            | "hs" | "ml" | "mli" | "clj" | "cljs" | "lisp" | "el" | "dart" | "vue" | "svelte"
            | "css" | "scss" | "sass" | "less" | "html" | "htm" | "sql" | "graphql" | "gql"
            | "proto" | "thrift" | "wasm" | "wat" => Self::Code,

            // Images
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "ico" | "webp" | "tiff" | "tif"
            | "psd" | "ai" | "eps" | "raw" | "cr2" | "nef" | "arw" | "dng" | "heic" | "heif"
            | "avif" | "jxl" => Self::Image,

            // Video ("ts" omitted — ambiguous with TypeScript, matched as Code above)
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg"
            | "3gp" | "ogv" => Self::Video,

            // Audio
            "mp3" | "flac" | "wav" | "aac" | "ogg" | "wma" | "m4a" | "opus" | "aiff" | "ape"
            | "alac" | "mid" | "midi" => Self::Audio,

            // Archives
            "zip" | "tar" | "gz" | "bz2" | "xz" | "zst" | "lz4" | "7z" | "rar" | "cab" | "iso"
            | "dmg" | "deb" | "rpm" | "pkg" | "msi" | "appimage" | "snap" | "flatpak" | "tgz"
            | "tbz2" | "txz" => Self::Archive,

            // Documents
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp"
            | "rtf" | "txt" | "md" | "rst" | "tex" | "latex" | "epub" | "mobi" | "pages"
            | "numbers" | "key" | "csv" => Self::Document,

            // Databases
            "db" | "sqlite" | "sqlite3" | "mdb" | "accdb" | "dbf" | "ldb" => Self::Database,

            // Executables / binaries
            "exe" | "dll" | "so" | "dylib" | "a" | "lib" | "o" | "obj" | "class" | "pyc"
            | "pyo" | "elc" | "beam" => Self::Executable,

            // Fonts
            "ttf" | "otf" | "woff" | "woff2" | "eot" => Self::Font,

            // Config
            "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "env" | "xml" | "plist"
            | "properties" | "reg" => Self::Config,

            // Data / serialized
            "bin" | "dat" | "parquet" | "arrow" | "avro" | "msgpack" | "cbor" | "pb" | "npy"
            | "npz" | "hdf5" | "h5" | "nc" | "fits" => Self::Data,

            _ => Self::Other,
        }
    }

    /// All category variants, for iteration (e.g. legends).
    pub const ALL: &[FileCategory] = &[
        Self::Code,
        Self::Image,
        Self::Video,
        Self::Audio,
        Self::Archive,
        Self::Document,
        Self::Database,
        Self::Executable,
        Self::Font,
        Self::Config,
        Self::Data,
        Self::Other,
    ];

    /// RGB color for this category. Single source of truth for both
    /// the HTML hex string and the GUI Color32.
    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Code => (86, 156, 214),      // blue
            Self::Image => (206, 145, 52),     // amber
            Self::Video => (214, 86, 86),      // red
            Self::Audio => (156, 86, 214),     // purple
            Self::Archive => (86, 214, 156),   // teal
            Self::Document => (214, 206, 86),  // yellow
            Self::Database => (214, 130, 86),  // orange
            Self::Executable => (180, 86, 86), // dark red
            Self::Font => (150, 150, 150),     // gray
            Self::Config => (120, 180, 120),   // green
            Self::Data => (86, 136, 214),      // steel blue
            Self::Other => (100, 100, 120),    // dark gray-blue
        }
    }

    /// CSS color for this category (hex string), derived from `rgb()`.
    pub fn color(&self) -> String {
        let (r, g, b) = self.rgb();
        format!("#{r:02x}{g:02x}{b:02x}")
    }

    /// Short label for the category.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Code => "Code",
            Self::Image => "Image",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Archive => "Archive",
            Self::Document => "Document",
            Self::Database => "Database",
            Self::Executable => "Executable",
            Self::Font => "Font",
            Self::Config => "Config",
            Self::Data => "Data",
            Self::Other => "Other",
        }
    }
}

/// Serialize FileCategory as its color string for ECharts.
impl Serialize for FileCategory {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.color())
    }
}

#[cfg(test)]
#[path = "category_tests.rs"]
mod category_tests;
