//! SVG path parsing, shape construction and curve flattening.

use super::*;

// ── paths ────────────────────────────────────────────────────────────────

/// A path as cubic segments: subpaths of a start point and the cubics
/// after it (lines are cubics with their control points on the line), and
/// whether each closes.
#[derive(Clone, Debug, Default)]
pub struct Path {
    pub subpaths: Vec<(Vec2, Vec<[Vec2; 3]>, bool)>,
}

impl Path {
    fn move_to(&mut self, p: Vec2) {
        self.subpaths.push((p, Vec::new(), false));
    }

    fn end(&self) -> Vec2 {
        self.subpaths
            .last()
            .map_or(Vec2::ZERO, |(s, c, _)| c.last().map_or(*s, |c| c[2]))
    }

    fn line_to(&mut self, p: Vec2) {
        let a = self.end();
        self.cubic_to(a + (p - a) / 3.0, a + (p - a) * (2.0 / 3.0), p);
    }

    fn cubic_to(&mut self, c1: Vec2, c2: Vec2, p: Vec2) {
        if self.subpaths.is_empty() {
            self.move_to(Vec2::ZERO);
        }
        self.subpaths
            .last_mut()
            .expect("a subpath")
            .1
            .push([c1, c2, p]);
    }

    fn close(&mut self) {
        if let Some(s) = self.subpaths.last_mut() {
            s.2 = true;
        }
    }

    /// An elliptical arc from the current point (SVG's endpoint
    /// parametrisation, F.6), as cubics of at most a quarter turn.
    fn arc_to(&mut self, r: Vec2, rotation: f32, large: bool, sweep: bool, p: Vec2) {
        let p0 = self.end();
        if (p - p0).length() < 1e-9 {
            return;
        }
        let (mut rx, mut ry) = (r.x.abs(), r.y.abs());
        if rx < 1e-9 || ry < 1e-9 {
            self.line_to(p);
            return;
        }
        let (sin, cos) = rotation.to_radians().sin_cos();
        let d = (p0 - p) * 0.5;
        let x1 = cos * d.x + sin * d.y;
        let y1 = -sin * d.x + cos * d.y;
        let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
        if lambda > 1.0 {
            rx *= lambda.sqrt();
            ry *= lambda.sqrt();
        }
        let num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
        let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
        let mut k = (num / den).max(0.0).sqrt();
        if large == sweep {
            k = -k;
        }
        let cx1 = k * rx * y1 / ry;
        let cy1 = -k * ry * x1 / rx;
        let mid = (p0 + p) * 0.5;
        let c = Vec2::new(cos * cx1 - sin * cy1 + mid.x, sin * cx1 + cos * cy1 + mid.y);
        // atan2, not acos: an arc between nearly coincident ends (a circle
        // drawn as one arc) turns by a tiny angle that acos rounds to 0.
        let angle = |u: Vec2, v: Vec2| u.perp_dot(v).atan2(u.dot(v));
        let u = Vec2::new((x1 - cx1) / rx, (y1 - cy1) / ry);
        let v = Vec2::new((-x1 - cx1) / rx, (-y1 - cy1) / ry);
        let theta = angle(Vec2::X, u);
        let mut delta = angle(u, v);
        if !sweep && delta > 0.0 {
            delta -= std::f32::consts::TAU;
        } else if sweep && delta < 0.0 {
            delta += std::f32::consts::TAU;
        }
        let n = (delta.abs() / std::f32::consts::FRAC_PI_2).ceil().max(1.0) as usize;
        let step = delta / n as f32;
        let kappa = 4.0 / 3.0 * (step / 4.0).tan();
        let point = |t: f32| {
            let (s, c0) = t.sin_cos();
            Vec2::new(rx * c0, ry * s)
        };
        let deriv = |t: f32| {
            let (s, c0) = t.sin_cos();
            Vec2::new(-rx * s, ry * c0)
        };
        let world = |q: Vec2| Vec2::new(cos * q.x - sin * q.y + c.x, sin * q.x + cos * q.y + c.y);
        for i in 0..n {
            let t0 = theta + step * i as f32;
            let t1 = t0 + step;
            let a = point(t0);
            let b = point(t1);
            let end = if i + 1 == n { p } else { world(b) };
            self.cubic_to(
                world(a + deriv(t0) * kappa),
                world(b - deriv(t1) * kappa),
                end,
            );
        }
    }

