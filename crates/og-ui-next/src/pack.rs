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
pub const THEME_KEYS: [&str; 18] = [
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
    "blur",
    "perspective",
    "aspect",
    "ease",
];

/// The seeds the design palette is derived from (`packages/design/tokens/seed.css`).
pub const SEEDS: [&str; 9] = [
    "primary", "accent", "neutral", "danger", "warn", "success", "info", "team", "hud",
];

/// The interaction events a pack gives sounds for.
pub const SOUND_EVENTS: [&str; 10] = [
    "press", "hover", "select", "confirm", "cancel", "back", "open", "close", "error", "reward",
];

/// One font face a pack ships. `src` is relative to the pack's directory
/// or names a shared asset (`@styles/shared/…`).
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub family: String,
    pub src: String,
    pub weight: u16,
    pub italic: bool,
    /// A CSS `unicode-range` (`U+4E00-9FFF, U+3000-303F`): the face draws
    /// only these characters, so a fallback face covers a script.
    pub unicode_range: Option<String>,
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
    pub fonts: Vec<Face>,
    pub icons: BTreeMap<String, String>,
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
    pub fonts: Vec<Face>,
    pub icons: BTreeMap<String, String>,
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
                    let Value::Array(faces) = v else {
                        return Err("fonts: an array of faces".into());
                    };
                    for (i, f) in faces.iter().enumerate() {
                        layer
                            .fonts
                            .push(face(f).map_err(|e| format!("fonts[{i}]: {e}"))?);
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
                        "{other}: not a pack key; one of about, scheme, seeds, theme, contrast, variants, images, fonts, icons, sounds"
                    ));
                }
            }
        }
        Ok(layer)
    }
}

