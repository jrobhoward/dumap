#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::generate_html;
use crate::category::FileCategory;

#[test]
fn generate_html____valid_data____contains_echarts_and_json() {
    let json = r#"[{"name":"test","value":100}]"#;
    let html = generate_html(json, 100, 1, "/tmp/test", 3);

    assert!(html.contains("echarts"));
    assert!(html.contains(json));
    assert!(html.contains("/tmp/test"));
    assert!(html.contains("1 files"));
    assert!(html.contains("100 B total"));
}

#[test]
fn generate_html____large_size____formats_correctly() {
    let json = "[]";
    let html = generate_html(json, 5_368_709_120, 42000, "/home", 3);

    assert!(html.contains("5.0 GB total"));
    assert!(html.contains("42000 files"));
}

#[test]
fn generate_html____custom_depth____sets_leaf_depth() {
    let json = "[]";
    let html = generate_html(json, 0, 0, "/", 5);

    assert!(html.contains("leafDepth: 5"));
}

#[test]
fn generate_html____output_is_valid_html____has_doctype_and_closing_tags() {
    let json = "[]";
    let html = generate_html(json, 0, 0, "/", 3);

    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("</html>"));
    assert!(html.contains("</body>"));
    assert!(html.contains("</script>"));
}

#[test]
fn generate_html____legend____includes_all_categories_with_correct_colors() {
    let json = "[]";
    let html = generate_html(json, 0, 0, "/", 3);

    for cat in FileCategory::ALL {
        let color = cat.color();
        let label = cat.label();
        assert!(
            html.contains(&color),
            "HTML legend missing color {color} for category {label}"
        );
        assert!(html.contains(label), "HTML legend missing label {label}");
    }
}

#[test]
fn generate_html____legend____category_count_matches() {
    let json = "[]";
    let html = generate_html(json, 0, 0, "/", 3);

    let swatch_count = html.matches("class=\"swatch\"").count();
    assert_eq!(
        swatch_count,
        FileCategory::ALL.len(),
        "Legend should have exactly {} swatches, found {swatch_count}",
        FileCategory::ALL.len()
    );
}
