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

/// The sounds of one scope, by event.
pub type Sounds = BTreeMap<String, Sound>;

/// One `pack.json` as read: nothing merged yet.
#[derive(Clone, Debug, Default)]
pub struct Layer {
    /// The scheme the pack is drawn in unless the game asks for another.
    pub scheme: Option<Scheme>,
    /// Seed overrides, by name, per scheme.
    seeds: Vec<(String, [String; 2])>,
    /// The theme per scheme ([`Scheme::ALL`] order).
    theme: [Map<String, Value>; 2],
    /// What the theme takes in high contrast (`prefers-contrast: more`),
    /// per scheme: entries over the theme's, as `theme` writes them.
    contrast: [Map<String, Value>; 2],
    pub variants: BTreeMap<String, String>,
    /// The font files it ships, each relative to the pack's directory or
    /// a shared asset (`@styles/shared/…`). A file's family, weight, style
    /// and width are its own: the theme's `fontFamily` lists name the
    /// families as the files do.
    pub fonts: Vec<String>,
    pub icons: BTreeMap<String, String>,
    /// The icon set (an `.ogt` of `type icons`) `<Icon name>` draws from,
    /// under the game's own icons: a later layer's over an earlier one's.
    pub icon_set: Option<String>,
    /// Nine-slice art by name, per scheme ([`Scheme::ALL`] order).
    pub images: [BTreeMap<String, NineSlice>; 2],
    /// The pack-wide sounds under `""`, a slot's under its key.
    pub sounds: BTreeMap<String, Sounds>,
}

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
}

impl Layer {
    /// The engine's default pack (`default/pack.json`).
    pub fn default_pack() -> Layer {
        Layer::parse(design::DEFAULT_PACK).expect("default/pack.json is generated valid")
    }

    /// Reads one `pack.json`. Every key and form is checked: an unknown
    /// key, a variant key not of the form `Component.slot[.variant]`, or a
    /// scheme pair missing a side is an error naming where it is.
    pub fn parse(text: &str) -> Result<Layer, String> {
        let root: Value = serde_json::from_str(text).map_err(|e| format!("pack.json: {e}"))?;
        let Value::Object(root) = root else {
            return Err("pack.json: not an object".into());
        };
        let mut layer = Layer::default();
        for (key, v) in &root {
            match key.as_str() {
                "about" => {}
                "scheme" => {
                    let s = v.as_str().and_then(Scheme::of);
                    layer.scheme = Some(s.ok_or("scheme: \"light\" or \"dark\"")?);
                }
                "seeds" => {
                    for (name, v) in object(v, "seeds")? {
                        if !SEEDS.contains(&name.as_str()) {
                            return Err(format!(
                                "seeds.{name}: not a seed; one of {}",
                                SEEDS.join(", ")
                            ));
                        }
                        let at = format!("seeds.{name}");
                        let side = |s| match split(v, s, &at)? {
                            Value::String(c) => Ok(c),
                            _ => Err(format!("{at}: a CSS colour")),
                        };
                        layer
                            .seeds
                            .push((name.clone(), [side(Scheme::Light)?, side(Scheme::Dark)?]));
                    }
                }
                "theme" | "contrast" => {
                    for (section, _) in object(v, key)? {
                        if !THEME_KEYS.contains(&section.as_str()) {
                            return Err(format!(
                                "{key}.{section}: not a theme key; one of {}",
                                THEME_KEYS.join(", ")
                            ));
                        }
                    }
                    for (i, s) in Scheme::ALL.into_iter().enumerate() {
                        let Value::Object(m) = split(v, s, key)? else {
                            unreachable!()
                        };
                        if key == "theme" {
                            layer.theme[i] = m;
                        } else {
                            layer.contrast[i] = m;
                        }
                    }
                }
                "variants" => {
                    for (k, v) in object(v, "variants")? {
                        if !variant_key(k) {
                            return Err(format!(
                                "variants.{k}: a key is Component.slot or Component.slot.variant (PascalCase, then lower-case names)"
                            ));
                        }
                        // An empty string declares the slot and adds nothing:
                        // the default pack's slots are the vocabulary.
                        let class = v
                            .as_str()
                            .ok_or_else(|| format!("variants.{k}: a class string"))?;
                        layer.variants.insert(k.clone(), class.trim().to_owned());
                    }
                }
                "fonts" => {
                    let Value::Array(files) = v else {
                        return Err("fonts: an array of font files".into());
                    };
                    for (i, f) in files.iter().enumerate() {
                        let src = f.as_str().ok_or_else(|| {
                            format!("fonts[{i}]: a font file's path; its family, weight and style are the file's own, and the theme's fontFamily names its family")
                        })?;
                        layer.fonts.push(src.to_owned());
                    }
                }
                "images" => {
                    for (name, v) in object(v, "images")? {
                        let at = format!("images.{name}");
                        if !kebab(name) {
                            return Err(format!("{at}: a name is lower-case words joined by `-`"));
                        }
                        for (i, s) in Scheme::ALL.into_iter().enumerate() {
                            let image =
                                nine(&split(v, s, &at)?).map_err(|e| format!("{at}: {e}"))?;
                            layer.images[i].insert(name.clone(), image);
                        }
                    }
                }
                "icons" => {
                    for (name, src) in object(v, "icons")? {
                        let src = src
                            .as_str()
                            .ok_or_else(|| format!("icons.{name}: a path"))?;
                        layer.icons.insert(name.clone(), src.to_owned());
                    }
                }
                "iconSet" => {
                    let src = v.as_str().ok_or("iconSet: a path to an icon set (.ogt)")?;
                    layer.icon_set = Some(src.to_owned());
                }
                "sounds" => {
                    for (k, v) in object(v, "sounds")? {
                        if SOUND_EVENTS.contains(&k.as_str()) {
                            let s = sound(v).map_err(|e| format!("sounds.{k}: {e}"))?;
                            layer
                                .sounds
                                .entry(String::new())
                                .or_default()
                                .insert(k.clone(), s);
                        } else if variant_key(k) {
                            let slot = layer.sounds.entry(k.clone()).or_default();
                            for (event, v) in object(v, &format!("sounds.{k}"))? {
                                if !SOUND_EVENTS.contains(&event.as_str()) {
                                    return Err(format!(
                                        "sounds.{k}.{event}: one of {}",
                                        SOUND_EVENTS.join(", ")
                                    ));
                                }
                                let s = sound(v).map_err(|e| format!("sounds.{k}.{event}: {e}"))?;
                                slot.insert(event.clone(), s);
                            }
                        } else {
                            return Err(format!(
                                "sounds.{k}: an event ({}) or Component.slot[.variant]",
                                SOUND_EVENTS.join(", ")
                            ));
                        }
                    }
                }
                other => {
                    return Err(format!(
                        "{other}: not a pack key; one of about, scheme, seeds, theme, contrast, variants, images, fonts, icons, iconSet, sounds"
                    ));
                }
            }
        }
        Ok(layer)
    }
}

