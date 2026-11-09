//! Style asset metadata retained until the CSS pack loader replaces `.ogt`
//! packs: fonts, icons, sounds, ornaments and layout tokens. Part appearance
//! is CSS; this parser does not create or resolve appearance rules.

use crate::motion::TransitionKind;
use crate::style::{Durations, Palette, Radius, Scheme, Srgba, Theme, TypeRamp, TypeStep};
use crate::tokens::{
    self, ElementClass, Level, RadiusStep, ShapeToken, SizeStep, SoundEvent, SpaceStep,
    StrokeWidth, StyleTokens, names_of,
};
use og_core::ogt::{Document, Entry, Value};
use og_render::TextureHandle;

/// The `.ogt` type of a UI style.
pub const STYLE_TYPE: &str = "style";

/// A style file parsed: the theme it makes and the assets it names.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleFile {
    pub name: String,
    /// The file's `version`: 1 or 2.
    pub version: u32,
    /// The theme with the style's palette, radii, ramp and durations; its
    /// art has no atlas or sounds yet (the loader adds the handles).
    pub theme: Theme,
    /// The atlas texture's asset id.
    pub atlas: Option<String>,
    /// The atlas at 2× and 3× its texels, when the `atlas` line names
    /// those densities (`atlas "atlas.png" 384 256 2 3`): the same layout
    /// at that many texels per CSS px, `atlas@2x.png` and `atlas@3x.png`
    /// beside it. The UI draws from the one nearest at or above its device
    /// pixels per CSS px ([`Art::atlas_for`]).
    pub atlas_dense: [Option<String>; 2],
    /// Font asset ids: body, display, numeric (the mono role), fallback
    /// (characters the others lack, a last resort), cjk and cjkDisplay (the
    /// style's own Simplified Chinese faces beside body and display: see
    /// [`crate::Fonts::companions`]).
    pub fonts: [Option<String>; 6],
    /// The weight of each of `fonts` (`body 300 "…"`; 0: unstated, 400).
    pub font_weights: [u16; 6],
    /// The style's other faces: a role's other weights and italics
    /// (`body 700 "…"`, `body italic "…"`, `display 800 italic "…"`) and
    /// the CJK companions of each locale's forms (`cjk ja "…"`, `cjkDisplay
    /// ko 700 "…"`): see [`crate::Fonts::faces`].
    pub font_faces: Vec<StyleFace>,
    /// The icon texture's asset id, its grid and names.
    pub icons: Option<(String, u32, u32, Vec<String>)>,
    /// Sound asset ids by [`UiSound`] (in version 2 the first of the
    /// event's set).
    pub sounds: [Option<String>; 6],
    /// Each [`SoundEvent`]'s sound set, asset ids (empty: silent).
    pub feedback_sounds: [Vec<String>; SoundEvent::COUNT],
    /// `recolour "<from>" <role>` lines: the painted art follows the
    /// palette (docs/art.md §2.7). Every authored slice and ornament the
    /// style cuts takes these unless its dressing sets its own.
    pub recolour: Vec<Recolour>,
}

/// One accent of painted art and the palette role it follows: the art was
/// painted with `from`, and the cook moves the pixels of its hue to the
/// role's colour (docs/art.md §2.7), keeping each pixel's lightness and
/// chroma relative to `from`. Where the role is `from`, nothing moves.
#[derive(Clone, Debug, PartialEq)]
pub struct Recolour {
    pub from: Srgba,
    /// A palette role (`primary`, `focus`, `gold` …).
    pub role: String,
}

impl StyleFile {
    /// Every asset id the file names (the cook checks they exist).
    pub fn references(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        out.extend(self.atlas.clone());
        out.extend(self.atlas_dense.iter().flatten().cloned());
        out.extend(self.fonts.iter().flatten().cloned());
        out.extend(self.font_faces.iter().map(|f| f.path.clone()));
        out.extend(self.icons.as_ref().map(|i| i.0.clone()));
        out.extend(self.sounds.iter().flatten().cloned());
        for s in self.feedback_sounds.iter().flatten() {
            if !out.contains(s) {
                out.push(s.clone());
            }
        }
        out
    }

