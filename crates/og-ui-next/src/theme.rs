//! Theme recipes (CONVENTIONS.md §Theme recipes): checked `file.theme` picks one
//! value per dimension (`templates/themes/dimensions/<dimension>/<value>.json`)
//! and may extend another theme, mix in partial recipes and lay a pack over
//! the result. [`resolve`] reads a recipe and everything it inherits into
//! one recipe; [`compile`] turns that into a pack layer and the art spec the
//! cook draws (og-cook's `cook/ui_art.rs`).
//!
//! The taste is in the dimension files: this module only merges and maps.
//! The order is the recipe plan's (§3): the parent (`extends`, resolved
//! first), each `with` mix-in in order, then the recipe's own keys; objects
//! merge key by key, anything else (arrays included) is replaced. `pack`, a
//! structured pack patch lies over the compiled layer and
//! the generated art, so a hand-drawn image replaces its generated one by
//! name and every other piece still comes from the generator.

use crate::pack::Layer;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// The pieces of art an ornament may be drawn on.
pub const ORNAMENT_SLOTS: [&str; 5] = ["button", "panel", "plate", "slot", "chip"];

/// Where a recipe, a mix-in or a dimension value is read from, relative to
/// `templates/` (the engine's themes) or a game's styles.
pub enum Source<'a> {
    /// A theme's canonical identity, project-owned or explicitly `@themes/…`.
    Theme(&'a str),
    /// A mix-in by name (`templates/themes/mixins/<name>.json`).
    Mixin(&'a str),
    /// One dimension's value (`templates/themes/dimensions/<dim>/<value>.json`).
    Dimension(&'a str, &'a str),
}

impl Source<'_> {
    /// The file it names, relative to `templates/`.
    pub fn path(&self) -> String {
        match self {
            Source::Theme(n) => format!("{n}.ts"),
            Source::Mixin(n) => format!("themes/mixins/{n}.ts"),
            Source::Dimension(d, v) => format!("themes/dimensions/{d}/{v}.json"),
        }
    }
}

/// Reads checked recipe fields or engine-owned dimension data, never author text.
pub type Read<'a> = dyn FnMut(&Source) -> Result<Map<String, Value>, String> + 'a;

fn load(read: &mut Read, src: &Source) -> Result<Map<String, Value>, String> {
    read(src)
}

