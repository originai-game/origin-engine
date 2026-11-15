//! Text on a path (`-og-text-path`, docs/ui.md §2): SVG 2's `<textPath>`
//! layout for one line of an element's text. Each glyph is placed so the
//! middle of its advance sits on the path at its distance along it, turned
//! to the path's direction there, its baseline on the path; a glyph whose
//! middle falls off an open path is not drawn (SVG 2 §11.8.3), and a
//! closed path (a circle) wraps. `text-align` is the text's anchor at
//! `-og-text-path-offset` (SVG's `text-anchor` at `startOffset`).
//!
//! Path data is SVG's (`M L H V C S Q T A Z`, absolute and relative),
//! kurbo's parser and flattening, once when the document is built.
//! CSS's `offset-path` moves the element itself, so it is not reused.

use og_core::glam::Vec2;

/// An element's text path ([`crate::Props::text_path`]).
#[derive(Clone, Debug, PartialEq)]
pub struct TextPath {
    pub shape: PathShape,
    /// Where the text's anchor sits along the path: CSS px, or a share of
    /// the path's length when `percent` (`-og-text-path-offset`).
    pub offset: f32,
    pub percent: bool,
    /// Runs the path backwards (SVG 2's `side="right"`): text on a circle
    /// reads counter-clockwise, inside it.
    pub reverse: bool,
}

/// The path's shape, in CSS px of the element's content box.
#[derive(Clone, Debug, PartialEq)]
pub enum PathShape {
    /// `path("<svg path data>")`: its subpaths, flattened.
    Data(Subpaths),
    /// `circle([<radius>])`: centred in the content box, from its top
    /// clockwise; no radius is the box's closest side.
    Circle(Option<f32>),
}

impl TextPath {
    /// A text path from its CSS (`-og-text-path`, `-og-text-path-offset`,
    /// `-og-text-path-side`); `None` for `none`.
    pub fn from_css(
        shape: &str,
        offset: Option<&str>,
        side: Option<&str>,
    ) -> Result<Option<TextPath>, String> {
        let Some(shape) = parse_shape(shape)? else {
            return Ok(None);
        };
        let (offset, percent) = match offset {
            Some(o) => parse_offset(o)?,
            None => (0.0, false),
        };
        let reverse = match side.map(str::trim) {
            None | Some("left") => false,
            Some("right") => true,
            Some(s) => return Err(format!("-og-text-path-side is left or right, not '{s}'")),
        };
        Ok(Some(TextPath {
            shape,
            offset,
            percent,
            reverse,
        }))
    }

    /// The size it gives an element with no size of its own: the path
    /// data's extent from the content box's origin, a circle's diameter
    /// (none without a radius).
    pub fn extent(&self) -> Option<Vec2> {
        match &self.shape {
            PathShape::Data(subs) => {
                let max = subs.iter().flatten().fold(Vec2::ZERO, |m, p| m.max(*p));
                Some(max)
            }
            PathShape::Circle(r) => r.map(|r| Vec2::splat(2.0 * r)),
        }
    }

    /// Its polyline in a content box of `size` (from the box's origin),
    /// and whether it closes.
    pub(crate) fn polyline(&self, size: Vec2) -> (Vec<Vec<Vec2>>, bool) {
        let (mut subs, closed) = match &self.shape {
            PathShape::Data(subs) => {
                let closed =
                    subs.len() == 1 && subs[0].len() > 2 && subs[0].first() == subs[0].last();
                ((**subs).clone(), closed)
            }
            PathShape::Circle(r) => {
                let r = r.unwrap_or(size.x.min(size.y) / 2.0).max(0.0);
                let c = size / 2.0;
                // Enough sides that a chord strays under a tenth of a px.
                let n = ((std::f32::consts::PI / (1.0 - 0.1 / r.max(0.2)).clamp(-1.0, 1.0).acos())
                    .ceil() as usize)
                    .clamp(16, 720);
                let ring: Vec<Vec2> = (0..=n)
                    .map(|k| {
                        // From the top (12 o'clock), clockwise on screen.
                        let a = k as f32 / n as f32 * std::f32::consts::TAU;
                        c + Vec2::new(a.sin(), -a.cos()) * r
                    })
                    .collect();
                (vec![ring], true)
            }
        };
        if self.reverse {
            subs.reverse();
            for s in &mut subs {
                s.reverse();
            }
        }
        (subs, closed)
    }
}

