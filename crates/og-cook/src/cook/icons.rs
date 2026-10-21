//! Icon sets → font blobs: SVG icons (iconoir's 24-unit line icons) cooked
//! like a font, so og-ui draws them through the MTSDF text path (crisp at
//! any size, tinted by their ink, with a text glow when a skin asks).
//!
//! An `.ogt` of `type icons` names its SVGs:
//!
//! ```text
//! type icons
//! version 1
//! ---
//! source "iconoir"       # the SVGs' directory, beside this file
//! size 48                # atlas pixels per icon (its em)
//! range 8                # the distance range, atlas pixels
//! join round             # the join of a path that names none (round,
//!                        # miter or bevel; SVG's miter when left out)
//! icon "heart"           # an icon: its name, and its SVG's file name without
//! icon "close" "xmark"   # .svg when the two differ
//! ```
//!
//! Each icon is a glyph at U+E000 + its index (the Private Use Area), one em
//! square: the SVG's view box is the em, the baseline its bottom edge, so a
//! font size of S draws the icon S × S with its top at the line's top
//! (ascender 1, descender 0). The blob is an `og.Font` with one more raw
//! section, `names` (the icons' names, one a line, in index order), which
//! is how the runtime knows it is an icon set.
//!
//! The field is a true signed distance in all four channels (the median of
//! three equal channels is the distance, as og-ui's run-time glyphs), of
//! the stroke's own outline: round caps and joins are arcs; a butt cap is
//! cut square at its end; a miter join reaches its point (SVG's
//! `stroke-miterlimit`, 4 unless the path says, beyond which it is
//! bevelled) and a bevel join is cut across, so the zero line keeps a
//! miter's corner sharp (a single channel rounds it only inside one atlas
//! texel, where three channels would not; at 48 px an em that is under a
//! twentieth of iconoir's 24 units). A path's join is its
//! `stroke-linejoin`, else the set's `join` line, else SVG's `miter`.
//! SVG support is what line icons use: `<path>` elements with absolute or
//! relative `M L H V C S Q T Z` commands, `stroke` with `stroke-width`,
//! `stroke-linecap` (`round`, `butt`, `square` taken as butt),
//! `stroke-linejoin` (`round`, `miter`, `miter-clip` and `arcs` taken as
//! miter, `bevel`), `stroke-miterlimit` and `fill` (`currentColor` or any
//! colour fills by the nonzero rule; `none` does not), inherited from the
//! `<svg>` element. Anything else is refused with the icon and element
//! named.

use super::{CookError, Source};
use crate::format::{self, *};
use crate::ktx2;
use og_blob::BlobWriter;
use og_core::ogt::Document;

/// The `.ogt` type of an icon set.
pub const ICONS_TYPE: &str = "icons";
/// The code point of the first icon.
pub const ICON_PUA: u32 = 0xE000;

/// A parsed icon set file.
#[derive(Clone, Debug, PartialEq)]
pub struct IconSet {
    pub source: String,
    pub size: u32,
    pub range: u32,
    /// The join of a stroked path that names none (`join`; SVG's miter).
    pub join: Join,
    /// (name, SVG file stem), in index order.
    pub icons: Vec<(String, String)>,
}

/// How a stroke's segments meet at a corner (SVG's `stroke-linejoin`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Join {
    /// Out to the point where the two edges meet (within the miter limit).
    #[default]
    Miter,
    /// An arc around the corner.
    Round,
    /// Cut straight across the corner.
    Bevel,
}

impl Join {
    fn parse(v: &str) -> Option<Join> {
        match v {
            "miter" | "miter-clip" | "arcs" => Some(Join::Miter),
            "round" => Some(Join::Round),
            "bevel" => Some(Join::Bevel),
            _ => None,
        }
    }
}

