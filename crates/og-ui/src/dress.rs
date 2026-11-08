//! Part dressings (docs/ui.md §3.2): a component names its parts
//! (`Button.base`, `Meter.fill`, `ShopCard.frame`) and a version 2 style
//! dresses any of them, by variant and state, with a selector:
//!
//! ```text
//! dress "Button.base[primary][pressed]" {
//!   slice 276 4 48 48 10 tint
//!   shadow 0 2 0 0 "#00000080"
//! }
//! dress "Slot.frame[rarity=epic]" {
//!   glow 6 primary 0.5
//!   shape notch 6
//! }
//! ```
//!
//! A [`Selector`] is `Component.part` then any number of qualifiers: a
//! flag (`[primary]`, `[pressed]`: a variant or a state, which the
//! component's state groups tell apart) or an attribute (`[rarity=epic]`).
//! A dressing matches an element of that component's part whose flags and
//! attributes include all of the selector's; the most specific dressing
//! wins property by property ([`Selector::specificity`]), and two blocks
//! of the same selector merge, the later line winning.
//!
//! What a dressing sets is two kinds of data:
//!
//! - [`DressFrame`]: what version 1 could already say of a part, typed
//!   (the nine-slice and its tone, the shadow, glow and glass, corner
//!   ornaments, the crest and edge ornaments and the tile, the shape, and
//!   a button's press and hover).
//!   The twelve version 1 parts ([`crate::StylePart`]) are the dressings of
//!   [`crate::StylePart::selector`]: a style's `dress "Button.base[hover]"`
//!   is what version 1 called `slice buttonHover`, and those land in the
//!   [`crate::Art`] the widgets draw today.
//! - [`DressDraw`]: paints, strokes, effects, a material, ornaments, a
//!   blend, a mask and sounds ([`DRAW_KEYS`]) in the UI pass's types
//!   ([`crate::paint`]): `fill linear 90 "#ffd34a" "#e08a1e"`, `stroke 2
//!   lineStrong inside`, `effect innerShadow 0 2 4 "#00000066"` (the
//!   grammar is [`crate::paint_entry`]'s). `fill`, `stroke` and `effect`
//!   lines stack in order; a later block of the same selector that sets
//!   one of them replaces the earlier block's list. On a text part
//!   ([`TEXT_PARTS`]: `Field.text`, `Field.placeholder`), `ink <colour>`
//!   and `step <TypeStepName>` set the text's colour and type step.

use crate::paint::{Blend, Effect, Layer, Mask, Paint, Stroke};
use crate::paint_entry::AtlasContext;
use crate::style_file::{Glass, Glow, Hover, Press, Shadow, Slice};
use crate::tokens::{ElementClass, Level, ShapeToken, SoundEvent};
use og_core::ogt::Entry;
use std::fmt;

/// The dressing keys of the UI pass's types ([`DressDraw`]).
pub const DRAW_KEYS: [&str; 13] = [
    "fill",
    "stroke",
    "effect",
    "material",
    "ornament",
    "ornamentTint",
    "blend",
    "mask",
    "sound",
    "segments",
    "ink",
    "step",
    "appearance",
];

/// Element properties a dressing can set, using the document reflection schema.
pub const APPEARANCE_PROPS: &[&str] = &[
    "opacity", "motion", "color", "step", "padding", "gap", "width", "height", "x", "y",
];

/// `Component.part[flag][key=value]…`.
#[derive(Clone, Debug, Eq)]
pub struct Selector {
    /// PascalCase: `Button`, `ShopCard`.
    pub component: String,
    /// camelCase: `base`, `frame`, `fill`.
    pub part: String,
    /// Bare qualifiers as written: variants and states (`primary`, `pressed`).
    pub flags: Vec<String>,
    /// `[key=value]` qualifiers as written (`rarity`, `epic`).
    pub attrs: Vec<(String, String)>,
}

fn ident(s: &str, upper: bool) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| {
        if upper {
            f.is_ascii_uppercase()
        } else {
            f.is_ascii_lowercase()
        }
    }) && c.all(|c| c.is_ascii_alphanumeric())
}

fn word(s: &str) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| f.is_ascii_alphanumeric())
        && c.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

impl Selector {
    /// Reads a selector (`Button.base[primary][pressed]`, `Slot.frame[rarity=epic]`).
    pub fn parse(text: &str) -> Result<Selector, String> {
        let bad = |why: &str| {
            Err(format!(
                "'{text}' is not a part selector ({why}): Component.part, then [flag] or [key=value] qualifiers, such as Button.base[primary][pressed] or Slot.frame[rarity=epic]"
            ))
        };
        let (head, mut rest) = match text.find('[') {
            Some(i) => text.split_at(i),
            None => (text, ""),
        };
        let Some((component, part)) = head.split_once('.') else {
            return bad("no `.` between the component and its part");
        };
        if !ident(component, true) {
            return bad("the component is a PascalCase name");
        }
        if !ident(part, false) {
            return bad("the part is a camelCase name");
        }
        let mut s = Selector {
            component: component.into(),
            part: part.into(),
            flags: Vec::new(),
            attrs: Vec::new(),
        };
        while !rest.is_empty() {
            let Some(inner) = rest.strip_prefix('[') else {
                return bad("text after a qualifier");
            };
            let Some(end) = inner.find(']') else {
                return bad("an unclosed [");
            };
            let q = &inner[..end];
            rest = &inner[end + 1..];
            match q.split_once('=') {
                Some((k, v)) if word(k) && word(v) => {
                    if s.attrs.iter().any(|(a, _)| a == k) {
                        return bad(&format!("[{k}=…] twice"));
                    }
                    s.attrs.push((k.into(), v.into()));
                }
                None if word(q) => {
                    if s.flags.iter().any(|f| f == q) {
                        return bad(&format!("[{q}] twice"));
                    }
                    s.flags.push(q.into());
                }
                _ => return bad(&format!("[{q}] is not a word or key=value")),
            }
        }
        Ok(s)
    }

    /// `Component.part`, no qualifiers.
    pub fn of(component: &str, part: &str) -> Selector {
        Selector {
            component: component.into(),
            part: part.into(),
            flags: Vec::new(),
            attrs: Vec::new(),
        }
    }

    /// How many qualifiers it has: the more, the later it applies.
    pub fn specificity(&self) -> usize {
        self.flags.len() + self.attrs.len()
    }

