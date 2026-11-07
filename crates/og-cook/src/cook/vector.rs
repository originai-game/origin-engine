//! Vectors (`.ogt` of `type vector`, `docs/ui.md` §3.3): an SVG subset
//! cooked to triangles that og-render's UI pass draws with an analytic
//! antialiased edge (`og_render::UiVectorData`).
//!
//! ```text
//! type vector
//! version 1
//! ---
//! source "ui/emblem.svg"   # the SVG, an asset path (or: svg "<svg …>")
//! tolerance 0.05           # curves flattened to within this many view-box
//!                          # units (default: the view box's longer side / 2000)
//! ```
//!
//! The SVG subset: `<svg>` with a `viewBox` (or `width` and `height`);
//! `<g>`, `<path>` (every command, arcs included), `<rect>` (with `rx`,
//! `ry`), `<circle>`, `<ellipse>`, `<line>`, `<polyline>` and `<polygon>`;
//! `transform` (matrix, translate, scale, rotate, skewX, skewY) on any of
//! them; `fill` and `stroke` as a colour (`#rgb`, `#rgba`, `#rrggbb`,
//! `#rrggbbaa`, `rgb()`, `rgba()`, the CSS basic names, `currentColor`,
//! which is white so the draw's tint colours it, or `none`) or
//! `url(#id)` of a `<linearGradient>` or `<radialGradient>` in the file
//! (`gradientUnits`, `spreadMethod`, `href` for stops, up to 8 `<stop>`s
//! with `offset`, `stop-color` and `stop-opacity`; a `gradientTransform`
//! of translate and scale); `fill-rule`, `fill-opacity`, `stroke-width`,
//! `stroke-opacity`, `stroke-linecap`, `stroke-linejoin`,
//! `stroke-miterlimit`, `stroke-dasharray`, `opacity` (a group's
//! multiplies its children's alpha); each as an attribute or in `style`.
//! `<defs>`, `<title>`, `<desc>` and `<metadata>` are read past. Anything
//! else (text, images, `use`, clips, masks, filters, patterns) is refused
//! with the element named.
//!
//! Fills and strokes both become triangles in document order, so they
//! layer as the SVG does: a stroke is its outline (segments, joins and
//! caps) filled by the nonzero rule. The fill is cut into trapezoids
//! between the outline's vertices; each vertex on the outline carries its
//! outward miter, along which the UI pass moves it half a device pixel in,
//! and a fringe quad along every outline edge ramps the coverage from one
//! to nothing over the pixel that straddles it: the edge's antialiasing is
//! exact at any size the vector is drawn, with nothing cooked per size.

use super::{CookError, Source};
use crate::format::{self, *};
use og_blob::BlobWriter;
use og_core::glam::Vec2;
use og_core::ogt::{Document, Number, Value};
use og_render::{
    UiGradient, UiGradientKind, UiSpread, UiStop, UiVectorData, UiVectorPaint, UiVectorVertex,
    flatten_cubic,
};
use std::collections::HashMap;

mod paths;
mod tessellation;

use paths::*;
use tessellation::*;

/// The `.ogt` type of a vector.
pub const VECTOR_TYPE: &str = "vector";

/// The most triangles a vector cooks to.
const MAX_TRIANGLES: usize = 400_000;

/// A parsed vector file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VectorFile {
    /// The SVG's asset path (`source`).
    pub source: Option<String>,
    /// The SVG inline (`svg`).
    pub svg: Option<String>,
    pub tolerance: Option<f32>,
}

/// Parses a vector file.
pub fn parse(text: &str) -> Result<VectorFile, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != VECTOR_TYPE {
        return Err(format!("a vector is `type vector`, not `type {}`", doc.ty));
    }
    if doc.version != 1 {
        return Err(format!(
            "vector version {} (this engine reads 1)",
            doc.version
        ));
    }
    let mut out = VectorFile::default();
    for e in &doc.body.entries {
        let line = e.line;
        match e.key.as_str() {
            "source" | "svg" => {
                if out.source.is_some() || out.svg.is_some() {
                    return Err(format!("line {line}: one `source` or `svg` per vector"));
                }
                let s = e
                    .values
                    .first()
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("line {line}: `{}` takes a string", e.key))?
                    .to_string();
                if e.key == "source" {
                    out.source = Some(s);
                } else {
                    out.svg = Some(s);
                }
            }
            "tolerance" => {
                let t = e
                    .values
                    .first()
                    .and_then(Value::as_number)
                    .map(Number::as_f32)
                    .filter(|t| t.is_finite() && *t > 0.0)
                    .ok_or_else(|| format!("line {line}: `tolerance` takes a positive number"))?;
                out.tolerance = Some(t);
            }
            other => return Err(format!("line {line}: unknown vector entry `{other}`")),
        }
    }
    if out.source.is_none() && out.svg.is_none() {
        return Err("a vector needs `source` (an SVG's asset path) or `svg`".into());
    }
    Ok(out)
}

/// What the cook stamp covers besides the file: the SVG's bytes.
pub(super) fn dependencies(s: &Source) -> String {
    let Ok(file) = parse(&String::from_utf8_lossy(&s.bytes)) else {
        return String::new();
    };
    file.source
        .map(|src| super::read_sibling(s, &src).unwrap_or_default())
        .unwrap_or_default()
}

