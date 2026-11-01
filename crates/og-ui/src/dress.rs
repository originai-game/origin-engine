//! Part dressings (docs/ui.md §12.3): a component names its parts
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
//!   ornaments and the tile, the shape, and a button's press and hover).
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
//!   one of them replaces the earlier block's list.

use crate::paint::{Blend, Effect, Layer, Mask, Paint, Stroke};
use crate::paint_entry::AtlasContext;
use crate::style_file::{Glass, Glow, Hover, Press, Shadow, Slice, UiSound};
use crate::tokens::ShapeToken;
use og_core::ogt::Entry;
use std::fmt;

/// The dressing keys of the UI pass's types ([`DressDraw`]).
pub const DRAW_KEYS: [&str; 8] = [
    "fill", "stroke", "effect", "material", "ornament", "blend", "mask", "sound",
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
    /// `shadow <dx> <dy> <blur> <spread> "<colour>"`.
    pub shadow: Option<Shadow>,
    /// `glow <size> <ink> <alpha>`.
    pub glow: Option<Glow>,
    /// `glass <blur> <fill> [tint]`.
    pub glass: Option<Glass>,
    /// `corners`: the style's corner ornament at its corners.
    pub corners: bool,
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
    pub blend: Option<Blend>,
    pub mask: Option<Mask>,
    /// Per UI event, an audio asset; a later line for one event wins.
    pub sounds: Vec<(UiSound, String)>,
}

/// Which list keys a block has set (a block's first line of one replaces
/// the earlier blocks' list).
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawSeen {
    fills: bool,
    strokes: bool,
    effects: bool,
}

impl DressDraw {
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
        match e.key.as_str() {
            "fill" => push(
                &mut self.fills,
                &mut seen.fills,
                Paint::from_entry(e, atlas)?,
            ),
            "stroke" => push(
                &mut self.strokes,
                &mut seen.strokes,
                Stroke::from_entry(e, atlas)?,
            ),
            "effect" => push(&mut self.effects, &mut seen.effects, Effect::from_entry(e)?),
            "material" => self.material = Some(DressMaterial::from_entry(e)?),
            "ornament" => self.ornaments = crate::paint_entry::ornaments_from_entry(e)?,
            "blend" => self.blend = Some(crate::paint_entry::blend_from_entry(e)?),
            "mask" => self.mask = Some(Mask::from_entry(e, atlas)?),
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
}
