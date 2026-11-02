//! UI styles: a game's whole look, one `.ogt` of `type style`. Its values
//! (every palette role, the radii, the type ramp, durations) go into the
//! [`Theme`] fields; its art is an [`Art`], which dresses the
//! element classes ([`StylePart`]: panel, card, button and its states, field,
//! slot, tooltip, hud, progress track and fill) in nine-slice frames from
//! one atlas texture, adds ornaments (corners, divider, heading flourish), a
//! tiled panel texture, drop shadows and glows per class, text shadows and
//! glows per font role, a press squash and a hover lift, motion choices
//! (entrances, stagger, count-ups, eased progress, refused-action shake,
//! bursts) and UI sounds.
//!
//! A style is complete: it sets every palette role and every radius, so
//! nothing of another look shows through it. There is no base to fall back
//! on; before a game picks a style it has the engine's default style
//! ([`crate::theme`], [`default_dressings`]). [`Art::default`] dresses nothing and
//! moves nothing: every draw path checks for a slice, an ornament or a
//! motion value first, so a style without art draws flat.
//!
//! Version 2 (docs/ui.md §12) adds the tokens of [`crate::tokens`] (carried
//! in [`Theme::tokens`], not yet drawn from) and dresses parts by selector
//! (`crate::dress`): its dressings of the twelve parts above fill the same
//! [`Art`], so a style upgraded by [`upgrade_style`] draws as before.
//!
//! The grammar is [`parse_style`]'s; the runtime loads a style's atlas,
//! fonts, icons and sounds through the bundle and hands the handles to
//! [`StyleFile::theme`].

use crate::dress::{DRAW_KEYS, DressFrame, Dressing, Selector};
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
    /// The icon texture's asset id, its grid and names.
    pub icons: Option<(String, u32, u32, Vec<String>)>,
    /// Sound asset ids by [`UiSound`] (in version 2 the first of the
    /// event's set).
    pub sounds: [Option<String>; 6],
    /// Each [`SoundEvent`]'s sound set, asset ids (empty: silent).
    pub feedback_sounds: [Vec<String>; SoundEvent::COUNT],
    /// The part dressings (`crate::dress`), in file order; a version 1
    /// style's are its twelve parts' ([`StylePart::selector`]).
    pub dressings: Vec<Dressing>,
}

impl StyleFile {
    /// Every asset id the file names (the cook checks they exist).
    pub fn references(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        out.extend(self.atlas.clone());
        out.extend(self.atlas_dense.iter().flatten().cloned());
        out.extend(self.fonts.iter().flatten().cloned());
        out.extend(self.icons.as_ref().map(|i| i.0.clone()));
        out.extend(self.sounds.iter().flatten().cloned());
        for s in self.feedback_sounds.iter().flatten() {
            if !out.contains(s) {
                out.push(s.clone());
            }
        }
        for v in self.vector_assets() {
            if !out.contains(&v) {
                out.push(v);
            }
        }
        out
    }

    /// The vector assets its dressings fill with (`fill vector "<asset>"`),
    /// each once, in file order.
    pub fn vector_assets(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for d in &self.dressings {
            for (_, v) in &d.draw.vectors {
                if !out.contains(v) {
                    out.push(v.clone());
                }
            }
        }
        out
    }