/// Cooks a vector into an `og.Vector` blob.
pub(super) fn cook(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text =
        std::str::from_utf8(&s.bytes).map_err(|_| CookError("a vector is text (UTF-8)".into()))?;
    let file = parse(text).map_err(CookError)?;
    let svg = match (&file.source, &file.svg) {
        (Some(src), _) => {
            super::read_sibling(s, src).map_err(|e| CookError(format!("source \"{src}\": {e}")))?
        }
        (None, Some(svg)) => svg.clone(),
        (None, None) => unreachable!("parse requires one"),
    };
    let data = tessellate(&svg, file.tolerance).map_err(CookError)?;
    let triangles = data.indices.len() / 3;
    Ok((
        write(&data, stamp),
        vec![format!(
            "{} triangle(s), {} paint(s), view box {}×{}",
            triangles,
            data.paints.len(),
            data.view_box[2],
            data.view_box[3]
        )],
    ))
}

/// The blob of a tessellated vector.
pub(crate) fn write(data: &UiVectorData, stamp: u64) -> Vec<u8> {
    let mut stops = Vec::new();
    let paints: Vec<VectorPaintRow> = data
        .paints
        .iter()
        .map(|p| match p {
            UiVectorPaint::Solid(c) => VectorPaintRow {
                r: c[0],
                g: c[1],
                b: c[2],
                a: c[3],
                ..VectorPaintRow::default()
            },
            UiVectorPaint::Gradient(g) => {
                let first = stops.len() as u32;
                stops.extend(g.stops.iter().map(|s| VectorStopRow {
                    offset: s.offset,
                    r: s.color[0],
                    g: s.color[1],
                    b: s.color[2],
                    a: s.color[3],
                }));
                VectorPaintRow {
                    kind: match g.kind {
                        UiGradientKind::Linear => 1,
                        UiGradientKind::Radial => 2,
                        UiGradientKind::Conic => 3,
                        UiGradientKind::Diamond => 4,
                    },
                    spread: match g.spread {
                        UiSpread::Pad => 0,
                        UiSpread::Repeat => 1,
                        UiSpread::Reflect => 2,
                    },
                    dither: u32::from(g.dither),
                    first_stop: first,
                    stop_count: g.stops.len() as u32,
                    from_x: g.from.x,
                    from_y: g.from.y,
                    to_x: g.to.x,
                    to_y: g.to.y,
                    ..VectorPaintRow::default()
                }
            }
        })
        .collect();
    let mut w = BlobWriter::new(
        format::Vector::type_hash(),
        format::Vector::schema_version(),
        stamp,
    );
    let [x, y, width, height] = data.view_box;
    w.table(
        "meta",
        &[VectorMeta {
            x,
            y,
            width,
            height,
            vertex_count: data.vertices.len() as u32,
            index_count: data.indices.len() as u32,
            paint_count: paints.len() as u32,
        }],
        format::row_hash::<VectorMeta>(),
    )
    .table("paints", &paints, format::row_hash::<VectorPaintRow>())
    .table("stops", &stops, format::row_hash::<VectorStopRow>())
    .raw(
        "vertices",
        bytemuck::cast_slice::<UiVectorVertex, u8>(&data.vertices).to_vec(),
        16,
    )
    .raw(
        "indices",
        bytemuck::cast_slice::<u32, u8>(&data.indices).to_vec(),
        16,
    );
    w.finish()
}

// ── the SVG document ─────────────────────────────────────────────────────

/// An element: its name, attributes (style declarations folded in) and
/// children.
#[derive(Debug, Default)]
struct Element {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Element>,
}

impl Element {
    fn get(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .rev()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.trim())
    }
}

/// The element tree of an SVG document (its root `<svg>`).
fn parse_xml(src: &str) -> Result<Element, String> {
    let mut stack: Vec<Element> = vec![Element::default()];
    let mut rest = src;
    while let Some(i) = rest.find('<') {
        rest = &rest[i..];
        if let Some(r) = rest.strip_prefix("<!--") {
            let end = r.find("-->").ok_or("an unclosed comment")?;
            rest = &r[end + 3..];
            continue;
        }
        if let Some(r) = rest.strip_prefix("<![CDATA[") {
            let end = r.find("]]>").ok_or("an unclosed CDATA section")?;
            rest = &r[end + 3..];
            continue;
        }
        if rest.starts_with("<?") || rest.starts_with("<!") {
            let end = rest.find('>').ok_or("an unclosed declaration")?;
            rest = &rest[end + 1..];
            continue;
        }
        let end = tag_end(rest).ok_or("an unclosed tag")?;
        let tag = &rest[1..end];
        rest = &rest[end + 1..];
        if let Some(name) = tag.strip_prefix('/') {
            let name = name.trim();
            let el = stack.pop().ok_or("a closing tag with nothing open")?;
            if el.name != name {
                return Err(format!("</{name}> closes <{}>", el.name));
            }
            stack
                .last_mut()
                .ok_or(format!("</{name}> closes nothing"))?
                .children
                .push(el);
            continue;
        }
        let (tag, closed) = match tag.strip_suffix('/') {
            Some(t) => (t, true),
            None => (tag, false),
        };
        let name_len = tag.find(|c: char| c.is_whitespace()).unwrap_or(tag.len());
        let name = tag[..name_len].to_string();
        let mut el = Element {
            name,
            attrs: attributes(&tag[name_len..])?,
            children: Vec::new(),
        };
        // `style` declarations override the attributes.
        if let Some(style) = el.get("style").map(str::to_string) {
            for decl in style.split(';') {
                if let Some((k, v)) = decl.split_once(':') {
                    el.attrs.push((k.trim().to_string(), v.trim().to_string()));
                }
            }
        }
        if closed {
            stack.last_mut().expect("the document").children.push(el);
        } else {
            stack.push(el);
        }
    }
    if stack.len() != 1 {
        return Err(format!(
            "<{}> is never closed",
            stack.last().map_or("", |e| e.name.as_str())
        ));
    }
    let doc = stack.pop().expect("the document");
    doc.children
        .into_iter()
        .find(|e| e.name == "svg")
        .ok_or_else(|| "no <svg> element".to_string())
}

