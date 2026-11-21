//! The engine's built-in icons (Lucide, `third_party/lucide`) as one table
//! of stroke segments in their 24-unit space, made once when the first
//! icon paints and shared by every paint arena: an icon is a quad naming
//! its span, which the paint shader strokes 2 units wide and round.

use og_draw::paint::IconSegment;
use std::sync::{Arc, OnceLock};

/// The table and each icon's span of it, by name.
pub struct Icons {
    pub segments: Arc<[IconSegment]>,
    spans: Vec<(&'static str, u32, u32)>,
}

impl Icons {
    /// Icon `name`'s first segment and their count.
    pub fn find(&self, name: &str) -> Option<(u32, u32)> {
        self.spans
            .iter()
            .find(|(n, ..)| *n == name)
            .map(|&(_, first, count)| (first, count))
    }
}

/// The built-in icons' table.
pub fn icons() -> &'static Icons {
    static ICONS: OnceLock<Icons> = OnceLock::new();
    ICONS.get_or_init(|| {
        let mut segments = Vec::new();
        let mut spans = Vec::new();
        for &name in og_ui::DEFAULT_ICON_NAMES {
            let first = segments.len() as u32;
            let svg = og_ui::builtin_icon_svg(name)
                .and_then(|s| og_ui::svg::parse_xml(s).ok())
                .expect("bundled Lucide SVG");
            for el in &svg.children {
                let Ok(Some(path)) =
                    og_ui::svg::shape_path(&el.name, &|n| el.get(n), [0.0, 0.0, 24.0, 24.0])
                else {
                    continue;
                };
                for (points, closed) in og_ui::svg::subpaths(&path) {
                    flatten(&points, closed, &mut segments);
                }
            }
            spans.push((name, first, segments.len() as u32 - first));
        }
        Icons {
            segments: segments.into(),
            spans,
        }
    })
}

/// A subpath (its start, then three points a cubic) as segments: a cubic
/// whose controls lie on its chord is one, a curve one per 1.5 units of
/// its control polygon (2 to 16).
fn flatten(points: &[[f32; 2]], closed: bool, out: &mut Vec<IconSegment>) {
    let Some(&start) = points.first() else {
        return;
    };
    let mut at = start;
    let (cubics, _) = points[1..].as_chunks::<3>();
    for &[p1, p2, p3] in cubics {
        let off = |p: [f32; 2]| {
            let (dx, dy) = (p3[0] - at[0], p3[1] - at[1]);
            let len = (dx * dx + dy * dy).sqrt().max(1e-6);
            ((p[0] - at[0]) * dy - (p[1] - at[1]) * dx).abs() / len
        };
        if off(p1) < 0.01 && off(p2) < 0.01 {
            out.push([at[0], at[1], p3[0], p3[1]]);
            at = p3;
            continue;
        }
        let d = |a: [f32; 2], b: [f32; 2]| {
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            (dx * dx + dy * dy).sqrt()
        };
        let n = ((d(at, p1) + d(p1, p2) + d(p2, p3)) / 1.5)
            .ceil()
            .clamp(2.0, 16.0) as usize;
        let mut prev = at;
        for k in 1..=n {
            let t = k as f32 / n as f32;
            let u = 1.0 - t;
            let b = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            let p = [
                b[0] * at[0] + b[1] * p1[0] + b[2] * p2[0] + b[3] * p3[0],
                b[0] * at[1] + b[1] * p1[1] + b[2] * p2[1] + b[3] * p3[1],
            ];
            out.push([prev[0], prev[1], p[0], p[1]]);
            prev = p;
        }
        at = p3;
    }
    if closed && (at[0] - start[0]).abs() + (at[1] - start[1]).abs() > 1e-4 {
        out.push([at[0], at[1], start[0], start[1]]);
    }
}