    /// Binds its dressings' vector fills to the loaded vectors (`handle`
    /// of an asset id; `None` leaves that fill unbound, drawing nothing):
    /// the style's asset binding of docs/ui.md §13.
    pub fn bind_vectors(&mut self, handle: impl Fn(&str) -> Option<crate::paint::UiVectorHandle>) {
        for d in &mut self.dressings {
            for (at, asset) in &d.draw.vectors {
                if let (Some(h), Some(crate::paint::Paint::Vector { vector, .. })) =
                    (handle(asset), d.draw.fills.get_mut(*at))
                {
                    *vector = h;
                }
            }
        }
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

    /// The line that sets `part`'s tone (`og cook` appends what it
    /// measured): `tone <part> …` in version 1, a `dress` block in version 2.
    pub fn tone_text(&self, part: StylePart, t: [f32; 4]) -> String {
        let v = format!("{:.3} {:.3} {:.3} {:.2}", t[0], t[1], t[2], t[3]);
        if self.version >= 2 {
            format!("dress \"{}\" {{\n  tone {v}\n}}", part.selector())
        } else {
            format!("tone {} {v}", part.name())
        }
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
        if let Some(i) = self.icons.as_mut() {
            at(&mut i.0);
        }
        for s in self.sounds.iter_mut().flatten() {
            at(s);
        }
        for s in self.feedback_sounds.iter_mut().flatten() {
            at(s);
        }
        for d in &mut self.dressings {
            for (_, v) in &mut d.draw.vectors {
                at(v);
            }
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
        "displayXl2" => &mut r.display_xl2,
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
        _ => return None,
    })
}

/// `#rrggbb` or `#rrggbbaa`.
pub fn parse_color(s: &str) -> Option<Srgba> {
    let h = s.strip_prefix('#')?;
    if !(h.len() == 6 || h.len() == 8) || !h.is_ascii() {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some(Srgba {
        rgb: [byte(0)?, byte(2)?, byte(4)?],
        alpha: if h.len() == 8 {
            f32::from(byte(6)?) / 255.0
        } else {
            1.0
        },
    })
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
            .ok_or_else(|| self.err(&format!("takes \"#rrggbb\" or \"#rrggbbaa\", not \"{s}\"")))
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

/// The dressings a version 1 style's art makes: each of the twelve parts
/// at its [`StylePart::selector`], its class's shadow, glow, glass, corners
/// and tile on the class, the press and hover on `Button.base`.
pub(crate) fn v1_dressings(art: &Art) -> Vec<Dressing> {
    let mut out = Vec::new();
    for part in StylePart::ALL {
        let mut f = DressFrame {
            slice: art.slices[part.index()],
            ..DressFrame::default()
        };
        if part.class() == part {
            let i = part.index();
            f.shadow = art.shadows[i];
            f.glow = art.glows[i];
            f.glass = art.glass[i];
            f.corners = art.corner_parts & part.bit() != 0;
            f.tile = art.tile.is_some_and(|t| t.parts & part.bit() != 0);
        }
        if part == StylePart::Button {
            f.press = art.press;
            f.hover = art.hover;
        }
        if f != DressFrame::default() {
            out.push(Dressing {
                selector: part.selector(),
                frame: f,
                draw: Default::default(),
            });
        }
    }
    out
}

/// The default style's part dressings ([`crate::theme::DRESSINGS`], the
/// design language's: glass on floating chrome, borderless controls),
/// parsed once: what a UI without a style is dressed with.
pub fn default_dressings() -> Vec<Dressing> {
    static PARSED: std::sync::OnceLock<Vec<Dressing>> = std::sync::OnceLock::new();
    PARSED
        .get_or_init(|| {
            use std::fmt::Write;
            // A complete style around them: the dark palette and radii as a
            // version 1 file, upgraded, then the dressings.
            let p = &crate::theme::DARK;
            let r = &crate::theme::RADIUS;
            let mut v1 = String::from(
                "type style\nversion 1\n---\nname \"default\"\nscheme dark\npalette {\n",
            );
            for name in Palette::NAMES {
                let c = p.get(name).unwrap_or(Srgba::BLACK);
                let a = (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
                let [x, y, z] = c.rgb;
                let _ = writeln!(v1, "  {name} \"#{x:02x}{y:02x}{z:02x}{a:02x}\"");
            }
            let _ = write!(
                v1,
                "}}\nradius {{\n  small {}\n  control {}\n  bar {}\n  panel {}\n}}\n",
                r.small, r.control, r.bar, r.panel
            );
            let v2 = upgrade_style(&v1).expect("the default style upgrades");
            parse_style(&(v2 + crate::theme::DRESSINGS))
                .expect("the default style's dressings parse")
                .dressings
        })
        .clone()
}

/// The dressings a theme draws with until a style sets its own: a
/// version 1 style's art ([`v1_dressings`]), or the default style's for a
/// theme with none.
pub(crate) fn theme_dressings(art: &Art) -> Vec<Dressing> {
    if *art == Art::default() {
        default_dressings()
    } else {
        v1_dressings(art)
    }
}

/// `og style upgrade`: a version 1 style written as version 2. Every
/// value version 1 set keeps its value (the palette, the ramp, the fonts,
/// the art and its motion, so the style draws the same); the radii become
/// the shape language's corners, the durations motion tokens, the per-part
/// lines dressings of [`StylePart::selector`], the sounds one-sound sets;
/// the new tokens are [`StyleTokens::derive`]d from the palette and scheme.
pub fn upgrade_style(text: &str) -> Result<String, String> {
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
    for k in ["name", "scheme", "atlas", "scale", "palette"] {
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
    for d in &old.dressings {
        let part = StylePart::from_selector(&d.selector).expect("a version 1 part");
        let mut entries: Vec<Entry> = Vec::new();
        let rest = |e: &Entry| {
            let mut n = e.clone();
            n.values.remove(0);
            n.line = 0;
            n
        };
        let of = |e: &Entry, class: bool| {
            e.values
                .first()
                .and_then(Value::as_word)
                .and_then(StylePart::from_name)
                .is_some_and(|p| if class { p.class() == part } else { p == part })
        };
        for k in ["slice", "tone"] {
            entries.extend(body.get_all(k).filter(|e| of(e, false)).map(rest));
        }
        if part.class() == part {
            for k in ["shadow", "glow", "glass"] {
                entries.extend(body.get_all(k).filter(|e| of(e, true)).map(rest));
            }
            if d.frame.corners {
                entries.push(Entry::new("corners"));
            }
            if d.frame.tile {
                entries.push(Entry::new("tile"));
            }
        }
        if part == StylePart::Button {
            copy_into(&mut entries, &body, "press");
            copy_into(&mut entries, &body, "hover");
        }
        if !entries.is_empty() {
            out.push(
                Entry::new("dress")
                    .with(Value::String(d.selector.to_string()))
                    .with_block(og_core::ogt::Block { entries }),
            );
        }
    }
    doc.version = 2;
    doc.body = out;
    let text2 = doc.to_text().map_err(|e| e.to_string())?.replacen(
        "---\n",
        "---\n# Upgraded to version 2 by og style upgrade: its version 1 values kept, the new tokens derived from its palette and scheme (docs/ui.md §12).\n",
        1,
    );
    let new =
        parse_style(&text2).map_err(|e| format!("the upgraded style does not read back: {e}"))?;
    let (a, b) = (&old.theme, &new.theme);
    if (
        a.palette,
        a.space,
        a.radius,
        a.type_ramp,
        a.durations_ms,
        a.art,
    ) != (
        b.palette,
        b.space,
        b.radius,
        b.type_ramp,
        b.durations_ms,
        b.art,
    ) || old.sounds != new.sounds
        || old.fonts != new.fonts
    {
        return Err(
            "the upgrade would change how the style draws; this is a bug in og style upgrade"
                .into(),
        );
    }
    Ok(text2)
}

fn copy_into(out: &mut Vec<Entry>, body: &og_core::ogt::Block, key: &str) {
    for e in body.get_all(key) {
        let mut n = e.clone();
        n.line = 0;
        out.push(n);
    }
}

mod art;
mod parse;
pub use art::*;
pub use parse::{dense_name, parse_style};