    /// The theme with the loaded atlas and sounds: `sounds` the version 1
    /// events' handles, `feedback` the handle of each asset id of the
    /// [`StyleFile::feedback_sounds`] sets (0: not loaded), so every
    /// event plays its whole set.
    pub fn theme_with_feedback(
        &self,
        atlas: Option<TextureHandle>,
        sounds: [u64; 6],
        feedback: impl Fn(&str) -> u64,
    ) -> Theme {
        let mut t = self.theme(atlas, sounds);
        for e in SoundEvent::ALL {
            let set = &self.feedback_sounds[e.index()];
            if set.is_empty() {
                continue;
            }
            let f = &mut t.tokens.feedback[e.index()];
            let n = set.len().min(crate::tokens::SET_SOUNDS);
            for (k, id) in set.iter().take(n).enumerate() {
                f.sounds[k] = feedback(id);
            }
            f.count = n as u8;
        }
        t
    }

    /// Resolves the asset ids the file names against the style file's own
    /// id (`style_id`, such as `styles/fantasy/style.ogt`), so a pack copied
    /// to another directory keeps working: an id is first read relative to
    /// the style's directory (`atlas.png` is `styles/fantasy/atlas.png`,
    /// `../shared/rope.png` climbs one), and taken as written, from
    /// `assets/`, when no asset `exists` there. The cook and the runtime
    /// resolve the same way, so what the cook checked is what loads.
    pub fn resolve(&mut self, style_id: &str, exists: impl Fn(&str) -> bool) {
        let at = |r: &mut String| *r = resolve_reference(style_id, r, &exists);
        if let Some(a) = self.atlas.as_mut() {
            at(a);
        }
        for a in self.atlas_dense.iter_mut().flatten() {
            at(a);
        }
        for f in self.fonts.iter_mut().flatten() {
            at(f);
        }
        for f in &mut self.font_faces {
            at(&mut f.path);
        }
        if let Some(i) = self.icons.as_mut() {
            at(&mut i.0);
        }
        for s in self.sounds.iter_mut().flatten() {
            at(s);
        }
        for s in self.feedback_sounds.iter_mut().flatten() {
            at(s);
        }
    }

    /// The theme with the loaded atlas and sound handles (the denser
    /// atlases join with [`Theme::with_dense_atlases`]).
    pub fn theme(&self, atlas: Option<TextureHandle>, sounds: [u64; 6]) -> Theme {
        let mut t = self.theme;
        t.art.atlas = atlas;
        t.art.sounds = sounds;
        for u in UiSound::ALL {
            let f = &mut t.tokens.feedback[u.event().index()];
            f.sounds[0] = sounds[u as usize];
            if sounds[u as usize] != 0 {
                f.count = f.count.max(1);
            }
        }
        t
    }
}

/// An asset id a style file names, resolved against the style's own id (see
/// [`StyleFile::resolve`]): relative to its directory when that asset
/// exists, else as written.
pub fn resolve_reference(style_id: &str, reference: &str, exists: impl Fn(&str) -> bool) -> String {
    let dir = style_id.rsplit_once('/').map_or("", |(d, _)| d);
    let mut parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
    let mut inside = true;
    for p in reference.split('/') {
        match p {
            "" | "." => {}
            ".." => inside &= parts.pop().is_some(),
            p => parts.push(p),
        }
    }
    let relative = parts.join("/");
    if inside && !relative.is_empty() && exists(&relative) {
        relative
    } else {
        reference.to_string()
    }
}

/// A palette colour by its ThemeColor name (`primary`, `inkMuted`, `panelRaised`).
pub fn palette_color<'a>(p: &'a mut Palette, name: &str) -> Option<&'a mut Srgba> {
    p.get_mut(name)
}

/// A type step by its TypeStepName (`body`, `xl2`, `displayLg`).
/// Every TypeStepName, smallest steps first in the ramp's own order.
pub const STEP_NAMES: [&str; 22] = [
    "body",
    "xs",
    "meta",
    "caption",
    "sm",
    "reading",
    "base",
    "lg",
    "xl",
    "xl2",
    "xl3",
    "xl4",
    "xl5",
    "xl6",
    "xl7",
    "xl8",
    "displaySm",
    "displayMd",
    "displayLg",
    "displayXl",
    "displayXl2",
    "hero",
];