impl Layer {
    /// Every asset the layer names: its faces, icons and sound clips.
    pub fn assets(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self.fonts.iter().map(|f| f.src.as_str()).collect();
        out.extend(self.icons.values().map(String::as_str));
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
    /// asset id, `styles/cartoon`): a path starting with `@` or `/` names
    /// an asset by id and stays (`/` dropped).
    pub fn resolve(&mut self, dir: &str) {
        let at = |p: &mut String| {
            if let Some(id) = p.strip_prefix('/') {
                *p = id.to_owned();
            } else if !p.starts_with('@') {
                *p = format!("{}/{p}", dir.trim_end_matches('/'));
            }
        };
        self.fonts.iter_mut().for_each(|f| at(&mut f.src));
        self.icons.values_mut().for_each(at);
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
            let faces: Vec<Value> = self
                .fonts
                .iter()
                .map(|f| {
                    let mut m = json!({ "family": f.family, "src": f.src, "weight": f.weight });
                    if f.italic {
                        m["style"] = json!("italic");
                    }
                    if let Some(r) = &f.unicode_range {
                        m["unicodeRange"] = json!(r);
                    }
                    m
                })
                .collect();
            root.insert("fonts".into(), Value::Array(faces));
        }
        if !self.icons.is_empty() {
            root.insert("icons".into(), json!(self.icons));
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

/// The names of the nine-slice art `classes` gives (`nine-<name>`, under
/// any variant: `hover:nine-plate-lit`), and the classes without them.
pub fn split_nine(classes: &str) -> (Vec<&str>, String) {
    let mut names = Vec::new();
    let mut rest = Vec::new();
    for class in classes.split_whitespace() {
        let utility = class.rsplit(':').next().unwrap_or(class);
        match utility.strip_prefix("nine-") {
            Some(name) => names.push(name),
            None => rest.push(class),
        }
    }
    (names, rest.join(" "))
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

fn face(v: &Value) -> Result<Face, String> {
    let m = object(v, "a face")?;
    let text = |k: &str| m.get(k).and_then(Value::as_str).map(str::to_owned);
    for k in m.keys() {
        if !["family", "src", "weight", "style", "unicodeRange"].contains(&k.as_str()) {
            return Err(format!(
                "{k}: one of family, src, weight, style, unicodeRange"
            ));
        }
    }
    let weight = match m.get("weight") {
        None => 400,
        Some(w) => w
            .as_u64()
            .filter(|w| (1..=1000).contains(w))
            .ok_or("weight: 1 to 1000")? as u16,
    };
    let italic = match text("style").as_deref() {
        None | Some("normal") => false,
        Some("italic") => true,
        Some(_) => return Err("style: \"normal\" or \"italic\"".into()),
    };
    Ok(Face {
        family: text("family").ok_or("family: a name")?,
        src: text("src").ok_or("src: a font file")?,
        weight,
        italic,
        unicode_range: text("unicodeRange"),
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
    if clips.is_empty() {
        return Err("clips: one or more sound assets".into());
    }
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
    Ok(Sound {
        clips,
        volume: range("volume", 1.0)?,
        pitch: range("pitch", 1.0)?,
    })
}

#[cfg(test)]
mod tests {
    /// High contrast is pack data: the default pack's from the design's
    /// `[data-contrast=more]` tokens, and a style pack's `contrast` over it.
    #[test]
    fn high_contrast_is_each_layers_contrast_over_its_theme() {
        let theme = |p: &super::Pack, high: bool| -> serde_json::Value {
            let t = if high {
                p.theme_json_high_contrast(super::Scheme::Dark)
            } else {
                p.theme_json(super::Scheme::Dark)
            };
            serde_json::from_str::<serde_json::Value>(&t).unwrap()["colors"].clone()
        };
        let base = super::Pack::new(&[super::Layer::default_pack()]);
        let (plain, high) = (theme(&base, false), theme(&base, true));
        assert_eq!(
            high["fg-muted"], plain["fg"],
            "muted ink reads as the body ink"
        );
        assert_eq!(high["tile"], plain["border-strong"]);
        assert_eq!(high["accent"], plain["accent"], "the rest stays");
        let own = super::Layer::parse(r##"{ "contrast": { "colors": { "tile": "#ffffff" } } }"##)
            .unwrap();
        let styled = super::Pack::new(&[super::Layer::default_pack(), own]);
        assert_eq!(theme(&styled, true)["tile"], "#ffffff");
        assert_eq!(
            theme(&styled, true)["fg-muted"],
            plain["fg"],
            "the default's other entries stay"
        );
        assert_eq!(theme(&styled, false)["tile"], plain["tile"]);
    }

    use super::*;

    fn pack(over: &str) -> Pack {
        Pack::new(&[Layer::default_pack(), Layer::parse(over).unwrap()])
    }

    fn color(p: &Pack, s: Scheme, name: &str) -> String {
        p.theme(s)["colors"][name].as_str().unwrap().to_owned()
    }

    #[test]
    fn a_scheme_pair_splits_at_any_depth_and_a_later_layer_replaces_entries() {
        let p = pack(
            r##"{ "theme": {
                "colors": { "surface": { "light": "#f3efe6", "dark": "#1b1d24" }, "brand": { "500": { "light": "#111111", "dark": "#eeeeee" } } },
                "radius": { "card": 3 } } }"##,
        );
        assert_eq!(color(&p, Scheme::Light, "surface"), "#f3efe6");
        assert_eq!(color(&p, Scheme::Dark, "surface"), "#1b1d24");
        assert_eq!(p.theme(Scheme::Dark)["colors"]["brand"]["500"], "#eeeeee");
        // The default's other entries stay beside the pack's.
        assert_eq!(p.theme(Scheme::Light)["radius"]["card"], 3);
        assert!(p.theme(Scheme::Light)["radius"]["panel"].is_number());
        assert!(p.theme(Scheme::Light)["colors"]["fg"].is_string());
        // What the builder reads is plain theme JSON, no pair left.
        assert!(!p.theme_json(Scheme::Dark).contains("\"light\""));
    }

    /// The class resolver reads the default theme whole in each scheme, and
    /// its names are classes.
    #[test]
    fn the_class_resolver_takes_every_default_entry() {
        let p = Pack::new(&[Layer::default_pack()]);
        for s in Scheme::ALL {
            let (mut c, ignored) = crate::classes::Classes::new(Some(&p.theme_json(s))).unwrap();
            assert!(ignored.is_empty(), "{s:?}: {ignored:?}");
            let classes = "bg-surface text-fg-muted border-border-strong rounded-card shadow-panel text-body font-display tracking-wide leading-snug ease-out bg-legendary";
            assert!(
                c.unknown(classes).is_empty(),
                "{s:?}: {:?}",
                c.unknown(classes)
            );
        }
    }

    #[test]
    fn seeds_derive_the_palette_and_written_colours_lie_over_it() {
        let base = Pack::new(&[Layer::default_pack()]);
        let p = pack(
            r##"{ "seeds": { "primary": "#c8102e" }, "theme": { "colors": { "bg": "#010203" } } }"##,
        );
        for s in Scheme::ALL {
            assert_ne!(color(&p, s, "accent"), color(&base, s, "accent"), "{s:?}");
            assert_eq!(color(&p, s, "bg"), "#010203");
        }
        // A red seed gives a red accent, in both schemes.
        for s in Scheme::ALL {
            let c = og_css::parse_color(&color(&p, s, "accent")).unwrap();
            assert!(
                c.rgb[0] > c.rgb[1] && c.rgb[0] > c.rgb[2],
                "{s:?}: {:?}",
                c.rgb
            );
        }
    }

    #[test]
    fn every_form_error_names_where_it_is() {
        for (text, wants) in [
            (r##"{ "theme": { "colours": {} } }"##, "theme.colours"),
            (
                r##"{ "theme": { "colors": { "a": { "light": "#000" } } } }"##,
                "theme.colors.a",
            ),
            (
                r##"{ "variants": { "panel.frame": "p-2" } }"##,
                "variants.panel.frame",
            ),
            (
                r##"{ "variants": { "Panel.Frame": "p-2" } }"##,
                "variants.Panel.Frame",
            ),
            (r##"{ "seeds": { "brand": "#000" } }"##, "seeds.brand"),
            (
                r##"{ "sounds": { "Button.base": { "tap": { "clips": ["a"] } } } }"##,
                "sounds.Button.base.tap",
            ),
            (r##"{ "fonts": [{ "src": "a.ttf" }] }"##, "fonts[0]: family"),
            (r##"{ "surfaces": {} }"##, "surfaces"),
        ] {
            let e = Layer::parse(text).unwrap_err();
            assert!(e.starts_with(wants), "{text}: {e}");
        }
    }

    #[test]
    fn a_resolved_layer_writes_back_to_itself() {
        let mut l = Layer::parse(r##"{ "scheme": "light", "seeds": { "primary": { "light": "#111111", "dark": "#eeeeee" } },
            "theme": { "colors": { "gold": "#ffd23f", "surface": { "light": "#ffffff", "dark": "#000000" } } },
            "variants": { "Panel.frame": "p-2" },
            "fonts": [{ "family": "A", "src": "a.ttf", "weight": 700, "unicodeRange": "U+4E00-9FFF" }, { "family": "B", "src": "@styles/shared/b.ttf" }],
            "icons": { "star": "art/star.svg" },
            "sounds": { "press": { "clips": ["/sfx/tap.ogt"], "volume": [0.2, 0.3] }, "Button.base": { "hover": { "clips": ["t.ogt"] } } } }"##).unwrap();
        l.resolve("styles/cartoon");
        assert_eq!(
            l.assets(),
            [
                "@styles/shared/b.ttf",
                "sfx/tap.ogt",
                "styles/cartoon/a.ttf",
                "styles/cartoon/art/star.svg",
                "styles/cartoon/t.ogt"
            ]
        );
        let again = Layer::parse(&l.to_json()).unwrap();
        assert_eq!(again.to_json(), l.to_json());
        assert_eq!(again.theme, l.theme);
        assert_eq!(again.seeds, l.seeds);
        assert_eq!(again.fonts, l.fonts);
        assert_eq!(again.sounds, l.sounds);
        assert_eq!(again.scheme, Some(Scheme::Light));
    }

    #[test]
    fn nine_slice_art_reads_per_scheme_and_a_class_names_it() {
        let mut l = Layer::parse(r##"{ "images": {
            "plate": { "src": "art/plate.png", "slice": [12, 16, 12, 16], "width": 8, "repeat": "tile", "tint": true },
            "frame": { "light": { "src": "art/frame-light.png", "slice": 20 }, "dark": { "src": "art/frame.png", "slice": 20, "fill": false } } },
            "variants": { "Panel.frame": "p-4 nine-frame hover:nine-plate" } }"##).unwrap();
        l.resolve("styles/x");
        let p = Pack::new(&[Layer::default_pack(), l.clone()]);
        let plate = &p.images(Scheme::Dark)["plate"];
        assert_eq!(
            (plate.slice, plate.width, plate.tile, plate.fill, plate.tint),
            ([12.0, 16.0, 12.0, 16.0], Some([8.0; 4]), true, true, true)
        );
        assert!(!p.images(Scheme::Dark)["frame"].tint);
        assert_eq!(
            p.images(Scheme::Light)["frame"].src,
            "styles/x/art/frame-light.png"
        );
        assert!(!p.images(Scheme::Dark)["frame"].fill);
        assert!(l.assets().contains(&"styles/x/art/frame.png"));
        assert_eq!(Layer::parse(&l.to_json()).unwrap().images, l.images);
        assert_eq!(
            split_nine("p-4 nine-frame hover:nine-plate"),
            (vec!["frame", "plate"], "p-4".to_owned())
        );
        for (text, wants) in [
            (
                r#"{ "images": { "a": { "src": "a.png" } } }"#,
                "images.a: slice",
            ),
            (
                r#"{ "images": { "a": { "src": "a.png", "slice": [1, 2] } } }"#,
                "images.a: slice",
            ),
            (
                r#"{ "images": { "A": { "src": "a.png", "slice": 1 } } }"#,
                "images.A",
            ),
            (
                r#"{ "images": { "a": { "src": "a.png", "slice": 1, "repeat": "round" } } }"#,
                "images.a: repeat",
            ),
        ] {
            let e = Layer::parse(text).unwrap_err();
            assert!(e.starts_with(wants), "{text}: {e}");
        }
    }

    #[test]
    fn slot_sounds_and_variants_merge_by_key() {
        let a = Layer::parse(r#"{ "variants": { "Panel.frame": "p-2", "Panel.frame.hud": "bg-plate-hi" },
            "sounds": { "press": { "clips": ["a"], "volume": [0.2, 0.3] }, "Button.base": { "hover": { "clips": ["b"], "pitch": 1.2 } } } }"#).unwrap();
        let b = Layer::parse(r#"{ "variants": { "Panel.frame": "p-4" }, "sounds": { "press": { "clips": ["c"] } } }"#).unwrap();
        let p = Pack::new(&[Layer::default_pack(), a, b]);
        assert_eq!(p.variants()["Panel.frame"], "p-4");
        assert_eq!(p.variants()["Panel.frame.hud"], "bg-plate-hi");
        assert_eq!(p.sounds[""]["press"].clips, ["c"]);
        assert_eq!(p.sounds["Button.base"]["hover"].pitch, [1.2, 1.2]);
    }

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
                    a.and_then(|a| a.strip_prefix('"')?.strip_suffix('"')).map(str::to_owned)
                };
                let (Some(component), Some(slot)) = (literal(parts.next()), literal(parts.next()))
                else {
                    continue;
                };
                let base = format!("{component}.{slot}");
                let variants = parts.next().unwrap_or("").split('"').skip(1).step_by(2);
                for key in std::iter::once(base.clone()).chain(variants.map(|v| format!("{base}.{v}"))) {
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
            let keys = keys.chain(layer.sounds.keys().filter(|k| !k.is_empty()).map(|k| ("sounds", k)));
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