/// The index of a tag's `>`, past quoted attribute values.
fn tag_end(s: &str) -> Option<usize> {
    let mut quote = None;
    for (i, c) in s.char_indices() {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(q), c) if c == q => quote = None,
            (None, '>') => return Some(i),
            _ => {}
        }
    }
    None
}

fn attributes(s: &str) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    let mut rest = s.trim_start();
    while !rest.is_empty() {
        let eq = rest
            .find('=')
            .ok_or_else(|| format!("an attribute without a value: `{}`", rest.trim()))?;
        let name = rest[..eq].trim().to_string();
        let v = rest[eq + 1..].trim_start();
        let q = v.chars().next().ok_or("an attribute without a value")?;
        if q != '"' && q != '\'' {
            return Err(format!("attribute `{name}`'s value is not quoted"));
        }
        let end = v[1..].find(q).ok_or("an unclosed attribute value")?;
        out.push((name, unescape(&v[1..1 + end])));
        rest = v[end + 2..].trim_start();
    }
    Ok(out)
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// The numbers in `s` (separated by spaces or commas, signs and exponents
/// as SVG writes them: `1-2` is two numbers, `.5.5` too).
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
            c if c.is_whitespace() || c == ',' => flush(&mut cur, &mut out)?,
            other => return Err(format!("'{other}' in a list of numbers")),
        }
    }
    flush(&mut cur, &mut out)?;
    Ok(out)
}

/// A length: a number, `px`, or a percentage of `whole`.
fn length(v: &str, whole: f32) -> Result<f32, String> {
    let v = v.trim();
    if let Some(p) = v.strip_suffix('%') {
        return p
            .trim()
            .parse::<f32>()
            .map(|p| p / 100.0 * whole)
            .map_err(|_| format!("'{v}' is not a length"));
    }
    v.trim_end_matches("px")
        .parse::<f32>()
        .map_err(|_| format!("'{v}' is not a length (units other than px are not supported)"))
}

fn length_attr(el: &Element, name: &str, whole: f32, default: f32) -> Result<f32, String> {
    el.get(name)
        .map_or(Ok(default), |v| length(v, whole))
        .map_err(|e| format!("<{}> {name}: {e}", el.name))
}

// ── transforms ───────────────────────────────────────────────────────────

/// An affine transform `[a, b, c, d, e, f]` as SVG's `matrix`: x' = a·x +
/// c·y + e, y' = b·x + d·y + f.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Affine([f32; 6]);

impl Affine {
    const IDENTITY: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    fn apply(&self, p: Vec2) -> Vec2 {
        let [a, b, c, d, e, f] = self.0;
        Vec2::new(a * p.x + c * p.y + e, b * p.x + d * p.y + f)
    }

    /// `self` after `inner` (inner applies first).
    fn then(&self, inner: &Affine) -> Affine {
        let [a, b, c, d, e, f] = self.0;
        let [a2, b2, c2, d2, e2, f2] = inner.0;
        Affine([
            a * a2 + c * b2,
            b * a2 + d * b2,
            a * c2 + c * d2,
            b * c2 + d * d2,
            a * e2 + c * f2 + e,
            b * e2 + d * f2 + f,
        ])
    }

    /// The mean scale (a stroke's width scales by it).
    fn scale(&self) -> f32 {
        let [a, b, c, d, ..] = self.0;
        (a * d - b * c).abs().sqrt()
    }

    fn parse(s: &str) -> Result<Affine, String> {
        let mut out = Affine::IDENTITY;
        let mut rest = s.trim();
        while !rest.is_empty() {
            let open = rest
                .find('(')
                .ok_or_else(|| format!("transform \"{s}\": expected `name(...)`"))?;
            let close = rest[open..]
                .find(')')
                .ok_or_else(|| format!("transform \"{s}\": an unclosed `(`"))?
                + open;
            let name = rest[..open].trim().trim_start_matches(',').trim();
            let n = numbers(&rest[open + 1..close])?;
            let bad = || format!("transform \"{s}\": {name} with {} number(s)", n.len());
            let t = match (name, n.len()) {
                ("matrix", 6) => Affine([n[0], n[1], n[2], n[3], n[4], n[5]]),
                ("translate", 1) => Affine([1.0, 0.0, 0.0, 1.0, n[0], 0.0]),
                ("translate", 2) => Affine([1.0, 0.0, 0.0, 1.0, n[0], n[1]]),
                ("scale", 1) => Affine([n[0], 0.0, 0.0, n[0], 0.0, 0.0]),
                ("scale", 2) => Affine([n[0], 0.0, 0.0, n[1], 0.0, 0.0]),
                ("rotate", 1 | 3) => {
                    let (s, c) = n[0].to_radians().sin_cos();
                    let r = Affine([c, s, -s, c, 0.0, 0.0]);
                    if n.len() == 3 {
                        let (x, y) = (n[1], n[2]);
                        Affine([1.0, 0.0, 0.0, 1.0, x, y])
                            .then(&r)
                            .then(&Affine([1.0, 0.0, 0.0, 1.0, -x, -y]))
                    } else {
                        r
                    }
                }
                ("skewX", 1) => Affine([1.0, 0.0, n[0].to_radians().tan(), 1.0, 0.0, 0.0]),
                ("skewY", 1) => Affine([1.0, n[0].to_radians().tan(), 0.0, 1.0, 0.0, 0.0]),
                _ => return Err(bad()),
            };
            out = out.then(&t);
            rest = rest[close + 1..].trim_start();
        }
        Ok(out)
    }
}