/// The steps an auto-fit text at `from` takes down to `min`: `from`, then
/// each smaller size of the ramp, largest first, with `from`'s tracking and
/// features (a display heading stays one as it shrinks).
pub(crate) fn fit_steps(ramp: &TypeRamp, from: TypeStep, min: TypeStep) -> Vec<TypeStep> {
    let mut r = *ramp;
    let mut out: Vec<TypeStep> = STEP_NAMES
        .iter()
        .filter_map(|n| ramp_step(&mut r, n).copied())
        .filter(|s| s.size < from.size && s.size >= min.size.min(from.size))
        .map(|s| TypeStep {
            size: s.size,
            line_height: s.line_height,
            cjk_line_height: s.cjk_line_height,
            ..from
        })
        .collect();
    out.sort_by(|a, b| b.size.total_cmp(&a.size));
    out.dedup_by(|a, b| a.size == b.size);
    out.insert(0, from);
    out
}

pub fn ramp_step<'a>(r: &'a mut TypeRamp, name: &str) -> Option<&'a mut TypeStep> {
    Some(match name {
        "body" => &mut r.body,
        "xs" => &mut r.xs,
        "meta" => &mut r.meta,
        "caption" => &mut r.caption,
        "sm" => &mut r.sm,
        "reading" => &mut r.reading,
        "base" => &mut r.base,
        "lg" => &mut r.lg,
        "xl" => &mut r.xl,
        "xl2" => &mut r.xl2,
        "xl3" => &mut r.xl3,
        "xl4" => &mut r.xl4,
        "xl5" => &mut r.xl5,
        "xl6" => &mut r.xl6,
        "xl7" => &mut r.xl7,
        "xl8" => &mut r.xl8,
        "displaySm" => &mut r.display_sm,
        "displayMd" => &mut r.display_md,
        "displayLg" => &mut r.display_lg,
        "displayXl" => &mut r.display_xl,
        "displayXl2" | "hero" => &mut r.display_xl2,
        _ => return None,
    })
}

/// The radius names, all of which a style sets.
const RADII: [&str; 4] = ["small", "control", "bar", "panel"];

fn radius_slot<'a>(r: &'a mut Radius, name: &str) -> Option<&'a mut f32> {
    Some(match name {
        "small" => &mut r.small,
        "control" => &mut r.control,
        "bar" => &mut r.bar,
        "panel" => &mut r.panel,
        _ => return None,
    })
}

fn duration_slot<'a>(d: &'a mut Durations, name: &str) -> Option<&'a mut f32> {
    Some(match name {
        "base" => &mut d.base,
        "bounce" => &mut d.bounce,
        "fast" => &mut d.fast,
        "instant" => &mut d.instant,
        "slow" => &mut d.slow,
        "spring" => &mut d.spring,
        _ => return None,
    })
}

fn kind_of(w: &str) -> Option<TransitionKind> {
    Some(match w {
        "fade" => TransitionKind::Fade,
        "slideUp" => TransitionKind::SlideUp,
        "slideDown" => TransitionKind::SlideDown,
        "slideLeft" => TransitionKind::SlideLeft,
        "slideRight" => TransitionKind::SlideRight,
        "scale" => TransitionKind::Scale,
        "pop" => TransitionKind::Pop,
        "glitch" => TransitionKind::Glitch,
        _ => return None,
    })
}

/// A CSS hex colour: `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa` (CSS Color 4
/// §5.2). Every reader of a literal colour parses it here.
pub fn parse_color(s: &str) -> Option<Srgba> {
    let h = s.strip_prefix('#')?;
    let d: Vec<u8> = h
        .chars()
        .map(|c| c.to_digit(16).map(|v| v as u8))
        .collect::<Option<_>>()?;
    let v: Vec<u8> = match d.len() {
        3 | 4 => d.iter().map(|x| x * 17).collect(),
        6 | 8 => d.chunks(2).map(|p| p[0] * 16 + p[1]).collect(),
        _ => return None,
    };
    Some(Srgba {
        rgb: [v[0], v[1], v[2]],
        alpha: v.get(3).map_or(1.0, |&a| f32::from(a) / 255.0),
    })
}

/// One of a style file's other faces ([`StyleFile::font_faces`]).
#[derive(Clone, Debug, PartialEq)]
pub struct StyleFace {
    /// Body, Display or Mono (`numeric`); with `cjk`, Body is `cjk` and
    /// Display `cjkDisplay`.
    pub role: crate::FontRole,
    pub cjk: bool,
    pub locale: crate::CjkLocale,
    /// 100 to 900.
    pub weight: u16,
    pub italic: bool,
    /// Its asset id.
    pub path: String,
}

