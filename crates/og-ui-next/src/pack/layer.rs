//! Reading, validating, and serialising individual `pack.json` layers.

use super::*;

/// One `pack.json` as read: nothing merged yet.
#[derive(Clone, Debug, Default)]
pub struct Layer {
    /// The scheme the pack is drawn in unless the game asks for another.
    pub scheme: Option<Scheme>,
    /// Seed overrides, by name, per scheme.
    pub(super) seeds: Vec<(String, [String; 2])>,
    /// The theme per scheme ([`Scheme::ALL`] order).
    pub(super) theme: [Map<String, Value>; 2],
    /// What the theme takes in high contrast (`prefers-contrast: more`),
    /// per scheme: entries over the theme's, as `theme` writes them.
    pub(super) contrast: [Map<String, Value>; 2],
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
    /// The pack-wide haptics under `""`, a slot's under its key.
    pub haptics: BTreeMap<String, Haptics>,
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
                "haptics" => {
                    let preset = |v: &Value, at: &str| -> Result<String, String> {
                        let name = v
                            .as_str()
                            .ok_or_else(|| format!("{at}: one of {}", HAPTIC_PRESETS.join(", ")))?;
                        if !HAPTIC_PRESETS.contains(&name) {
                            return Err(format!("{at}: one of {}", HAPTIC_PRESETS.join(", ")));
                        }
                        Ok(name.to_owned())
                    };
                    for (k, v) in object(v, "haptics")? {
                        if SOUND_EVENTS.contains(&k.as_str()) {
                            let h = preset(v, &format!("haptics.{k}"))?;
                            layer
                                .haptics
                                .entry(String::new())
                                .or_default()
                                .insert(k.clone(), h);
                        } else if variant_key(k) {
                            let slot = layer.haptics.entry(k.clone()).or_default();
                            for (event, v) in object(v, &format!("haptics.{k}"))? {
                                if !SOUND_EVENTS.contains(&event.as_str()) {
                                    return Err(format!(
                                        "haptics.{k}.{event}: one of {}",
                                        SOUND_EVENTS.join(", ")
                                    ));
                                }
                                let h = preset(v, &format!("haptics.{k}.{event}"))?;
                                slot.insert(event.clone(), h);
                            }
                        } else {
                            return Err(format!(
                                "haptics.{k}: an event ({}) or Component.slot[.variant]",
                                SOUND_EVENTS.join(", ")
                            ));
                        }
                    }
                }
                other => {
                    return Err(format!(
                        "{other}: not a pack key; one of about, scheme, seeds, theme, contrast, variants, images, fonts, icons, iconSet, sounds, haptics"
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
        if !self.haptics.is_empty() {
            let mut m = Map::new();
            for (scope, haptics) in &self.haptics {
                let events: Map<String, Value> = haptics
                    .iter()
                    .map(|(e, p)| (e.clone(), Value::String(p.clone())))
                    .collect();
                if scope.is_empty() {
                    m.extend(events);
                } else {
                    m.insert(scope.clone(), Value::Object(events));
                }
            }
            root.insert("haptics".into(), Value::Object(m));
        }
        Value::Object(root).to_string()
    }
}

/// `over`'s sections over `have`'s, entry by entry.
pub(super) fn merge(have: &mut Map<String, Value>, over: &Map<String, Value>) {
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
pub(super) fn pair(m: &Map<String, Value>) -> Option<(&Value, &Value)> {
    (m.len() == 2)
        .then(|| Some((m.get("light")?, m.get("dark")?)))
        .flatten()
}

/// `v` with every scheme pair replaced by its `scheme` side.
pub(super) fn split(v: &Value, scheme: Scheme, at: &str) -> Result<Value, String> {
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
pub(super) fn variant_key(k: &str) -> bool {
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

pub(super) fn kebab(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|w| {
            !w.is_empty()
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// Four insets: one number for all, or `[top, right, bottom, left]`.
pub(super) fn insets(v: &Value, what: &str) -> Result<[f32; 4], String> {
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

pub(super) fn nine(v: &Value) -> Result<NineSlice, String> {
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

pub(super) fn sound(v: &Value) -> Result<Sound, String> {
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