// ── style ────────────────────────────────────────────────────────────────

/// A fill or stroke paint before it is placed.
#[derive(Clone, Debug, PartialEq)]
enum PaintRef {
    None,
    Color([f32; 4]),
    Url(String),
}

/// The inherited presentation attributes.
#[derive(Clone, Debug)]
struct Style {
    fill: PaintRef,
    fill_opacity: f32,
    even_odd: bool,
    stroke: PaintRef,
    stroke_opacity: f32,
    stroke_width: f32,
    cap: Cap,
    join: Join,
    miter_limit: f32,
    dashes: Vec<f32>,
    /// The product of the ancestors' and the element's `opacity`.
    opacity: f32,
    transform: Affine,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            fill: PaintRef::Color([0.0, 0.0, 0.0, 1.0]),
            fill_opacity: 1.0,
            even_odd: false,
            stroke: PaintRef::None,
            stroke_opacity: 1.0,
            stroke_width: 1.0,
            cap: Cap::Butt,
            join: Join::Miter,
            miter_limit: 4.0,
            dashes: Vec::new(),
            opacity: 1.0,
            transform: Affine::IDENTITY,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cap {
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Join {
    Miter,
    Round,
    Bevel,
}

fn opacity(v: &str) -> Result<f32, String> {
    let v = v.trim();
    let n = match v.strip_suffix('%') {
        Some(p) => p.trim().parse::<f32>().map(|p| p / 100.0),
        None => v.parse::<f32>(),
    };
    n.map(|n| n.clamp(0.0, 1.0))
        .map_err(|_| format!("'{v}' is not an opacity"))
}

/// An sRGB byte or unit channel to linear.
fn linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// A colour as linear RGBA (straight alpha).
fn color(v: &str) -> Result<[f32; 4], String> {
    let v = v.trim();
    let srgb = |r: f32, g: f32, b: f32, a: f32| [linear(r), linear(g), linear(b), a];
    if let Some(hex) = v.strip_prefix('#') {
        let digit = |i: usize| {
            u8::from_str_radix(&hex[i..i + 1], 16)
                .map(f32::from)
                .map_err(|_| format!("'{v}' is not a colour"))
        };
        let byte = |i: usize| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map(|b| f32::from(b) / 255.0)
                .map_err(|_| format!("'{v}' is not a colour"))
        };
        return match hex.len() {
            3 | 4 => {
                let a = if hex.len() == 4 {
                    digit(3)? / 15.0
                } else {
                    1.0
                };
                Ok(srgb(
                    digit(0)? / 15.0,
                    digit(1)? / 15.0,
                    digit(2)? / 15.0,
                    a,
                ))
            }
            6 | 8 => {
                let a = if hex.len() == 8 { byte(6)? } else { 1.0 };
                Ok(srgb(byte(0)?, byte(2)?, byte(4)?, a))
            }
            _ => Err(format!("'{v}' is not a colour")),
        };
    }
    if let Some(args) = v
        .strip_prefix("rgba(")
        .or_else(|| v.strip_prefix("rgb("))
        .and_then(|a| a.strip_suffix(')'))
    {
        let parts: Vec<&str> = args
            .split([',', ' ', '/'])
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() != 3 && parts.len() != 4 {
            return Err(format!("'{v}' is not a colour"));
        }
        let ch = |s: &str| -> Result<f32, String> {
            match s.strip_suffix('%') {
                Some(p) => p.parse::<f32>().map(|p| p / 100.0),
                None => s.parse::<f32>().map(|n| n / 255.0),
            }
            .map(|c| c.clamp(0.0, 1.0))
            .map_err(|_| format!("'{v}' is not a colour"))
        };
        let a = match parts.get(3) {
            Some(a) => opacity(a)?,
            None => 1.0,
        };
        return Ok(srgb(ch(parts[0])?, ch(parts[1])?, ch(parts[2])?, a));
    }
    let named: [u8; 3] = match v.to_ascii_lowercase().as_str() {
        // White, so the draw's tint is the colour.
        "currentcolor" | "white" => [255, 255, 255],
        "black" => [0, 0, 0],
        "red" => [255, 0, 0],
        "lime" => [0, 255, 0],
        "green" => [0, 128, 0],
        "blue" => [0, 0, 255],
        "yellow" => [255, 255, 0],
        "cyan" | "aqua" => [0, 255, 255],
        "magenta" | "fuchsia" => [255, 0, 255],
        "gray" | "grey" => [128, 128, 128],
        "silver" => [192, 192, 192],
        "maroon" => [128, 0, 0],
        "olive" => [128, 128, 0],
        "navy" => [0, 0, 128],
        "purple" => [128, 0, 128],
        "teal" => [0, 128, 128],
        "orange" => [255, 165, 0],
        "transparent" => return Ok([0.0; 4]),
        _ => return Err(format!("'{v}' is not a colour")),
    };
    Ok(srgb(
        f32::from(named[0]) / 255.0,
        f32::from(named[1]) / 255.0,
        f32::from(named[2]) / 255.0,
        1.0,
    ))
}

fn paint_ref(v: &str) -> Result<PaintRef, String> {
    let v = v.trim();
    if v == "none" {
        return Ok(PaintRef::None);
    }
    if let Some(url) = v.strip_prefix("url(") {
        let end = url
            .find(')')
            .ok_or_else(|| format!("'{v}': an unclosed url("))?;
        let id = url[..end].trim().trim_matches(['"', '\'']);
        let id = id
            .strip_prefix('#')
            .ok_or_else(|| format!("'{v}': only a reference into this file (url(#id))"))?;
        return Ok(PaintRef::Url(id.to_string()));
    }
    color(v).map(PaintRef::Color)
}

impl Style {
    /// `parent`'s style with `el`'s attributes applied.
    fn of(el: &Element, parent: &Style) -> Result<Style, String> {
        let mut s = parent.clone();
        let at = |e: String| format!("<{}>: {e}", el.name);
        for (k, v) in &el.attrs {
            let v = v.trim();
            if v == "inherit" {
                continue;
            }
            match k.as_str() {
                "fill" => s.fill = paint_ref(v).map_err(at)?,
                "stroke" => s.stroke = paint_ref(v).map_err(at)?,
                "fill-opacity" => s.fill_opacity = opacity(v).map_err(at)?,
                "stroke-opacity" => s.stroke_opacity = opacity(v).map_err(at)?,
                "fill-rule" => {
                    s.even_odd = match v {
                        "evenodd" => true,
                        "nonzero" => false,
                        _ => return Err(at(format!("fill-rule \"{v}\""))),
                    }
                }
                "stroke-width" => {
                    s.stroke_width = length(v, 1.0).map_err(at)?.max(0.0);
                }
                "stroke-linecap" => {
                    s.cap = match v {
                        "butt" => Cap::Butt,
                        "round" => Cap::Round,
                        "square" => Cap::Square,
                        _ => return Err(at(format!("stroke-linecap \"{v}\""))),
                    }
                }
                "stroke-linejoin" => {
                    s.join = match v {
                        "miter" | "miter-clip" | "arcs" => Join::Miter,
                        "round" => Join::Round,
                        "bevel" => Join::Bevel,
                        _ => return Err(at(format!("stroke-linejoin \"{v}\""))),
                    }
                }
                "stroke-miterlimit" => {
                    s.miter_limit = numbers(v)
                        .ok()
                        .and_then(|n| n.first().copied())
                        .filter(|l| *l >= 1.0)
                        .ok_or_else(|| at("stroke-miterlimit is a number of at least 1".into()))?;
                }
                "stroke-dasharray" => {
                    s.dashes = if v == "none" {
                        Vec::new()
                    } else {
                        let d = numbers(&v.replace("px", "")).map_err(at)?;
                        if d.iter().any(|x| *x < 0.0) {
                            return Err(at("stroke-dasharray has a negative length".into()));
                        }
                        if d.iter().sum::<f32>() <= 0.0 {
                            Vec::new()
                        } else if d.len() % 2 == 1 {
                            d.repeat(2)
                        } else {
                            d
                        }
                    };
                }
                "stroke-dashoffset" => {
                    if length(v, 1.0).map_err(at)? != 0.0 {
                        return Err(at("stroke-dashoffset is not supported".into()));
                    }
                }
                "opacity" => s.opacity *= opacity(v).map_err(at)?,
                "transform" => {
                    s.transform = parent.transform.then(&Affine::parse(v).map_err(at)?);
                }
                "display" if v == "none" => s.opacity = 0.0,
                "visibility" if v == "hidden" => s.opacity = 0.0,
                "clip-path" | "mask" | "filter" if v != "none" => {
                    return Err(at(format!("{k} is not supported")));
                }
                _ => {}
            }
        }
        Ok(s)
    }
}

// ── gradients ────────────────────────────────────────────────────────────

/// A gradient element's resolved attributes (after `href`).
#[derive(Clone, Debug)]
struct GradientDef {
    radial: bool,
    user_space: bool,
    spread: UiSpread,
    /// x1 y1 x2 y2, or cx cy r (fx, fy unused), as written (lengths).
    coords: HashMap<String, String>,
    stops: Vec<UiStop>,
    transform: Affine,
}

fn gradient_defs(root: &Element) -> Result<HashMap<String, GradientDef>, String> {
    let mut raw: HashMap<String, &Element> = HashMap::new();
    fn walk<'a>(e: &'a Element, raw: &mut HashMap<String, &'a Element>) {
        if (e.name == "linearGradient" || e.name == "radialGradient")
            && let Some(id) = e.get("id")
        {
            raw.insert(id.to_string(), e);
        }
        for c in &e.children {
            walk(c, raw);
        }
    }
    walk(root, &mut raw);
    let mut out = HashMap::new();
    for (id, e) in &raw {
        // Attributes and stops follow `href` (nearest first).
        let mut chain = vec![*e];
        while let Some(h) = chain
            .last()
            .and_then(|e| e.get("href").or_else(|| e.get("xlink:href")))
        {
            let target = h
                .strip_prefix('#')
                .and_then(|h| raw.get(h))
                .ok_or_else(|| format!("gradient \"{id}\": href \"{h}\" names no gradient"))?;
            if chain.len() > 16 || chain.iter().any(|c| std::ptr::eq(*c, *target)) {
                return Err(format!("gradient \"{id}\": an href loop"));
            }
            chain.push(target);
        }
        let get = |name: &str| chain.iter().find_map(|c| c.get(name));
        let mut coords = HashMap::new();
        for k in ["x1", "y1", "x2", "y2", "cx", "cy", "r", "fx", "fy"] {
            if let Some(v) = get(k) {
                coords.insert(k.to_string(), v.to_string());
            }
        }
        let radial = e.name == "radialGradient";
        if radial {
            for (f, c) in [("fx", "cx"), ("fy", "cy")] {
                if coords.contains_key(f) && coords.get(f) != coords.get(c) {
                    return Err(format!(
                        "gradient \"{id}\": a focal point apart from the centre is not supported"
                    ));
                }
            }
        }
        let stops_of = chain
            .iter()
            .find(|c| c.children.iter().any(|s| s.name == "stop"))
            .map_or(&[][..], |c| c.children.as_slice());
        let mut stops = Vec::new();
        let mut last = 0.0f32;
        for s in stops_of.iter().filter(|s| s.name == "stop") {
            let offset = match s.get("offset") {
                Some(v) => opacity(v).map_err(|e| format!("gradient \"{id}\" stop: {e}"))?,
                None => 0.0,
            }
            .max(last);
            last = offset;
            let mut c = match s.get("stop-color") {
                Some(v) => color(v).map_err(|e| format!("gradient \"{id}\" stop: {e}"))?,
                None => [0.0, 0.0, 0.0, 1.0],
            };
            if let Some(o) = s.get("stop-opacity") {
                c[3] *= opacity(o).map_err(|e| format!("gradient \"{id}\" stop: {e}"))?;
            }
            stops.push(UiStop { offset, color: c });
        }
        if stops.len() > og_render::UI_GRADIENT_STOPS {
            return Err(format!(
                "gradient \"{id}\" has {} stops (at most {})",
                stops.len(),
                og_render::UI_GRADIENT_STOPS
            ));
        }
        let transform = match get("gradientTransform") {
            Some(t) => {
                let a = Affine::parse(t).map_err(|e| format!("gradient \"{id}\": {e}"))?;
                if a.0[1].abs() > 1e-6 || a.0[2].abs() > 1e-6 {
                    return Err(format!(
                        "gradient \"{id}\": a gradientTransform that rotates or skews is not supported"
                    ));
                }
                a
            }
            None => Affine::IDENTITY,
        };
        out.insert(
            id.clone(),
            GradientDef {
                radial,
                user_space: get("gradientUnits") == Some("userSpaceOnUse"),
                spread: match get("spreadMethod") {
                    Some("repeat") => UiSpread::Repeat,
                    Some("reflect") => UiSpread::Reflect,
                    _ => UiSpread::Pad,
                },
                coords,
                stops,
                transform,
            },
        );
    }
    Ok(out)
}