impl StyleFile {
    /// Every CJK companion's asset id with the forms it draws: `cjk` and
    /// `cjkDisplay` (Simplified Chinese) and the other locales' faces. The
    /// cook bundles each only for a game that shows its locale.
    pub fn companions(&self) -> Vec<(crate::CjkLocale, String)> {
        self.fonts[4..]
            .iter()
            .flatten()
            .map(|p| (crate::CjkLocale::Hans, p.clone()))
            .chain(
                self.font_faces
                    .iter()
                    .filter(|f| f.cjk)
                    .map(|f| (f.locale, f.path.clone())),
            )
            .collect()
    }
}

pub(crate) struct P<'a> {
    pub(crate) e: &'a Entry,
}

impl<'a> P<'a> {
    pub(crate) fn err(&self, what: &str) -> String {
        format!("line {}: `{}` {what}", self.e.line, self.e.key)
    }
    pub(crate) fn num(&self, i: usize) -> Result<f32, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_number)
            .map(|n| n.as_f32())
            .filter(|v| v.is_finite())
            .ok_or_else(|| self.err(&format!("takes a number at position {}", i + 1)))
    }
    pub(crate) fn num_or(&self, i: usize, d: f32) -> Result<f32, String> {
        if self.e.values.len() > i {
            self.num(i)
        } else {
            Ok(d)
        }
    }
    pub(crate) fn word(&self, i: usize) -> Result<&'a str, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_word)
            .ok_or_else(|| self.err(&format!("takes a word at position {}", i + 1)))
    }
    pub(crate) fn string(&self, i: usize) -> Result<&'a str, String> {
        self.e
            .values
            .get(i)
            .and_then(Value::as_str)
            .ok_or_else(|| self.err(&format!("takes a quoted string at position {}", i + 1)))
    }
    pub(crate) fn color(&self, i: usize) -> Result<Srgba, String> {
        let s = self.string(i)?;
        parse_color(s)
            .ok_or_else(|| self.err(&format!("takes a hex colour (\"#rgb\", \"#rgba\", \"#rrggbb\" or \"#rrggbbaa\"), not \"{s}\"")))
    }
    pub(crate) fn block(&self) -> Result<&'a og_core::ogt::Block, String> {
        self.e
            .block
            .as_ref()
            .ok_or_else(|| self.err("takes a { } block"))
    }
}

/// The atlas region `x y w h` (texels) at `values[i..i + 4]` as UVs.
fn region(p: &P, i: usize, size: (f32, f32)) -> Result<([f32; 4], [f32; 4]), String> {
    let (x, y, w, h) = (p.num(i)?, p.num(i + 1)?, p.num(i + 2)?, p.num(i + 3)?);
    if w <= 0.0 || h <= 0.0 || x < 0.0 || y < 0.0 || x + w > size.0 || y + h > size.1 {
        return Err(p.err(&format!(
            "region {x} {y} {w} {h} is not inside the {}×{} atlas",
            size.0, size.1
        )));
    }
    Ok((
        [x / size.0, y / size.1, (x + w) / size.0, (y + h) / size.1],
        [x, y, w, h],
    ))
}

fn parts_mask(p: &P, from: usize) -> Result<u32, String> {
    let mut mask = 0;
    for i in from..p.e.values.len() {
        let w = p.word(i)?;
        let part = StylePart::from_name(w)
            .ok_or_else(|| p.err(&format!("names an unknown part '{w}'")))?;
        mask |= part.bit();
    }
    Ok(mask)
}

fn role_index(p: &P, w: &str) -> Result<usize, String> {
    match w {
        "body" => Ok(0),
        "display" => Ok(1),
        "numeric" | "mono" => Ok(2),
        _ => Err(p.err(&format!("names the role '{w}' (body, display or numeric)"))),
    }
}

