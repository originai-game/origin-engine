//! Looks (`.ogt` of `type look`): a frame's art direction as one
//! git-diffable file. A look names the engine resources it sets (the IDL's
//! `RenderSettings`, `PostEffects`, `Stylise`, `MaterialStyle`, `Fog`, `Sky`
//! and `Environment`) with their fields by reflected name, the UI style, and
//! the antialiasing it wants, and may build on another look:
//!
//! ```text
//! type look
//! version 1
//! ---
//! name "Dusk ink"
//! extends "anime-cel-3d"        # a built-in preset (templates/looks/<slug>.ogt),
//!                               # the game's own assets/looks/<slug>.ogt first,
//!                               # or another look's asset id ("looks/base.ogt")
//! ui "paper"                   # the UI style applied with the look: a built-in
//!                               # (templates/styles/<slug>), the game's own
//!                               # styles/<slug>/style.ogt first, or an asset id
//! antialias off                 # tier, off, msaa4 or taa
//! RenderSettings {
//!   toneMapping agx
//!   exposure 1.2
//! }
//! PostEffects {
//!   outline true
//!   outlineColor "#1b1420"      # a colour as three linear numbers or an sRGB hex
//!   lut "luts/dusk.cube"        # a handle field takes the asset's id
//! }
//! Stylise {
//!   posterize 6
//!   dither bayer4
//!   palette "palettes/dusk.hex"
//! }
//! MaterialStyle {
//!   toon true
//! }
//! ```
//!
//! This module reads and writes the text ([`LookFile`]) and merges a look
//! over the one it extends; the cook (`cook::look`) resolves `extends`,
//! checks every field against the resources' reflected schema (the IDL's
//! records), turns hex colours into linear numbers, and writes the flattened
//! result, so the cooked blob ([`crate::format::Look`]) is the whole look
//! and the runtime never resolves anything but the assets it names.

use og_core::ogt::{Block, Document, Entry, Value};

/// The `.ogt` type of a look.
pub const LOOK_TYPE: &str = "look";

/// The resources a look may set, by IDL name (each registers as `og.<name>`).
pub const LOOK_RESOURCES: [&str; 7] = [
    "RenderSettings",
    "PostEffects",
    "Stylise",
    "MaterialStyle",
    "Fog",
    "Sky",
    "Environment",
];

/// Fields game code animates tick by tick: a look never sets them, and
/// applying one keeps their current values.
pub const ANIMATED_FIELDS: [(&str, &str); 2] =
    [("PostEffects", "flash"), ("PostEffects", "shakeTrauma")];

/// The values of `antialias` (the IDL's `render.Antialias`).
pub const ANTIALIAS: [&str; 4] = ["tier", "off", "msaa4", "taa"];

/// The longest `extends` chain the cook follows.
pub const MAX_EXTENDS: usize = 8;

/// The top-level keys of a look file.
const TOP: [&str; 4] = ["name", "extends", "ui", "antialias"];

/// One resource block: its IDL name and its fields in file order, each with
/// its values as written (numbers, `true`/`false`, a variant word, or a
/// string: an asset id or an sRGB hex colour).
#[derive(Clone, Debug, PartialEq)]
pub struct LookBlock {
    pub resource: String,
    pub fields: Vec<(String, Vec<Value>, usize)>,
}

impl LookBlock {
    pub fn get(&self, field: &str) -> Option<&[Value]> {
        self.fields
            .iter()
            .find(|(f, _, _)| f == field)
            .map(|(_, v, _)| v.as_slice())
    }
}

/// A look file as written (or as cooked: flattened, `extends` then lists
/// the whole chain, nearest first).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LookFile {
    pub name: Option<String>,
    pub extends: Vec<String>,
    pub ui: Option<String>,
    pub antialias: Option<String>,
    pub blocks: Vec<LookBlock>,
}

fn one_string(e: &Entry) -> Result<String, String> {
    match e.values.as_slice() {
        [Value::String(s)] => Ok(s.clone()),
        _ => Err(format!("line {}: '{}' takes one string", e.line, e.key)),
    }
}