    /// Whether it dresses the `component`'s `part` carrying `flags` and `attrs`.
    pub fn matches(
        &self,
        component: &str,
        part: &str,
        flags: &[&str],
        attrs: &[(&str, &str)],
    ) -> bool {
        self.component == component
            && self.part == part
            && self.flags.iter().all(|f| flags.contains(&f.as_str()))
            && self
                .attrs
                .iter()
                .all(|(k, v)| attrs.contains(&(k.as_str(), v.as_str())))
    }

    fn canonical(&self) -> (Vec<&str>, Vec<(&str, &str)>) {
        let mut f: Vec<&str> = self.flags.iter().map(String::as_str).collect();
        let mut a: Vec<(&str, &str)> = self
            .attrs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        f.sort_unstable();
        a.sort_unstable();
        (f, a)
    }
}

/// The order of qualifiers does not matter.
impl PartialEq for Selector {
    fn eq(&self, o: &Selector) -> bool {
        self.component == o.component && self.part == o.part && self.canonical() == o.canonical()
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.component, self.part)?;
        for q in &self.flags {
            write!(f, "[{q}]")?;
        }
        for (k, v) in &self.attrs {
            write!(f, "[{k}={v}]")?;
        }
        Ok(())
    }
}

/// What a dressing sets that version 1 could already say, typed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DressFrame {
    /// `slice <x> <y> <w> <h> <inset>… [tint|authored] [stretch|repeat|round]`.
    pub slice: Option<Slice>,
    /// `slice none`: no art, not even a less specific dressing's (a red
    /// Delete drawn by its fill under a pack's sliced buttons).
    pub unsliced: bool,
    /// `shadow <dx> <dy> <blur> <spread> "<colour>"`.
    pub shadow: Option<Shadow>,
    /// `glow <size> <ink> <alpha>`.
    pub glow: Option<Glow>,
    /// `glass <blur> <fill> [tint]`.
    pub glass: Option<Glass>,
    /// `corners`: the style's corner ornament at its corners.
    pub corners: bool,
    /// `crest`: the style's crest at its top centre.
    pub crest: bool,
    /// `edges`: the style's edge ornament at the middle of its left and
    /// right edges, the right one mirrored.
    pub edges: bool,
    /// `tile`: the style's tile over its fill.
    pub tile: bool,
    /// `shape <corner> <radius>`: its corner, over its element class's.
    pub shape: Option<ShapeToken>,
    /// `press <sx> <sy> [dy]`.
    pub press: Option<Press>,
    /// `hover <dy> [scale]`.
    pub hover: Option<Hover>,
}

/// A dressing's material: a built-in's layer, or a `type ui-material`
/// asset named by path (`asset`, bound to its handle when the style's
/// assets load; `layer.material` is plain until then), with the `params`
/// numbers it set (the rest are the material's defaults).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DressMaterial {
    pub layer: Layer,
    pub asset: Option<String>,
    pub params: usize,
}

/// The style's ornament art a dressing puts on its part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrnamentKind {
    Corner,
    Divider,
    Flourish,
}

/// What a dressing draws with in the UI pass ([`DRAW_KEYS`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DressDraw {
    /// `fill` lines, bottom first.
    pub fills: Vec<Paint>,
    pub strokes: Vec<Stroke>,
    pub effects: Vec<Effect>,
    pub material: Option<DressMaterial>,
    pub ornaments: Vec<OrnamentKind>,
    /// Overrides this part's ornament ink without changing the shared atlas.
    pub ornament_tint: Option<crate::paint::ColorRef>,
    pub blend: Option<Blend>,
    pub mask: Option<Mask>,
    /// Per UI event, an audio asset; a later line for one event wins.
    pub sounds: Vec<(SoundEvent, String)>,
    /// The `fill vector "<asset>"` lines: the index of their paint in
    /// `fills` and the asset, bound to a handle when the style's assets
    /// load ([`crate::StyleFile::bind`]); unbound, a vector fill draws nothing.
    pub vectors: Vec<(usize, String)>,
    /// `fill none`, `stroke none`, `effect none` ([`CLEAR_FILLS`]…): the
    /// part draws none of them, not the tokens' default.
    pub cleared: u8,
    /// `segments <count> <gap> [ramp <from> <to>]`, on a slider's or a
    /// meter's track: the track drawn as segments, filled whole ones.
    pub segments: Option<Segments>,
    /// `ink <colour>`, on a text part (`Field.text`, `Field.placeholder`):
    /// the text's colour.
    pub ink: Option<crate::paint::ColorRef>,
    /// `step <TypeStepName>`, on a text part: the text's type step, scaled
    /// with the player's text size as every step is.
    pub step: Option<&'static str>,
    /// Literal element properties, indexed by the document reflection schema.
    pub appearance: Vec<(usize, crate::doc::Lit)>,
}

/// The parts the `ink` and `step` dress keys apply to: the text a widget
/// draws as a part of its own (a field's value and placeholder).
pub const TEXT_PARTS: [&str; 2] = ["text", "placeholder"];

/// A track drawn as `count` segments `gap` CSS px apart, its fill a whole
/// number of segments (the nearest to the value); with `ramp`, filled
/// segment k in `from` mixed towards `to` by k / (count − 1) (cyber's
/// signal bars, a pixel meter's blocks).
#[derive(Clone, Debug, PartialEq)]
pub struct Segments {
    pub count: u32,
    pub gap: f32,
    pub ramp: Option<[ColorRef; 2]>,
}

impl Segments {
    /// The segments' rects across `track`, start to end.
    pub fn rects(&self, track: crate::Rect) -> Vec<crate::Rect> {
        let n = self.count.max(1) as f32;
        let gap = self.gap.min(track.width / n / 2.0).max(0.0);
        let w = ((track.width - gap * (n - 1.0)) / n).max(0.0);
        (0..self.count)
            .map(|k| crate::Rect::new(track.x + k as f32 * (w + gap), track.y, w, track.height))
            .collect()
    }

    /// How many segments a value 0–1 fills: the nearest whole number.
    pub fn filled(&self, value: f32) -> u32 {
        ((value.clamp(0.0, 1.0) * self.count as f32).round() as u32).min(self.count)
    }