/// A nine-slice at `values[at..]`: the region `x y w h` in texels, one
/// corner inset or four, then `tint` or `authored` and `stretch`, `repeat`
/// or `round`.
fn slice_at(p: &P, at: usize, size: (f32, f32), scale: f32) -> Result<Slice, String> {
    let e = p.e;
    let (uv, px) = region(p, at, size)?;
    let numbers = e.values[(at + 4).min(e.values.len())..]
        .iter()
        .take_while(|v| v.as_number().is_some())
        .count();
    let b = match numbers {
        1 => {
            let v = p.num(at + 4)?;
            [v; 4]
        }
        4 => [
            p.num(at + 4)?,
            p.num(at + 5)?,
            p.num(at + 6)?,
            p.num(at + 7)?,
        ],
        _ => {
            return Err(p.err("takes one corner inset or four (left top right bottom), in texels"));
        }
    };
    if b[0] + b[2] > px[2] || b[1] + b[3] > px[3] || b.iter().any(|v| *v < 0.0) {
        return Err(p.err("has corner insets larger than its region"));
    }
    let (mut tint, mut fill) = (true, SliceFill::Stretch);
    for v in &e.values[at + 4 + numbers..] {
        match v.as_word() {
            Some("tint") => tint = true,
            Some("authored") => tint = false,
            Some("stretch") => fill = SliceFill::Stretch,
            Some("repeat") => fill = SliceFill::Repeat,
            Some("round") => fill = SliceFill::Round,
            _ => {
                return Err(p.err(&format!(
                    "ends with tint or authored and stretch, repeat or round, not '{}'",
                    v.as_word().unwrap_or("a number")
                )));
            }
        }
    }
    let middle = [(px[2] - b[0] - b[2]) * scale, (px[3] - b[1] - b[3]) * scale];
    if fill != SliceFill::Stretch && (middle[0] < 1.0 || middle[1] < 1.0) {
        return Err(p.err("tiles its edges but leaves no middle between the corners to tile"));
    }
    Ok(Slice {
        uv,
        border_uv: [b[0] / size.0, b[1] / size.1, b[2] / size.0, b[3] / size.1],
        border: [b[0] * scale, b[1] * scale, b[2] * scale, b[3] * scale],
        tint,
        fill,
        middle,
        tone: Slice::WHITE,
    })
}

/// A slice's tone at `values[at..]`: a grey and an alpha, or r g b a (linear).
fn tone_at(p: &P, at: usize) -> Result<[f32; 4], String> {
    let tone = match p.e.values.len().saturating_sub(at) {
        1 | 2 => {
            let v = p.num(at)?;
            [v, v, v, p.num_or(at + 1, 1.0)?]
        }
        4 => [p.num(at)?, p.num(at + 1)?, p.num(at + 2)?, p.num(at + 3)?],
        _ => return Err(p.err("takes a grey and an alpha, or r g b and an alpha")),
    };
    if tone.iter().any(|v| !(0.0..=1.0).contains(v)) {
        return Err(p.err("takes values from 0 to 1 (linear light)"));
    }
    Ok(tone)
}

fn ink_at(p: &P, i: usize) -> Result<Ink, String> {
    let iw = p.word(i)?;
    Ink::parse(iw).ok_or_else(|| {
        p.err(&format!(
            "ink '{iw}' is authored, primary, ink, muted, line or focus"
        ))
    })
}

fn glow_at(p: &P, at: usize) -> Result<Glow, String> {
    Ok(Glow {
        size: p.num(at)?.max(0.0),
        ink: ink_at(p, at + 1)?,
        alpha: p.num(at + 2)?.clamp(0.0, 1.0),
    })
}

fn glass_at(p: &P, at: usize) -> Result<Glass, String> {
    Ok(Glass {
        blur: p.num(at)?.max(0.0),
        fill: p.num(at + 1)?.clamp(0.0, 1.0),
        tint: p.num_or(at + 2, 0.25)?.clamp(0.0, 1.0),
    })
}

fn press_at(p: &P) -> Result<Press, String> {
    Ok(Press {
        sx: p.num(0)?,
        sy: p.num(1)?,
        dy: p.num_or(2, 0.0)?,
    })
}

fn hover_at(p: &P) -> Result<Hover, String> {
    Ok(Hover {
        dy: p.num(0)?,
        scale: p.num_or(1, 1.0)?,
    })
}

fn part_at(p: &P, i: usize) -> Result<StylePart, String> {
    let w = p.word(i)?;
    StylePart::from_name(w).ok_or_else(|| p.err(&format!("names an unknown part '{w}'")))
}

/// `og style upgrade`: a version 1 style written as version 2. Every
/// value version 1 set keeps its value (the palette, the ramp, the fonts,
/// the art and its motion, so the style draws the same); the radii become
/// the shape language's corners, the durations motion tokens, the per-part
/// lines dressings of [`StylePart::selector`], the sounds one-sound sets;
/// the new tokens are [`StyleTokens::derive`]d from the palette and scheme.
pub fn upgrade_style(text: &str) -> Result<String, String> {
    migrate_style(&upgrade_style_v2(text)?)
}