impl GradientDef {
    /// The gradient placed for a shape whose local bounding box is `bbox`
    /// (`[x, y, w, h]`) under `t` (to view-box units), `alpha` multiplying
    /// its stops.
    fn place(&self, bbox: [f32; 4], t: &Affine, alpha: f32) -> Result<UiGradient, String> {
        let [bx, by, bw, bh] = bbox;
        // A number or a percentage: of the box in its units, of one
        // view-box unit in user space.
        let c = |k: &str, default: &str| -> Result<f32, String> {
            length(self.coords.get(k).map_or(default, String::as_str), 1.0)
        };
        // Into the shape's user space.
        let local = |x: f32, y: f32| {
            let p = self.transform.apply(Vec2::new(x, y));
            if self.user_space {
                p
            } else {
                Vec2::new(bx + p.x * bw, by + p.y * bh)
            }
        };
        let stops = self
            .stops
            .iter()
            .map(|s| UiStop {
                offset: s.offset,
                color: [s.color[0], s.color[1], s.color[2], s.color[3] * alpha],
            })
            .collect();
        let (kind, from, to) = if self.radial {
            let (cx, cy, r) = (c("cx", "50%")?, c("cy", "50%")?, c("r", "50%")?);
            let center = local(cx, cy);
            let edge_x = local(cx + r, cy) - center;
            let edge_y = local(cx, cy + r) - center;
            let s = t.scale();
            (
                UiGradientKind::Radial,
                t.apply(center),
                Vec2::new(edge_x.length() * s, edge_y.length() * s),
            )
        } else {
            let (x1, y1) = (c("x1", "0%")?, c("y1", "0%")?);
            let (x2, y2) = (c("x2", "100%")?, c("y2", "0%")?);
            (
                UiGradientKind::Linear,
                t.apply(local(x1, y1)),
                t.apply(local(x2, y2)),
            )
        };
        Ok(UiGradient {
            kind,
            from,
            to,
            stops,
            spread: self.spread,
            dither: true,
        })
    }
}

