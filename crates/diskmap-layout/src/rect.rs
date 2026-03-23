/// A rectangle in layout space (f64 for precision during subdivision).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl LayoutRect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn area(&self) -> f64 {
        self.w * self.h
    }

    pub fn shorter_side(&self) -> f64 {
        self.w.min(self.h)
    }

    pub fn is_wide(&self) -> bool {
        self.w >= self.h
    }

    /// Whether this rect is large enough to be visible.
    pub fn is_visible(&self, min_dimension: f64) -> bool {
        self.w >= min_dimension && self.h >= min_dimension
    }

    /// Inset the rectangle by `amount` on all sides. Returns a zero-area rect
    /// if the inset would collapse it.
    pub fn inset(&self, amount: f64) -> Self {
        let double = amount * 2.0;
        if self.w <= double || self.h <= double {
            return Self {
                x: self.x + self.w / 2.0,
                y: self.y + self.h / 2.0,
                w: 0.0,
                h: 0.0,
            };
        }
        Self {
            x: self.x + amount,
            y: self.y + amount,
            w: self.w - double,
            h: self.h - double,
        }
    }

    /// Split off a strip from the shorter side direction.
    /// `fraction` is the proportion of area the strip occupies (0..1).
    /// Returns (strip, remainder).
    pub fn split_strip(&self, fraction: f64) -> (Self, Self) {
        let fraction = fraction.clamp(0.0, 1.0);
        if self.is_wide() {
            // Split vertically — strip on the left
            let strip_w = self.w * fraction;
            let strip = Self {
                x: self.x,
                y: self.y,
                w: strip_w,
                h: self.h,
            };
            let remainder = Self {
                x: self.x + strip_w,
                y: self.y,
                w: self.w - strip_w,
                h: self.h,
            };
            (strip, remainder)
        } else {
            // Split horizontally — strip on top
            let strip_h = self.h * fraction;
            let strip = Self {
                x: self.x,
                y: self.y,
                w: self.w,
                h: strip_h,
            };
            let remainder = Self {
                x: self.x,
                y: self.y + strip_h,
                w: self.w,
                h: self.h - strip_h,
            };
            (strip, remainder)
        }
    }
}

#[cfg(test)]
#[path = "rect_tests.rs"]
mod rect_tests;