/// [`upgrade_style`]'s version 2 text (what `og style new` builds on).
pub(crate) fn upgrade_style_v2(text: &str) -> Result<String, String> {
    let old = parse_style(text)?;
    if old.version != 1 {
        return Err(format!(
            "the style is version {} already: nothing to upgrade",
            old.version
        ));
    }
    let mut doc = Document::parse(text).map_err(|e| e.to_string())?;
    let body = std::mem::take(&mut doc.body);
    let icons: Vec<String> = old.icons.as_ref().map(|i| i.3.clone()).unwrap_or_default();
    let mut sounds: [Vec<String>; SoundEvent::COUNT] = Default::default();
    for e in body.get_all("sound") {
        let p = P { e };
        if let Some(u) = UiSound::from_name(p.word(0)?) {
            sounds[u.event().index()] = vec![p.string(1)?.to_string()];
        }
    }
    let (token_blocks, motion_tokens, sounds_block) = old.theme.tokens.entries(&icons, &sounds);
    let mut out = og_core::ogt::Block::new();
    let copy = |out: &mut og_core::ogt::Block, key: &str| {
        for e in body.get_all(key) {
            out.push(e.clone());
        }
    };
    for k in ["name", "scheme"] {
        copy(&mut out, k);
    }
    // A version 1 style's screens stay as they shipped (`Art::legacy`).
    out.push(Entry::new("screens").with(Value::Number(1u32.into())));
    for k in ["atlas", "scale", "palette"] {
        copy(&mut out, k);
    }
    out.entries.extend(token_blocks);
    for k in [
        "ramp", "fonts", "icons", "soft", "corner", "divider", "flourish", "crest", "edge",
    ] {
        copy(&mut out, k);
    }
    for e in body.get_all("tile") {
        let mut e = e.clone();
        let numbers = e.values[4.min(e.values.len())..]
            .iter()
            .take_while(|v| v.as_number().is_some())
            .count();
        e.values.truncate(4 + numbers);
        out.push(e);
    }
    for k in ["textShadow", "textGlow", "textWeight"] {
        copy(&mut out, k);
    }
    let mut motion: Vec<Entry> = Vec::new();
    if let Some(e) = body.get("motion")
        && let Some(b) = &e.block
    {
        for c in &b.entries {
            match c.key.as_str() {
                "stagger" => {}
                "hoverSpring" | "pressSpring" => {
                    let q = P { e: c };
                    let s = tokens::SpringToken::from_spring(crate::theme::Spring {
                        damping: q.num(0)?.clamp(0.05, 4.0),
                        response_ms: q.num(1)?.clamp(16.0, 4000.0),
                    });
                    let mut n = Entry::new(c.key.clone());
                    n.values = vec![
                        tokens::num_value(s.duration_ms),
                        tokens::num_value(s.bounce),
                    ];
                    motion.push(n);
                }
                _ => motion.push(c.clone()),
            }
        }
    }
    motion.extend(motion_tokens);
    out.push(Entry::new("motion").with_block(og_core::ogt::Block { entries: motion }));
    out.push(sounds_block);
    doc.version = 2;
    doc.body = out;
    let text2 = doc.to_text().map_err(|e| e.to_string())?.replacen(
        "---\n",
        "---\n# Upgraded to version 2 by og style upgrade: its version 1 values kept, the new tokens derived from its palette and scheme (docs/ui.md §3).\n",
        1,
    );
    let new =
        parse_style(&text2).map_err(|e| format!("the upgraded style does not read back: {e}"))?;
    let (a, b) = (&old.theme, &new.theme);
    // Its knobs and switches draw round once upgraded (Art::v1_controls).
    let mut b = *b;
    b.art.v1_controls = a.art.v1_controls;
    let b = &b;
    if (a.palette, a.space, a.radius, a.type_ramp, a.durations_ms)
        != (b.palette, b.space, b.radius, b.type_ramp, b.durations_ms)
        || old.sounds != new.sounds
        || old.fonts != new.fonts
    {
        return Err(
            "the upgrade would change how the style draws; this is a bug in og style upgrade"
                .into(),
        );
    }
    Ok(text2)
}

mod art;
pub mod css;
mod parse;
pub use art::*;
pub use css::{migrate_style, read_style_doc, write_style_doc};
pub use parse::{dense_name, parse_style};
