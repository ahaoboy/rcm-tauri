//! Screen-space geometry, in physical pixels.
//!
//! These are deliberately tiny value types with no toolkit dependency, so the
//! same layout maths can run inside a Tauri command handler, a Reactor
//! component, or a unit test.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A point on the virtual desktop (physical pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Offset from the virtual-desktop origin.
    pub const fn offset(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A width/height pair (physical pixels, unless documented otherwise).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Whether both dimensions are positive.
    pub const fn is_positive(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// An axis-aligned rectangle (physical pixels).
///
/// A monitor's *work area* is a `Rect`. For compatibility with Tauri's
/// `Monitor` type, `x`/`y` are the top-left corner and `width`/`height` extend
/// right/down — i.e. this is *not* a corner-pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Left edge (inclusive).
    pub const fn left(&self) -> i32 {
        self.x
    }

    /// Top edge (inclusive).
    pub const fn top(&self) -> i32 {
        self.y
    }

    /// Right edge (inclusive).
    pub const fn right(&self) -> i32 {
        self.x + self.width
    }

    /// Bottom edge (inclusive).
    pub const fn bottom(&self) -> i32 {
        self.y + self.height
    }

    /// Whether `point` lies inside the rectangle.
    ///
    /// Edges are inclusive, matching `rcm-ui/utils/layout.ts`, so a point on a
    /// shared monitor border resolves to the first monitor that claims it.
    pub const fn contains(&self, point: Point) -> bool {
        point.x >= self.left()
            && point.x <= self.right()
            && point.y >= self.top()
            && point.y <= self.bottom()
    }

    /// Squared distance from `point` to the nearest point inside the rectangle.
    ///
    /// Zero when `point` is inside. Used to pick the closest monitor.
    pub fn distance_sq(&self, point: Point) -> i64 {
        let cx = point.x.clamp(self.left(), self.right()) as i64;
        let cy = point.y.clamp(self.top(), self.bottom()) as i64;
        let dx = point.x as i64 - cx;
        let dy = point.y as i64 - cy;
        dx * dx + dy * dy
    }

    /// Shrink the rectangle by `gap` on every edge.
    pub const fn inset(&self, gap: i32) -> Self {
        Self {
            x: self.x + gap,
            y: self.y + gap,
            width: self.width - gap * 2,
            height: self.height - gap * 2,
        }
    }

    /// Place a `size`-sized rectangle at `point`.
    pub const fn at(point: Point, size: Size) -> Self {
        Self {
            x: point.x,
            y: point.y,
            width: size.width,
            height: size.height,
        }
    }
}

/// Renders as `(x, y) WxH` — the form used in menu placement logs.
impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}) {}x{}", self.x, self.y, self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_includes_edges() {
        let r = Rect::new(0, 0, 100, 200);
        assert!(r.contains(Point::new(0, 0)));
        assert!(r.contains(Point::new(100, 200)));
        assert!(r.contains(Point::new(50, 100)));
        assert!(!r.contains(Point::new(101, 100)));
        assert!(!r.contains(Point::new(50, -1)));
    }

    #[test]
    fn distance_is_zero_inside() {
        let r = Rect::new(10, 10, 100, 100);
        assert_eq!(r.distance_sq(Point::new(50, 50)), 0);
        assert_eq!(r.distance_sq(Point::new(5, 50)), 25);
    }

    #[test]
    fn inset_shrinks_each_edge() {
        let r = Rect::new(0, 0, 100, 100).inset(8);
        assert_eq!(r, Rect::new(8, 8, 84, 84));
        assert_eq!(r.right(), 92);
    }

    #[test]
    fn rect_displays_position_and_size() {
        assert_eq!(Rect::new(10, 20, 300, 449).to_string(), "(10, 20) 300x449");
        assert_eq!(Rect::new(-5, -3, 1, 2).to_string(), "(-5, -3) 1x2");
    }
}
