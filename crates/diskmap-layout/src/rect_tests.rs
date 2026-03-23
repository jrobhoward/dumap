#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(non_snake_case)]

use super::LayoutRect;

const EPSILON: f64 = 1e-10;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

#[test]
fn area____simple____correct() {
    let r = LayoutRect::new(0.0, 0.0, 10.0, 5.0);
    assert!(approx_eq(r.area(), 50.0));
}

#[test]
fn shorter_side____wide_rect____returns_height() {
    let r = LayoutRect::new(0.0, 0.0, 10.0, 3.0);
    assert!(approx_eq(r.shorter_side(), 3.0));
}

#[test]
fn shorter_side____tall_rect____returns_width() {
    let r = LayoutRect::new(0.0, 0.0, 3.0, 10.0);
    assert!(approx_eq(r.shorter_side(), 3.0));
}

#[test]
fn is_wide____wide____true() {
    assert!(LayoutRect::new(0.0, 0.0, 10.0, 5.0).is_wide());
}

#[test]
fn is_wide____tall____false() {
    assert!(!LayoutRect::new(0.0, 0.0, 5.0, 10.0).is_wide());
}

#[test]
fn is_wide____square____true() {
    assert!(LayoutRect::new(0.0, 0.0, 5.0, 5.0).is_wide());
}

#[test]
fn inset____normal____shrinks_correctly() {
    let r = LayoutRect::new(10.0, 20.0, 100.0, 50.0);
    let i = r.inset(5.0);
    assert!(approx_eq(i.x, 15.0));
    assert!(approx_eq(i.y, 25.0));
    assert!(approx_eq(i.w, 90.0));
    assert!(approx_eq(i.h, 40.0));
}

#[test]
fn inset____too_large____returns_zero_area() {
    let r = LayoutRect::new(0.0, 0.0, 4.0, 4.0);
    let i = r.inset(3.0);
    assert!(approx_eq(i.w, 0.0));
    assert!(approx_eq(i.h, 0.0));
}

#[test]
fn split_strip____wide_rect____splits_vertically() {
    let r = LayoutRect::new(0.0, 0.0, 100.0, 50.0);
    let (strip, remainder) = r.split_strip(0.3);

    assert!(approx_eq(strip.x, 0.0));
    assert!(approx_eq(strip.w, 30.0));
    assert!(approx_eq(strip.h, 50.0));

    assert!(approx_eq(remainder.x, 30.0));
    assert!(approx_eq(remainder.w, 70.0));
    assert!(approx_eq(remainder.h, 50.0));

    // Total area preserved
    assert!(approx_eq(strip.area() + remainder.area(), r.area()));
}

#[test]
fn split_strip____tall_rect____splits_horizontally() {
    let r = LayoutRect::new(0.0, 0.0, 50.0, 100.0);
    let (strip, remainder) = r.split_strip(0.4);

    assert!(approx_eq(strip.y, 0.0));
    assert!(approx_eq(strip.h, 40.0));
    assert!(approx_eq(strip.w, 50.0));

    assert!(approx_eq(remainder.y, 40.0));
    assert!(approx_eq(remainder.h, 60.0));

    assert!(approx_eq(strip.area() + remainder.area(), r.area()));
}

#[test]
fn is_visible____large_enough____true() {
    assert!(LayoutRect::new(0.0, 0.0, 5.0, 5.0).is_visible(2.0));
}

#[test]
fn is_visible____too_small____false() {
    assert!(!LayoutRect::new(0.0, 0.0, 1.0, 5.0).is_visible(2.0));
}