/// Parses an icon set file.
pub fn parse(text: &str) -> Result<IconSet, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != ICONS_TYPE {
        return Err(format!("an icon set is `type icons`, not `{}`", doc.ty));
    }
    let mut set = IconSet {
        source: ".".into(),
        size: 48,
        range: 8,
        join: Join::Miter,
        icons: Vec::new(),
    };
    for e in &doc.body.entries {
        let at = |m: String| format!("line {}: {m}", e.line);
        match e.key.as_str() {
            "source" => set.source = e.string(0).map_err(|x| at(x.to_string()))?.to_owned(),
            "size" | "range" => {
                let v = e.f32(0).map_err(|x| at(x.to_string()))?;
                if !((1.0..=512.0).contains(&v) && v.fract() == 0.0) {
                    return Err(at(format!(
                        "{} {v}: a whole number of pixels, 1 to 512",
                        e.key
                    )));
                }
                if e.key == "size" {
                    set.size = v as u32;
                } else {
                    set.range = v as u32;
                }
            }
            "join" => {
                let v = e
                    .word(0)
                    .or_else(|_| e.string(0))
                    .map_err(|x| at(x.to_string()))?;
                set.join = Join::parse(v)
                    .filter(|_| matches!(v, "round" | "miter" | "bevel"))
                    .ok_or_else(|| at(format!("join \"{v}\": round, miter or bevel")))?;
            }
            "icon" => {
                let name = e.string(0).map_err(|x| at(x.to_string()))?.to_owned();
                let file = match e.values.len() {
                    1 => name.clone(),
                    _ => e.string(1).map_err(|x| at(x.to_string()))?.to_owned(),
                };
                if name.is_empty() || name.contains('\n') {
                    return Err(at("an icon's name is one non-empty line".into()));
                }
                if set.icons.iter().any(|(n, _)| *n == name) {
                    return Err(at(format!("icon \"{name}\" is named twice")));
                }
                if file.split('/').any(|c| c == ".." || c.is_empty()) {
                    return Err(at(format!("\"{file}\" is not a file under the source")));
                }
                set.icons.push((name, file));
            }
            other => {
                return Err(at(format!(
                    "`{other}` is not an icon set line (source, size, range, join, icon)"
                )));
            }
        }
    }
    if set.icons.is_empty() {
        return Err("an icon set names at least one `icon`".into());
    }
    if set.icons.len() > 0x1900 {
        return Err("an icon set holds at most 6,400 icons (the Private Use Area)".into());
    }
    Ok(set)
}

/// The directory the set's SVGs are read from.
fn source_dir(s: &Source, set: &IconSet) -> std::path::PathBuf {
    let mut dir = s.path.clone();
    dir.pop();
    dir.join(&set.source)
}