impl LookFile {
    /// Parses a look's text. Checks the grammar only: which resources and
    /// keys exist; the fields' names and values are the cook's to check
    /// against the reflected schema.
    pub fn parse(text: &str) -> Result<LookFile, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != LOOK_TYPE {
            return Err(format!("type {}: a look is `type look`", doc.ty));
        }
        if doc.version != 1 {
            return Err(format!("version {}: looks are version 1", doc.version));
        }
        let mut look = LookFile::default();
        let mut seen_top: Vec<&str> = Vec::new();
        for e in &doc.body.entries {
            if let Some(block) = &e.block {
                if !LOOK_RESOURCES.contains(&e.key.as_str()) {
                    return Err(format!(
                        "line {}: '{}' is not a resource a look sets: {}",
                        e.line,
                        e.key,
                        LOOK_RESOURCES.join(", ")
                    ));
                }
                if !e.values.is_empty() {
                    return Err(format!("line {}: '{} {{' takes no values", e.line, e.key));
                }
                if look.blocks.iter().any(|b| b.resource == e.key) {
                    return Err(format!("line {}: a second '{}' block", e.line, e.key));
                }
                let mut fields: Vec<(String, Vec<Value>, usize)> = Vec::new();
                for f in &block.entries {
                    if f.block.is_some() {
                        return Err(format!(
                            "line {}: {}.{} opens a block; a field takes values only",
                            f.line, e.key, f.key
                        ));
                    }
                    // A key alone is an empty list; the cook refuses it for
                    // any other field (`cook::look`, which reads the records).
                    if fields.iter().any(|(n, _, _)| n == &f.key) {
                        return Err(format!("line {}: {}.{} is set twice", f.line, e.key, f.key));
                    }
                    if ANIMATED_FIELDS.contains(&(e.key.as_str(), f.key.as_str())) {
                        return Err(format!(
                            "line {}: {}.{} is animated by game code tick by tick; a look does not set it",
                            f.line, e.key, f.key
                        ));
                    }
                    fields.push((f.key.clone(), f.values.clone(), f.line));
                }
                look.blocks.push(LookBlock {
                    resource: e.key.clone(),
                    fields,
                });
                continue;
            }
            if e.key == "skin" {
                return Err(format!(
                    "line {}: 'skin' is now 'ui': `ui \"<slug>\"` names a built-in style (pixel, cartoon, fantasy, scifi, paper, neon) or the game's own styles/<slug>/style.ogt",
                    e.line
                ));
            }
            if !TOP.contains(&e.key.as_str()) {
                return Err(format!(
                    "line {}: '{}' is not a look entry: {}, or a resource block ({})",
                    e.line,
                    e.key,
                    TOP.join(", "),
                    LOOK_RESOURCES.join(", ")
                ));
            }
            if seen_top.contains(&e.key.as_str()) {
                return Err(format!("line {}: '{}' is given twice", e.line, e.key));
            }
            seen_top.push(TOP.iter().find(|k| **k == e.key).expect("a top key"));
            match e.key.as_str() {
                "name" => look.name = Some(one_string(e)?),
                "ui" => look.ui = Some(one_string(e)?),
                "extends" => {
                    look.extends = e
                        .values
                        .iter()
                        .map(|v| match v {
                            Value::String(s) if !s.is_empty() => Ok(s.clone()),
                            _ => Err(format!(
                                "line {}: 'extends' takes a preset's slug or a look's asset id, as a string",
                                e.line
                            )),
                        })
                        .collect::<Result<_, _>>()?;
                    if look.extends.is_empty() {
                        return Err(format!("line {}: 'extends' names no look", e.line));
                    }
                }
                _ => match e.values.as_slice() {
                    [Value::Word(w)] if ANTIALIAS.contains(&w.as_str()) => {
                        look.antialias = Some(w.clone())
                    }
                    _ => {
                        return Err(format!(
                            "line {}: 'antialias' is one of {}",
                            e.line,
                            ANTIALIAS.join(", ")
                        ));
                    }
                },
            }
        }
        Ok(look)
    }

    /// The canonical text: top-level keys, then the blocks in
    /// [`LOOK_RESOURCES`] order, fields as given.
    pub fn to_text(&self) -> String {
        let mut doc = Document::new(LOOK_TYPE, 1);
        if let Some(n) = &self.name {
            doc.body
                .push(Entry::new("name").with(Value::String(n.clone())));
        }
        if !self.extends.is_empty() {
            let mut e = Entry::new("extends");
            e.values = self.extends.iter().cloned().map(Value::String).collect();
            doc.body.push(e);
        }
        if let Some(s) = &self.ui {
            doc.body
                .push(Entry::new("ui").with(Value::String(s.clone())));
        }
        if let Some(a) = &self.antialias {
            doc.body
                .push(Entry::new("antialias").with(Value::Word(a.clone())));
        }
        for r in LOOK_RESOURCES {
            if let Some(b) = self.blocks.iter().find(|b| b.resource == r) {
                let mut inner = Block::new();
                for (f, v, _) in &b.fields {
                    let mut e = Entry::new(f.clone());
                    e.values = v.clone();
                    inner.push(e);
                }
                doc.body.push(Entry::new(r).with_block(inner));
            }
        }
        doc.to_text().expect("a parsed look writes back")
    }

    /// This look laid over `base` (the look it extends): the name is this
    /// look's own (a look is not called after its base), its ui style and
    /// antialias win where given, a resource either names takes both
    /// blocks' fields with this look's winning field by field, and `extends`
    /// becomes `base_id` followed by `base`'s own chain.
    pub fn over(&self, base: &LookFile, base_id: &str) -> LookFile {
        let mut blocks: Vec<LookBlock> = base.blocks.clone();
        for b in &self.blocks {
            match blocks.iter_mut().find(|x| x.resource == b.resource) {
                Some(x) => {
                    for (f, v, line) in &b.fields {
                        match x.fields.iter_mut().find(|(n, _, _)| n == f) {
                            Some(slot) => *slot = (f.clone(), v.clone(), *line),
                            None => x.fields.push((f.clone(), v.clone(), *line)),
                        }
                    }
                }
                None => blocks.push(b.clone()),
            }
        }
        let mut extends = vec![base_id.to_string()];
        extends.extend(base.extends.iter().cloned());
        LookFile {
            name: self.name.clone(),
            extends,
            ui: self.ui.clone().or_else(|| base.ui.clone()),
            antialias: self.antialias.clone().or_else(|| base.antialias.clone()),
            blocks,
        }
    }

    /// The resources the look sets, in [`LOOK_RESOURCES`] order.
    pub fn resources(&self) -> Vec<&'static str> {
        LOOK_RESOURCES
            .into_iter()
            .filter(|r| self.blocks.iter().any(|b| b.resource == *r))
            .collect()
    }
}