    /// Filled segment `k`'s colour on its ramp, or `fill`.
    pub fn color(&self, k: u32, fill: Srgba, theme: &Theme) -> Srgba {
        let Some([a, b]) = &self.ramp else {
            return fill;
        };
        let of = |c: &ColorRef| match c {
            ColorRef::Token { name, alpha } => theme.color(name).map_or(fill, |c| c.fade(*alpha)),
            ColorRef::Literal(c) => *c,
        };
        let t = k as f32 / (self.count.max(2) - 1) as f32;
        of(a).mix(of(b), t)
    }
}

/// The handle an unbound vector fill carries until its style's assets load.
pub const UNBOUND_VECTOR: crate::paint::UiVectorHandle = crate::paint::UiVectorHandle(u32::MAX);

/// Which list keys a block has set (a block's first line of one replaces
/// the earlier blocks' list).
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawSeen {
    fills: bool,
    strokes: bool,
    effects: bool,
}

impl DressDraw {
    pub(crate) fn appearance_value(&self, name: &str) -> Option<&crate::doc::Lit> {
        self.appearance
            .iter()
            .find(|(i, _)| crate::doc::schema::PROPS[*i].name == name)
            .map(|(_, value)| value)
    }

    pub(crate) fn subpart_color(&self, fallback: Srgba, theme: &Theme) -> Srgba {
        use crate::doc::{Lit, Paint};
        match self.appearance_value("color") {
            Some(Lit::Paint(Paint::Color(c))) => Some(*c),
            Some(Lit::Paint(Paint::Token(name))) => theme.color(name),
            _ => self.ink.as_ref().and_then(|ink| ink.srgb(theme)),
        }
        .unwrap_or(fallback)
    }

    pub(crate) fn subpart_opacity(&self) -> f32 {
        match self.appearance_value("opacity") {
            Some(crate::doc::Lit::Num(opacity)) => opacity.clamp(0.0, 1.0),
            _ => 1.0,
        }
    }

    /// Resize a decorative subpart about its centre, then offset it.
    /// Percent and grow use the owning widget's box, not the previous size.
    pub(crate) fn subpart_rect(&self, rect: crate::Rect, parent: crate::Rect) -> crate::Rect {
        use crate::Size;
        use crate::doc::Lit;
        let extent = |name, current: f32, available: f32| match self.appearance_value(name) {
            Some(Lit::Size(size)) => match *size {
                Size::Fixed(n) => n.max(0.0),
                Size::Percent(p) => (available * p).max(0.0),
                Size::Fit { min, max } => current.clamp(min, max),
                Size::Grow { min, max } => available.clamp(min, max),
            },
            _ => current,
        };
        let offset = |name| match self.appearance_value(name) {
            Some(Lit::Num(n)) => *n,
            _ => 0.0,
        };
        let width = extent("width", rect.width, parent.width);
        let height = extent("height", rect.height, parent.height);
        crate::Rect::new(
            rect.x + (rect.width - width) / 2.0 + offset("x"),
            rect.y + (rect.height - height) / 2.0 + offset("y"),
            width,
            height,
        )
    }

    /// Reads one [`DRAW_KEYS`] line of a block into its field.
    pub fn apply(
        &mut self,
        e: &Entry,
        atlas: &AtlasContext,
        seen: &mut DrawSeen,
    ) -> Result<(), String> {
        fn push<T>(list: &mut Vec<T>, seen: &mut bool, v: T) {
            if !*seen {
                list.clear();
                *seen = true;
            }
            list.push(v);
        }
        let none = matches!(e.values.as_slice(), [og_core::ogt::Value::Word(w)] if w == "none");
        match (e.key.as_str(), none) {
            ("fill", true) => {
                self.fills.clear();
                self.vectors.clear();
                seen.fills = true;
                self.cleared |= CLEAR_FILLS;
                return Ok(());
            }
            ("stroke", true) => {
                self.strokes.clear();
                seen.strokes = true;
                self.cleared |= CLEAR_STROKES;
                return Ok(());
            }
            ("effect", true) => {
                self.effects.clear();
                seen.effects = true;
                self.cleared |= CLEAR_EFFECTS;
                return Ok(());
            }
            _ => {}
        }
        match e.key.as_str() {
            "fill" => {
                let vector = crate::paint_entry::vector_fill_from_entry(e, atlas)?;
                // The block's first fill replaces the list, its vectors too.
                if !seen.fills {
                    self.vectors.clear();
                }
                let at = if seen.fills { self.fills.len() } else { 0 };
                let paint = match vector {
                    Some((path, tint)) => {
                        self.vectors.push((at, path));
                        Paint::Vector {
                            vector: UNBOUND_VECTOR,
                            tint,
                        }
                    }
                    None => Paint::from_entry(e, atlas)?,
                };
                self.cleared &= !CLEAR_FILLS;
                push(&mut self.fills, &mut seen.fills, paint)
            }
            "stroke" => {
                self.cleared &= !CLEAR_STROKES;
                push(
                    &mut self.strokes,
                    &mut seen.strokes,
                    Stroke::from_entry(e, atlas)?,
                )
            }
            "effect" => {
                self.cleared &= !CLEAR_EFFECTS;
                push(
                    &mut self.effects,
                    &mut seen.effects,
                    Effect::from_entry(e, atlas)?,
                )
            }
            "material" => self.material = Some(DressMaterial::from_entry(e, atlas)?),
            "ornament" => self.ornaments = crate::paint_entry::ornaments_from_entry(e)?,
            "ornamentTint" => {
                self.ornament_tint = Some(crate::paint_entry::color_from_entry(e, atlas)?)
            }
            "blend" => self.blend = Some(crate::paint_entry::blend_from_entry(e)?),
            "mask" => self.mask = Some(Mask::from_entry(e, atlas)?),
            "segments" => self.segments = Some(crate::paint_entry::segments_from_entry(e, atlas)?),
            "ink" => self.ink = Some(crate::paint_entry::color_from_entry(e, atlas)?),
            "appearance" => {
                let p = crate::style_file::P { e };
                if !e.values.is_empty() {
                    return Err(p.err("takes a { } block of element properties"));
                }
                for entry in &p.block()?.entries {
                    let q = crate::style_file::P { e: entry };
                    if !APPEARANCE_PROPS.contains(&entry.key.as_str()) {
                        return Err(q.err(&format!("takes {}", APPEARANCE_PROPS.join(", "))));
                    }
                    let (index, spec) = crate::doc::schema::prop(&entry.key)
                        .expect("reflected appearance property");
                    let value =
                        crate::doc::parse_value(spec.ty, &entry.values).map_err(|e| q.err(&e))?;
                    let crate::doc::PropValue::Lit(value) = value else {
                        return Err(q.err("takes a literal, not a binding or template"));
                    };
                    self.appearance.retain(|(i, _)| *i != index);
                    self.appearance.push((index, value));
                }
            }
            "step" => {
                let name = match e.values.as_slice() {
                    [og_core::ogt::Value::Word(w)] => w.as_str(),
                    _ => {
                        return Err(format!(
                            "line {}: `step` takes one TypeStepName (body, caption, lg…)",
                            e.line
                        ));
                    }
                };
                let mut ramp = crate::theme::TYPE_RAMP;
                if crate::style_file::ramp_step(&mut ramp, name).is_none() {
                    return Err(format!(
                        "line {}: `step {name}`: not a TypeStepName (body, caption, lg…)",
                        e.line
                    ));
                }
                self.step = Some(intern(name));
            }
            "sound" => {
                let (s, path) = crate::paint_entry::sound_from_entry(e)?;
                self.sounds.retain(|(k, _)| *k != s);
                self.sounds.push((s, path));
            }
            k => return Err(format!("line {}: `{k}` is not a draw setting", e.line)),
        }
        Ok(())
    }
}

