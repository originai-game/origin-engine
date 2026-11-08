//! SVG's drawing model, shared by the cook and documents (`docs/ui.md`
//! §3.3): the XML reader of a `.svg` file ([`parse_xml`]), path data and
//! the basic shapes as cubics ([`Path`], [`shape_path`]), transforms
//! ([`Affine`]), the presentation attributes ([`Style`]) and gradients
//! ([`GradientDef`]).
//!
//! The cook tessellates a file's shapes into a `type vector`
//! (`og-cook/src/cook/vector.rs`); a document's inline `<svg>` lowers each
//! shape to an element's draw, its path drawn analytically by og-render's
//! UI pass (`crate::doc::build::svg`).

use og_core::glam::Vec2;

mod path;
pub mod smil;

pub use path::{Path, bounds, shape_path};

// ── the SVG document ─────────────────────────────────────────────────────

/// An element: its name, attributes (style declarations folded in) and
/// children.
#[derive(Debug, Default)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Element>,
}

impl Element {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .rev()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.trim())
    }
}

/// The element tree of an SVG document (its root `<svg>`).
pub fn parse_xml(src: &str) -> Result<Element, String> {
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
pub fn numbers(s: &str) -> Result<Vec<f32>, String> {
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
pub fn length(v: &str, whole: f32) -> Result<f32, String> {
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

pub fn length_attr(el: &Element, name: &str, whole: f32, default: f32) -> Result<f32, String> {
    el.get(name)
        .map_or(Ok(default), |v| length(v, whole))
        .map_err(|e| format!("<{}> {name}: {e}", el.name))
}

// ── transforms ───────────────────────────────────────────────────────────

/// An affine transform `[a, b, c, d, e, f]` as SVG's `matrix`: x' = a·x +
/// c·y + e, y' = b·x + d·y + f.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine(pub [f32; 6]);

impl Affine {
    pub const IDENTITY: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub fn apply(&self, p: Vec2) -> Vec2 {
        let [a, b, c, d, e, f] = self.0;
        Vec2::new(a * p.x + c * p.y + e, b * p.x + d * p.y + f)
    }

    /// `self` after `inner` (inner applies first).
    pub fn then(&self, inner: &Affine) -> Affine {
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
    pub fn scale(&self) -> f32 {
        let [a, b, c, d, ..] = self.0;
        (a * d - b * c).abs().sqrt()
    }

    pub fn parse(s: &str) -> Result<Affine, String> {
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
pub enum PaintRef {
    None,
    /// sRGB, straight alpha, each channel 0 to 1.
    Color([f32; 4]),
    /// `currentColor`: the element's CSS `color`.
    Current,
    /// `var(--role)`: a colour of the style (documents only; CSS reaches
    /// SVG's presentation properties, its attributes take no `var()`).
    Token(String),
    /// `url(#id)`: a gradient.
    Url(String),
}

/// The inherited presentation attributes, and the element's transform
/// to its outermost `<svg>`'s user space.
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub fill: PaintRef,
    pub fill_opacity: f32,
    pub even_odd: bool,
    pub stroke: PaintRef,
    pub stroke_opacity: f32,
    pub stroke_width: f32,
    pub cap: Cap,
    pub join: Join,
    pub miter_limit: f32,
    /// Dash and gap lengths, even in number (an odd list repeated); empty:
    /// solid.
    pub dashes: Vec<f32>,
    pub dash_offset: f32,
    /// The product of the ancestors' and the element's `opacity`.
    pub opacity: f32,
    pub transform: Affine,
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
            dash_offset: 0.0,
            opacity: 1.0,
            transform: Affine::IDENTITY,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cap {
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Join {
    Miter,
    Round,
    Bevel,
}

/// An opacity: a number or a percentage, clamped to 0–1.
pub fn opacity(v: &str) -> Result<f32, String> {
    let v = v.trim();
    let n = match v.strip_suffix('%') {
        Some(p) => p.trim().parse::<f32>().map(|p| p / 100.0),
        None => v.parse::<f32>(),
    };
    n.map(|n| n.clamp(0.0, 1.0))
        .map_err(|_| format!("'{v}' is not an opacity"))
}

/// A colour as sRGB, straight alpha, each channel 0 to 1 (`currentColor`
/// is white: the cook's vectors take their tint as their colour).
/// A literal CSS colour as sRGB channels 0–1 ([`crate::css_color`]: hex,
/// named colours, every colour function); `currentColor` reads as white
/// here, its callers resolve it first.
pub fn color(v: &str) -> Result<[f32; 4], String> {
    let v = v.trim();
    if v.eq_ignore_ascii_case("currentcolor") {
        return Ok([1.0; 4]);
    }
    let c = crate::css_color::parse_str(v)
        .ok()
        .and_then(|c| c.literal())
        .ok_or_else(|| format!("'{v}' is not a colour"))?;
    Ok([
        f32::from(c.rgb[0]) / 255.0,
        f32::from(c.rgb[1]) / 255.0,
        f32::from(c.rgb[2]) / 255.0,
        c.alpha,
    ])
}

pub fn paint_ref(v: &str) -> Result<PaintRef, String> {
    let v = v.trim();
    if v == "none" {
        return Ok(PaintRef::None);
    }
    if v.eq_ignore_ascii_case("currentcolor") {
        return Ok(PaintRef::Current);
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
    if let Some(name) = v
        .strip_prefix("var(--")
        .and_then(|n| n.strip_suffix(')'))
        .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
    {
        return Ok(PaintRef::Token(name.to_string()));
    }
    color(v).map(PaintRef::Color)
}

impl Style {
    /// `parent`'s style with `el`'s attributes applied.
    pub fn of(el: &Element, parent: &Style) -> Result<Style, String> {
        let mut s = parent.clone();
        for (k, v) in &el.attrs {
            s.apply(k, v, parent)
                .map_err(|e| format!("<{}>: {e}", el.name))?;
        }
        Ok(s)
    }

    /// One presentation attribute (or CSS property) `k` written `v`, onto
    /// the style inherited from `parent`. Other names are read past.
    pub fn apply(&mut self, k: &str, v: &str, parent: &Style) -> Result<(), String> {
        let v = v.trim();
        if v == "inherit" {
            return Ok(());
        }
        match k {
            "fill" => self.fill = paint_ref(v)?,
            "stroke" => self.stroke = paint_ref(v)?,
            "fill-opacity" => self.fill_opacity = opacity(v)?,
            "stroke-opacity" => self.stroke_opacity = opacity(v)?,
            "fill-rule" => {
                self.even_odd = match v {
                    "evenodd" => true,
                    "nonzero" => false,
                    _ => return Err(format!("fill-rule \"{v}\"")),
                }
            }
            "stroke-width" => {
                self.stroke_width = length(v, 1.0)?.max(0.0);
            }
            "stroke-linecap" => {
                self.cap = match v {
                    "butt" => Cap::Butt,
                    "round" => Cap::Round,
                    "square" => Cap::Square,
                    _ => return Err(format!("stroke-linecap \"{v}\"")),
                }
            }
            "stroke-linejoin" => {
                self.join = match v {
                    "miter" | "miter-clip" | "arcs" => Join::Miter,
                    "round" => Join::Round,
                    "bevel" => Join::Bevel,
                    _ => return Err(format!("stroke-linejoin \"{v}\"")),
                }
            }
            "stroke-miterlimit" => {
                self.miter_limit = numbers(v)
                    .ok()
                    .and_then(|n| n.first().copied())
                    .filter(|l| *l >= 1.0)
                    .ok_or("stroke-miterlimit is a number of at least 1")?;
            }
            "stroke-dasharray" => {
                self.dashes = if v == "none" {
                    Vec::new()
                } else {
                    let d = numbers(&v.replace("px", ""))?;
                    if d.iter().any(|x| *x < 0.0) {
                        return Err("stroke-dasharray has a negative length".into());
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
            "stroke-dashoffset" => self.dash_offset = length(v, 1.0)?,
            "opacity" => self.opacity = parent.opacity * opacity(v)?,
            "transform" => {
                self.transform = parent.transform.then(&Affine::parse(v)?);
            }
            "display" if v == "none" => self.opacity = 0.0,
            "visibility" if v == "hidden" => self.opacity = 0.0,
            "clip-path" | "mask" | "filter" if v != "none" => {
                return Err(format!("{k} is not supported"));
            }
            _ => {}
        }
        Ok(())
    }
}

// ── gradients ────────────────────────────────────────────────────────────

/// A gradient element's resolved attributes (after `href`).
#[derive(Clone, Debug, PartialEq)]
pub struct GradientDef {
    pub radial: bool,
    pub user_space: bool,
    pub spread: og_render::UiSpread,
    /// x1 y1 x2 y2, or cx cy r (fx, fy unused), as written (lengths).
    pub coords: Vec<(String, String)>,
    /// Offsets and sRGB colours (straight alpha, `stop-opacity` applied).
    pub stops: Vec<(f32, [f32; 4])>,
    pub transform: Affine,
}

/// A gradient placed for a shape: in its target space, linear from `from`
/// to `to`, or radial about `from` with radii `to`.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacedGradient {
    pub radial: bool,
    pub from: Vec2,
    pub to: Vec2,
    pub stops: Vec<(f32, [f32; 4])>,
    pub spread: og_render::UiSpread,
}

/// Every gradient of a file's tree, by id.
pub fn gradient_defs(
    root: &Element,
) -> Result<std::collections::HashMap<String, GradientDef>, String> {
    use std::collections::HashMap;
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
        let stops: Vec<&Element> = chain
            .iter()
            .find(|c| c.children.iter().any(|s| s.name == "stop"))
            .map_or(&[][..], |c| c.children.as_slice())
            .iter()
            .filter(|s| s.name == "stop")
            .collect();
        let stops: Vec<_> = stops.iter().map(|s| |k: &str| s.get(k)).collect();
        out.insert(
            id.clone(),
            GradientDef::new(id, e.name == "radialGradient", &get, &stops)?,
        );
    }
    Ok(out)
}

impl GradientDef {
    /// The gradient `id` (radial or linear) whose attributes `get` reads
    /// and whose stops `stops` read.
    pub fn new<'a>(
        id: &str,
        radial: bool,
        get: &dyn Fn(&str) -> Option<&'a str>,
        stops: &[impl Fn(&str) -> Option<&'a str>],
    ) -> Result<GradientDef, String> {
        let mut coords = Vec::new();
        for k in ["x1", "y1", "x2", "y2", "cx", "cy", "r", "fx", "fy"] {
            if let Some(v) = get(k) {
                coords.push((k.to_string(), v.to_string()));
            }
        }
        let coord = |k: &str| coords.iter().find(|(n, _)| n == k).map(|(_, v)| v);
        if radial {
            for (f, c) in [("fx", "cx"), ("fy", "cy")] {
                if coord(f).is_some() && coord(f) != coord(c) {
                    return Err(format!(
                        "gradient \"{id}\": a focal point apart from the centre is not supported"
                    ));
                }
            }
        }
        let mut out = Vec::new();
        let mut last = 0.0f32;
        for s in stops {
            let offset = match s("offset") {
                Some(v) => opacity(v).map_err(|e| format!("gradient \"{id}\" stop: {e}"))?,
                None => 0.0,
            }
            .max(last);
            last = offset;
            let mut c = match s("stop-color") {
                Some(v) => color(v).map_err(|e| format!("gradient \"{id}\" stop: {e}"))?,
                None => [0.0, 0.0, 0.0, 1.0],
            };
            if let Some(o) = s("stop-opacity") {
                c[3] *= opacity(o).map_err(|e| format!("gradient \"{id}\" stop: {e}"))?;
            }
            out.push((offset, c));
        }
        if out.len() > og_render::UI_GRADIENT_STOPS {
            return Err(format!(
                "gradient \"{id}\" has {} stops (at most {})",
                out.len(),
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
        Ok(GradientDef {
            radial,
            user_space: get("gradientUnits") == Some("userSpaceOnUse"),
            spread: match get("spreadMethod") {
                Some("repeat") => og_render::UiSpread::Repeat,
                Some("reflect") => og_render::UiSpread::Reflect,
                _ => og_render::UiSpread::Pad,
            },
            coords,
            stops: out,
            transform,
        })
    }

    /// The gradient placed for a shape whose local bounding box is `bbox`
    /// (`[x, y, w, h]`) under `t` (to the target space), `alpha`
    /// multiplying its stops.
    pub fn place(&self, bbox: [f32; 4], t: &Affine, alpha: f32) -> Result<PlacedGradient, String> {
        let [bx, by, bw, bh] = bbox;
        // A number or a percentage: of the box in its units, of one
        // view-box unit in user space.
        let c = |k: &str, default: &str| -> Result<f32, String> {
            length(
                self.coords
                    .iter()
                    .find(|(n, _)| n == k)
                    .map_or(default, |(_, v)| v.as_str()),
                1.0,
            )
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
            .map(|(o, c)| (*o, [c[0], c[1], c[2], c[3] * alpha]))
            .collect();
        let (from, to) = if self.radial {
            let (cx, cy, r) = (c("cx", "50%")?, c("cy", "50%")?, c("r", "50%")?);
            let center = local(cx, cy);
            let edge_x = local(cx + r, cy) - center;
            let edge_y = local(cx, cy + r) - center;
            let s = t.scale();
            (
                t.apply(center),
                Vec2::new(edge_x.length() * s, edge_y.length() * s),
            )
        } else {
            let (x1, y1) = (c("x1", "0%")?, c("y1", "0%")?);
            let (x2, y2) = (c("x2", "100%")?, c("y2", "0%")?);
            (t.apply(local(x1, y1)), t.apply(local(x2, y2)))
        };
        Ok(PlacedGradient {
            radial: self.radial,
            from,
            to,
            stops,
            spread: self.spread,
        })
    }
}
