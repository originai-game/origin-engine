//! A style pack: one `pack.json` (CONVENTIONS.md §Theme and packs). A
//! pack is a Tailwind theme, the class strings its components' slots
//! append, and the fonts, icons and sounds it ships. Packs lay over one
//! another as layers: the engine's default pack (generated from the design
//! tokens, [`design`]), then the pack a look names, then the game's own.
//!
//! Any leaf of `theme` or `seeds` may be `{ "light": …, "dark": … }`; a
//! layer is split per scheme as it is read, so each scheme's theme is
//! plain Tailwind theme JSON ([`Pack::theme_json`]) and switching scheme is
//! switching tables, never a cascade.

pub mod design;

/// The default UI face, bundled for games with UI documents and drawn
/// first wherever no game font or style names a role.
pub const DEFAULT_FONT_ASSET: &str = "@styles/shared/InstrumentSans[wdth,wght].ttf";

use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// A colour scheme: each has its own theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scheme {
    Light,
    Dark,
}

impl Scheme {
    pub const ALL: [Scheme; 2] = [Scheme::Light, Scheme::Dark];

    pub fn name(self) -> &'static str {
        match self {
            Scheme::Light => "light",
            Scheme::Dark => "dark",
        }
    }

    fn of(name: &str) -> Option<Scheme> {
        Scheme::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// The theme sections a pack may set: NSH's `Theme::from_json` keys.
pub const THEME_KEYS: [&str; 23] = [
    "colors",
    "spacing",
    "rem",
    "breakpoints",
    "container",
    "fontSize",
    "fontWeight",
    "fontFamily",
    "tracking",
    "leading",
    "radius",
    "shadow",
    "insetShadow",
    "dropShadow",
    "textShadow",
    "blur",
    "perspective",
    "aspect",
    "ease",
    "duration",
    "opacity",
    "stroke",
    "scale",
];

/// The seeds the design palette is derived from (`packages/design/tokens/seed.css`).
pub const SEEDS: [&str; 9] = [
    "primary", "accent", "neutral", "danger", "warn", "success", "info", "team", "hud",
];

/// The interaction events a pack gives sounds for.
pub const SOUND_EVENTS: [&str; 20] = [
    "press",
    "hover",
    "select",
    "confirm",
    "cancel",
    "back",
    "open",
    "close",
    "error",
    "reward",
    "toggle",
    "tab",
    "levelup",
    "purchase",
    "countdown",
    "tick",
    "step",
    "swipe",
    "notify",
    "unlock",
];

/// The event an event's sound falls back to when no scope has a clip for
/// it: the nearest one a pack without its own clips already has.
pub fn sound_fallback(event: &str) -> Option<&'static str> {
    Some(match event {
        "confirm" | "back" | "countdown" => "press",
        "toggle" | "tab" => "select",
        "purchase" => "confirm",
        "levelup" | "unlock" => "reward",
        "tick" | "step" | "swipe" => "select",
        "notify" => "open",
        _ => return None,
    })
}

/// A sound an event plays: one of `clips` at random, at a volume and
/// pitch drawn from their ranges.
#[derive(Clone, Debug, PartialEq)]
pub struct Sound {
    pub clips: Vec<String>,
    pub volume: [f32; 2],
    pub pitch: [f32; 2],
}

/// Nine-slice art: an image cut by four insets into corners that keep
/// their size, edges that stretch or tile along their side, and a centre
/// drawn or left out. A class `nine-<name>` gives an element it.
#[derive(Clone, Debug, PartialEq)]
pub struct NineSlice {
    /// The image, relative to the pack or an asset id (`@styles/…`).
    pub src: String,
    /// The insets into the image, in its pixels: top, right, bottom, left.
    pub slice: [f32; 4],
    /// The insets as drawn on the element, in points; the slice's when
    /// unset.
    pub width: Option<[f32; 4]>,
    /// Whether the centre is drawn.
    pub fill: bool,
    /// Whether the edges tile (`true`) or stretch along their side.
    pub tile: bool,
    /// Whether the image is a shape the element's fill colours (its alpha
    /// kept, its colour the fill's), rather than a picture drawn as it is.
    pub tint: bool,
}

/// The haptic preset names a pack or element prop accepts.
pub const HAPTIC_PRESETS: [&str; 9] = [
    "tick", "tap", "thud", "double", "buzz", "success", "warning", "error", "none",
];