/// A style's dressing of the parts one selector matches.
#[derive(Clone, Debug, PartialEq)]
pub struct Dressing {
    pub selector: Selector,
    pub frame: DressFrame,
    /// Its paints, strokes, effects, material, ornaments, blend, mask and
    /// sounds (see the module docs).
    pub draw: DressDraw,
    /// `recolour "<from>" <role>` lines (or `recolour none`) for its
    /// authored slice, over the style's (`StyleFile::recolour`).
    pub recolour: Option<Vec<crate::style_file::Recolour>>,
    /// The asset id of the style it was read from (`StyleFile::source`;
    /// empty: a style made in memory, the default style's).
    pub source: String,
    /// The lines of its blocks in that file (two blocks of one selector
    /// merge), for `pick` and `dump_ui` (docs/ui.md §6).
    pub lines: Vec<usize>,
}

impl Dressing {
    /// Where it is written: `styles/fantasy/style.ogt:210` (every block's
    /// line, comma separated), or its selector's text alone when it was
    /// not read from a file.
    pub fn written_at(&self) -> String {
        let lines: Vec<String> = self.lines.iter().map(usize::to_string).collect();
        match (self.source.is_empty(), lines.is_empty()) {
            (false, false) => format!("{}:{}", self.source, lines.join(",")),
            (false, true) => self.source.clone(),
            _ => String::new(),
        }
    }
}

// ───────────────────────────────────────────── parts and their resolution

/// Qualifiers a [`PartRef`] carries (variants and states).
pub const MAX_FLAGS: usize = 6;
/// Attributes a [`PartRef`] carries (`rarity=epic`, `tone=success`).
pub const MAX_ATTRS: usize = 3;

/// A name made `'static` once, so a part a game names at run time
/// (`og.ui` element descs, documents) is as cheap to carry as a widget's
/// own: the set of component, part, flag and attribute names is small.
pub fn intern(s: &str) -> &'static str {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static NAMES: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut set = NAMES
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(n) = set.get(s) {
        return n;
    }
    let n: &'static str = Box::leak(s.to_string().into_boxed_str());
    set.insert(n);
    n
}

/// What a widget's element is to its style (docs/ui.md §3.2): the
/// component and part a dressing's selector names, the variant and state
/// flags and the attributes it matches against, the element class whose
/// shape language and the elevation level it draws with when no dressing
/// says otherwise, and how far into hover and press it is (0–1, moving on
/// the style's `ui` spring, so a state change is a transition, not a jump).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartRef {
    pub component: &'static str,
    pub part: &'static str,
    /// The element class whose tokens an undressed part takes (its corner,
    /// outline and elevation); `None`: a game's decoration, which takes
    /// nothing from the tokens, only what a style dresses it with.
    pub class: Option<ElementClass>,
    pub level: Option<Level>,
    /// Empty entries are unused.
    pub flags: [&'static str; MAX_FLAGS],
    pub attrs: [(&'static str, &'static str); MAX_ATTRS],
    pub hover: f32,
    pub press: f32,
}

impl Default for PartRef {
    fn default() -> PartRef {
        PartRef::new("Element", "base", ElementClass::Panel)
    }
}

impl PartRef {
    pub const fn new(component: &'static str, part: &'static str, class: ElementClass) -> PartRef {
        PartRef::of(component, part, Some(class))
    }

    /// A part of `class`, or of none (see [`PartRef::class`]).
    pub const fn of(
        component: &'static str,
        part: &'static str,
        class: Option<ElementClass>,
    ) -> PartRef {
        PartRef {
            component,
            part,
            class,
            level: None,
            flags: [""; MAX_FLAGS],
            attrs: [("", ""); MAX_ATTRS],
            hover: 0.0,
            press: 0.0,
        }
    }

    /// With a flag (a variant or a state); an empty one, a repeat or one
    /// past [`MAX_FLAGS`] changes nothing.
    pub fn flag(mut self, f: &'static str) -> PartRef {
        if f.is_empty() || self.has(f) {
            return self;
        }
        if let Some(slot) = self.flags.iter_mut().find(|s| s.is_empty()) {
            *slot = f;
        }
        self
    }

    /// With `flag` when `on`.
    pub fn flag_if(self, on: bool, f: &'static str) -> PartRef {
        if on { self.flag(f) } else { self }
    }

    /// With an attribute (a later one of the same key replaces it).
    pub fn attr(mut self, k: &'static str, v: &'static str) -> PartRef {
        if k.is_empty() || v.is_empty() {
            return self;
        }
        if let Some(slot) = self.attrs.iter_mut().find(|(a, _)| *a == k) {
            slot.1 = v;
        } else if let Some(slot) = self.attrs.iter_mut().find(|(a, _)| a.is_empty()) {
            *slot = (k, v);
        }
        self
    }

    pub fn at_level(mut self, l: Level) -> PartRef {
        self.level = Some(l);
        self
    }

    pub fn has(&self, f: &str) -> bool {
        self.flags.contains(&f) && !f.is_empty()
    }