    /// An ellipse at `c` with radii `r`, clockwise from its right.
    pub fn ellipse(c: Vec2, r: Vec2) -> Path {
        let mut p = Path::default();
        p.move_to(c + Vec2::new(r.x, 0.0));
        for q in [
            Vec2::new(0.0, r.y),
            Vec2::new(-r.x, 0.0),
            Vec2::new(0.0, -r.y),
            Vec2::new(r.x, 0.0),
        ] {
            p.arc_to(r, 0.0, false, true, c + q);
        }
        p.close();
        p
    }

    pub fn transformed(&self, t: &Affine) -> Path {
        Path {
            subpaths: self
                .subpaths
                .iter()
                .map(|(s, cs, closed)| {
                    (
                        t.apply(*s),
                        cs.iter()
                            .map(|c| [t.apply(c[0]), t.apply(c[1]), t.apply(c[2])])
                            .collect(),
                        *closed,
                    )
                })
                .collect(),
        }
    }

    /// The subpaths as polylines (within `tolerance`), each with whether
    /// it closes.
    pub fn flatten(&self, tolerance: f32) -> Vec<(Vec<Vec2>, bool)> {
        self.subpaths
            .iter()
            .filter_map(|(s, cs, closed)| {
                let mut pts = vec![*s];
                let mut at = *s;
                for c in cs {
                    let straight = (c[0] - (at + (c[2] - at) / 3.0)).length() < 1e-6
                        && (c[1] - (at + (c[2] - at) * (2.0 / 3.0))).length() < 1e-6;
                    if straight {
                        pts.push(c[2]);
                    } else {
                        pts.extend(
                            og_render::flatten_cubic(at, c[0], c[1], c[2], tolerance)
                                .into_iter()
                                .skip(1),
                        );
                    }
                    at = c[2];
                }
                pts.dedup_by(|a, b| (*a - *b).length() < 1e-6);
                (pts.len() > 1).then_some((pts, *closed))
            })
            .collect()
    }

    /// Its length along every subpath (within `tolerance`).
    pub fn length(&self, tolerance: f32) -> f32 {
        self.flatten(tolerance)
            .iter()
            .map(|(pts, closed)| {
                let open: f32 = pts.windows(2).map(|w| (w[1] - w[0]).length()).sum();
                match (closed, pts.first(), pts.last()) {
                    (true, Some(a), Some(b)) => open + (*a - *b).length(),
                    _ => open,
                }
            })
            .sum()
    }

