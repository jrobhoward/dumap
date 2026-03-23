#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::FileCategory;
use rstest::rstest;
use std::path::Path;

#[rstest]
#[case("main.rs", FileCategory::Code)]
#[case("app.py", FileCategory::Code)]
#[case("index.tsx", FileCategory::Code)]
#[case("Makefile.sh", FileCategory::Code)]
#[case("styles.css", FileCategory::Code)]
fn from_path____code_extensions____returns_code(
    #[case] filename: &str,
    #[case] expected: FileCategory,
) {
    assert_eq!(FileCategory::from_path(Path::new(filename)), expected);
}

#[rstest]
#[case("photo.jpg", FileCategory::Image)]
#[case("icon.PNG", FileCategory::Image)]
#[case("logo.svg", FileCategory::Image)]
#[case("raw.CR2", FileCategory::Image)]
fn from_path____image_extensions____returns_image(
    #[case] filename: &str,
    #[case] expected: FileCategory,
) {
    assert_eq!(FileCategory::from_path(Path::new(filename)), expected);
}

#[rstest]
#[case("movie.mp4", FileCategory::Video)]
#[case("clip.mkv", FileCategory::Video)]
#[case("film.avi", FileCategory::Video)]
fn from_path____video_extensions____returns_video(
    #[case] filename: &str,
    #[case] expected: FileCategory,
) {
    assert_eq!(FileCategory::from_path(Path::new(filename)), expected);
}

#[rstest]
#[case("song.mp3", FileCategory::Audio)]
#[case("track.flac", FileCategory::Audio)]
#[case("sound.wav", FileCategory::Audio)]
fn from_path____audio_extensions____returns_audio(
    #[case] filename: &str,
    #[case] expected: FileCategory,
) {
    assert_eq!(FileCategory::from_path(Path::new(filename)), expected);
}

#[rstest]
#[case("backup.tar.gz", FileCategory::Archive)]
#[case("package.zip", FileCategory::Archive)]
#[case("image.iso", FileCategory::Archive)]
fn from_path____archive_extensions____returns_archive(
    #[case] filename: &str,
    #[case] expected: FileCategory,
) {
    assert_eq!(FileCategory::from_path(Path::new(filename)), expected);
}

#[rstest]
#[case("readme.md", FileCategory::Document)]
#[case("report.pdf", FileCategory::Document)]
#[case("data.csv", FileCategory::Document)]
fn from_path____document_extensions____returns_document(
    #[case] filename: &str,
    #[case] expected: FileCategory,
) {
    assert_eq!(FileCategory::from_path(Path::new(filename)), expected);
}

#[rstest]
#[case("config.toml", FileCategory::Config)]
#[case("settings.json", FileCategory::Config)]
#[case("app.yaml", FileCategory::Config)]
fn from_path____config_extensions____returns_config(
    #[case] filename: &str,
    #[case] expected: FileCategory,
) {
    assert_eq!(FileCategory::from_path(Path::new(filename)), expected);
}

#[test]
fn from_path____no_extension____returns_other() {
    assert_eq!(
        FileCategory::from_path(Path::new("Makefile")),
        FileCategory::Other
    );
}

#[test]
fn from_path____unknown_extension____returns_other() {
    assert_eq!(
        FileCategory::from_path(Path::new("file.xyz123")),
        FileCategory::Other
    );
}

#[test]
fn from_path____case_insensitive____matches() {
    assert_eq!(
        FileCategory::from_path(Path::new("FILE.RS")),
        FileCategory::Code
    );
    assert_eq!(
        FileCategory::from_path(Path::new("photo.JPG")),
        FileCategory::Image
    );
}

#[test]
fn color____all_categories____return_valid_hex() {
    let categories = [
        FileCategory::Code,
        FileCategory::Image,
        FileCategory::Video,
        FileCategory::Audio,
        FileCategory::Archive,
        FileCategory::Document,
        FileCategory::Database,
        FileCategory::Executable,
        FileCategory::Font,
        FileCategory::Config,
        FileCategory::Data,
        FileCategory::Other,
    ];
    for cat in &categories {
        let color = cat.color();
        assert!(color.starts_with('#'), "Color should start with #: {color}");
        assert_eq!(color.len(), 7, "Color should be #rrggbb: {color}");
    }
}

#[test]
fn serialize____category____produces_color_string() {
    let json = serde_json::to_string(&FileCategory::Code).unwrap();
    assert_eq!(json, "\"#569cd6\"");
}