/// Parses a haptic preset name (`"none"` yields [`og_ui_model::Haptic::NONE`]).
pub fn parse_haptic(name: &str) -> Option<og_ui_model::Haptic> {
    use og_ui_model::{Haptic, HapticPreset as P};
    Some(match name {
        "tick" => Haptic::preset(P::Tick),
        "tap" => Haptic::preset(P::Tap),
        "thud" => Haptic::preset(P::Thud),
        "double" => Haptic::preset(P::Double),
        "buzz" => Haptic::preset(P::Buzz),
        "success" => Haptic::preset(P::Success),
        "warning" => Haptic::preset(P::Warning),
        "error" => Haptic::preset(P::Error),
        "none" => Haptic::NONE,
        _ => return None,
    })
}

/// The sounds of one scope, by event.
pub type Sounds = BTreeMap<String, Sound>;

/// The haptic preset names of one scope, by event.
pub type Haptics = BTreeMap<String, String>;

mod layer;
pub use layer::Layer;
use layer::merge;

/// The layers merged: what the cook compiles and the runtime loads.
#[derive(Clone, Debug)]
pub struct Pack {
    pub scheme: Option<Scheme>,
    themes: [Map<String, Value>; 2],
    /// The high-contrast entries over each scheme's theme, merged.
    contrast: [Map<String, Value>; 2],
    variants: BTreeMap<String, String>,
    pub fonts: Vec<String>,
    pub icons: BTreeMap<String, String>,
    /// The layers' icon sets, the last layer's first: an icon is drawn
    /// from the first that names it.
    pub icon_sets: Vec<String>,
    images: [BTreeMap<String, NineSlice>; 2],
    pub sounds: BTreeMap<String, Sounds>,
    pub haptics: BTreeMap<String, Haptics>,
}

impl Pack {
    /// `layers` merged in order, the default pack first: a later layer
    /// replaces an earlier one's entry of the same name (a theme entry, a
    /// variant, an icon, an event's sound), and its fonts add faces. When
    /// a layer sets seeds, the design palette is derived again from every
    /// layer's seeds and stands in for the default's colours; colours a
    /// layer writes out lie over the derived ones.
    pub fn new(layers: &[Layer]) -> Pack {
        let mut themes: [Map<String, Value>; 2] = Default::default();
        let mut pack = Pack {
            scheme: None,
            themes: Default::default(),
            contrast: Default::default(),
            variants: BTreeMap::new(),
            fonts: Vec::new(),
            icons: BTreeMap::new(),
            icon_sets: Vec::new(),
            images: Default::default(),
            sounds: BTreeMap::new(),
            haptics: BTreeMap::new(),
        };
        let seeds: Vec<_> = layers.iter().flat_map(|l| &l.seeds).collect();
        let derived = (!seeds.is_empty()).then(|| {
            let mut css = String::new();
            for (i, s) in Scheme::ALL.into_iter().enumerate() {
                css.push_str(match s {
                    Scheme::Light => ":root {",
                    Scheme::Dark => ":root[data-scheme=dark] {",
                });
                for (name, v) in &seeds {
                    css.push_str(&format!(" --seed-{name}: {};", v[i]));
                }
                css.push_str(" }\n");
            }
            design::colors(&css)
        });
        for (n, layer) in layers.iter().enumerate() {
            pack.scheme = layer.scheme.or(pack.scheme);
            for (i, theme) in themes.iter_mut().enumerate() {
                for (section, v) in &layer.theme[i] {
                    let v = match (n, section.as_str(), &derived) {
                        (0, "colors", Some(d)) => Value::Object(d[i].clone()),
                        _ => v.clone(),
                    };
                    match (theme.get_mut(section), v) {
                        (Some(Value::Object(have)), Value::Object(over)) => have.extend(over),
                        (_, v) => {
                            theme.insert(section.clone(), v);
                        }
                    }
                }
            }
            for (have, over) in pack.contrast.iter_mut().zip(&layer.contrast) {
                merge(have, over);
            }
            pack.variants.extend(layer.variants.clone());
            pack.fonts.extend(layer.fonts.iter().cloned());
            pack.icons.extend(layer.icons.clone());
            pack.icon_sets.splice(0..0, layer.icon_set.iter().cloned());
            for (have, over) in pack.images.iter_mut().zip(&layer.images) {
                have.extend(over.clone());
            }
            for (scope, sounds) in &layer.sounds {
                pack.sounds
                    .entry(scope.clone())
                    .or_default()
                    .extend(sounds.clone());
            }
            for (scope, haptics) in &layer.haptics {
                pack.haptics
                    .entry(scope.clone())
                    .or_default()
                    .extend(haptics.clone());
            }
        }
        pack.themes = themes;
        pack
    }