    pub fn attr_value(&self, k: &str) -> Option<&'static str> {
        self.attrs.iter().find(|(a, _)| *a == k).map(|(_, v)| *v)
    }

    pub fn flags(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.flags.iter().copied().filter(|f| !f.is_empty())
    }

    /// The same part in another interaction state: `hover` and `pressed`
    /// as given (the rest of its flags kept).
    pub fn in_state(&self, hover: bool, pressed: bool) -> PartRef {
        let mut p = *self;
        for f in p.flags.iter_mut() {
            if *f == "hover" || *f == "pressed" {
                *f = "";
            }
        }
        p.flag_if(hover, "hover").flag_if(pressed, "pressed")
    }

    /// A part from its selector text (`ShopCard.frame[rarity=epic][selected]`),
    /// its names interned: how game code and documents name their own parts.
    pub fn parse(text: &str, class: impl Into<Option<ElementClass>>) -> Result<PartRef, String> {
        let s = Selector::parse(text)?;
        let mut p = PartRef::of(intern(&s.component), intern(&s.part), class.into());
        for f in &s.flags {
            p = p.flag(intern(f));
        }
        for (k, v) in &s.attrs {
            p = p.attr(intern(k), intern(v));
        }
        Ok(p)
    }

    /// Whether `sel` dresses it.
    pub fn matched_by(&self, sel: &Selector) -> bool {
        sel.component == self.component
            && sel.part == self.part
            && sel.flags.iter().all(|f| self.has(f))
            && sel
                .attrs
                .iter()
                .all(|(k, v)| self.attr_value(k) == Some(v.as_str()))
    }
}

impl fmt::Display for PartRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.component, self.part)?;
        for q in self.flags() {
            write!(f, "[{q}]")?;
        }
        for (k, v) in self.attrs.iter().filter(|(k, _)| !k.is_empty()) {
            write!(f, "[{k}={v}]")?;
        }
        Ok(())
    }
}

/// A style's dressings ready to match: in the order they apply (least
/// specific first; of equal specificity, file order), so the most
/// specific wins property by property.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DressSet {
    list: Vec<Dressing>,
}

/// What every dressing matching a part sets, merged.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resolved {
    pub frame: DressFrame,
    pub draw: DressDraw,
    /// How many dressings matched (0: the part draws from the tokens).
    pub matched: u32,
}

impl DressSet {
    pub fn new(mut list: Vec<Dressing>) -> DressSet {
        // A stable sort keeps file order among equals.
        list.sort_by_key(|d| d.selector.specificity());
        DressSet { list }
    }

    pub fn dressings(&self) -> &[Dressing] {
        &self.list
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Whether any dressing names `component.part`.
    pub fn dresses(&self, component: &str, part: &str) -> bool {
        self.list
            .iter()
            .any(|d| d.selector.component == component && d.selector.part == part)
    }

    /// Every dressing of `part` merged, the more specific over the less.
    /// Whether some dressing of `part`'s component and part is qualified by
    /// the attribute `key` (any value): the style tells that attribute's
    /// values apart itself (a gilded slot recolours its frame per tier).
    pub fn dresses_key(&self, part: &PartRef, key: &str) -> bool {
        self.list.iter().any(|d| {
            d.selector.component == part.component
                && d.selector.part == part.part
                && d.selector.attrs.iter().any(|(k, _)| k == key)
        })
    }

    /// The dressings that match `part`, in the order they apply: the last
    /// wins a property they both set.
    pub fn matching<'a>(&'a self, part: &'a PartRef) -> impl Iterator<Item = &'a Dressing> + 'a {
        self.list
            .iter()
            .filter(move |d| part.matched_by(&d.selector))
    }

    pub fn resolve(&self, part: &PartRef) -> Resolved {
        let mut r = Resolved::default();
        for d in self.list.iter().filter(|d| part.matched_by(&d.selector)) {
            r.matched += 1;
            let (f, g) = (&d.frame, &mut r.frame);
            g.slice = if f.unsliced {
                None
            } else {
                f.slice.or(g.slice)
            };
            g.shadow = f.shadow.or(g.shadow);
            g.glow = f.glow.or(g.glow);
            g.glass = f.glass.or(g.glass);
            g.corners |= f.corners;
            g.crest |= f.crest;
            g.edges |= f.edges;
            g.tile |= f.tile;
            g.shape = f.shape.or(g.shape);
            g.press = f.press.or(g.press);
            g.hover = f.hover.or(g.hover);
            let (a, b) = (&d.draw, &mut r.draw);
            if !a.fills.is_empty() || a.cleared & CLEAR_FILLS != 0 {
                b.fills = a.fills.clone();
                b.vectors = a.vectors.clone();
                b.cleared = (b.cleared & !CLEAR_FILLS) | (a.cleared & CLEAR_FILLS);
            }
            if !a.strokes.is_empty() || a.cleared & CLEAR_STROKES != 0 {
                b.strokes = a.strokes.clone();
                b.cleared = (b.cleared & !CLEAR_STROKES) | (a.cleared & CLEAR_STROKES);
            }
            if !a.effects.is_empty() || a.cleared & CLEAR_EFFECTS != 0 {
                b.effects = a.effects.clone();
                b.cleared = (b.cleared & !CLEAR_EFFECTS) | (a.cleared & CLEAR_EFFECTS);
            }
            if a.material.is_some() {
                b.material = a.material.clone();
            }
            if !a.ornaments.is_empty() {
                b.ornaments = a.ornaments.clone();
            }
            if a.ornament_tint.is_some() {
                b.ornament_tint = a.ornament_tint.clone();
            }
            b.blend = a.blend.or(b.blend);
            if a.mask.is_some() {
                b.mask = a.mask.clone();
            }
            if a.segments.is_some() {
                b.segments = a.segments.clone();
            }
            if a.ink.is_some() {
                b.ink = a.ink.clone();
            }
            b.step = a.step.or(b.step);
            for (index, value) in &a.appearance {
                b.appearance.retain(|(i, _)| i != index);
                b.appearance.push((*index, value.clone()));
            }
            for (e, s) in &a.sounds {
                b.sounds.retain(|(k, _)| k != e);
                b.sounds.push((*e, s.clone()));
            }
        }
        r
    }
}

/// [`DressDraw::cleared`]: `fill none`.
pub const CLEAR_FILLS: u8 = 1;
/// `stroke none`.
pub const CLEAR_STROKES: u8 = 2;
/// `effect none`.
pub const CLEAR_EFFECTS: u8 = 4;

// ──────────────────────────────────────────── a part drawn in the UI pass