/// Tessellates an SVG (the subset in the module documentation).
pub fn tessellate(svg: &str, tolerance: Option<f32>) -> Result<UiVectorData, String> {
    let root = parse_xml(svg)?;
    let view_box = match root.get("viewBox") {
        Some(v) => {
            let n = numbers(v)?;
            if n.len() != 4 || n[2] <= 0.0 || n[3] <= 0.0 {
                return Err(format!("viewBox \"{v}\" is not four numbers with a size"));
            }
            [n[0], n[1], n[2], n[3]]
        }
        None => {
            let w = length_attr(&root, "width", 0.0, 0.0)?;
            let h = length_attr(&root, "height", 0.0, 0.0)?;
            if w <= 0.0 || h <= 0.0 {
                return Err("an <svg> needs a viewBox, or a width and a height".into());
            }
            [0.0, 0.0, w, h]
        }
    };
    let tolerance = tolerance.unwrap_or(view_box[2].max(view_box[3]) / 2000.0);
    let gradients = gradient_defs(&root)?;
    let mut mesh = Mesh::default();
    let style = Style::of(&root, &Style::default())?;
    let ctx = Ctx {
        gradients: &gradients,
        tolerance,
        view_box,
    };
    for child in &root.children {
        draw(&ctx, child, &style, &mut mesh)?;
    }
    Ok(UiVectorData {
        view_box,
        vertices: mesh.vertices,
        indices: mesh.indices,
        paints: mesh.paints,
        strokes: Vec::new(),
    })
}