    /// The theme in `scheme` as `Theme::from_json` reads it: the default
    /// Tailwind theme extended by every layer.
    pub fn theme_json(&self, scheme: Scheme) -> String {
        Value::Object(self.themes[scheme as usize].clone()).to_string()
    }

    /// The theme in `scheme` in high contrast (`prefers-contrast:
    /// more`), as [`Pack::theme_json`] gives it: every layer's `contrast`
    /// entries over the theme (the default pack's from the design's
    /// `:root[data-contrast=more]` tokens).
    pub fn theme_json_high_contrast(&self, scheme: Scheme) -> String {
        let mut theme = self.themes[scheme as usize].clone();
        merge(&mut theme, &self.contrast[scheme as usize]);
        Value::Object(theme).to_string()
    }

    /// The theme in `scheme`, as a JSON object.
    pub fn theme(&self, scheme: Scheme) -> &Map<String, Value> {
        &self.themes[scheme as usize]
    }

    /// The nine-slice art in `scheme`, by the name `nine-<name>` gives.
    pub fn images(&self, scheme: Scheme) -> &BTreeMap<String, NineSlice> {
        &self.images[scheme as usize]
    }

    /// The class strings each `Component.slot[.variant]` appends.
    pub fn variants(&self) -> &BTreeMap<String, String> {
        &self.variants
    }
}

/// `nine-none`: no nine-slice art, so a state can drop the art an earlier
/// `nine-<name>` gave (`data-kind-owned:nine-none`) and its fill shows. No
/// pack image takes this name.
pub const NINE_NONE: &str = "none";

/// The names of the nine-slice art `classes` gives (`nine-<name>`, under
/// any variant: `hover:nine-plate-lit`), [`NINE_NONE`] left out, and the
/// classes without any `nine-` class.
pub fn split_nine(classes: &str) -> (Vec<&str>, String) {
    let mut names = Vec::new();
    let mut rest = Vec::new();
    for class in classes.split_whitespace() {
        let utility = class.rsplit(':').next().unwrap_or(class);
        match utility.strip_prefix("nine-") {
            Some(NINE_NONE) => {}
            Some(name) => names.push(name),
            None => rest.push(class),
        }
    }
    (names, rest.join(" "))
}

