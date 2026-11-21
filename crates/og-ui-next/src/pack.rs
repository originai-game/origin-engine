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
    pub variants: BTreeMap<String, String>,
    pub fonts: Vec<Face>,
    pub icons: BTreeMap<String, String>,
    /// The pack-wide sounds under `""`, a slot's under its key.
    pub sounds: BTreeMap<String, Sounds>,
}

/// The layers merged: what the cook compiles and the runtime loads.
#[derive(Clone, Debug)]
pub struct Pack {
    pub scheme: Option<Scheme>,
    themes: [Map<String, Value>; 2],
    variants: BTreeMap<String, String>,
    pub fonts: Vec<Face>,
    pub icons: BTreeMap<String, String>,
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
                            return Err(format!("seeds.{name}: not a seed; one of {}", SEEDS.join(", ")));
                        }
                        let at = format!("seeds.{name}");
                        let side = |s| match split(v, s, &at)? {
                            Value::String(c) => Ok(c),
                            _ => Err(format!("{at}: a CSS colour")),
                        };
                        layer.seeds.push((name.clone(), [side(Scheme::Light)?, side(Scheme::Dark)?]));
                    }
                }
                "theme" => {
                    for (section, _) in object(v, "theme")? {
                        if !THEME_KEYS.contains(&section.as_str()) {
                            return Err(format!("theme.{section}: not a theme key; one of {}", THEME_KEYS.join(", ")));
                        }
                    }
                    for (i, s) in Scheme::ALL.into_iter().enumerate() {
                        let Value::Object(m) = split(v, s, "theme")? else { unreachable!() };
                        layer.theme[i] = m;
                    }
                }
                "variants" => {
                    for (k, v) in object(v, "variants")? {
                        if !variant_key(k) {
                            return Err(format!("variants.{k}: a key is Component.slot or Component.slot.variant (PascalCase, then lower-case names)"));
                        }
                        let class = v.as_str().filter(|c| !c.trim().is_empty());
                        let class = class.ok_or_else(|| format!("variants.{k}: a class string"))?;
                        layer.variants.insert(k.clone(), class.trim().to_owned());
                    }
                }
                "fonts" => {
                    let Value::Array(faces) = v else {
                        return Err("fonts: an array of faces".into());
                    };
                    for (i, f) in faces.iter().enumerate() {
                        layer.fonts.push(face(f).map_err(|e| format!("fonts[{i}]: {e}"))?);
                    }
                }
                "icons" => {
                    for (name, src) in object(v, "icons")? {
                        let src = src.as_str().ok_or_else(|| format!("icons.{name}: a path"))?;
                        layer.icons.insert(name.clone(), src.to_owned());
                    }
                }
                "sounds" => {
                    for (k, v) in object(v, "sounds")? {
                        if SOUND_EVENTS.contains(&k.as_str()) {
                            let s = sound(v).map_err(|e| format!("sounds.{k}: {e}"))?;
                            layer.sounds.entry(String::new()).or_default().insert(k.clone(), s);
                        } else if variant_key(k) {
                            let slot = layer.sounds.entry(k.clone()).or_default();
                            for (event, v) in object(v, &format!("sounds.{k}"))? {
                                if !SOUND_EVENTS.contains(&event.as_str()) {
                                    return Err(format!("sounds.{k}.{event}: one of {}", SOUND_EVENTS.join(", ")));
                                }
                                let s = sound(v).map_err(|e| format!("sounds.{k}.{event}: {e}"))?;
                                slot.insert(event.clone(), s);
                            }
                        } else {
                            return Err(format!("sounds.{k}: an event ({}) or Component.slot[.variant]", SOUND_EVENTS.join(", ")));
                        }
                    }
                }
                other => {
                    return Err(format!(
                        "{other}: not a pack key; one of about, scheme, seeds, theme, variants, fonts, icons, sounds"
                    ));
                }
            }
        }
        Ok(layer)
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
            variants: BTreeMap::new(),
            fonts: Vec::new(),
            icons: BTreeMap::new(),
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
            pack.variants.extend(layer.variants.clone());
            pack.fonts.extend(layer.fonts.iter().cloned());
            pack.icons.extend(layer.icons.clone());
            for (scope, sounds) in &layer.sounds {
                pack.sounds.entry(scope.clone()).or_default().extend(sounds.clone());
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

    /// The theme in `scheme`, as a JSON object.
    pub fn theme(&self, scheme: Scheme) -> &Map<String, Value> {
        &self.themes[scheme as usize]
    }

    /// The class strings each `Component.slot[.variant]` appends.
    pub fn variants(&self) -> &BTreeMap<String, String> {
        &self.variants
    }
}