/// `-og-text-path`'s value: `none`, `path("<svg path data>")` or
/// `circle([<radius>px])`. Path data is flattened once per distinct text.
pub fn parse_shape(css: &str) -> Result<Option<PathShape>, String> {
    let v = css.trim();
    if v == "none" {
        return Ok(None);
    }
    let inner = |f: &str| {
        v.strip_prefix(f)
            .and_then(|r| r.trim_start().strip_prefix('('))
            .and_then(|r| r.strip_suffix(')'))
            .map(str::trim)
    };
    if let Some(arg) = inner("circle") {
        if arg.is_empty() {
            return Ok(Some(PathShape::Circle(None)));
        }
        let n = arg.strip_suffix("px").unwrap_or(arg).trim();
        return match n.parse::<f32>() {
            Ok(r) if r.is_finite() && r > 0.0 => Ok(Some(PathShape::Circle(Some(r)))),
            _ => Err(format!(
                "circle() takes a radius in px (circle(80px)), not '{arg}'"
            )),
        };
    }
    if let Some(arg) = inner("path") {
        let data = arg
            .strip_prefix('"')
            .and_then(|a| a.strip_suffix('"'))
            .or_else(|| arg.strip_prefix('\'').and_then(|a| a.strip_suffix('\'')))
            .ok_or_else(|| {
                format!("path() takes quoted path data (path(\"M0 40 Q100 0 200 40\")), not {arg}")
            })?;
        return Ok(Some(PathShape::Data(flattened(data)?)));
    }
    Err(format!(
        "-og-text-path is none, path(\"<svg path data>\") or circle([<n>px]), not '{v}' (offset-path moves the element itself)"
    ))
}

/// `-og-text-path-offset`: `<n>px` or `<n>%` (a share of the path's length).
pub fn parse_offset(css: &str) -> Result<(f32, bool), String> {
    let v = css.trim();
    let (n, percent) = match v.strip_suffix('%') {
        Some(n) => (n, true),
        None => (v.strip_suffix("px").unwrap_or(v), false),
    };
    match n.trim().parse::<f32>() {
        Ok(x) if x.is_finite() => Ok((if percent { x / 100.0 } else { x }, percent)),
        _ => Err(format!("-og-text-path-offset is <n>px or <n>%, not '{v}'")),
    }
}

/// Flattened subpaths, shared by every element with the same data.
type Subpaths = std::sync::Arc<Vec<Vec<Vec2>>>;

/// Path data flattened, kept by its text: a document built every frame
/// flattens each path once.
fn flattened(data: &str) -> Result<Subpaths, String> {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<String, Subpaths>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(hit) = cache.lock().expect("path cache").get(data) {
        return Ok(hit.clone());
    }
    let subs = Arc::new(parse_path_data(data)?);
    let mut c = cache.lock().expect("path cache");
    if c.len() >= 256 {
        c.clear();
    }
    c.insert(data.to_owned(), subs.clone());
    Ok(subs)
}

/// A polyline walked by distance: each subpath's points and the distance
/// at each (a move between subpaths adds none).
pub(crate) struct Walk {
    points: Vec<Vec2>,
    at: Vec<f32>,
    /// Where a subpath starts: no segment joins the point before it.
    starts: Vec<usize>,
    pub length: f32,
    pub closed: bool,
}

impl Walk {
    pub fn new(subs: &[Vec<Vec2>], closed: bool) -> Walk {
        let mut w = Walk {
            points: Vec::new(),
            at: Vec::new(),
            starts: Vec::new(),
            length: 0.0,
            closed,
        };
        for s in subs.iter().filter(|s| !s.is_empty()) {
            w.starts.push(w.points.len());
            for (k, p) in s.iter().enumerate() {
                if k > 0 {
                    w.length += p.distance(s[k - 1]);
                }
                w.points.push(*p);
                w.at.push(w.length);
            }
        }
        w
    }

    /// The point `d` along it and its direction there (radians, screen y
    /// down); `None` off an open path's ends or on an empty one.
    pub fn at(&self, d: f32) -> Option<(Vec2, f32)> {
        if self.points.len() < 2 || self.length <= 0.0 {
            return None;
        }
        let d = if self.closed {
            d.rem_euclid(self.length)
        } else if (0.0..=self.length).contains(&d) {
            d
        } else {
            return None;
        };
        // The segment holding `d`: the last point at or before it whose
        // next point is in the same subpath.
        let mut k = self.at.partition_point(|&a| a <= d).saturating_sub(1);
        loop {
            let next = k + 1;
            if next < self.points.len() && !self.starts.contains(&next) {
                break;
            }
            if k == 0 {
                return None;
            }
            k -= 1;
        }
        let (a, b) = (self.points[k], self.points[k + 1]);
        let len = self.at[k + 1] - self.at[k];
        let t = if len > 0.0 {
            (d - self.at[k]) / len
        } else {
            0.0
        };
        let dir = b - a;
        Some((a + dir * t.clamp(0.0, 1.0), dir.y.atan2(dir.x)))
    }
}