use crate::paint::{ColorRef, Corner, ElementDraw, ShapeKind, StrokeAlign};
use crate::style::{Srgba, Theme};
use crate::tokens::{CornerShape, StrokeWidth};

/// The UI pass's corner for a shape token, and its radius.
pub fn corner_of(s: ShapeToken) -> (Corner, f32) {
    match s.corner {
        CornerShape::Round => (Corner::Round, s.radius),
        CornerShape::Squircle => (Corner::Squircle(4.0), s.radius),
        CornerShape::Notch => (Corner::Notch, s.radius),
        CornerShape::Bevel => (Corner::Bevel, s.radius),
        CornerShape::Square => (Corner::Round, 0.0),
    }
}

fn tok(name: &str, alpha: f32) -> ColorRef {
    ColorRef::Token {
        name: name.to_string(),
        alpha,
    }
}

fn stroke(width: f32, color: ColorRef) -> Stroke {
    Stroke {
        width,
        align: StrokeAlign::Inside,
        dash: [0.0; 2],
        dash_offset: 0.0,
        dash_cap: crate::paint::Cap::Butt,
        paint: Paint::Solid(color),
    }
}

fn inner(dy: f32, blur: f32, color: ColorRef) -> Effect {
    Effect::InnerShadow {
        offset: [0.0, dy],
        blur,
        spread: 0.0,
        color,
    }
}

/// The level a part's shadows come from when it names none.
fn default_level(part: &PartRef) -> Option<Level> {
    if part.level.is_some() {
        return part.level;
    }
    match part.class? {
        ElementClass::Dialog => Some(Level::Overlay),
        ElementClass::Tooltip => Some(Level::Floating),
        ElementClass::Card => Some(Level::Raised),
        _ => None,
    }
}

/// Whether a class is a surface (it bounds content), not a control.
fn surface(class: ElementClass) -> bool {
    use ElementClass as C;
    matches!(class, C::Panel | C::Dialog | C::Card | C::Tooltip | C::Hud)
}

/// What an undressed part's outline is drawn with: the style's stroke
/// widths in its line colours, by class and state (docs/ui.md §3.2: a
/// part no style dressed is drawn from the tokens).
pub fn token_strokes(part: &PartRef, theme: &Theme) -> Vec<Stroke> {
    let t = &theme.tokens;
    let hair = t.stroke(StrokeWidth::Hairline);
    let thin = t.stroke(StrokeWidth::Thin);
    use ElementClass as C;
    let Some(class) = part.class else {
        return Vec::new();
    };
    match class {
        C::Panel | C::Dialog | C::Card | C::Tooltip => vec![stroke(hair, tok("line", 1.0))],
        C::Hud => vec![stroke(hair, tok("line", 0.7))],
        C::Button if part.has("disabled") || part.has("quiet") => Vec::new(),
        C::Button if part.has("danger") => vec![stroke(hair, tok("danger", 0.45))],
        C::Button if part.has("primary") => Vec::new(),
        C::Button if part.has("ghost") => vec![stroke(thin, tok("lineStrong", 1.0))],
        C::Button => vec![stroke(hair, tok("line", 1.0))],
        C::Field if mark_box(part) && part.has("invalid") => vec![stroke(thin, tok("danger", 1.0))],
        // An unchecked box or radio is a well that must be found at 3:1;
        // a filled one is its fill alone.
        C::Field | C::Avatar if well_mark(part) => {
            vec![stroke(hair, ColorRef::Literal(well_rim(theme)))]
        }
        C::Field if mark_box(part) => Vec::new(),
        C::Field if part.has("invalid") => vec![stroke(thin, tok("danger", 1.0))],
        C::Field => vec![stroke(thin, tok("line", 1.0))],
        // One selection grammar (docs/ui.md §6.3): a pack's selected chip,
        // slot or tab is an accent hairline on a ground one step up, its
        // words in the accent ink ([`pack_grammar`]).
        C::Slot | C::Chip | C::Tab if selected_grammar(part, theme) => {
            vec![stroke(hair, tok("primary", 1.0))]
        }
        C::Slot if part.has("selected") => vec![stroke(thin, tok("primary", 1.0))],
        C::Slot => vec![stroke(hair, tok("line", 1.0))],
        C::Chip if part.has("selected") => Vec::new(),
        C::Chip => vec![stroke(hair, tok("line", 1.0))],
        C::Avatar => vec![stroke(thin, tok("lineStrong", 1.0))],
        C::Badge | C::Tab | C::Bar => Vec::new(),
    }
}

/// Whether a selected chip, slot or tab takes the one selection grammar.
/// A tab bar's underlined tab is marked by its indicator alone.
fn selected_grammar(part: &PartRef, theme: &Theme) -> bool {
    part.has("selected") && part.part != "track" && !part.has("underline") && pack_grammar(theme)
}

/// Whether a theme's selected chips, slots and tabs follow the style
/// packs' one selection grammar (an accent hairline, a ground one step
/// up, the accent ink): every style's but the default's, whose grammar is
/// the design language's, all neutral (a selected chip inverts), and a
/// version 1 style's, which keeps the grammar its game shipped with.
pub fn pack_grammar(theme: &Theme) -> bool {
    !theme.art.legacy && theme.art != crate::style_file::Art::default()
}

/// The accent's ink on a selected tile's `ground`: `primary`, moved as
/// little as it takes to read 4.5:1 there, else the ink that does.
pub fn accent_ink(theme: &Theme, ground: Srgba) -> Srgba {
    let p = &theme.palette;
    p.primary
        .fade(1.0)
        .readable_on(ground, 4.5)
        .or_else(|| p.ink.readable_on(ground, 4.5))
        .unwrap_or(p.ink_inverse)
}

/// Whether a part is a checkbox's box or a radio's ring.
fn mark_box(part: &PartRef) -> bool {
    matches!(
        (part.component, part.part),
        ("Checkbox", "box") | ("Radio", "box" | "ring")
    )
}

/// Whether a part is a checkbox's box or a radio's ring, empty.
fn well_mark(part: &PartRef) -> bool {
    mark_box(part)
        && !["on", "checked", "mixed", "selected"]
            .iter()
            .any(|f| part.has(f))
}

