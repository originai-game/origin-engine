//! Rectangles and insets in CSS pixels, and the device pixel grid.

/// A rectangle in CSS pixels, origin at the top left, y down.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn intersect(&self, o: &Rect) -> Rect {
        let x = self.x.max(o.x);
        let y = self.y.max(o.y);
        let r = self.right().min(o.right());
        let b = self.bottom().min(o.bottom());
        Rect::new(x, y, (r - x).max(0.0), (b - y).max(0.0))
    }

    pub fn overlaps(&self, o: &Rect) -> bool {
        self.x < o.right() && o.x < self.right() && self.y < o.bottom() && o.y < self.bottom()
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Grown by `d` on every side.
    /// The rectangle less `pad` on each side (never negative).
    pub fn inset(&self, pad: &Insets) -> Rect {
        Rect::new(
            self.x + pad.left,
            self.y + pad.top,
            (self.width - pad.horizontal()).max(0.0),
            (self.height - pad.vertical()).max(0.0),
        )
    }

    pub fn outset(&self, d: f32) -> Rect {
        Rect::new(
            self.x - d,
            self.y - d,
            self.width + 2.0 * d,
            self.height + 2.0 * d,
        )
    }

    /// Scaled by `k` around its centre.
    pub fn scaled(&self, k: f32) -> Rect {
        let (cx, cy) = self.center();
        Rect::new(
            cx - self.width * k / 2.0,
            cy - self.height * k / 2.0,
            self.width * k,
            self.height * k,
        )
    }

    /// Edges rounded to the device pixel grid ([`snap`]).
    pub fn snapped(&self, dpr: f32) -> Rect {
        let s = |v: f32| snap(v, dpr);
        let (x, y) = (s(self.x), s(self.y));
        Rect::new(x, y, s(self.right()) - x, s(self.bottom()) - y)
    }
}

/// `v` CSS px on the device pixel grid of `dpr`, held to a multiple of
/// 1/256 px (less than 1/512 px off it, unseen): at a ratio like 1.5 a
/// device pixel is no exact `f32`, and a grid of exact values keeps every
/// sum of edges and moves exact, so a scrolled row's commands moved by the
/// scroll are the ones painting it there gives, and overlap what they did.
pub fn snap(v: f32, dpr: f32) -> f32 {
    ((v * dpr).round() / dpr * 256.0).round() / 256.0
}

/// Space inside an element's edges (padding) or a device's unsafe margins.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Insets {
    pub const fn all(v: f32) -> Insets {
        Insets {
            left: v,
            top: v,
            right: v,
            bottom: v,
        }
    }

    /// `x` left and right, `y` top and bottom.
    pub const fn xy(x: f32, y: f32) -> Insets {
        Insets {
            left: x,
            top: y,
            right: x,
            bottom: y,
        }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    pub fn add(&self, o: &Insets) -> Insets {
        Insets {
            left: self.left + o.left,
            top: self.top + o.top,
            right: self.right + o.right,
            bottom: self.bottom + o.bottom,
        }
    }
}