    /// Parses path data (`d`).
    pub fn parse(d: &str) -> Result<Path, String> {
        let mut path = Path::default();
        let chars: Vec<char> = d.chars().collect();
        let mut i = 0;
        let mut start = Vec2::ZERO;
        let mut last_c: Option<Vec2> = None;
        let mut last_q: Option<Vec2> = None;
        while i < chars.len() {
            let c = chars[i];
            if c.is_whitespace() || c == ',' {
                i += 1;
                continue;
            }
            if !c.is_ascii_alphabetic() || c == 'e' || c == 'E' {
                return Err(format!("path data: '{c}' where a command was expected"));
            }
            let mut j = i + 1;
            while j < chars.len()
                && !(chars[j].is_ascii_alphabetic() && chars[j] != 'e' && chars[j] != 'E')
            {
                j += 1;
            }
            let args: Vec<f32> = if c.eq_ignore_ascii_case(&'a') {
                arc_numbers(&chars[i + 1..j].iter().collect::<String>())?
            } else {
                numbers(&chars[i + 1..j].iter().collect::<String>())?
            };
            i = j;
            let rel = c.is_ascii_lowercase();
            let arity = match c.to_ascii_uppercase() {
                'M' | 'L' | 'T' => 2,
                'H' | 'V' => 1,
                'C' => 6,
                'S' | 'Q' => 4,
                'A' => 7,
                'Z' => 0,
                other => return Err(format!("path data: '{other}' is not a command")),
            };
            if arity == 0 {
                path.close();
                // The next command starts at the subpath's start.
                path.move_to(start);
                last_c = None;
                last_q = None;
                continue;
            }
            if args.is_empty() || !args.len().is_multiple_of(arity) {
                return Err(format!(
                    "path data: '{c}' takes numbers in groups of {arity}, not {}",
                    args.len()
                ));
            }
            for (k, a) in args.chunks(arity).enumerate() {
                let at = path.end();
                let pt = |x: f32, y: f32| {
                    if rel {
                        at + Vec2::new(x, y)
                    } else {
                        Vec2::new(x, y)
                    }
                };
                let cmd = if k > 0 && c.eq_ignore_ascii_case(&'m') {
                    if rel { 'l' } else { 'L' }
                } else {
                    c
                };
                let (mut nc, mut nq) = (None, None);
                match cmd.to_ascii_uppercase() {
                    'M' => {
                        // A pending start left by Z is replaced.
                        if path.subpaths.last().is_some_and(|s| s.1.is_empty()) {
                            path.subpaths.pop();
                        }
                        start = pt(a[0], a[1]);
                        path.move_to(start);
                    }
                    'L' => path.line_to(pt(a[0], a[1])),
                    'H' => path.line_to(Vec2::new(if rel { at.x + a[0] } else { a[0] }, at.y)),
                    'V' => path.line_to(Vec2::new(at.x, if rel { at.y + a[0] } else { a[0] })),
                    'C' => {
                        let (c1, c2, p) = (pt(a[0], a[1]), pt(a[2], a[3]), pt(a[4], a[5]));
                        path.cubic_to(c1, c2, p);
                        nc = Some(c2);
                    }
                    'S' => {
                        let c1 = last_c.map_or(at, |c| 2.0 * at - c);
                        let (c2, p) = (pt(a[0], a[1]), pt(a[2], a[3]));
                        path.cubic_to(c1, c2, p);
                        nc = Some(c2);
                    }
                    'Q' | 'T' => {
                        let (q, p) = if cmd.eq_ignore_ascii_case(&'q') {
                            (pt(a[0], a[1]), pt(a[2], a[3]))
                        } else {
                            (last_q.map_or(at, |q| 2.0 * at - q), pt(a[0], a[1]))
                        };
                        path.cubic_to(at + (q - at) * (2.0 / 3.0), p + (q - p) * (2.0 / 3.0), p);
                        nq = Some(q);
                    }
                    'A' => {
                        path.arc_to(
                            Vec2::new(a[0], a[1]),
                            a[2],
                            a[3] != 0.0,
                            a[4] != 0.0,
                            pt(a[5], a[6]),
                        );
                    }
                    _ => unreachable!("arity checked above"),
                }
                last_c = nc;
                last_q = nq;
            }
        }
        path.subpaths.retain(|s| !s.1.is_empty());
        Ok(path)
    }
}

/// An arc command's numbers, whose flags may be written without a
/// separator (`a1 1 0 00 1 1`).
fn arc_numbers(s: &str) -> Result<Vec<f32>, String> {
    let mut out = Vec::new();
    let mut rest = s.trim_start_matches(|c: char| c.is_whitespace() || c == ',');
    while !rest.is_empty() {
        let k = out.len() % 7;
        if k == 3 || k == 4 {
            let c = rest.chars().next().expect("not empty");
            if c != '0' && c != '1' {
                return Err(format!("an arc flag is 0 or 1, not '{c}'"));
            }
            out.push(if c == '1' { 1.0 } else { 0.0 });
            rest = &rest[1..];
        } else {
            let end = number_end(rest);
            if end == 0 {
                return Err(format!("'{rest}' in an arc's numbers"));
            }
            out.push(
                rest[..end]
                    .parse::<f32>()
                    .map_err(|_| format!("'{}' is not a number", &rest[..end]))?,
            );
            rest = &rest[end..];
        }
        rest = rest.trim_start_matches(|c: char| c.is_whitespace() || c == ',');
    }
    Ok(out)
}

/// The length of the number at the start of `s`.
fn number_end(s: &str) -> usize {
    let b = s.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'-' || b[i] == b'+') {
        i += 1;
    }
    let mut dot = false;
    while i < b.len() && (b[i].is_ascii_digit() || (b[i] == b'.' && !dot)) {
        dot |= b[i] == b'.';
        i += 1;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'-' || b[j] == b'+') {
            j += 1;
        }
        if j < b.len() && b[j].is_ascii_digit() {
            i = j;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
        }
    }
    i
}