/// What the cook stamp covers besides the file: each SVG's bytes.
pub(super) fn dependencies(s: &Source) -> String {
    let Ok(set) = parse(&String::from_utf8_lossy(&s.bytes)) else {
        return String::new();
    };
    let dir = source_dir(s, &set);
    set.icons
        .iter()
        .map(|(_, f)| {
            let h = std::fs::read(dir.join(format!("{f}.svg")))
                .map(|b| super::hash_bytes(&b))
                .unwrap_or(0);
            format!("{f}={h:016x}")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A flattened subpath: its points, and whether it closes.
type Subpath = (Vec<(f32, f32)>, bool);

/// One drawn shape of an icon, in view-box units (y down).
struct Shape {
    /// Flattened subpaths: points, closed.
    subpaths: Vec<Subpath>,
    /// Half the stroke's width; 0: not stroked.
    half: f32,
    /// Butt (or square) caps instead of round.
    butt: bool,
    /// How its segments meet, and the miter limit (a miter longer than
    /// `limit` stroke widths is bevelled).
    join: Join,
    limit: f32,
    filled: bool,
}

/// Pieces per curve: well under a tenth of a pixel at 48 px per 24 units.
const CURVE_STEPS: u32 = 16;

/// The value of attribute `name` in a start tag's text.
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = tag;
    while let Some(i) = rest.find(name) {
        let before = rest[..i].chars().last();
        let after = &rest[i + name.len()..];
        if before.is_some_and(char::is_whitespace) && after.trim_start().starts_with('=') {
            let v = after.trim_start()[1..].trim_start();
            let q = v.chars().next()?;
            if q == '"' || q == '\'' {
                let v = &v[1..];
                return v.find(q).map(|end| &v[..end]);
            }
        }
        rest = &rest[i + name.len()..];
    }
    None
}

/// The numbers of a path's data, after its command letters were split off.
fn numbers(s: &str) -> Result<Vec<f32>, String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let flush = |cur: &mut String, out: &mut Vec<f32>| -> Result<(), String> {
        if !cur.is_empty() {
            out.push(
                cur.parse::<f32>()
                    .map_err(|_| format!("'{cur}' is not a number"))?,
            );
            cur.clear();
        }
        Ok(())
    };
    for c in s.chars() {
        match c {
            '0'..='9' => cur.push(c),
            '.' if cur.contains('.') && !cur.contains(['e', 'E']) => {
                flush(&mut cur, &mut out)?;
                cur.push('.');
            }
            '.' | 'e' | 'E' => cur.push(c),
            '-' | '+' if !cur.ends_with(['e', 'E']) => {
                flush(&mut cur, &mut out)?;
                cur.push(c);
            }
            '-' | '+' => cur.push(c),
            _ => flush(&mut cur, &mut out)?,
        }
    }
    flush(&mut cur, &mut out)?;
    Ok(out)
}

/// A path's data flattened into subpaths.
fn flatten(d: &str) -> Result<Vec<Subpath>, String> {
    let mut out: Vec<Subpath> = Vec::new();
    let mut cur: Vec<(f32, f32)> = Vec::new();
    let (mut at, mut start) = ((0.0f32, 0.0f32), (0.0f32, 0.0f32));
    // The last curve's second control point, for S and T.
    let mut last_c: Option<(f32, f32)> = None;
    let mut last_q: Option<(f32, f32)> = None;
    let mut i = 0;
    let bytes: Vec<char> = d.chars().collect();
    while i < bytes.len() {
        let c = bytes[i];
        if !c.is_ascii_alphabetic() {
            if c.is_whitespace() || c == ',' {
                i += 1;
                continue;
            }
            return Err(format!("path data starts with '{c}', not a command"));
        }
        let mut j = i + 1;
        while j < bytes.len()
            && !(bytes[j].is_ascii_alphabetic() && bytes[j] != 'e' && bytes[j] != 'E')
        {
            j += 1;
        }
        let args = numbers(&bytes[i + 1..j].iter().collect::<String>())?;
        let rel = c.is_ascii_lowercase();
        let base = |at: (f32, f32), x: f32, y: f32| if rel { (at.0 + x, at.1 + y) } else { (x, y) };
        let arity = match c.to_ascii_uppercase() {
            'M' | 'L' | 'T' => 2,
            'H' | 'V' => 1,
            'C' => 6,
            'S' | 'Q' => 4,
            'Z' => 0,
            'A' => return Err("arcs (A) are not supported: convert them to curves".into()),
            other => return Err(format!("'{other}' is not a path command")),
        };
        if arity == 0 {
            if !cur.is_empty() {
                out.push((std::mem::take(&mut cur), true));
            }
            at = start;
            last_c = None;
            last_q = None;
            i = j;
            continue;
        }
        if args.is_empty() || args.len() % arity != 0 {
            return Err(format!(
                "'{c}' takes numbers in groups of {arity}, got {}",
                args.len()
            ));
        }
        for (k, a) in args.chunks(arity).enumerate() {
            let cmd = if k > 0 && c.eq_ignore_ascii_case(&'M') {
                if rel { 'l' } else { 'L' }
            } else {
                c
            };
            let (mut nc, mut nq) = (None, None);
            match cmd.to_ascii_uppercase() {
                'M' => {
                    if cur.len() > 1 {
                        out.push((std::mem::take(&mut cur), false));
                    }
                    cur.clear();
                    at = base(at, a[0], a[1]);
                    start = at;
                    cur.push(at);
                }
                'L' => {
                    at = base(at, a[0], a[1]);
                    cur.push(at);
                }
                'H' => {
                    at = (if rel { at.0 + a[0] } else { a[0] }, at.1);
                    cur.push(at);
                }
                'V' => {
                    at = (at.0, if rel { at.1 + a[0] } else { a[0] });
                    cur.push(at);
                }
                'C' | 'S' => {
                    let (p1, p2, p3) = if cmd.eq_ignore_ascii_case(&'C') {
                        (
                            base(at, a[0], a[1]),
                            base(at, a[2], a[3]),
                            base(at, a[4], a[5]),
                        )
                    } else {
                        let p1 = last_c.map_or(at, |c| (2.0 * at.0 - c.0, 2.0 * at.1 - c.1));
                        (p1, base(at, a[0], a[1]), base(at, a[2], a[3]))
                    };
                    let p0 = at;
                    for s in 1..=CURVE_STEPS {
                        let t = s as f32 / CURVE_STEPS as f32;
                        let u = 1.0 - t;
                        let (b0, b1, b2, b3) =
                            (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                        cur.push(if s == CURVE_STEPS {
                            p3
                        } else {
                            (
                                b0 * p0.0 + b1 * p1.0 + b2 * p2.0 + b3 * p3.0,
                                b0 * p0.1 + b1 * p1.1 + b2 * p2.1 + b3 * p3.1,
                            )
                        });
                    }
                    at = p3;
                    nc = Some(p2);
                }
                'Q' | 'T' => {
                    let (p1, p2) = if cmd.eq_ignore_ascii_case(&'Q') {
                        (base(at, a[0], a[1]), base(at, a[2], a[3]))
                    } else {
                        let p1 = last_q.map_or(at, |q| (2.0 * at.0 - q.0, 2.0 * at.1 - q.1));
                        (p1, base(at, a[0], a[1]))
                    };
                    let p0 = at;
                    for s in 1..=CURVE_STEPS {
                        let t = s as f32 / CURVE_STEPS as f32;
                        let u = 1.0 - t;
                        cur.push(if s == CURVE_STEPS {
                            p2
                        } else {
                            (
                                u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0,
                                u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1,
                            )
                        });
                    }
                    at = p2;
                    nq = Some(p1);
                }
                _ => unreachable!("arity checked above"),
            }
            last_c = nc;
            last_q = nq;
        }
        i = j;
    }
    if cur.len() > 1 {
        out.push((cur, false));
    }
    Ok(out)
}

/// The shapes of an SVG icon and its view box (x, y, width, height); a
/// stroked path that names no join takes `join`.
fn shapes(svg: &str, join: Join) -> Result<(Vec<Shape>, [f32; 4]), String> {
    let open = svg.find("<svg").ok_or("no <svg> element")?;
    let svg_tag = &svg[open..open + svg[open..].find('>').ok_or("an unclosed <svg> tag")?];
    let view = match attr(svg_tag, "viewBox") {
        Some(v) => {
            let n = numbers(v)?;
            if n.len() != 4 || n[2] <= 0.0 || n[3] <= 0.0 {
                return Err(format!("viewBox \"{v}\" is not four numbers with a size"));
            }
            [n[0], n[1], n[2], n[3]]
        }
        None => [0.0, 0.0, 24.0, 24.0],
    };
    fn inherit<'a>(tag: &'a str, svg: &'a str, name: &str) -> Option<&'a str> {
        attr(tag, name).or_else(|| attr(svg, name))
    }
    let mut out = Vec::new();
    let mut rest = &svg[open + svg_tag.len()..];
    while let Some(i) = rest.find('<') {
        rest = &rest[i + 1..];
        let end = rest.find('>').ok_or("an unclosed tag")?;
        let tag = &rest[..end];
        rest = &rest[end + 1..];
        let name: String = tag
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        match name.as_str() {
            "" => continue, // a closing tag, a comment
            "path" => {
                let d = attr(tag, "d").ok_or("a <path> without d")?;
                let tag = &format!(" {tag}");
                let stroke = inherit(tag, svg_tag, "stroke").is_some_and(|v| v != "none");
                let fill = inherit(tag, svg_tag, "fill").is_none_or(|v| v != "none");
                let width = match inherit(tag, svg_tag, "stroke-width") {
                    Some(w) => numbers(w)?
                        .first()
                        .copied()
                        .ok_or("an empty stroke-width")?,
                    None => 1.0,
                };
                let cap = inherit(tag, svg_tag, "stroke-linecap").unwrap_or("butt");
                let join = match inherit(tag, svg_tag, "stroke-linejoin") {
                    Some(v) => Join::parse(v)
                        .ok_or_else(|| format!("stroke-linejoin \"{v}\" is not a join"))?,
                    None => join,
                };
                let limit = match inherit(tag, svg_tag, "stroke-miterlimit") {
                    Some(v) => numbers(v)?
                        .first()
                        .copied()
                        .filter(|l| *l >= 1.0)
                        .ok_or("a stroke-miterlimit is a number of at least 1")?,
                    None => 4.0,
                };
                out.push(Shape {
                    subpaths: flatten(d)?,
                    half: if stroke { width / 2.0 } else { 0.0 },
                    butt: cap != "round",
                    join,
                    limit,
                    filled: fill,
                });
            }
            "g" | "defs" | "title" | "desc" => {
                if attr(&format!(" {tag}"), "transform").is_some() {
                    return Err(format!("<{name}> with a transform is not supported"));
                }
            }
            other => return Err(format!("<{other}> is not supported (paths only)")),
        }
    }
    Ok((out, view))
}

/// Distance from `p` to segment `a`-`b`, and whether the nearest point is
/// `a` (t at 0) or `b` (t at 1).
fn segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = dx * dx + dy * dy;
    let t = if len > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (ex, ey) = (a.0 + t * dx - p.0, a.1 + t * dy - p.1);
    ((ex * ex + ey * ey).sqrt(), t)
}

/// The signed distance (negative inside) from `p` to a convex polygon
/// (its corners in order, either way round).
fn convex(p: (f32, f32), pts: &[(f32, f32)]) -> f32 {
    let n = pts.len();
    let mut d = f32::MAX;
    let (mut pos, mut neg) = (false, false);
    for k in 0..n {
        let (a, b) = (pts[k], pts[(k + 1) % n]);
        d = d.min(segment(p, a, b).0);
        let c = (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
        pos |= c > 0.0;
        neg |= c < 0.0;
    }
    if pos && neg { d } else { -d }
}

/// The signed distance from `p` to a stroke whose segments meet in miter
/// or bevel joins: the union of each segment's band (cut square at its
/// ends), the joins' wedges on the outer side of each corner (to the miter
/// point within the limit, else across) and round caps' discs.
fn stroke_joined(p: (f32, f32), pts: &[(f32, f32)], closed: bool, s: &Shape) -> f32 {
    // Corners without the zero-length segments a path can draw (a curve
    // ending on its start, then Z).
    let mut v: Vec<(f32, f32)> = Vec::with_capacity(pts.len());
    for &q in pts {
        if v.last()
            .is_none_or(|l| (l.0 - q.0).abs() + (l.1 - q.1).abs() > 1e-6)
        {
            v.push(q);
        }
    }
    if closed && v.len() > 2 {
        let (f, l) = (v[0], v[v.len() - 1]);
        if (f.0 - l.0).abs() + (f.1 - l.1).abs() <= 1e-6 {
            v.pop();
        }
    }
    let h = s.half;
    let n = v.len();
    if n < 2 {
        return v.first().map_or(f32::MAX, |c| {
            if s.butt {
                f32::MAX
            } else {
                ((p.0 - c.0).powi(2) + (p.1 - c.1).powi(2)).sqrt() - h
            }
        });
    }
    let edges = if closed { n } else { n - 1 };
    let dir = |k: usize| {
        let (a, b) = (v[k % n], v[(k + 1) % n]);
        let l = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        ((b.0 - a.0) / l, (b.1 - a.1) / l)
    };
    let mut best = f32::MAX;
    for k in 0..edges {
        let (a, b) = (v[k], v[(k + 1) % n]);
        let t = dir(k);
        let nrm = (-t.1 * h, t.0 * h);
        best = best.min(convex(
            p,
            &[
                (a.0 + nrm.0, a.1 + nrm.1),
                (b.0 + nrm.0, b.1 + nrm.1),
                (b.0 - nrm.0, b.1 - nrm.1),
                (a.0 - nrm.0, a.1 - nrm.1),
            ],
        ));
    }
    // The joins: at every corner of a closed path, the inner ones of an open one.
    let corners: Vec<usize> = if closed {
        (0..n).collect()
    } else {
        (1..n - 1).collect()
    };
    for c in corners {
        let t1 = dir((c + n - 1) % n);
        let t2 = dir(c);
        let cross = t1.0 * t2.1 - t1.1 * t2.0;
        let dot = t1.0 * t2.0 + t1.1 * t2.1;
        if cross.abs() < 1e-6 && dot > 0.0 {
            continue; // straight on
        }
        // The outer side of the turn, and the bands' corners there.
        let side = if cross > 0.0 { -1.0 } else { 1.0 };
        let (n1, n2) = ((-t1.1, t1.0), (-t2.1, t2.0));
        let q = v[c];
        let p1 = (q.0 + side * n1.0 * h, q.1 + side * n1.1 * h);
        let p2 = (q.0 + side * n2.0 * h, q.1 + side * n2.1 * h);
        // The miter's length over the stroke's width: 1 / cos(α/2), α the
        // angle between the normals.
        let cos_half = ((1.0 + n1.0 * n2.0 + n1.1 * n2.1) / 2.0).max(0.0).sqrt();
        if s.join == Join::Miter && cos_half > 0.0 && 1.0 / cos_half <= s.limit {
            let m = (n1.0 + n2.0, n1.1 + n2.1);
            let ml = (m.0 * m.0 + m.1 * m.1).sqrt();
            let reach = h / cos_half;
            let tip = (q.0 + side * m.0 / ml * reach, q.1 + side * m.1 / ml * reach);
            best = best.min(convex(p, &[q, p1, tip, p2]));
        } else {
            best = best.min(convex(p, &[q, p1, p2]));
        }
    }
    // Round caps: a disc at each end of an open path.
    if !closed && !s.butt {
        for e in [v[0], v[n - 1]] {
            best = best.min(((p.0 - e.0).powi(2) + (p.1 - e.1).powi(2)).sqrt() - h);
        }
    }
    best
}

/// The signed distance (negative inside) from `p` to the icon, view-box units.
fn signed_distance(shapes: &[Shape], p: (f32, f32)) -> f32 {
    let mut best = f32::MAX;
    for s in shapes {
        if s.filled {
            let mut d = f32::MAX;
            let mut winding = 0i32;
            for (pts, _) in &s.subpaths {
                // Filled subpaths close implicitly.
                for k in 0..pts.len() {
                    let (a, b) = (pts[k], pts[(k + 1) % pts.len()]);
                    d = d.min(segment(p, a, b).0);
                    if (a.1 <= p.1) != (b.1 <= p.1) {
                        let x = a.0 + (p.1 - a.1) / (b.1 - a.1) * (b.0 - a.0);
                        if x > p.0 {
                            winding += if b.1 > a.1 { 1 } else { -1 };
                        }
                    }
                }
            }
            best = best.min(if winding != 0 { -d } else { d });
        }
        if s.half > 0.0 && s.join != Join::Round {
            for (pts, closed) in &s.subpaths {
                best = best.min(stroke_joined(p, pts, *closed, s));
            }
        } else if s.half > 0.0 {
            for (pts, closed) in &s.subpaths {
                let n = pts.len();
                let edges = if *closed { n } else { n - 1 };
                let (mut d, mut at_start, mut at_end) = (f32::MAX, false, false);
                for k in 0..edges {
                    let (e, t) = segment(p, pts[k], pts[(k + 1) % n]);
                    if e < d {
                        d = e;
                        at_start = !*closed && k == 0 && t == 0.0;
                        at_end = !*closed && k + 1 == edges && t == 1.0;
                    }
                }
                let mut sd = d - s.half;
                // A butt cap: cut square at the end the point lies beyond.
                if s.butt && (at_start || at_end) && n >= 2 {
                    let (p0, p1) = if at_start {
                        (pts[0], pts[1])
                    } else {
                        (pts[n - 1], pts[n - 2])
                    };
                    let (tx, ty) = (p1.0 - p0.0, p1.1 - p0.1);
                    let l = (tx * tx + ty * ty).sqrt();
                    if l > 0.0 {
                        let behind = -((p.0 - p0.0) * tx + (p.1 - p0.1) * ty) / l;
                        sd = sd.max(behind);
                    }
                }
                best = best.min(sd);
            }
        }
    }
    best
}

/// One icon rendered: `cell`×`cell` RGBA8 rows top to bottom, the em at
/// `pad` pixels in from each side.
fn render(shapes: &[Shape], view: [f32; 4], size: u32, range: u32, pad: u32) -> Vec<u8> {
    let cell = size + 2 * pad;
    let units = view[2].max(view[3]) / size as f32;
    let mut rgba = vec![0u8; (cell * cell * 4) as usize];
    for row in 0..cell {
        for col in 0..cell {
            let p = (
                view[0] + (col as f32 + 0.5 - pad as f32) * units,
                view[1] + (row as f32 + 0.5 - pad as f32) * units,
            );
            let sd_px = signed_distance(shapes, p) / units;
            let v = (0.5 - sd_px / range as f32).clamp(0.0, 1.0);
            let byte = (f64::from(v) * 255.0) as u8;
            let o = ((row * cell + col) * 4) as usize;
            rgba[o..o + 4].copy_from_slice(&[byte; 4]);
        }
    }
    rgba
}

/// Cooks an icon set into an `og.Font` blob with a `names` section.
pub(super) fn cook(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("an icon set is text (UTF-8)".into()))?;
    let set = parse(text).map_err(CookError)?;
    let dir = source_dir(s, &set);
    let pad = set.range.div_ceil(2) + 1;
    let cell = set.size + 2 * pad;
    let count = set.icons.len() as u32;
    let cols = (f64::from(count).sqrt().ceil() as u32).max(1);
    let rows = count.div_ceil(cols);
    let width = cols * (cell + 1) + 1;
    let height = (rows * (cell + 1) + 1).div_ceil(4) * 4;
    if width > 8192 || height > 8192 {
        return Err(CookError(format!(
            "{count} icons at size {} need a {width}x{height} page; lower size",
            set.size
        )));
    }
    let mut page = vec![0u8; (width * height * 4) as usize];
    let mut glyphs = Vec::with_capacity(set.icons.len());
    let edge = pad as f32 / set.size as f32;
    for (i, (name, file)) in set.icons.iter().enumerate() {
        let path = dir.join(format!("{file}.svg"));
        let svg = std::fs::read_to_string(&path)
            .map_err(|e| CookError(format!("icon \"{name}\": {}: {e}", path.display())))?;
        let (sh, view) = shapes(&svg, set.join)
            .map_err(|e| CookError(format!("icon \"{name}\" ({file}.svg): {e}")))?;
        let pixels = render(&sh, view, set.size, set.range, pad);
        let (gx, gy) = (
            1 + (i as u32 % cols) * (cell + 1),
            1 + (i as u32 / cols) * (cell + 1),
        );
        for row in 0..cell {
            let d0 = (((gy + row) * width + gx) * 4) as usize;
            let s0 = (row * cell * 4) as usize;
            page[d0..d0 + (cell * 4) as usize]
                .copy_from_slice(&pixels[s0..s0 + (cell * 4) as usize]);
        }
        glyphs.push(GlyphRow {
            codepoint: ICON_PUA + i as u32,
            page: 0,
            advance: 1.0,
            plane_left: -edge,
            plane_bottom: -edge,
            plane_right: 1.0 + edge,
            plane_top: 1.0 + edge,
            atlas_x: gx,
            atlas_y: gy,
            atlas_w: cell,
            atlas_h: cell,
        });
    }
    let meta = FontMeta {
        units_per_em: set.size,
        em_px: set.size as f32,
        px_range: set.range as f32,
        ascender: 1.0,
        descender: 0.0,
        line_gap: 0.0,
        glyph_count: count,
        page_count: 1,
        page_width: width,
        page_height: height,
        uncooked_codepoints: 0,
    };
    let names: Vec<&str> = set.icons.iter().map(|(n, _)| n.as_str()).collect();
    let mut w = BlobWriter::new(
        format::Font::type_hash(),
        format::Font::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], row_hash::<FontMeta>())
        .table("glyphs", &glyphs, row_hash::<GlyphRow>())
        .table("kerning", &Vec::<KernRow>::new(), row_hash::<KernRow>())
        .raw(
            "atlas.0",
            ktx2::write_rgba8_zlib(width, height, &page, false, false, format::COOKER_VERSION),
            16,
        )
        .raw("names", names.join("\n").into_bytes(), 16);
    Ok((
        w.finish(),
        vec![format!(
            "{count} icon(s) from {}/, {} px each, range {}: a {width}x{height} page",
            set.source, set.size, set.range
        )],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_data_flattens_with_relative_and_smooth_commands() {
        let p = flatten("M2 2h4v4H2z m10 0 l2 2").unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].0, vec![(2.0, 2.0), (6.0, 2.0), (6.0, 6.0), (2.0, 6.0)]);
        assert!(p[0].1, "z closes the first");
        // m after z starts from the closed subpath's start.
        assert_eq!(p[1].0, vec![(12.0, 2.0), (14.0, 4.0)]);
        assert!(!p[1].1);
        assert_eq!(
            numbers("1.5-2.25.5e1,3").unwrap(),
            vec![1.5, -2.25, 5.0, 3.0]
        );
        assert!(flatten("M0 0A1 1 0 0 0 2 2").is_err(), "arcs are refused");
    }

    #[test]
    fn a_round_stroke_is_its_width_and_a_butt_end_is_cut() {
        let svg = r#"<svg viewBox="0 0 24 24" stroke-width="2" fill="none"><path d="M4 12H20" stroke="currentColor" stroke-linecap="round"/></svg>"#;
        let (sh, view) = shapes(svg, Join::Round).unwrap();
        assert_eq!(view, [0.0, 0.0, 24.0, 24.0]);
        assert_eq!(
            signed_distance(&sh, (12.0, 12.0)),
            -1.0,
            "the middle is 1 unit inside"
        );
        assert_eq!(signed_distance(&sh, (12.0, 15.0)), 2.0);
        assert_eq!(
            signed_distance(&sh, (2.0, 12.0)),
            1.0,
            "a round cap reaches past the end"
        );
        let butt = svg.replace(r#" stroke-linecap="round""#, "");
        let (sh, _) = shapes(&butt, Join::Round).unwrap();
        assert_eq!(
            signed_distance(&sh, (2.0, 12.0)),
            2.0,
            "a butt cap ends at the end"
        );
        // A filled square: inside negative.
        let fill =
            r#"<svg viewBox="0 0 24 24"><path d="M4 4H20V20H4Z" fill="currentColor"/></svg>"#;
        let (sh, _) = shapes(fill, Join::Round).unwrap();
        assert_eq!(signed_distance(&sh, (12.0, 12.0)), -8.0);
        assert!(shapes(r#"<svg><circle r="2"/></svg>"#, Join::Miter).is_err());
    }

    /// An L of stroke width 2 turning at (4, 4): its outer corner as each
    /// join draws it.
    fn corner(join: &str) -> Vec<Shape> {
        let svg = format!(
            r#"<svg viewBox="0 0 24 24" stroke-width="2" fill="none"><path d="M4 20V4H20" stroke="currentColor" {join}/></svg>"#
        );
        shapes(&svg, Join::Miter).unwrap().0
    }

    #[test]
    fn a_miter_join_keeps_its_corner_and_a_bevel_cuts_it() {
        // Miter (SVG's default): the outer corner reaches (3, 3), sharp.
        let miter = corner("");
        assert!(
            signed_distance(&miter, (3.0, 3.0)).abs() < 1e-4,
            "the point is on the outline"
        );
        assert!(
            signed_distance(&miter, (3.2, 3.2)) < -0.19,
            "inside the corner"
        );
        assert!(
            (signed_distance(&miter, (2.0, 3.0)) - 1.0).abs() < 1e-4,
            "one unit out of the side"
        );
        // The outline beyond the corner's point is a distance to that point:
        // the zero line meets it at a right angle, not an arc.
        let d = signed_distance(&miter, (2.0, 2.0));
        assert!((d - 2f32.sqrt()).abs() < 1e-4, "{d}");
        // Round: the arc of radius 1 around (4, 4); the corner point is out.
        let round = corner(r#"stroke-linejoin="round""#);
        assert!((signed_distance(&round, (3.0, 3.0)) - (2f32.sqrt() - 1.0)).abs() < 1e-4);
        // Bevel: cut across from (3, 4) to (4, 3).
        let bevel = corner(r#"stroke-linejoin="bevel""#);
        assert!(signed_distance(&bevel, (3.2, 3.2)) > 0.0);
        assert!(signed_distance(&bevel, (3.6, 3.6)) < 0.0);
        // The rest of the stroke is the same whatever the join.
        for p in [
            (4.0, 12.0),
            (5.5, 12.0),
            (12.0, 2.5),
            (20.5, 4.0),
            (4.0, 21.5),
        ] {
            let (m, r) = (signed_distance(&miter, p), signed_distance(&round, p));
            assert!((m - r).abs() < 1e-4, "{p:?}: miter {m}, round {r}");
        }
        // Past the miter limit a sharp turn is bevelled.
        let sharp = r#"<svg viewBox="0 0 24 24" stroke-width="2" fill="none"><path d="M4 20L12 4L20 20" stroke="currentColor"/></svg>"#;
        let (sh, _) = shapes(sharp, Join::Miter).unwrap();
        // The miter point is 1/sin(θ/2) ≈ 2.24 half-widths over the apex,
        // within the limit 4: drawn.
        assert!(signed_distance(&sh, (12.0, 4.0 - 2.0)) < 0.0);
        let limited = sharp.replace(
            r#"stroke="currentColor""#,
            r#"stroke="currentColor" stroke-miterlimit="2""#,
        );
        let (sh, _) = shapes(&limited, Join::Miter).unwrap();
        assert!(
            signed_distance(&sh, (12.0, 4.0 - 2.0)) > 0.0,
            "bevelled past the limit"
        );
    }

    #[test]
    fn the_rendered_field_keeps_a_miter_corner_sharp() {
        // At 48 px an em of 24 units (2 px a unit), the texel over the
        // miter's point reads on the outline (0.5) and the one inside the
        // corner well within; round, the corner texel falls outside.
        let px = |sh: &[Shape], x: u32, y: u32| {
            let rgba = render(sh, [0.0, 0.0, 24.0, 24.0], 48, 8, 0);
            f32::from(rgba[((y * 48 + x) * 4) as usize]) / 255.0
        };
        // Texel (6, 6) covers units 3..3.5: its centre (3.25, 3.25).
        let (miter, round) = (corner(""), corner(r#"stroke-linejoin="round""#));
        assert!(
            px(&miter, 6, 6) > 0.5,
            "inside the miter: {}",
            px(&miter, 6, 6)
        );
        assert!(
            px(&round, 6, 6) < 0.5,
            "outside the round join: {}",
            px(&round, 6, 6)
        );
    }

    #[test]
    fn a_set_declares_the_join_its_paths_leave_out() {
        let set = parse("type icons\nversion 1\n---\njoin round\nicon \"a\"\n").unwrap();
        assert_eq!(set.join, Join::Round);
        assert_eq!(
            parse("type icons\nversion 1\n---\nicon \"a\"\n")
                .unwrap()
                .join,
            Join::Miter
        );
        assert!(parse("type icons\nversion 1\n---\njoin arcs\nicon \"a\"\n").is_err());
        let svg = r#"<svg viewBox="0 0 24 24" stroke-width="2" fill="none"><path d="M4 20V4H20" stroke="currentColor"/></svg>"#;
        let (sh, _) = shapes(svg, Join::Round).unwrap();
        assert!(
            signed_distance(&sh, (3.2, 3.2)) > 0.0,
            "the set's round join"
        );
        let named = svg.replace("<path ", r#"<path stroke-linejoin="miter" "#);
        let (sh, _) = shapes(&named, Join::Round).unwrap();
        assert!(
            signed_distance(&sh, (3.2, 3.2)) < 0.0,
            "a path's own join wins"
        );
    }
}