/// SVG path data (kurbo's) flattened to polylines within a tenth of a CSS
/// px, a closed subpath ending on its first point again; or what is wrong
/// with it.
pub fn parse_path_data(d: &str) -> Result<Vec<Vec<Vec2>>, String> {
    let path = og_draw::parse_svg_path(d)?;
    let subs: Vec<Vec<Vec2>> = og_draw::polylines(&path, 0.1)
        .into_iter()
        .map(|(mut pts, closed)| {
            if closed {
                pts.push(pts[0]);
            }
            pts
        })
        .collect();
    if subs.is_empty() {
        return Err("path data with no segment: draw at least one line or curve after M".into());
    }
    Ok(subs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn length(subs: &[Vec<Vec2>]) -> f32 {
        subs.iter()
            .map(|s| s.windows(2).map(|w| w[0].distance(w[1])).sum::<f32>())
            .sum()
    }

    #[test]
    fn arcs_curves_and_relative_commands_flatten_to_their_shapes() {
        // A half circle of radius 50 by an arc: its length is 50π; the
        // small-arc flag with sweep 1 goes through y > 50 (below).
        let half = parse_path_data("M0 50 A50 50 0 0 1 100 50").unwrap();
        assert!((length(&half) - 50.0 * std::f32::consts::PI).abs() < 0.3);
        assert!(
            half[0].iter().all(|p| p.y <= 50.001),
            "sweep 1 runs above on screen: {:?}",
            half[0][half[0].len() / 2]
        );
        let other = parse_path_data("M0 50 A50 50 0 0 0 100 50").unwrap();
        assert!(other[0][other[0].len() / 2].y > 99.0);
        // Too-small radii scale up to just reach (F.6.6): still a half circle.
        let small = parse_path_data("M0 0 A1 1 0 0 1 10 0").unwrap();
        assert!((length(&small) - 5.0 * std::f32::consts::PI).abs() < 0.2);
        // Relative moves from the current point; H and V; Z closes.
        let box_ = parse_path_data("m10 10 h20 v20 h-20 z").unwrap();
        assert_eq!(box_[0].first(), box_[0].last());
        assert_eq!(box_[0][2], Vec2::new(30.0, 30.0));
        // S reflects the last control: a symmetric S-curve's middle is its centre.
        let s = parse_path_data("M0 0 C0 50 50 50 50 0 S100 -50 100 0").unwrap();
        assert!(
            s[0].iter()
                .any(|p| (p.x - 50.0).abs() < 0.01 && p.y.abs() < 0.01)
        );
        assert!(s[0].iter().any(|p| p.y < -30.0));
        // Numbers run together as SVG allows.
        let tight = parse_path_data("M0,0L10-5.5.5.5").unwrap();
        assert_eq!(tight[0][2], Vec2::new(0.5, 0.5));
    }

    #[test]
    fn a_walk_wraps_a_closed_path_and_ends_an_open_one() {
        let line = vec![vec![Vec2::ZERO, Vec2::new(10.0, 0.0)]];
        let w = Walk::new(&line, false);
        assert!(w.at(-0.1).is_none() && w.at(10.1).is_none());
        assert_eq!(w.at(4.0).unwrap().0, Vec2::new(4.0, 0.0));
        let sq = vec![vec![
            Vec2::ZERO,
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
            Vec2::ZERO,
        ]];
        let w = Walk::new(&sq, true);
        let (p, a) = w.at(45.0).unwrap();
        assert_eq!(p, Vec2::new(5.0, 0.0));
        assert!(a.abs() < 1e-6);
        let (p, a) = w.at(15.0).unwrap();
        assert_eq!(p, Vec2::new(10.0, 5.0));
        assert!((a - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        // Two subpaths: nothing between them.
        let two = vec![
            vec![Vec2::ZERO, Vec2::new(10.0, 0.0)],
            vec![Vec2::new(0.0, 50.0), Vec2::new(10.0, 50.0)],
        ];
        let w = Walk::new(&two, false);
        assert_eq!(w.length, 20.0);
        assert_eq!(w.at(12.0).unwrap().0, Vec2::new(2.0, 50.0));
    }
}
