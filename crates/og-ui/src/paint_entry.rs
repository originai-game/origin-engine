//! A style's dressing lines read into the UI pass's types (docs/ui.md
//! §5, §12.3): what [`crate::dress::DressDraw`] holds. One line per entry,
//! values in order; a colour is a quoted literal (`"#rrggbb"`,
//! `"#rrggbbaa"`) or a palette token (`primary`, `lineStrong`), resolved
//! through the style when the frame is drawn. Angles are degrees, lengths
//! CSS px, atlas regions texels of the style's `atlas` line.
//!
//! ```text
//! fill <paint>
//! stroke <width> <paint> [inside|center|outside] [dash <len> <gap> [offset <o>]] [butt|round|square]
//! effect dropShadow|innerShadow <dx> <dy> <blur> [<spread>] <colour>
//! effect outerGlow|innerGlow <blur> [<spread>] <colour>
//! effect blur <radius>
//! effect backdrop [<blur>] [saturation <s>] [brightness <b>] [tint <colour>]
//!     [glass thin|regular|thick] [elevation raised|float|lift] [edge <w> [<colour>]] [solid] [css]
//! material <builtin>|"<ui-material asset>" [<param>…]   (at most 16 numbers)
//! blend normal|multiply|screen|add|overlay|softLight|colorDodge
//! mask <gradient> | mask image <x> <y> <w> <h>
//! ornament corner|divider|flourish…
//! ornamentTint <colour>
//! sound <event> "<asset>"                               (an event of docs/ui.md §7.8: press, confirm, toggleOn…)
//! ```
//!
//! where a paint is
//!
//! ```text
//! <colour>
//! linear <angle> <stops> [repeat|reflect]      (CSS angles: 0 up, 90 right)
//! radial <stops> [repeat|reflect]              (centre to the box's edges)
//! conic <angle> <stops> [repeat|reflect]       (from the angle, clockwise)
//! diamond <stops> [repeat|reflect]
//! image <x> <y> <w> <h> [fill|cover|contain|none] [focal <fx> <fy>] [tint <colour>]
//! pattern <x> <y> <w> <h> <tile w> <tile h> [rotate <deg>] [scroll <sx> <sy>] [tint <colour>]
//! vector "<asset>" [tint <colour>]             (a `type vector` the style loads, contained; fills only)
//! ```
//!
//! and stops are two to eight colours, each followed by its offset (0 to
//! 1) or none (spread evenly between its neighbours).

use crate::dress::{DressMaterial, OrnamentKind};
use crate::paint::{
    AtlasPaint, Blend, Builtin, Cap, ColorRef, Effect, Fit, Gradient, GradientKind, Layer, Mask,
    Paint, Spread, Stop, Stroke, StrokeAlign,
};
use crate::style::{Palette, Roles};
use crate::style_file::{P, parse_color};
use crate::tokens::SoundEvent;
use og_core::ogt::{Entry, Value};

/// What a dressing's atlas regions are cut from: the atlas's size in
/// texels (`None`: the style has no `atlas` line) and the style's scale.
/// It also carries the colour roles the style declares of its own
/// (`roles { … }`, docs/ui.md §12.2), which a paint names like a palette
/// role.
#[derive(Clone, Copy, Debug)]
pub struct AtlasContext {
    pub size: Option<(f32, f32)>,
    pub scale: f32,
    pub roles: Roles,
}

impl Default for AtlasContext {
    fn default() -> AtlasContext {
        AtlasContext {
            size: None,
            scale: 1.0,
            roles: Roles::NONE,
        }
    }
}

const BUILTINS: [(&str, Builtin); 9] = [
    ("dissolve", Builtin::Dissolve),
    ("shine", Builtin::Shine),
    ("hologram", Builtin::Hologram),
    ("glitch", Builtin::Glitch),
    ("ripple", Builtin::Ripple),
    ("pixelate", Builtin::Pixelate),
    ("colorAdjust", Builtin::ColorAdjust),
    ("outline", Builtin::Outline),
    ("blur", Builtin::Blur),
];