/// `over` merged into `have`: objects key by key, anything else replaced.
fn deep(have: &mut Value, over: &Value) {
    match (have, over) {
        (Value::Object(h), Value::Object(o)) => {
            for (k, v) in o {
                match h.get_mut(k) {
                    Some(slot) => deep(slot, v),
                    None => {
                        h.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (h, o) => *h = o.clone(),
    }
}

/// Checks a recipe's (or a mix-in's) keys and forms, and writes an
/// ornament named once (`"ornament": "studs"`) as one per piece, so a later
/// layer can change one piece's.
fn ornaments(mut r: Map<String, Value>) -> Map<String, Value> {
    if let Some(Value::String(o)) = r.get("ornament") {
        let all: Map<String, Value> = ORNAMENT_SLOTS
            .iter()
            .map(|s| (s.to_string(), json!(o)))
            .collect();
        r.insert("ornament".into(), Value::Object(all));
    }
    r
}

/// The checked recipe `top`, identified by `at`, with
/// everything it inherits merged in: its parent's resolved recipe, then
/// each mix-in in order, then its own keys. `extends` and `with` are gone
/// from the result. A theme that extends itself through any chain is an
/// error naming the chain.
pub fn resolve(
    top: Map<String, Value>,
    at: &str,
    read: &mut Read,
) -> Result<Map<String, Value>, String> {
    resolve_with(top, at, read, &mut vec![at.to_owned()])
}

fn resolve_with(
    own: Map<String, Value>,
    at: &str,
    read: &mut Read,
    chain: &mut Vec<String>,
) -> Result<Map<String, Value>, String> {
    let mut own = ornaments(own);
    let mut out = Value::Object(Map::new());
    if let Some(parent) = own.remove("extends") {
        let name = parent
            .as_str()
            .or_else(|| parent.get("$ogRef")?.as_str())
            .ok_or("invalid checked theme base")?
            .to_owned();
        if chain.contains(&name) {
            chain.push(name);
            return Err(format!("{at}: `extends` loops: {}", chain.join(" → ")));
        }
        chain.push(name.clone());
        let src = Source::Theme(&name);
        let path = src.path();
        let p = load(read, &src)?;
        out = Value::Object(resolve_with(p, &path, read, chain)?);
        chain.pop();
    }
    if let Some(Value::Array(with)) = own.remove("with") {
        for m in with {
            let src = Source::Mixin(m.as_str().unwrap_or_default());
            let path = src.path();
            let mix = ornaments(load(read, &src)?);
            if mix.contains_key("extends") || mix.contains_key("with") {
                return Err(format!("{path}: a mix-in neither extends nor mixes in"));
            }
            deep(&mut out, &Value::Object(mix));
        }
    }
    deep(&mut out, &Value::Object(own));
    let Value::Object(out) = out else {
        unreachable!()
    };
    Ok(out)
}

/// A compiled theme: the pack layer its dimensions make, the art spec the
/// cook draws, and the recipe's `pack`, which lies over both.
#[derive(Clone, Debug)]
pub struct Theme {
    /// The structured fields the dimensions compile to, without art.
    pub pack: Map<String, Value>,
    /// The art spec (`templates/tools/theme-art.ts`'s input): the palette,
    /// material, shape and ornaments.
    pub art: Value,
    /// The recipe's own pack, laid last.
    pub over: Map<String, Value>,
    /// The `effects` value (its textures, text glow and shadow), for the
    /// cook to carry.
    pub effects: Option<Map<String, Value>>,
    /// The `density` value (`compact`, `comfortable` or `touch`).
    pub density: Option<String>,
}

impl Theme {
    /// Ground registration from the drawn art (plan §4): `faces` are each
    /// piece's ground as drawn, by piece name (`button`, `button-hover`,
    /// `panel` …): the 10th and 90th percentile of its centre's opaque
    /// pixels, a gloss or a grain included, where the recipe's faces are
    /// only the colours the face was painted from. Each piece's ink
    /// (`art-panel-fg`, for a game that draws the piece) is moved until it
    /// holds 4.5:1 on the piece as drawn. The design's inks are not: no
    /// built-in component stands on the art. Returns what still misses its
    /// ratio, for the cook to report.
    pub fn register(&mut self, faces: &BTreeMap<String, Vec<String>>) -> Vec<String> {
        let mut missed = Vec::new();
        let Some(colors) = self
            .pack
            .get_mut("theme")
            .and_then(|t| t.get_mut("colors"))
            .and_then(Value::as_object_mut)
        else {
            return missed;
        };
        for (_, image) in ART_SLOTS {
            let drawn: Vec<String> = faces
                .iter()
                .filter(|(n, _)| {
                    *n == image
                        || n.strip_prefix(image)
                            .is_some_and(|s| s == "-hover" || s == "-pressed")
                })
                .flat_map(|(_, f)| f.iter().cloned())
                .collect();
            if drawn.is_empty() {
                continue;
            }
            let name = format!("art-{image}-fg");
            if let Some(ink) = colors.get(&name).and_then(Value::as_str).map(str::to_owned) {
                let worst = drawn
                    .iter()
                    .map(|g| contrast(&ink, g))
                    .fold(f32::MAX, f32::min);
                if worst < 4.5 {
                    let hue = lch(&ink).unwrap_or(Lch {
                        l: 0.5,
                        c: 0.0,
                        h: 0.0,
                    });
                    let (fixed, r) = solve(hue, &drawn, 4.5);
                    if r < 4.5 {
                        missed.push(format!("{name} {fixed}: {r:.2}:1 on the drawn {image}"));
                    }
                    colors.insert(name, json!(fixed));
                }
            }
        }
        missed
    }

    /// The layer, with `images` (the drawn art's pack entries, by name)
    /// under the recipe's own pack: what the cook lays over the default.
    pub fn layer(&self, images: Map<String, Value>) -> Result<Layer, String> {
        let mut pack = Value::Object(self.pack.clone());
        if !images.is_empty() {
            deep(&mut pack, &json!({ "images": images }));
        }
        deep(&mut pack, &Value::Object(self.over.clone()));
        let input: crate::pack::input::PackInput =
            serde_json::from_value(pack).map_err(|e| format!("the compiled theme: {e}"))?;
        Layer::from_input(input, Clone::clone)
    }
}

mod color;
mod compile;
pub use color::{INKS, Lch, contrast, hex, lch, solve};
use compile::ART_SLOTS;
pub use compile::compile;