/// An sRGB `#rrggbb` (or `rrggbb`) colour as linear RGB, as three.js
/// `Color.setHex` gives under its colour management.
pub fn hex_to_linear(hex: &str) -> Option<[f64; 3]> {
    let h = hex.strip_prefix('#').unwrap_or(hex);
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let ch = |i: usize| {
        let c = f64::from(u8::from_str_radix(&h[i..i + 2], 16).ok()?) / 255.0;
        Some(if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        })
    };
    Some([ch(0)?, ch(2)?, ch(4)?])
}

/// The colours of a palette file: `.hex` (one `rrggbb` per line, `#`
/// optional, blank lines and `;` comments skipped) or GIMP `.gpl` (a
/// `GIMP Palette` header, `Name:`/`Columns:` lines and `#` comments, then
/// `r g b [name]` per colour). sRGB bytes in file order, at most 256.
pub fn parse_palette(name: &str, text: &str) -> Result<Vec<[u8; 3]>, String> {
    let gpl = name.to_ascii_lowercase().ends_with(".gpl");
    let mut out = Vec::new();
    let mut lines = text.lines().enumerate().peekable();
    if gpl {
        match lines.next() {
            Some((_, l)) if l.trim() == "GIMP Palette" => {}
            _ => return Err("a .gpl palette starts with the line `GIMP Palette`".into()),
        }
    }
    for (i, raw) in lines {
        let line = raw.trim();
        let n = i + 1;
        if line.is_empty() || line.starts_with(';') || (gpl && line.starts_with('#')) {
            continue;
        }
        if gpl {
            if line.starts_with("Name:") || line.starts_with("Columns:") {
                continue;
            }
            let rgb: Vec<u8> = line
                .split_whitespace()
                .take(3)
                .map(|t| t.parse::<u8>())
                .collect::<Result<_, _>>()
                .map_err(|_| format!("line {n}: a colour is `r g b [name]` with 0-255 channels"))?;
            if rgb.len() != 3 {
                return Err(format!(
                    "line {n}: a colour is `r g b [name]` with 0-255 channels"
                ));
            }
            out.push([rgb[0], rgb[1], rgb[2]]);
        } else {
            let h = line.strip_prefix('#').unwrap_or(line);
            let h = h.split_whitespace().next().unwrap_or("");
            let byte = |k: usize| u8::from_str_radix(h.get(k..k + 2).unwrap_or("x"), 16);
            match (h.len(), byte(0), byte(2), byte(4)) {
                (6, Ok(r), Ok(g), Ok(b)) => out.push([r, g, b]),
                _ => return Err(format!("line {n}: '{line}' is not an rrggbb colour")),
            }
        }
        if out.len() > 256 {
            return Err("a palette holds at most 256 colours".into());
        }
    }
    if out.is_empty() {
        return Err("the palette holds no colours".into());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOOK: &str = "type look\nversion 1\n---\nname \"Dusk\"\nextends \"anime-cel-3d\"\nantialias off\nStylise {\n  posterize 6\n  dither bayer4\n}\nPostEffects {\n  outline true\n  outlineColor \"#1b1420\"\n}\n";

    #[test]
    fn parses_and_writes_canonical_text() {
        let l = LookFile::parse(LOOK).unwrap();
        assert_eq!(l.name.as_deref(), Some("Dusk"));
        assert_eq!(l.extends, vec!["anime-cel-3d"]);
        assert_eq!(l.antialias.as_deref(), Some("off"));
        assert_eq!(l.resources(), vec!["PostEffects", "Stylise"]);
        let text = l.to_text();
        // Blocks in resource order, whatever the file's order.
        assert!(text.find("PostEffects").unwrap() < text.find("Stylise").unwrap());
        assert_eq!(LookFile::parse(&text).unwrap().to_text(), text);
    }

    #[test]
    fn merges_over_its_base() {
        let base = LookFile::parse(
            "type look\nversion 1\n---\nname \"Base\"\nui \"styles/a/style.ogt\"\nStylise {\n  posterize 4\n  halftone 0.5\n}\nFog {\n  enabled true\n}\n",
        )
        .unwrap();
        let l = LookFile::parse(LOOK).unwrap().over(&base, "looks/base.ogt");
        assert_eq!(l.name.as_deref(), Some("Dusk"));
        assert_eq!(LookFile::default().over(&base, "b").name, None);
        assert_eq!(l.ui.as_deref(), Some("styles/a/style.ogt"));
        assert_eq!(l.extends, vec!["looks/base.ogt"]);
        let s = l.blocks.iter().find(|b| b.resource == "Stylise").unwrap();
        assert_eq!(
            s.get("posterize").unwrap()[0].as_number().unwrap().text(),
            "6"
        );
        assert!(s.get("halftone").is_some() && s.get("dither").is_some());
        assert_eq!(l.resources(), vec!["PostEffects", "Stylise", "Fog"]);
    }

    #[test]
    fn refuses_what_is_not_a_look() {
        let bad = |body: &str| {
            LookFile::parse(&format!("type look\nversion 1\n---\n{body}")).unwrap_err()
        };
        assert!(bad("Camera {\n  fov 30\n}\n").contains("not a resource a look sets"));
        assert!(bad("PostEffects {\n  flash 1\n}\n").contains("animated by game code"));
        assert!(bad("antialias fxaa\n").contains("one of tier, off, msaa4, taa"));
        assert!(bad("palette \"p.hex\"\n").contains("not a look entry"));
        assert!(bad("Stylise {\n  posterize 4\n  posterize 5\n}\n").contains("set twice"));
    }

    #[test]
    fn hex_colours_are_linear() {
        let [r, g, b] = hex_to_linear("#ffffff").unwrap();
        assert!((r - 1.0).abs() < 1e-12 && (g - 1.0).abs() < 1e-12 && (b - 1.0).abs() < 1e-12);
        let [m, _, _] = hex_to_linear("808080").unwrap();
        assert!((m - 0.2158605).abs() < 1e-6);
        assert!(hex_to_linear("#12345").is_none());
    }

    #[test]
    fn reads_hex_and_gpl_palettes() {
        let hex = parse_palette("p.hex", "; a palette\n1a1c2c\n#5d275d\n\nb13e53\n").unwrap();
        assert_eq!(
            hex,
            vec![[0x1a, 0x1c, 0x2c], [0x5d, 0x27, 0x5d], [0xb1, 0x3e, 0x53]]
        );
        let gpl = parse_palette(
            "p.gpl",
            "GIMP Palette\nName: Dusk\nColumns: 4\n# comment\n 26  28  44\tInk\n255 255 255 Paper\n",
        )
        .unwrap();
        assert_eq!(gpl, vec![[26, 28, 44], [255, 255, 255]]);
        assert!(parse_palette("p.gpl", "1a1c2c\n").is_err());
        assert!(
            parse_palette("p.hex", "zz0000\n")
                .unwrap_err()
                .contains("line 1")
        );
        assert!(
            parse_palette("p.hex", "\n")
                .unwrap_err()
                .contains("no colours")
        );
    }
}