fn object<'a>(v: &'a Value, at: &str) -> Result<&'a Map<String, Value>, String> {
    v.as_object().ok_or_else(|| format!("{at}: an object"))
}

/// Whether `v` is a scheme pair: exactly `light` and `dark`.
fn pair(m: &Map<String, Value>) -> Option<(&Value, &Value)> {
    (m.len() == 2).then(|| Some((m.get("light")?, m.get("dark")?))).flatten()
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
        return Err(format!("{at}: a scheme pair has exactly `light` and `dark`"));
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
            && p.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    };
    matches!(parts.len(), 2 | 3)
        && parts[0].starts_with(|c: char| c.is_ascii_uppercase())
        && parts[0].chars().all(|c| c.is_ascii_alphanumeric())
        && parts[1..].iter().all(|p| name(p))
}

fn face(v: &Value) -> Result<Face, String> {
    let m = object(v, "a face")?;
    let text = |k: &str| m.get(k).and_then(Value::as_str).map(str::to_owned);
    for k in m.keys() {
        if !["family", "src", "weight", "style", "unicodeRange"].contains(&k.as_str()) {
            return Err(format!("{k}: one of family, src, weight, style, unicodeRange"));
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
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).map(str::to_owned).collect(),
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
    Ok(Sound { clips, volume: range("volume", 1.0)?, pitch: range("pitch", 1.0)? })
}

#[cfg(test)]
mod tests {
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
            assert!(c.unknown(classes).is_empty(), "{s:?}: {:?}", c.unknown(classes));
        }
    }

    #[test]
    fn seeds_derive_the_palette_and_written_colours_lie_over_it() {
        let base = Pack::new(&[Layer::default_pack()]);
        let p = pack(r##"{ "seeds": { "primary": "#c8102e" }, "theme": { "colors": { "bg": "#010203" } } }"##);
        for s in Scheme::ALL {
            assert_ne!(color(&p, s, "accent"), color(&base, s, "accent"), "{s:?}");
            assert_eq!(color(&p, s, "bg"), "#010203");
        }
        // A red seed gives a red accent, in both schemes.
        for s in Scheme::ALL {
            let c = og_css::parse_color(&color(&p, s, "accent")).unwrap();
            assert!(c.rgb[0] > c.rgb[1] && c.rgb[0] > c.rgb[2], "{s:?}: {:?}", c.rgb);
        }
    }

    #[test]
    fn every_form_error_names_where_it_is() {
        for (text, wants) in [
            (r##"{ "theme": { "colours": {} } }"##, "theme.colours"),
            (r##"{ "theme": { "colors": { "a": { "light": "#000" } } } }"##, "theme.colors.a"),
            (r##"{ "variants": { "panel.frame": "p-2" } }"##, "variants.panel.frame"),
            (r##"{ "variants": { "Panel.Frame": "p-2" } }"##, "variants.Panel.Frame"),
            (r##"{ "seeds": { "brand": "#000" } }"##, "seeds.brand"),
            (r##"{ "sounds": { "Button.base": { "tap": { "clips": ["a"] } } } }"##, "sounds.Button.base.tap"),
            (r##"{ "fonts": [{ "src": "a.ttf" }] }"##, "fonts[0]: family"),
            (r##"{ "surfaces": {} }"##, "surfaces"),
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
}