struct Ctx<'a> {
    gradients: &'a HashMap<String, GradientDef>,
    tolerance: f32,
    view_box: [f32; 4],
}

fn draw(ctx: &Ctx, el: &Element, parent: &Style, mesh: &mut Mesh) -> Result<(), String> {
    match el.name.as_str() {
        "defs" | "title" | "desc" | "metadata" | "linearGradient" | "radialGradient" | "style" => {
            return Ok(());
        }
        "g" | "path" | "rect" | "circle" | "ellipse" | "line" | "polyline" | "polygon" => {}
        other => {
            return Err(format!(
                "<{other}> is not supported (paths, basic shapes, groups and gradients)"
            ));
        }
    }
    let style = Style::of(el, parent)?;
    if style.opacity <= 0.0 {
        return Ok(());
    }
    if el.name == "g" {
        for c in &el.children {
            draw(ctx, c, &style, mesh)?;
        }
        return Ok(());
    }
    let Some(local) = shape_path(el, ctx.view_box)? else {
        return Ok(());
    };
    let bbox = bounds(&local);
    let t = style.transform;
    let path = local.transformed(&t);
    let tol = ctx.tolerance;
    let eps = (ctx.view_box[2].max(ctx.view_box[3]) * 1e-5).max(1e-6);
    let resolve = |p: &PaintRef, alpha: f32, mesh: &mut Mesh| -> Result<Option<u32>, String> {
        Ok(match p {
            PaintRef::None => None,
            PaintRef::Color(c) => {
                Some(mesh.paint(UiVectorPaint::Solid([c[0], c[1], c[2], c[3] * alpha])))
            }
            PaintRef::Url(id) => {
                let g = ctx
                    .gradients
                    .get(id)
                    .ok_or_else(|| format!("<{}>: url(#{id}) names no gradient", el.name))?;
                Some(mesh.paint(UiVectorPaint::Gradient(g.place(bbox, &t, alpha)?)))
            }
        })
    };
    let lines = path.flatten(tol);
    // A line has no inside; a polyline fills as SVG does, closed.
    if el.name != "line"
        && let Some(paint) = resolve(&style.fill, style.fill_opacity * style.opacity, mesh)?
    {
        let contours: Vec<Vec<Vec2>> = lines
            .iter()
            .filter(|(p, _)| p.len() > 2)
            .map(|(p, _)| p.clone())
            .collect();
        fill(mesh, &contours, style.even_odd, paint, eps)?;
    }
    let width = style.stroke_width * t.scale();
    if width > 0.0
        && let Some(paint) = resolve(&style.stroke, style.stroke_opacity * style.opacity, mesh)?
    {
        let dashes: Vec<f32> = style.dashes.iter().map(|d| d * t.scale()).collect();
        let mut contours = Vec::new();
        for (pts, closed) in &lines {
            let pieces = if dashes.is_empty() {
                vec![(pts.clone(), *closed)]
            } else {
                dash(pts, *closed, &dashes)
                    .into_iter()
                    .map(|d| (d, false))
                    .collect()
            };
            for (p, c) in pieces {
                contours.extend(stroke_outline(
                    &p,
                    c,
                    width / 2.0,
                    style.cap,
                    style.join,
                    style.miter_limit,
                    tol,
                ));
            }
        }
        fill(mesh, &contours, false, paint, eps)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The area the mesh covers before the UI pass moves its outline (the
    /// fringe quads have none there), and its triangles' total.
    fn area(d: &UiVectorData) -> f32 {
        d.indices
            .chunks(3)
            .map(|t| {
                let p = |i: u32| Vec2::from(d.vertices[i as usize].position);
                ((p(t[1]) - p(t[0])).perp_dot(p(t[2]) - p(t[0]))).abs() / 2.0
            })
            .sum()
    }

    fn svg(body: &str) -> String {
        format!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\">{body}</svg>")
    }

    /// A rectangle covers its area once; an even-odd hole and a nonzero
    /// overlap cover what the rules say, so trapezoids neither overlap nor
    /// leave gaps.
    #[test]
    fn fills_cover_their_area_by_the_fill_rule() {
        let d = tessellate(
            &svg(r#"<rect x="10" y="20" width="30" height="40"/>"#),
            None,
        )
        .unwrap();
        assert!((area(&d) - 1200.0).abs() < 1e-2, "{}", area(&d));
        let ring = r#"<path fill-rule="evenodd" d="M0 0H100V100H0Z M25 25H75V75H25Z"/>"#;
        let d = tessellate(&svg(ring), None).unwrap();
        assert!((area(&d) - 7500.0).abs() < 1e-1, "{}", area(&d));
        // The same with nonzero fills the hole (both wound clockwise).
        let d = tessellate(&svg(&ring.replace("evenodd", "nonzero")), None).unwrap();
        assert!((area(&d) - 10000.0).abs() < 1e-1, "{}", area(&d));
        // Two squares overlapping by a quarter: the union.
        let two = r#"<path d="M0 0H50V50H0Z M25 25H75V75H25Z"/>"#;
        let d = tessellate(&svg(two), None).unwrap();
        assert!(
            (area(&d) - (2500.0 * 2.0 - 625.0)).abs() < 1e-1,
            "{}",
            area(&d)
        );
    }

    /// A circle by arcs, and by `<circle>`, is a circle's area within the
    /// flattening's tolerance; every outline vertex moves along a unit-ish
    /// miter, and the fringe quads hold an inner and an outer vertex.
    #[test]
    fn curves_flatten_within_the_tolerance_and_the_outline_carries_its_miters() {
        let want = std::f32::consts::PI * 40.0 * 40.0;
        for body in [
            r#"<circle cx="50" cy="50" r="40"/>"#,
            r#"<path d="M10 50a40 40 0 1 0 80 0A40 40 0 1 0 10 50z"/>"#,
            // One arc between nearly coincident ends: a whole circle.
            r#"<path d="M50 10 a40 40 0 1 0 0.001 0 Z"/>"#,
        ] {
            let d = tessellate(&svg(body), Some(0.01)).unwrap();
            let a = area(&d);
            assert!(a < want && a > want * 0.999, "{body}: {a} vs {want}");
            let moved: Vec<f32> = d
                .vertices
                .iter()
                .map(|v| Vec2::from(v.normal).length())
                .filter(|l| *l > 0.0)
                .collect();
            assert!(!moved.is_empty());
            assert!(moved.iter().all(|l| (0.99..1.01).contains(l)), "{moved:?}");
            assert!(d.vertices.iter().any(|v| v.aa > 0.0) && d.vertices.iter().any(|v| v.aa < 0.0));
        }
    }

    /// A stroke is its outline: a straight butt line is length × width, a
    /// square cap adds half a width at each end, a dashed line half of it.
    #[test]
    fn strokes_cover_their_outline_with_caps_and_dashes() {
        let line = |extra: &str| {
            tessellate(
                &svg(&format!(r#"<line x1="10" y1="50" x2="90" y2="50" stroke="red" stroke-width="10" {extra}/>"#)),
                None,
            )
            .unwrap()
        };
        assert!((area(&line("")) - 800.0).abs() < 0.1, "{}", area(&line("")));
        assert!((area(&line(r#"stroke-linecap="square""#)) - 900.0).abs() < 0.1);
        let round = area(&line(r#"stroke-linecap="round""#));
        let want = 800.0 + std::f32::consts::PI * 25.0;
        assert!((round - want).abs() < 1.0, "{round} vs {want}");
        let dashed = area(&line(r#"stroke-dasharray="10 10""#));
        assert!((dashed - 400.0).abs() < 0.1, "{dashed}");
        // A polyline's miter join fills its corner: an L of two 40-long arms
        // 10 wide is the two arms (overlapping by 5×5 inside the corner)
        // and the join's 5×5 outside it.
        let l = tessellate(
            &svg(r##"<polyline points="10 10 50 10 50 50" fill="none" stroke="#000" stroke-width="10"/>"##),
            None,
        )
        .unwrap();
        assert!(
            (area(&l) - (400.0 + 400.0 - 25.0 + 25.0)).abs() < 0.1,
            "{}",
            area(&l)
        );
    }

    /// Paints: a colour is linear, `currentColor` white; a gradient in
    /// bounding box units lands on the shape's box under its transform; a
    /// group's opacity multiplies its children's alpha.
    #[test]
    fn paints_resolve_colours_gradients_and_opacity() {
        let d = tessellate(
            &svg(r##"<defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#f00"/><stop offset="1" stop-color="#00f" stop-opacity="0.5"/></linearGradient></defs>
            <g opacity="0.5" transform="translate(10 0)"><rect x="10" y="10" width="40" height="20" fill="url(#g)"/></g>
            <rect x="0" y="80" width="10" height="10" fill="currentColor"/>
            <rect x="20" y="80" width="10" height="10" fill="#808080"/>"##),
            None,
        )
        .unwrap();
        let UiVectorPaint::Gradient(g) = &d.paints[0] else {
            panic!("{:?}", d.paints)
        };
        assert_eq!(g.kind, UiGradientKind::Linear);
        assert_eq!(
            (g.from, g.to),
            (Vec2::new(20.0, 10.0), Vec2::new(60.0, 10.0))
        );
        assert_eq!(g.stops[0].color, [1.0, 0.0, 0.0, 0.5]);
        assert_eq!(g.stops[1].color, [0.0, 0.0, 1.0, 0.25]);
        assert_eq!(d.paints[1], UiVectorPaint::Solid([1.0, 1.0, 1.0, 1.0]));
        let UiVectorPaint::Solid(grey) = d.paints[2] else {
            panic!()
        };
        assert!((grey[0] - 0.2158).abs() < 1e-3, "{grey:?}");
    }

    /// What the subset does not hold is refused with the element named.
    #[test]
    fn unsupported_svg_is_refused_by_name() {
        for (body, want) in [
            ("<text>hi</text>", "<text>"),
            (r#"<image href="a.png"/>"#, "<image>"),
            (
                r#"<rect width="1" height="1" clip-path="url(#c)"/>"#,
                "clip-path",
            ),
            (
                r#"<rect width="1" height="1" fill="url(#nowhere)"/>"#,
                "nowhere",
            ),
        ] {
            let e = tessellate(&svg(body), None).expect_err(body);
            assert!(e.contains(want), "{body}: {e}");
        }
        assert!(
            tessellate("<svg><path d='M0 0L1 1'/></svg>", None)
                .unwrap_err()
                .contains("viewBox")
        );
    }

    /// The vector file names its SVG once.
    #[test]
    fn the_file_names_one_svg() {
        let f = parse("type vector\nversion 1\n---\nsource \"ui/a.svg\"\ntolerance 0.1\n").unwrap();
        assert_eq!(f.source.as_deref(), Some("ui/a.svg"));
        assert_eq!(f.tolerance, Some(0.1));
        assert!(parse("type vector\nversion 1\n---\n").is_err());
    }
}