impl Layer {
    /// The family names its theme's `fontFamily` lists for the text roles
    /// body (`sans`), display, mono and rounded, first first, quotes
    /// removed (the light scheme's; a pack's fonts do not change with
    /// the scheme). A role the theme leaves out is empty.
    pub fn role_families(&self) -> [Vec<String>; 4] {
        let list = |key: &str| -> Vec<String> {
            self.theme[0]
                .get("fontFamily")
                .and_then(|f| f.get(key))
                .and_then(Value::as_str)
                .map(|l| {
                    l.split(',')
                        .map(|n| n.trim().trim_matches(['"', '\'']).to_owned())
                        .filter(|n| !n.is_empty())
                        .collect()
                })
                .unwrap_or_default()
        };
        ["sans", "display", "mono", "rounded"].map(list)
    }

    /// Its theme's `fontFamily` list for `key` (`sans`, `display`, `mono`,
    /// `rounded`, `serif`), as CSS writes a `font-family` (the light
    /// scheme's; a pack's fonts do not change with the scheme).
    pub fn font_family(&self, key: &str) -> Option<String> {
        let list = self.theme[0].get("fontFamily")?.get(key)?.as_str()?;
        (!list.trim().is_empty()).then(|| list.trim().to_owned())
    }

    /// Every asset the layer names: its fonts, icons and sound clips.
    pub fn assets(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self.fonts.iter().map(String::as_str).collect();
        out.extend(self.icons.values().map(String::as_str));
        out.extend(self.icon_set.as_deref());
        out.extend(
            self.images
                .iter()
                .flat_map(|m| m.values().map(|n| n.src.as_str())),
        );
        for sounds in self.sounds.values() {
            out.extend(
                sounds
                    .values()
                    .flat_map(|s| s.clips.iter().map(String::as_str)),
            );
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Resolves every asset path against the pack's directory `dir` (its
    /// asset id, `styles/mine`): a path starting with `@` or `/` names
    /// an asset by id and stays (`/` dropped).
    pub fn resolve(&mut self, dir: &str) {
        let at = |p: &mut String| {
            if let Some(id) = p.strip_prefix('/') {
                *p = id.to_owned();
            } else if !p.starts_with('@') {
                *p = format!("{}/{p}", dir.trim_end_matches('/'));
            }
        };
        self.fonts.iter_mut().for_each(at);
        self.icons.values_mut().for_each(at);
        self.icon_set.iter_mut().for_each(at);
        for m in &mut self.images {
            m.values_mut().for_each(|n| at(&mut n.src));
        }
        for sounds in self.sounds.values_mut() {
            sounds
                .values_mut()
                .flat_map(|s| s.clips.iter_mut())
                .for_each(at);
        }
    }

    /// The layer as `pack.json` again ([`Layer::parse`] reads it back to
    /// the same layer): a cooked pack's blob.
    pub fn to_json(&self) -> String {
        use serde_json::json;
        let mut root = Map::new();
        if let Some(s) = self.scheme {
            root.insert("scheme".into(), json!(s.name()));
        }
        if !self.seeds.is_empty() {
            let seeds: Map<String, Value> = self
                .seeds
                .iter()
                .map(|(n, [l, d])| (n.clone(), design::join(&json!(l), &json!(d))))
                .collect();
            root.insert("seeds".into(), Value::Object(seeds));
        }
        let [light, dark] = &self.theme;
        if !light.is_empty() || !dark.is_empty() {
            let theme = design::join(&Value::Object(light.clone()), &Value::Object(dark.clone()));
            root.insert("theme".into(), theme);
        }
        let [light, dark] = &self.contrast;
        if !light.is_empty() || !dark.is_empty() {
            let contrast =
                design::join(&Value::Object(light.clone()), &Value::Object(dark.clone()));
            root.insert("contrast".into(), contrast);
        }
        if !self.variants.is_empty() {
            root.insert("variants".into(), json!(self.variants));
        }
        if !self.fonts.is_empty() {
            root.insert("fonts".into(), json!(self.fonts));
        }
        if !self.icons.is_empty() {
            root.insert("icons".into(), json!(self.icons));
        }
        if let Some(set) = &self.icon_set {
            root.insert("iconSet".into(), json!(set));
        }
        let [light, dark] = &self.images;
        if !light.is_empty() || !dark.is_empty() {
            let one = |n: &NineSlice| {
                let mut m = json!({ "src": n.src, "slice": n.slice, "fill": n.fill, "repeat": if n.tile { "tile" } else { "stretch" }, "tint": n.tint });
                if let Some(w) = n.width {
                    m["width"] = json!(w);
                }
                m
            };
            let side = |m: &BTreeMap<String, NineSlice>| {
                Value::Object(m.iter().map(|(k, n)| (k.clone(), one(n))).collect())
            };
            root.insert("images".into(), design::join(&side(light), &side(dark)));
        }
        if !self.sounds.is_empty() {
            let sound =
                |s: &Sound| json!({ "clips": s.clips, "volume": s.volume, "pitch": s.pitch });
            let mut m = Map::new();
            for (scope, sounds) in &self.sounds {
                let events: Map<String, Value> =
                    sounds.iter().map(|(e, s)| (e.clone(), sound(s))).collect();
                if scope.is_empty() {
                    m.extend(events);
                } else {
                    m.insert(scope.clone(), Value::Object(events));
                }
            }
            root.insert("sounds".into(), Value::Object(m));
        }
        Value::Object(root).to_string()
    }
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

/// `over`'s sections over `have`'s, entry by entry.
fn merge(have: &mut Map<String, Value>, over: &Map<String, Value>) {
    for (section, v) in over {
        match (have.get_mut(section), v) {
            (Some(Value::Object(h)), Value::Object(o)) => h.extend(o.clone()),
            (_, v) => {
                have.insert(section.clone(), v.clone());
            }
        }
    }
}

fn object<'a>(v: &'a Value, at: &str) -> Result<&'a Map<String, Value>, String> {
    v.as_object().ok_or_else(|| format!("{at}: an object"))
}

/// Whether `v` is a scheme pair: exactly `light` and `dark`.
fn pair(m: &Map<String, Value>) -> Option<(&Value, &Value)> {
    (m.len() == 2)
        .then(|| Some((m.get("light")?, m.get("dark")?)))
        .flatten()
}

/// `v` with every scheme pair replaced by its `scheme` side.
fn split(v: &Value, scheme: Scheme, at: &str) -> Result<Value, String> {
    let Value::Object(m) = v else {
        return Ok(v.clone());
    };
    if let Some((light, dark)) = pair(m) {
        let side = if scheme == Scheme::Dark { dark } else { light };
        return split(side, scheme, at);
    }
    if m.contains_key("light") || m.contains_key("dark") {
        return Err(format!(
            "{at}: a scheme pair has exactly `light` and `dark`"
        ));
    }
    let mut out = Map::new();
    for (k, v) in m {
        out.insert(k.clone(), split(v, scheme, &format!("{at}.{k}"))?);
    }
    Ok(Value::Object(out))
}

/// `Component.slot` or `Component.slot.variant`.
fn variant_key(k: &str) -> bool {
    let parts: Vec<&str> = k.split('.').collect();
    let name = |p: &str| {
        !p.is_empty()
            && p.starts_with(|c: char| c.is_ascii_lowercase())
            && p.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    };
    matches!(parts.len(), 2 | 3)
        && parts[0].starts_with(|c: char| c.is_ascii_uppercase())
        && parts[0].chars().all(|c| c.is_ascii_alphanumeric())
        && parts[1..].iter().all(|p| name(p))
}

fn kebab(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|w| {
            !w.is_empty()
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
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

/// Four insets: one number for all, or `[top, right, bottom, left]`.
fn insets(v: &Value, what: &str) -> Result<[f32; 4], String> {
    let n = |v: &Value| v.as_f64().filter(|n| *n >= 0.0).map(|n| n as f32);
    match v {
        Value::Number(_) => n(v).map(|x| [x; 4]),
        Value::Array(a) if a.len() == 4 => {
            let s: Option<Vec<f32>> = a.iter().map(n).collect();
            s.map(|s| [s[0], s[1], s[2], s[3]])
        }
        _ => None,
    }
    .ok_or_else(|| format!("{what}: a number of 0 or more, or [top, right, bottom, left]"))
}

fn nine(v: &Value) -> Result<NineSlice, String> {
    let m = object(v, "nine-slice art")?;
    for k in m.keys() {
        if !["src", "slice", "width", "fill", "repeat", "tint"].contains(&k.as_str()) {
            return Err(format!("{k}: one of src, slice, width, fill, repeat, tint"));
        }
    }
    let src = m
        .get("src")
        .and_then(Value::as_str)
        .ok_or("src: an image")?
        .to_owned();
    let slice = insets(
        m.get("slice")
            .ok_or("slice: the insets into the image, in its pixels")?,
        "slice",
    )?;
    let width = m.get("width").map(|w| insets(w, "width")).transpose()?;
    let fill = match m.get("fill") {
        None => true,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err("fill: true or false".into()),
    };
    let tile = match m.get("repeat").map(|r| r.as_str()) {
        None | Some(Some("stretch")) => false,
        Some(Some("tile")) => true,
        _ => return Err("repeat: \"stretch\" or \"tile\"".into()),
    };
    let tint = match m.get("tint") {
        None => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err("tint: true or false".into()),
    };
    Ok(NineSlice {
        src,
        slice,
        width,
        fill,
        tile,
        tint,
    })
}

fn sound(v: &Value) -> Result<Sound, String> {
    let m = object(v, "a sound")?;
    for k in m.keys() {
        if !["clips", "volume", "pitch"].contains(&k.as_str()) {
            return Err(format!("{k}: one of clips, volume, pitch"));
        }
    }
    let clips: Vec<String> = match m.get("clips") {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    };
    // A number is a fixed value, a pair a range.
    let range = |k: &str, default: f32| -> Result<[f32; 2], String> {
        match m.get(k) {
            None => Ok([default; 2]),
            Some(Value::Number(n)) => Ok([n.as_f64().unwrap_or(0.0) as f32; 2]),
            Some(Value::Array(a)) if a.len() == 2 && a.iter().all(Value::is_number) => {
                Ok([a[0].as_f64().unwrap() as f32, a[1].as_f64().unwrap() as f32])
            }
            _ => Err(format!("{k}: a number or [low, high]")),
        }
    };
    let volume = range("volume", 1.0)?;
    // An event at volume 0 is silent and needs no clip: it silences the
    // layers' under it.
    if clips.is_empty() && volume != [0.0; 2] {
        return Err("clips: one or more sound assets (or volume 0, for silence)".into());
    }
    Ok(Sound {
        clips,
        volume,
        pitch: range("pitch", 1.0)?,
    })
}

#[cfg(test)]
mod tests {
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
}