const BLENDS: [(&str, Blend); 7] = [
    ("normal", Blend::Normal),
    ("multiply", Blend::Multiply),
    ("screen", Blend::Screen),
    ("add", Blend::Add),
    ("overlay", Blend::Overlay),
    ("softLight", Blend::SoftLight),
    ("colorDodge", Blend::ColorDodge),
];

fn names<T>(all: &[(&str, T)]) -> String {
    all.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", ")
}

/// A reader of one entry's values from a position on.
struct Cursor<'a> {
    p: P<'a>,
    at: usize,
    /// The style's own colour roles, named like palette roles.
    roles: Option<&'a Roles>,
}

impl<'a> Cursor<'a> {
    fn new(e: &'a Entry, at: usize) -> Cursor<'a> {
        Cursor {
            p: P { e },
            at,
            roles: None,
        }
    }
    /// The same, naming the style's own roles too.
    fn of(e: &'a Entry, ctx: &'a AtlasContext) -> Cursor<'a> {
        Cursor {
            roles: Some(&ctx.roles),
            ..Cursor::new(e, 0)
        }
    }
    /// `w` names a colour role: the palette's or one the style declares.
    fn is_role(&self, w: &str) -> bool {
        Palette::NAMES.contains(&w)
            || crate::tokens::colour_token_names().iter().any(|n| n == w)
            || self.roles.is_some_and(|r| r.get(w).is_some())
    }
    fn peek(&self) -> Option<&'a Value> {
        self.p.e.values.get(self.at)
    }
    fn peek_word(&self) -> Option<&'a str> {
        self.peek().and_then(Value::as_word)
    }
    fn peek_number(&self) -> bool {
        self.peek().is_some_and(|v| v.as_number().is_some())
    }
    fn done(&self) -> bool {
        self.at >= self.p.e.values.len()
    }
    fn err(&self, what: &str) -> String {
        self.p.err(what)
    }
    fn num(&mut self) -> Result<f32, String> {
        let v = self.p.num(self.at)?;
        self.at += 1;
        Ok(v)
    }
    fn word(&mut self) -> Result<&'a str, String> {
        let w = self.p.word(self.at)?;
        self.at += 1;
        Ok(w)
    }
    /// The next value if it is the word `w`.
    fn eat(&mut self, w: &str) -> bool {
        if self.peek_word() == Some(w) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn end(&self) -> Result<(), String> {
        match self.peek() {
            None => Ok(()),
            Some(v) => Err(self.err(&format!(
                "has '{}' left over at position {}",
                v.as_word()
                    .or(v.as_str())
                    .map_or_else(|| "a number".to_string(), str::to_string),
                self.at + 1
            ))),
        }
    }

    /// A colour: a quoted literal or a palette token.
    fn color(&mut self) -> Result<ColorRef, String> {
        let mut c = match self.peek() {
            Some(Value::String(s)) => ColorRef::Literal(parse_color(s).ok_or_else(|| {
                self.err(&format!("takes \"#rrggbb\" or \"#rrggbbaa\", not \"{s}\""))
            })?),
            Some(Value::Word(w)) if self.is_role(w) => ColorRef::token(w),
            Some(Value::Word(w)) => {
                return Err(self.err(&format!(
                    "names '{w}', which is not a palette colour (a token such as primary or lineStrong, a role the style declares in `roles {{ }}`, or a quoted \"#rrggbb\")"
                )));
            }
            _ => {
                return Err(self.err(&format!(
                    "takes a colour at position {} (a palette token or a quoted \"#rrggbb\")",
                    self.at + 1
                )));
            }
        };
        self.at += 1;
        if self.eat("alpha") {
            let opacity = self.num()?;
            if !(0.0..=1.0).contains(&opacity) {
                return Err(self.err("takes alpha from 0 to 1"));
            }
            match &mut c {
                ColorRef::Token { alpha, .. } => *alpha *= opacity,
                ColorRef::Literal(color) => *color = color.fade(opacity),
            }
        }
        Ok(c)
    }

    fn is_color(&self) -> bool {
        match self.peek() {
            Some(Value::String(_)) => true,
            Some(Value::Word(w)) => self.is_role(w),
            _ => false,
        }
    }

    /// Two to eight stops, then an optional spread word.
    fn stops(&mut self) -> Result<(Vec<Stop>, Spread), String> {
        let mut raw: Vec<(ColorRef, Option<f32>)> = Vec::new();
        while self.is_color() {
            let c = self.color()?;
            let o = if self.peek_number() {
                let o = self.num()?;
                if !(0.0..=1.0).contains(&o) {
                    return Err(self.err("has a stop offset outside 0 to 1"));
                }
                Some(o)
            } else {
                None
            };
            raw.push((c, o));
        }
        if raw.len() < 2
            && let Some(w) = self.peek_word()
        {
            return Err(self.err(&format!(
                "names '{w}' among its stops, which is not a palette colour (a token such as primary or lineStrong, or a quoted \"#rrggbb\")"
            )));
        }
        if !(2..=og_render::UI_GRADIENT_STOPS).contains(&raw.len()) {
            return Err(self.err(&format!(
                "takes 2 to {} gradient stops, not {}",
                og_render::UI_GRADIENT_STOPS,
                raw.len()
            )));
        }
        // Unset offsets: the ends at 0 and 1, the rest even between set ones.
        let n = raw.len();
        let mut offsets: Vec<Option<f32>> = raw.iter().map(|(_, o)| *o).collect();
        offsets[0] = offsets[0].or(Some(0.0));
        offsets[n - 1] = offsets[n - 1].or(Some(1.0));
        let mut i = 0;
        while i < n {
            if offsets[i].is_some() {
                i += 1;
                continue;
            }
            let lo = i - 1;
            let hi = (i..n)
                .find(|&j| offsets[j].is_some())
                .expect("the last is set");
            let (a, b) = (offsets[lo].expect("set"), offsets[hi].expect("set"));
            for (k, slot) in offsets.iter_mut().enumerate().take(hi).skip(i) {
                *slot = Some(a + (b - a) * (k - lo) as f32 / (hi - lo) as f32);
            }
            i = hi;
        }
        let offsets: Vec<f32> = offsets.into_iter().map(|o| o.expect("set")).collect();
        if offsets.windows(2).any(|w| w[1] < w[0]) {
            return Err(self.err("has gradient stops out of order"));
        }
        let spread = if self.eat("repeat") {
            Spread::Repeat
        } else if self.eat("reflect") {
            Spread::Reflect
        } else {
            Spread::Pad
        };
        let stops = raw
            .into_iter()
            .zip(offsets)
            .map(|((color, _), offset)| Stop { offset, color })
            .collect();
        Ok((stops, spread))
    }

    /// A gradient paint after its kind word, or `None` for another word.
    fn gradient(&mut self, kind: &str) -> Result<Option<Gradient>, String> {
        let g = match kind {
            "linear" => {
                // CSS: 0deg points up, 90deg right; the line reaches the
                // box's corners.
                let (s, c) = self.num()?.to_radians().sin_cos();
                // A quarter turn's zero is exact (90° is left to right).
                let exact = |v: f32| if v.abs() < 1e-6 { 0.0 } else { v };
                let (s, c) = (exact(s), exact(c));
                let reach = 0.5 * (s.abs() + c.abs());
                let (stops, spread) = self.stops()?;
                Gradient {
                    kind: GradientKind::Linear,
                    from: [0.5 - s * reach, 0.5 + c * reach],
                    to: [0.5 + s * reach, 0.5 - c * reach],
                    stops,
                    spread,
                    dither: true,
                }
            }
            "radial" | "diamond" => {
                let (stops, spread) = self.stops()?;
                Gradient {
                    kind: if kind == "radial" {
                        GradientKind::Radial
                    } else {
                        GradientKind::Diamond
                    },
                    from: [0.5, 0.5],
                    to: [0.5, 0.5],
                    stops,
                    spread,
                    dither: true,
                }
            }
            "conic" => {
                // CSS: from 0deg up, clockwise; the paint's start is from +x.
                let start = (self.num()? - 90.0).to_radians();
                let (stops, spread) = self.stops()?;
                Gradient {
                    kind: GradientKind::Conic,
                    from: [0.5, 0.5],
                    to: [start, 0.0],
                    stops,
                    spread,
                    dither: true,
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(g))
    }

    /// An atlas region `x y w h`: its UVs and its CSS px size.
    fn region(&mut self, atlas: &AtlasContext) -> Result<([f32; 4], [f32; 2]), String> {
        let Some((aw, ah)) = atlas.size else {
            return Err(self.err("cuts from the style's atlas, but it has no `atlas` line"));
        };
        let (x, y, w, h) = (self.num()?, self.num()?, self.num()?, self.num()?);
        if w <= 0.0 || h <= 0.0 || x < 0.0 || y < 0.0 || x + w > aw || y + h > ah {
            return Err(self.err(&format!(
                "region {x} {y} {w} {h} is not inside the {aw}×{ah} atlas"
            )));
        }
        Ok((
            [x / aw, y / ah, (x + w) / aw, (y + h) / ah],
            [w * atlas.scale, h * atlas.scale],
        ))
    }

    fn paint(&mut self, atlas: &AtlasContext) -> Result<Paint, String> {
        if self.is_color() {
            return Ok(Paint::Solid(self.color()?));
        }
        let kind = self.word()?;
        if let Some(g) = self.gradient(kind)? {
            return Ok(Paint::Gradient(g));
        }
        let white = || {
            ColorRef::Literal(crate::style::Srgba {
                rgb: [255, 255, 255],
                alpha: 1.0,
            })
        };
        match kind {
            "image" => {
                let (uv, size) = self.region(atlas)?;
                let mut a = AtlasPaint {
                    uv,
                    size,
                    tint: white(),
                    fit: Fit::Fill,
                    focal: [0.5, 0.5],
                };
                loop {
                    if self.eat("fill") {
                        a.fit = Fit::Fill;
                    } else if self.eat("cover") {
                        a.fit = Fit::Cover;
                    } else if self.eat("contain") {
                        a.fit = Fit::Contain;
                    } else if self.eat("none") {
                        a.fit = Fit::None;
                    } else if self.eat("focal") {
                        a.focal = [self.num()?.clamp(0.0, 1.0), self.num()?.clamp(0.0, 1.0)];
                    } else if self.eat("tint") {
                        a.tint = self.color()?;
                    } else {
                        break;
                    }
                }
                Ok(Paint::Atlas(a))
            }
            "pattern" => {
                let (uv, size) = self.region(atlas)?;
                let tile = og_core::glam::Vec2::new(self.num()?, self.num()?);
                if tile.x <= 0.0 || tile.y <= 0.0 {
                    return Err(self.err("takes a tile size above 0"));
                }
                let (mut rotation, mut scroll) = (0.0, og_core::glam::Vec2::ZERO);
                let mut tint = white();
                loop {
                    if self.eat("rotate") {
                        rotation = self.num()?.to_radians();
                    } else if self.eat("scroll") {
                        scroll = og_core::glam::Vec2::new(self.num()?, self.num()?);
                    } else if self.eat("tint") {
                        tint = self.color()?;
                    } else {
                        break;
                    }
                }
                Ok(Paint::Atlas(AtlasPaint {
                    uv,
                    size,
                    tint,
                    fit: Fit::Pattern {
                        tile,
                        rotation,
                        offset: og_core::glam::Vec2::ZERO,
                        scroll,
                    },
                    focal: [0.5, 0.5],
                }))
            }
            w => Err(self.err(&format!(
                "has '{w}', which is not a paint: a colour (a palette role, a role the style declares in `roles {{ }}`, or \"#rrggbb\"), linear, radial, conic, diamond, image or pattern"
            ))),
        }
    }
}

/// `fill vector "<asset>" [tint <colour>]`: a `type vector` asset the
/// style's assets bind ([`crate::StyleFile::vectors`]), contained in the
/// part and tinted (white: its own colours); `None` for any other fill.
pub fn vector_fill_from_entry(
    e: &Entry,
    atlas: &AtlasContext,
) -> Result<Option<(String, ColorRef)>, String> {
    let mut c = Cursor::of(e, atlas);
    if !c.eat("vector") {
        return Ok(None);
    }
    let path = c.p.string(c.at)?.to_string();
    c.at += 1;
    let tint = if c.eat("tint") {
        c.color()?
    } else {
        ColorRef::Literal(crate::style::Srgba::WHITE)
    };
    c.end()?;
    Ok(Some((path, tint)))
}

/// A symbolic colour used by a dressing outside a paint.
pub fn color_from_entry(e: &Entry, atlas: &AtlasContext) -> Result<ColorRef, String> {
    let mut c = Cursor::of(e, atlas);
    let color = c.color()?;
    c.end()?;
    Ok(color)
}

impl Paint {
    /// `fill <paint>` (see the module documentation).
    pub fn from_entry(e: &Entry, atlas: &AtlasContext) -> Result<Paint, String> {
        let mut c = Cursor::of(e, atlas);
        let p = c.paint(atlas)?;
        c.end()?;
        Ok(p)
    }
}

impl Stroke {
    /// `stroke <width> <paint> [align] [dash <len> <gap> [offset <o>]] [cap]`.
    pub fn from_entry(e: &Entry, atlas: &AtlasContext) -> Result<Stroke, String> {
        let mut c = Cursor::of(e, atlas);
        let width = c.num()?;
        if width <= 0.0 {
            return Err(c.err("takes a width above 0"));
        }
        let paint = c.paint(atlas)?;
        let mut s = Stroke {
            width,
            align: StrokeAlign::Center,
            dash: [0.0, 0.0],
            dash_offset: 0.0,
            dash_cap: Cap::Butt,
            paint,
        };
        while !c.done() {
            match c.word()? {
                "inside" => s.align = StrokeAlign::Inside,
                "center" => s.align = StrokeAlign::Center,
                "outside" => s.align = StrokeAlign::Outside,
                "dash" => {
                    s.dash = [c.num()?, c.num()?];
                    if s.dash[0] <= 0.0 || s.dash[1] < 0.0 {
                        return Err(c.err("takes a dash above 0 and a gap of 0 or more"));
                    }
                    if c.eat("offset") {
                        s.dash_offset = c.num()?;
                    }
                }
                "butt" => s.dash_cap = Cap::Butt,
                "round" => s.dash_cap = Cap::Round,
                "square" => s.dash_cap = Cap::Square,
                w => {
                    return Err(c.err(&format!(
                        "has '{w}': after the paint come inside, center or outside, dash <len> <gap> [offset <o>], and butt, round or square"
                    )));
                }
            }
        }
        Ok(s)
    }
}

impl Effect {
    /// `effect <kind> …` (see the module documentation).
    pub fn from_entry(e: &Entry, atlas: &AtlasContext) -> Result<Effect, String> {
        let mut c = Cursor::of(e, atlas);
        let fx = match c.word()? {
            k @ ("dropShadow" | "innerShadow") => {
                let offset = [c.num()?, c.num()?];
                let blur = c.num()?.max(0.0);
                let spread = if c.peek_number() { c.num()? } else { 0.0 };
                let color = c.color()?;
                if k == "dropShadow" {
                    Effect::DropShadow {
                        offset,
                        blur,
                        spread,
                        color,
                    }
                } else {
                    Effect::InnerShadow {
                        offset,
                        blur,
                        spread,
                        color,
                    }
                }
            }
            k @ ("outerGlow" | "innerGlow") => {
                let blur = c.num()?.max(0.0);
                let spread = if c.peek_number() { c.num()? } else { 0.0 };
                let color = c.color()?;
                if k == "outerGlow" {
                    Effect::OuterGlow {
                        blur,
                        spread,
                        color,
                    }
                } else {
                    Effect::InnerGlow {
                        blur,
                        spread,
                        color,
                    }
                }
            }
            "blur" => Effect::LayerBlur {
                radius: c.num()?.max(0.0),
            },
            "backdrop" => {
                let blur = if c.peek_number() {
                    c.num()?.max(0.0)
                } else {
                    0.0
                };
                let (mut saturation, mut brightness) = (1.0, 1.0);
                let mut tint = ColorRef::Literal(crate::style::Srgba {
                    rgb: [0, 0, 0],
                    alpha: 0.0,
                });
                let mut glass = crate::paint::GlassParts::default();
                loop {
                    if c.eat("saturation") {
                        saturation = c.num()?.max(0.0);
                    } else if c.eat("brightness") {
                        brightness = c.num()?.max(0.0);
                    } else if c.eat("tint") {
                        tint = c.color()?;
                        glass.own_tint = true;
                    } else if c.eat("glass") {
                        glass.weight = Some(match c.word()? {
                            "thin" => og_render::UiGlassWeight::Thin,
                            "regular" => og_render::UiGlassWeight::Regular,
                            "thick" => og_render::UiGlassWeight::Thick,
                            w => {
                                return Err(c.err(&format!(
                                    "has glass '{w}', which is not a weight: thin, regular or thick"
                                )));
                            }
                        });
                    } else if c.eat("elevation") {
                        glass.elevation = Some(match c.word()? {
                            "raised" => og_render::UiElevation::Raised,
                            "float" => og_render::UiElevation::Float,
                            "lift" => og_render::UiElevation::Lift,
                            w => {
                                return Err(c.err(&format!(
                                    "has elevation '{w}', which is not one: raised, float or lift"
                                )));
                            }
                        });
                    } else if c.eat("edge") {
                        glass.edge = c.num()?.max(0.0);
                        if c.is_color() {
                            glass.edge_color = Some(c.color()?);
                        }
                    } else if c.eat("solid") {
                        glass.solid = true;
                    } else if c.eat("css") {
                        glass.css_mix = true;
                    } else {
                        break;
                    }
                }
                Effect::Backdrop {
                    blur,
                    saturation,
                    brightness,
                    tint,
                    glass,
                }
            }
            w => {
                return Err(c.err(&format!(
                    "has '{w}', which is not an effect: dropShadow, innerShadow, outerGlow, innerGlow, blur or backdrop"
                )));
            }
        };
        c.end()?;
        Ok(fx)
    }
}

impl Mask {
    /// `mask <gradient>` (its colours' alpha) or `mask image <x> <y> <w> <h>`.
    pub fn from_entry(e: &Entry, atlas: &AtlasContext) -> Result<Mask, String> {
        let mut c = Cursor::of(e, atlas);
        let kind = c.word()?;
        let m = if kind == "image" {
            Mask::Atlas {
                uv: c.region(atlas)?.0,
            }
        } else {
            match c.gradient(kind)? {
                Some(g) => Mask::Gradient(g),
                None => {
                    return Err(c.err(&format!(
                        "has '{kind}': a mask is a gradient (linear, radial, conic, diamond) or image"
                    )));
                }
            }
        };
        c.end()?;
        Ok(m)
    }
}

impl DressMaterial {
    /// `material <builtin>|"<asset>" [<param>…]`.
    pub fn from_entry(e: &Entry, atlas: &AtlasContext) -> Result<DressMaterial, String> {
        let mut c = Cursor::of(e, atlas);
        let (layer, asset) = match c.peek() {
            Some(Value::String(path)) => {
                c.at += 1;
                (Layer::default(), Some(path.clone()))
            }
            _ => {
                let w = c.word()?;
                let b = BUILTINS
                    .iter()
                    .find(|(n, _)| *n == w)
                    .map(|(_, b)| *b)
                    .ok_or_else(|| {
                        c.err(&format!(
                            "names '{w}', which is not a built-in UI material ({}) or a quoted `type ui-material` asset",
                            names(&BUILTINS)
                        ))
                    })?;
                (
                    Layer {
                        material: crate::paint::Material::Builtin(b),
                        ..Layer::default()
                    },
                    None,
                )
            }
        };
        let mut layer = layer;
        let mut k = 0;
        while !c.done() {
            if k >= 4 * og_render::UI_MATERIAL_PARAMS {
                return Err(c.err(&format!(
                    "takes at most {} parameter numbers",
                    4 * og_render::UI_MATERIAL_PARAMS
                )));
            }
            layer.params[k / 4][k % 4] = c.num()?;
            k += 1;
        }
        Ok(DressMaterial {
            layer,
            asset,
            params: k,
        })
    }
}

/// `blend <mode>`.
pub fn blend_from_entry(e: &Entry) -> Result<Blend, String> {
    let mut c = Cursor::new(e, 0);
    let w = c.word()?;
    let b = BLENDS
        .iter()
        .find(|(n, _)| *n == w)
        .map(|(_, b)| *b)
        .ok_or_else(|| {
            c.err(&format!(
                "names '{w}', not a blend mode ({})",
                names(&BLENDS)
            ))
        })?;
    c.end()?;
    Ok(b)
}

/// `ornament <kind>…`.
pub fn ornaments_from_entry(e: &Entry) -> Result<Vec<OrnamentKind>, String> {
    let mut c = Cursor::new(e, 0);
    let mut out = Vec::new();
    while !c.done() {
        let k = match c.word()? {
            "corner" => OrnamentKind::Corner,
            "divider" => OrnamentKind::Divider,
            "flourish" => OrnamentKind::Flourish,
            w => {
                return Err(c.err(&format!(
                    "names '{w}', not an ornament (corner, divider or flourish)"
                )));
            }
        };
        if !out.contains(&k) {
            out.push(k);
        }
    }
    if out.is_empty() {
        return Err(c.err("takes one or more of corner, divider and flourish"));
    }
    Ok(out)
}

/// `segments <count> <gap> [ramp <from> <to>]` ([`crate::dress::Segments`]).
pub fn segments_from_entry(
    e: &Entry,
    ctx: &AtlasContext,
) -> Result<crate::dress::Segments, String> {
    let mut c = Cursor::of(e, ctx);
    let n = c.num()?;
    if !(2.0..=64.0).contains(&n) || n.fract() != 0.0 {
        return Err(c.err("takes a whole count of 2 to 64 segments"));
    }
    let gap = c.num()?;
    if !(0.0..=32.0).contains(&gap) {
        return Err(c.err("takes a gap of 0 to 32 CSS px"));
    }
    let ramp = if c.eat("ramp") {
        Some([c.color()?, c.color()?])
    } else {
        None
    };
    c.end()?;
    Ok(crate::dress::Segments {
        count: n as u32,
        gap,
        ramp,
    })
}

/// `sound <event> "<asset>"`.
pub fn sound_from_entry(e: &Entry) -> Result<(SoundEvent, String), String> {
    let mut c = Cursor::new(e, 0);
    let w = c.word()?;
    let s = SoundEvent::from_name(w).ok_or_else(|| {
        c.err(&format!(
            "names '{w}', not a UI event ({})",
            SoundEvent::names()
        ))
    })?;
    let path = c.p.string(c.at)?.to_string();
    c.at += 1;
    c.end()?;
    Ok((s, path))
}