/// The rim of an unchecked box or radio: `lineStrong`, moved as little as
/// it takes to read 3:1 on both the well (`panelRaised`) and the panel it
/// sits on (non-text contrast, WCAG 1.4.11).
pub fn well_rim(theme: &Theme) -> Srgba {
    let p = &theme.palette;
    let mut c = p.line_strong.fade(1.0);
    for ground in [p.panel_raised, p.panel] {
        let ground = ground.fade(1.0);
        if c.contrast(ground) < 3.0 {
            c = c.readable_on(ground, 3.0).unwrap_or(p.ink.fade(1.0));
        }
    }
    c
}

/// An undressed part's effects: its level's shadows (elevation), a lit
/// top edge on raised controls and a recess on wells, by class and state.
pub fn token_effects(part: &PartRef, theme: &Theme) -> Vec<Effect> {
    let mut out = Vec::new();
    let lifted = !part.has("pressed") && !part.has("disabled");
    if let Some(l) = default_level(part).filter(|_| lifted) {
        for s in theme.tokens.elevation(l).shadows() {
            out.push(Effect::DropShadow {
                offset: [s.dx, s.dy],
                blur: s.blur,
                spread: s.spread,
                color: ColorRef::Literal(s.color),
            });
        }
    }
    use ElementClass as C;
    let Some(class) = part.class else {
        return out;
    };
    match class {
        C::Button if part.has("disabled") || part.has("quiet") || part.has("ghost") => {}
        C::Button if part.has("pressed") => out.push(inner(1.0, 3.0, tok("shadow", 0.45))),
        C::Button => out.push(inner(1.0, 0.0, tok("sheen", 0.6))),
        // A selected tile is lifted a step, not recessed.
        C::Slot | C::Chip | C::Tab if selected_grammar(part, theme) => {}
        C::Field if mark_box(part) && !well_mark(part) => {}
        C::Field | C::Slot => out.push(inner(1.0, 3.0, tok("shadow", 0.3))),
        C::Bar if part.part == "fill" => out.push(inner(1.0, 0.0, tok("sheen", 0.5))),
        C::Bar => out.push(inner(1.0, 2.0, tok("shadow", 0.35))),
        C::Tab if part.has("selected") => out.push(Effect::DropShadow {
            offset: [0.0, 1.0],
            blur: 2.0,
            spread: 0.0,
            color: tok("shadow", 0.35),
        }),
        _ => {}
    }
    out
}

impl Resolved {
    /// The corner the part draws with: its dressing's `shape`, else its
    /// element class's in the style's shape language (a classless part's
    /// is square: its element's own radius applies, [`Resolved::element_draw`]).
    pub fn shape(&self, part: &PartRef, theme: &Theme) -> ShapeToken {
        self.frame.shape.unwrap_or_else(|| {
            // A part round by what it is (a radio's mark, a knob, a switch's
            // track) stays round in every shape language unless dressed.
            if part.has("round")
                && theme.art.v1_controls
                && matches!(part.component, "Slider" | "Toggle")
            {
                // A version 1 style's knobs and switches: the control radius.
                let inset = if (part.component, part.part) == ("Toggle", "knob") {
                    theme.space.xxs
                } else {
                    0.0
                };
                ShapeToken {
                    corner: crate::tokens::CornerShape::Round,
                    radius: (theme.radius.control - inset).max(0.0),
                }
            } else if part.has("round") {
                ShapeToken {
                    corner: crate::tokens::CornerShape::Round,
                    radius: 9999.0,
                }
            } else {
                part.class.map_or(
                    ShapeToken {
                        corner: crate::tokens::CornerShape::Square,
                        radius: 0.0,
                    },
                    |c| theme.tokens.shape(c),
                )
            }
        })
    }

    /// What the part draws in the UI pass (docs/ui.md §3.2): the
    /// dressing's paints, strokes, effects, blend, mask and material over
    /// the corner of its shape; each of fills, strokes and effects a style
    /// did not dress comes from the tokens ([`token_strokes`],
    /// [`token_effects`]; the fill is the widget's own colour `fill` for
    /// its state, none for a quiet control at rest).
    pub fn element_draw(&self, part: &PartRef, theme: &Theme, fill: Option<Srgba>) -> ElementDraw {
        let (corner, radius) = corner_of(self.shape(part, theme));
        let d = &self.draw;
        let fills = if !d.fills.is_empty() || d.cleared & CLEAR_FILLS != 0 {
            d.fills.clone()
        } else {
            fill.map(|c| vec![Paint::Solid(ColorRef::Literal(c))])
                .unwrap_or_default()
        };
        let strokes = if !d.strokes.is_empty() || d.cleared & CLEAR_STROKES != 0 {
            d.strokes.clone()
        } else if fills.is_empty() && part.class.is_some_and(surface) {
            // A surface with no fill is a layout group a pattern named
            // (so a style can dress it): undressed, it draws no outline.
            Vec::new()
        } else {
            token_strokes(part, theme)
        };
        let effects = if !d.effects.is_empty() || d.cleared & CLEAR_EFFECTS != 0 {
            d.effects.clone()
        } else {
            token_effects(part, theme)
        };
        // A transparent part (a quiet button at rest) keeps no shadow.
        let effects = if fills.is_empty() && d.fills.is_empty() {
            effects
                .into_iter()
                .filter(|e| !matches!(e, Effect::DropShadow { .. }))
                .collect()
        } else {
            effects
        };
        let mut layer = d.material.as_ref().map(|m| m.layer.clone());
        if layer.is_none() && d.mask.is_some() {
            layer = Some(Layer::default());
        }
        // A classless part the style gave no shape keeps its element's radius.
        let classless = part.class.is_none() && self.frame.shape.is_none() && !part.has("round");
        ElementDraw {
            shape: ShapeKind::Rect {
                radii: (!classless).then_some([radius; 4]),
                corner,
            },
            fills,
            strokes,
            effects,
            opacity: 1.0,
            blend: d.blend.unwrap_or_default(),
            clip: match &d.mask {
                Some(m) => crate::paint::Clip::Mask(m.clone()),
                None => crate::paint::Clip::None,
            },
            layer,
            text: None,
            rive: None,
            outlines: Vec::new(),
            view_box: None,
            view_rect: None,
        }
    }
}