/// The local path of the shape element `name` whose attributes `get`
/// reads, in a view box `vb` (percentages are of its size).
pub fn shape_path<'a>(
    name: &str,
    get: &dyn Fn(&str) -> Option<&'a str>,
    vb: [f32; 4],
) -> Result<Option<Path>, String> {
    let (w, h) = (vb[2], vb[3]);
    let diag = (w * w + h * h).sqrt() / std::f32::consts::SQRT_2;
    let len = |attr: &str, whole: f32| {
        get(attr)
            .map_or(Ok(0.0), |v| length(v, whole))
            .map_err(|e| format!("<{name}> {attr}: {e}"))
    };
    Ok(Some(match name {
        "path" => match get("d") {
            Some(d) => Path::parse(d).map_err(|e| format!("<path>: {e}"))?,
            None => return Ok(None),
        },
        "rect" => {
            let (x, y, rw, rh) = (
                len("x", w)?,
                len("y", h)?,
                len("width", w)?,
                len("height", h)?,
            );
            if rw <= 0.0 || rh <= 0.0 {
                return Ok(None);
            }
            let opt = |name: &str, whole: f32| {
                get(name)
                    .map(|v| length(v, whole))
                    .transpose()
                    .map_err(|e| format!("<rect> {name}: {e}"))
            };
            // One radius given stands for both.
            let (rx_v, ry_v) = match (opt("rx", w)?, opt("ry", h)?) {
                (Some(a), Some(b)) => (a, b),
                (Some(a), None) => (a, a),
                (None, Some(b)) => (b, b),
                (None, None) => (0.0, 0.0),
            };
            let (rx, ry) = (rx_v.clamp(0.0, rw / 2.0), ry_v.clamp(0.0, rh / 2.0));
            let mut p = Path::default();
            if rx <= 0.0 || ry <= 0.0 {
                p.move_to(Vec2::new(x, y));
                p.line_to(Vec2::new(x + rw, y));
                p.line_to(Vec2::new(x + rw, y + rh));
                p.line_to(Vec2::new(x, y + rh));
            } else {
                let r = Vec2::new(rx, ry);
                p.move_to(Vec2::new(x + rx, y));
                p.line_to(Vec2::new(x + rw - rx, y));
                p.arc_to(r, 0.0, false, true, Vec2::new(x + rw, y + ry));
                p.line_to(Vec2::new(x + rw, y + rh - ry));
                p.arc_to(r, 0.0, false, true, Vec2::new(x + rw - rx, y + rh));
                p.line_to(Vec2::new(x + rx, y + rh));
                p.arc_to(r, 0.0, false, true, Vec2::new(x, y + rh - ry));
                p.line_to(Vec2::new(x, y + ry));
                p.arc_to(r, 0.0, false, true, Vec2::new(x + rx, y));
            }
            p.close();
            p
        }
        "circle" => {
            let r = len("r", diag)?;
            if r <= 0.0 {
                return Ok(None);
            }
            Path::ellipse(Vec2::new(len("cx", w)?, len("cy", h)?), Vec2::splat(r))
        }
        "ellipse" => {
            let (rx, ry) = (len("rx", w)?, len("ry", h)?);
            if rx <= 0.0 || ry <= 0.0 {
                return Ok(None);
            }
            Path::ellipse(Vec2::new(len("cx", w)?, len("cy", h)?), Vec2::new(rx, ry))
        }
        "line" => {
            let mut p = Path::default();
            p.move_to(Vec2::new(len("x1", w)?, len("y1", h)?));
            p.line_to(Vec2::new(len("x2", w)?, len("y2", h)?));
            p
        }
        "polyline" | "polygon" => {
            let n = numbers(get("points").unwrap_or(""))
                .map_err(|e| format!("<{name}> points: {e}"))?;
            if n.len() < 4 {
                return Ok(None);
            }
            let mut p = Path::default();
            p.move_to(Vec2::new(n[0], n[1]));
            for q in n[2..].as_chunks::<2>().0 {
                p.line_to(Vec2::new(q[0], q[1]));
            }
            if name == "polygon" {
                p.close();
            }
            p
        }
        _ => return Ok(None),
    }))
}

/// The bounding box `[x, y, w, h]` of a path's points (control points
/// included, which bound the curve).
pub fn bounds(p: &Path) -> [f32; 4] {
    let mut lo = Vec2::splat(f32::MAX);
    let mut hi = Vec2::splat(f32::MIN);
    for (s, cs, _) in &p.subpaths {
        for q in std::iter::once(s).chain(cs.iter().flatten()) {
            lo = lo.min(*q);
            hi = hi.max(*q);
        }
    }
    [lo.x, lo.y, (hi.x - lo.x).max(1e-6), (hi.y - lo.y).max(1e-6)]
}
