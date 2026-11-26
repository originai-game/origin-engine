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
//! SVG support is what line icons use: `<path>` elements with path data
//! (kurbo's parser: every command, arcs included), `stroke` with `stroke-width`,
//! `stroke-linecap` (`round`, `butt`, `square` taken as butt),
//! `stroke-linejoin` (`round`, `miter`, `miter-clip` and `arcs` taken as
//! miter, `bevel`), `stroke-miterlimit` and `fill` (`currentColor` or any
//! colour fills by the nonzero rule; `none` does not), inherited from the
//! `<svg>` element. Anything else is refused with the icon and element
//! named.
//!
//! An icon is drawn in tones when a path's `fill` or `stroke` is a theme
//! colour, `var(--color-<role>)`: each run of consecutive paths in one
//! tone (the icon's ink for any other paint) is a glyph of its own, past
//! the icons' glyphs, and the icon's own glyph is its silhouette, every
//! path together (what a host that draws one tone draws). The `tones`
//! section lists them, one line an icon, `<icon> <glyph>:<role> …` in
//! paint order (an empty role: the ink), after the set's line
//! `sticker <outline> <dx> <dy>`: a toned icon is drawn over its
//! silhouette grown by `outline` in the `icon-outline` colour and its
//! silhouette moved by `dx`, `dy` in `icon-shadow`, each in em (the
//! set's `sticker` line gives them in 24ths of the em, a 24-unit grid's
//! units; 0 0 0 when it has none).

use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::format::{self, *};
use crate::ktx2;
use og_blob::BlobWriter;
use og_core::ogt::Document;

/// The `.ogt` type of an icon set.
pub const ICONS_TYPE: &str = "icons";
/// The code point of the first icon.
pub const ICON_PUA: u32 = 0xE000;