/// The variants of `layer`, laid over `base` (the packs under it), that
/// paint the ground under the text, a fill or art with a centre drawn in
/// its own colours (`fill` and no `tint`), but set no ink for it in their
/// base, hover, pressed or disabled state: one message per variant and
/// scheme. A component's ink was chosen for the component's own ground, so
/// a variant that replaces the ground names the ink too.
pub fn grounds_without_ink(base: &[Layer], layer: &Layer) -> Vec<String> {
    use crate::classes::{Classes, Env, state};
    let mut layers = base.to_vec();
    layers.push(layer.clone());
    let pack = Pack::new(&layers);
    let mut out = Vec::new();
    for scheme in Scheme::ALL {
        let Ok((mut classes, _)) = Classes::new(Some(&pack.theme_json(scheme))) else {
            continue;
        };
        for (key, class) in &layer.variants {
            let (images, class) = split_nine(class);
            let art = images.iter().any(|n| {
                pack.images(scheme)
                    .get(*n)
                    .is_some_and(|i| i.fill && !i.tint)
            });
            for states in [0, state::HOVERED, state::PRESSED, state::DISABLED] {
                let env = Env {
                    window: [1280.0, 720.0],
                    dark: scheme == Scheme::Dark,
                    rtl: false,
                    touch: false,
                    states,
                };
                let r = classes.record(&class, &env);
                let fill = r.skin.fill[3] > 0.0 || r.skin.gradient.is_some();
                if (art || fill) && !r.inked {
                    out.push(format!(
                        "variants.{key}: paints {} but sets no ink (a `text-*` colour for that ground) ({})",
                        if art { "art in its own colours" } else { "a fill" },
                        scheme.name()
                    ));
                    break;
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::layer::variant_key;
    use super::*;

    /// The slots the engine's TSX components declare (`packClass("C",
    /// "slot"[, variant])` with literal names; a variant written as an
    /// expression declares each string literal in it). A declaration that
    /// is no `Component.slot[.variant]` key (a lowercase component) goes
    /// to `bad` with its file: no pack could key it.
    fn declared_slots(
        dir: &std::path::Path,
        out: &mut std::collections::BTreeSet<String>,
        bad: &mut Vec<String>,
    ) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                declared_slots(&path, out, bad);
                continue;
            }
            if !kind.is_file() || !path.extension().is_some_and(|e| e == "tsx" || e == "ts") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for (at, _) in text.match_indices("packClass(") {
                let args = &text[at + "packClass(".len()..];
                let mut depth = 0;
                let end = args
                    .char_indices()
                    .find(|&(_, c)| {
                        depth += match c {
                            '(' => 1,
                            ')' => -1,
                            _ => 0,
                        };
                        depth < 0
                    })
                    .map_or(args.len(), |(i, _)| i);
                let args = &args[..end];
                let mut parts = args.splitn(3, ',').map(str::trim);
                let literal = |a: Option<&str>| {
                    a.and_then(|a| a.strip_prefix('"')?.strip_suffix('"'))
                        .map(str::to_owned)
                };
                let (Some(component), Some(slot)) = (literal(parts.next()), literal(parts.next()))
                else {
                    continue;
                };
                let base = format!("{component}.{slot}");
                let variants = parts.next().unwrap_or("").split('"').skip(1).step_by(2);
                for key in
                    std::iter::once(base.clone()).chain(variants.map(|v| format!("{base}.{v}")))
                {
                    if !variant_key(&key) {
                        bad.push(format!("{}: packClass {key}", path.display()));
                    }
                    out.insert(key);
                }
            }
        }
    }

    /// Every slot an engine component declares is a valid pack key, and
    /// every slot a shipped pack gives sounds or classes names a slot an
    /// engine component declares: a key for another name (an og-ui part
    /// such as `Button.base`) never matches and its sounds never play.
    #[test]
    fn shipped_packs_key_only_the_slots_components_declare() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let (mut declared, mut bad) = (std::collections::BTreeSet::new(), Vec::new());
        declared_slots(&root.join("packages/engine/src"), &mut declared, &mut bad);
        assert!(
            bad.is_empty(),
            "slots that are no Component.slot[.variant] key (a PascalCase component, lowercase slot and variant):\n{}",
            bad.join("\n")
        );
        assert!(declared.contains("Button.plate.primary"), "{declared:?}");
        let mut stray = Vec::new();
        for entry in std::fs::read_dir(root.join("templates/styles")).unwrap() {
            let file = entry.unwrap().path().join("pack.json");
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            let layer = Layer::parse(&text).unwrap();
            let keys = layer.variants.keys().map(|k| ("variants", k));
            let keys = keys.chain(
                layer
                    .sounds
                    .keys()
                    .filter(|k| !k.is_empty())
                    .map(|k| ("sounds", k)),
            );
            let keys = keys.chain(
                layer
                    .haptics
                    .keys()
                    .filter(|k| !k.is_empty())
                    .map(|k| ("haptics", k)),
            );
            for (what, key) in keys {
                if !declared.contains(key) {
                    stray.push(format!("{}: {what}.{key}", file.display()));
                }
            }
        }
        assert!(
            stray.is_empty(),
            "pack keys no component declares (declared: {declared:?}):\n{}",
            stray.join("\n")
        );
    }

    #[test]
    fn haptics_parse_roundtrip_and_merge() {
        let a = Layer::parse(
            r#"{ "haptics": { "press": "tick", "Button.plate.danger": { "confirm": "thud", "hover": "none" } } }"#,
        )
        .unwrap();
        let again = Layer::parse(&a.to_json()).unwrap();
        assert_eq!(again.haptics, a.haptics);
        let b = Layer::parse(r#"{ "haptics": { "press": "tap" } }"#).unwrap();
        let p = Pack::new(&[Layer::default_pack(), a, b]);
        assert_eq!(p.haptics[""]["press"], "tap");
        assert_eq!(p.haptics["Button.plate.danger"]["confirm"], "thud");
        assert_eq!(p.haptics["Button.plate.danger"]["hover"], "none");
        assert!(Layer::parse(r#"{ "haptics": { "press": "bogus" } }"#).is_err());
        assert!(Layer::parse(r#"{ "haptics": { "not_a_slot": { "press": "tap" } } }"#).is_err());
    }
}