/// The colour a draw shows under its text: its fills' mean over `below`
/// (a gradient's stops averaged), in linear light as the renderer blends.
pub fn ground_of(d: &ElementDraw, theme: &Theme, below: Srgba) -> Srgba {
    let mut c = below;
    for f in &d.fills {
        let top = match f {
            Paint::Solid(r) => Srgba::from_linear(r.resolve(theme)),
            Paint::Gradient(g) if !g.stops.is_empty() => {
                let n = g.stops.len() as f32;
                let mut sum = [0.0f32; 4];
                for s in &g.stops {
                    let l = s.color.resolve(theme);
                    for k in 0..4 {
                        sum[k] += l[k] / n;
                    }
                }
                Srgba::from_linear(sum)
            }
            _ => continue,
        };
        c = top.over_linear(c);
    }
    c
}

fn mix_lin(a: [f32; 4], b: [f32; 4], t: f32) -> ColorRef {
    let m = |k: usize| a[k] + (b[k] - a[k]) * t;
    ColorRef::Literal(Srgba::from_linear([m(0), m(1), m(2), m(3)]))
}

fn mix_paint(theme: &Theme, a: &Paint, b: &Paint, t: f32) -> Option<Paint> {
    Some(match (a, b) {
        (Paint::Solid(x), Paint::Solid(y)) => {
            Paint::Solid(mix_lin(x.resolve(theme), y.resolve(theme), t))
        }
        (Paint::Gradient(x), Paint::Gradient(y))
            if x.kind == y.kind && x.stops.len() == y.stops.len() =>
        {
            let mut g = x.clone();
            let l = |p: [f32; 2], q: [f32; 2]| [p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t];
            g.from = l(x.from, y.from);
            g.to = l(x.to, y.to);
            for (s, (p, q)) in g.stops.iter_mut().zip(x.stops.iter().zip(&y.stops)) {
                s.offset = p.offset + (q.offset - p.offset) * t;
                s.color = mix_lin(p.color.resolve(theme), q.color.resolve(theme), t);
            }
            Paint::Gradient(g)
        }
        _ if a == b => a.clone(),
        _ => return None,
    })
}

fn mix_effect(theme: &Theme, a: &Effect, b: &Effect, t: f32) -> Option<Effect> {
    let l = |x: f32, y: f32| x + (y - x) * t;
    let c = |x: &ColorRef, y: &ColorRef| mix_lin(x.resolve(theme), y.resolve(theme), t);
    Some(match (a, b) {
        (
            Effect::DropShadow {
                offset: o1,
                blur: b1,
                spread: s1,
                color: c1,
            },
            Effect::DropShadow {
                offset: o2,
                blur: b2,
                spread: s2,
                color: c2,
            },
        ) => Effect::DropShadow {
            offset: [l(o1[0], o2[0]), l(o1[1], o2[1])],
            blur: l(*b1, *b2),
            spread: l(*s1, *s2),
            color: c(c1, c2),
        },
        (
            Effect::InnerShadow {
                offset: o1,
                blur: b1,
                spread: s1,
                color: c1,
            },
            Effect::InnerShadow {
                offset: o2,
                blur: b2,
                spread: s2,
                color: c2,
            },
        ) => Effect::InnerShadow {
            offset: [l(o1[0], o2[0]), l(o1[1], o2[1])],
            blur: l(*b1, *b2),
            spread: l(*s1, *s2),
            color: c(c1, c2),
        },
        (
            Effect::OuterGlow {
                blur: b1,
                spread: s1,
                color: c1,
            },
            Effect::OuterGlow {
                blur: b2,
                spread: s2,
                color: c2,
            },
        ) => Effect::OuterGlow {
            blur: l(*b1, *b2),
            spread: l(*s1, *s2),
            color: c(c1, c2),
        },
        (
            Effect::InnerGlow {
                blur: b1,
                spread: s1,
                color: c1,
            },
            Effect::InnerGlow {
                blur: b2,
                spread: s2,
                color: c2,
            },
        ) => Effect::InnerGlow {
            blur: l(*b1, *b2),
            spread: l(*s1, *s2),
            color: c(c1, c2),
        },
        _ if a == b => a.clone(),
        _ => return None,
    })
}

/// `a` moved `t` of the way to `b` (0–1): colours mixed in linear light,
/// lengths, offsets and corner radii in between, where the two have the
/// same paints, strokes and effects in the same order; otherwise whichever
/// of the two `t` is nearer (a state whose dressing draws something else
/// switches at the transition's middle).
pub fn mix_draw(theme: &Theme, a: &ElementDraw, b: &ElementDraw, t: f32) -> ElementDraw {
    if t <= 0.0 {
        return a.clone();
    }
    if t >= 1.0 {
        return b.clone();
    }
    let nearer = || if t < 0.5 { a.clone() } else { b.clone() };
    let shape = match (&a.shape, &b.shape) {
        (
            ShapeKind::Rect {
                radii: Some(r1),
                corner: k1,
            },
            ShapeKind::Rect {
                radii: Some(r2),
                corner: k2,
            },
        ) if k1 == k2 => ShapeKind::Rect {
            radii: Some(std::array::from_fn(|k| r1[k] + (r2[k] - r1[k]) * t)),
            corner: *k1,
        },
        (x, y) if x == y => x.clone(),
        _ => return nearer(),
    };
    if a.fills.len() != b.fills.len()
        || a.strokes.len() != b.strokes.len()
        || a.effects.len() != b.effects.len()
        || a.clip != b.clip
        || a.layer != b.layer
        || a.blend != b.blend
    {
        // A fill appearing (a quiet button hovered) fades in instead.
        if a.fills.is_empty() && b.fills.len() == 1 && a.strokes == b.strokes {
            let mut out = b.clone();
            out.shape = shape;
            out.opacity = b.opacity * t;
            return out;
        }
        return nearer();
    }
    let mut out = a.clone();
    out.shape = shape;
    for (k, (p, q)) in a.fills.iter().zip(&b.fills).enumerate() {
        match mix_paint(theme, p, q, t) {
            Some(m) => out.fills[k] = m,
            None => return nearer(),
        }
    }
    for (k, (p, q)) in a.strokes.iter().zip(&b.strokes).enumerate() {
        let Some(paint) = mix_paint(theme, &p.paint, &q.paint, t) else {
            return nearer();
        };
        out.strokes[k].paint = paint;
        out.strokes[k].width = p.width + (q.width - p.width) * t;
    }
    for (k, (p, q)) in a.effects.iter().zip(&b.effects).enumerate() {
        match mix_effect(theme, p, q, t) {
            Some(m) => out.effects[k] = m,
            None => return nearer(),
        }
    }
    out.opacity = a.opacity + (b.opacity - a.opacity) * t;
    out
}