pub(crate) const LINES: &[crate::grammar::Line] = &[
    crate::grammar::Line::new(
        "source",
        "\"<directory>\"",
        ".",
        "SVG directory, relative to this file.",
        "",
    ),
    crate::grammar::Line::new(
        "size",
        "<integer, 1..512>",
        "48",
        "Distance-field em size in pixels.",
        "",
    ),
    crate::grammar::Line::new(
        "range",
        "<integer, 1..512>",
        "8",
        "Distance-field range in pixels.",
        "",
    ),
    crate::grammar::Line::new(
        "join",
        "miter | round | bevel",
        "miter",
        "Join used by stroked SVG paths without their own join.",
        "",
    ),
    crate::grammar::Line::new(
        "icon",
        "\"<name>\" [\"<SVG stem>\"]",
        "",
        "Repeat named glyphs in index order; omitted stem uses name. Names must be unique.",
        "",
    ),
    crate::grammar::Line::new(
        "sticker",
        "<outline> <dx> <dy>",
        "0 0 0",
        "Outline 0..4 and offsets -4..4, in 24ths of the em.",
        "",
    ),
];

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
    /// A toned icon's outline width and shadow offset, in 24ths of the em.
    pub sticker: [f32; 3],
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
        sticker: [0.0; 3],
    };
    for e in &doc.body.entries {
        let at = |m: String| format!("line {}: {m}", e.line);
        if !LINES.iter().any(|f| f.key == e.key) {
            return Err(at(format!("unknown icons field '{}'", e.key)));
        }
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
            "sticker" => {
                let mut v = [0.0; 3];
                for (k, x) in v.iter_mut().enumerate() {
                    *x = e.f32(k).map_err(|x| at(x.to_string()))?;
                }
                if !(0.0..=4.0).contains(&v[0]) || v[1].abs() > 4.0 || v[2].abs() > 4.0 {
                    return Err(at(
                        "sticker <outline> <dx> <dy>: 24ths of the em, 0 to 4 (an offset -4 to 4)"
                            .into(),
                    ));
                }
                set.sticker = v;
            }
            other => {
                return Err(at(format!(
                    "`{other}` is not an icon set line (source, size, range, join, sticker, icon)"
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
pub(crate) fn dependencies(s: &Source) -> String {
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
    /// The theme colour it is painted in (`var(--color-<role>)`); empty:
    /// the icon's ink.
    tone: String,
}

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

/// A path's data (kurbo's parser, arcs included) flattened into
/// subpaths within a hundredth of a unit: a fiftieth of a pixel at 48 px
/// per 24 units.
fn flatten(d: &str) -> Result<Vec<Subpath>, String> {
    let path = og_draw::parse_svg_path(d)?;
    Ok(og_draw::polylines(&path, 0.01)
        .into_iter()
        .map(|(pts, closed)| (pts.into_iter().map(|p| (p.x, p.y)).collect(), closed))
        .collect())
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
                // Its tone: the theme colour its paint names. A path that
                // fills and strokes paints both in one.
                fn role(v: Option<&str>) -> Option<&str> {
                    v.and_then(|v| v.trim().strip_prefix("var(--color-")?.strip_suffix(')'))
                        .map(str::trim)
                }
                let fill_role = role(inherit(tag, svg_tag, "fill")).filter(|_| fill);
                let stroke_role = role(inherit(tag, svg_tag, "stroke")).filter(|_| stroke);
                let tone = match (fill_role, stroke_role) {
                    (Some(f), Some(s)) if f != s => {
                        return Err(format!(
                            "a <path> fills in {f} and strokes in {s}: one tone a path"
                        ));
                    }
                    (Some(r), _) | (_, Some(r)) => {
                        if r.is_empty() || !r.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                        {
                            return Err(format!("var(--color-{r}) does not name a colour role"));
                        }
                        r.to_owned()
                    }
                    (None, None) => String::new(),
                };
                out.push(Shape {
                    subpaths: flatten(d)?,
                    half: if stroke { width / 2.0 } else { 0.0 },
                    butt: cap != "round",
                    join,
                    limit,
                    filled: fill,
                    tone,
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

/// A toned icon's glyphs: one a run of consecutive shapes in one tone,
/// numbered from `first`, with its tone.
fn runs_of(shapes: &[Shape], first: u32) -> Vec<(u32, String)> {
    shapes
        .chunk_by(|a, b| a.tone == b.tone)
        .zip(first..)
        .map(|(run, g)| (g, run[0].tone.clone()))
        .collect()
}

/// The `tones` section: the sticker (given in 24ths of the em, written in
/// em), then each toned icon's glyphs and tones.
fn tones_section(sticker: [f32; 3], runs: &[(u32, Vec<(u32, String)>)]) -> String {
    let [o, dx, dy] = sticker.map(|v| v / 24.0);
    let mut lines = vec![format!("sticker {o} {dx} {dy}")];
    lines.extend(runs.iter().map(|(i, tones)| {
        let t: Vec<String> = tones.iter().map(|(g, r)| format!("{g}:{r}")).collect();
        format!("{i} {}", t.join(" "))
    }));
    lines.join("\n")
}

/// Cooks an icon set into an `og.Font` blob with a `names` section.
pub(crate) fn cook(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("an icon set is text (UTF-8)".into()))?;
    let set = parse(text).map_err(CookError)?;
    let dir = source_dir(s, &set);
    let pad = set.range.div_ceil(2) + 1;
    let cell = set.size + 2 * pad;
    let count = set.icons.len() as u32;
    // Every icon's field (its silhouette), then each toned icon's runs of
    // one tone, a glyph each, and the `tones` lines naming them.
    // Each icon renders on its own (a field per pixel against every edge,
    // the cook's slowest step), on every core; the page is laid out in
    // the set's order after, so the output does not depend on the threads.
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let rendered = crate::cook::each(&set.icons, threads, |(name, file)| {
        let path = dir.join(format!("{file}.svg"));
        let svg = std::fs::read_to_string(&path)
            .map_err(|e| CookError(format!("icon \"{name}\": {}: {e}", path.display())))?;
        let (sh, view) = shapes(&svg, set.join)
            .map_err(|e| CookError(format!("icon \"{name}\" ({file}.svg): {e}")))?;
        let field = render(&sh, view, set.size, set.range, pad);
        let toned: Vec<Vec<u8>> = if sh.iter().all(|s| s.tone.is_empty()) {
            Vec::new()
        } else {
            sh.chunk_by(|a, b| a.tone == b.tone)
                .map(|run| render(run, view, set.size, set.range, pad))
                .collect()
        };
        Ok::<_, CookError>((field, sh, toned))
    });
    let mut fields = Vec::with_capacity(set.icons.len());
    let mut runs: Vec<(u32, Vec<(u32, String)>)> = Vec::new();
    let mut extra = Vec::new();
    for (i, r) in rendered.into_iter().enumerate() {
        let (field, sh, toned) = r?;
        fields.push(field);
        if toned.is_empty() {
            continue;
        }
        runs.push((i as u32, runs_of(&sh, count + extra.len() as u32)));
        extra.extend(toned);
    }
    let total = count + extra.len() as u32;
    let cols = (f64::from(total).sqrt().ceil() as u32).max(1);
    let rows = total.div_ceil(cols);
    let width = cols * (cell + 1) + 1;
    let height = (rows * (cell + 1) + 1).div_ceil(4) * 4;
    if width > 8192 || height > 8192 {
        return Err(CookError(format!(
            "{total} icon glyphs at size {} need a {width}x{height} page; lower size",
            set.size
        )));
    }
    let mut page = vec![0u8; (width * height * 4) as usize];
    let mut glyphs = Vec::with_capacity(total as usize);
    let edge = pad as f32 / set.size as f32;
    for (i, pixels) in fields.iter().chain(&extra).enumerate() {
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
        glyph_count: total,
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
            ktx2::write_rgba8_zlib(width, height, &page, false, false, format::KTX_WRITER),
            16,
        )
        .raw("names", names.join("\n").into_bytes(), 16);
    if !runs.is_empty() {
        w.raw("tones", tones_section(set.sticker, &runs).into_bytes(), 16);
    }
    Ok((
        w.finish(),
        vec![format!(
            "{count} icon(s), {} in tones, from {}/, {} px each, range {}: a {width}x{height} page",
            runs.len(),
            set.source,
            set.size,
            set.range
        )],
    ))
}

pub(crate) static ICONS: KindDef = KindDef {
    name: "icons",
    extensions: &[],
    types: &[ICONS_TYPE],
    by_content: false,
    streams: false,
    plan: plan_icons,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&ICONS];

fn plan_icons<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // The stamp covers every SVG the set names.
    let deps = dependencies(s);
    let st = stamp(&[Area::Font], &[b"icons", &hash, deps.as_bytes()]);
    outs.push((s.rel.clone(), format::Font::NAME, st, Box::new(IconsJob(s))));
}

/// An icon set's one output.
struct IconsJob<'a>(&'a Source);

impl Job for IconsJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook(self.0, p.stamp)
    }
}
