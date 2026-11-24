//! Theme recipes (CONVENTIONS.md §Theme recipes): a `theme.json` picks one
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
//! pack.json written as a pack writes it, lies over the compiled layer and
//! the generated art, so a hand-drawn image replaces its generated one by
//! name and every other piece still comes from the generator.

use crate::pack::{Layer, Scheme};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// The keys a recipe may set.
pub const RECIPE_KEYS: [&str; 14] = [
    "about", "extends", "with", "color", "material", "shape", "type", "motion", "ornament",
    "effects", "sound", "icons", "density", "pack",
];

/// The keys of a recipe's `color`.
pub const COLOR_KEYS: [&str; 6] = ["source", "accent", "neutral", "danger", "variant", "scheme"];

/// The pieces of art an ornament may be drawn on.
pub const ORNAMENT_SLOTS: [&str; 5] = ["button", "panel", "plate", "slot", "chip"];

/// Where a recipe, a mix-in or a dimension value is read from, relative to
/// `templates/` (the engine's themes) or a game's styles.
pub enum Source<'a> {
    /// A theme by name (`@themes/tactical` or `tactical`): its `theme.json`.
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
            Source::Theme(n) => format!("themes/{}/theme.json", theme_name(n)),
            Source::Mixin(n) => format!("themes/mixins/{n}.json"),
            Source::Dimension(d, v) => format!("themes/dimensions/{d}/{v}.json"),
        }
    }
}

/// `@themes/tactical` and `tactical` both name `tactical`.
pub fn theme_name(name: &str) -> &str {
    name.strip_prefix("@themes/").unwrap_or(name)
}

/// Reads a source's JSON text, or says why it cannot.
pub type Read<'a> = dyn FnMut(&Source) -> Result<String, String> + 'a;

fn load(read: &mut Read, src: &Source) -> Result<Map<String, Value>, String> {
    let at = src.path();
    let text = read(src)?;
    match serde_json::from_str(&text).map_err(|e| format!("{at}: {e}"))? {
        Value::Object(m) => Ok(m),
        _ => Err(format!("{at}: not an object")),
    }
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
fn checked(mut r: Map<String, Value>, at: &str) -> Result<Map<String, Value>, String> {
    for (k, v) in &r {
        if !RECIPE_KEYS.contains(&k.as_str()) {
            return Err(format!(
                "{at}: `{k}` is not a recipe key; one of {}",
                RECIPE_KEYS.join(", ")
            ));
        }
        let ok = match k.as_str() {
            "about" | "extends" | "material" | "shape" | "type" | "motion" | "effects"
            | "sound" | "icons" | "density" => v.is_string(),
            "with" => v.as_array().is_some_and(|a| a.iter().all(Value::is_string)),
            "color" | "pack" => v.is_object(),
            "ornament" => v.is_string() || v.is_object(),
            _ => true,
        };
        if !ok {
            let want = match k.as_str() {
                "with" => "a list of mix-in names",
                "color" | "pack" => "an object",
                "ornament" => "an ornament's name, or one per piece",
                _ => "a name",
            };
            return Err(format!("{at}: `{k}` is {want}"));
        }
    }
    if let Some(Value::Object(c)) = r.get("color") {
        for (k, v) in c {
            if !COLOR_KEYS.contains(&k.as_str()) {
                return Err(format!(
                    "{at}: `color.{k}` is not a colour key; one of {}",
                    COLOR_KEYS.join(", ")
                ));
            }
            if !v.is_string() {
                return Err(format!("{at}: `color.{k}` is a string"));
            }
        }
        if let Some(s) = c.get("scheme").and_then(Value::as_str)
            && Scheme::ALL.iter().all(|x| x.name() != s)
        {
            return Err(format!("{at}: `color.scheme` is \"light\" or \"dark\""));
        }
    }
    match r.get("ornament") {
        Some(Value::String(o)) => {
            let all: Map<String, Value> = ORNAMENT_SLOTS
                .iter()
                .map(|s| (s.to_string(), json!(o)))
                .collect();
            r.insert("ornament".into(), Value::Object(all));
        }
        Some(Value::Object(o)) => {
            for (k, v) in o {
                if !ORNAMENT_SLOTS.contains(&k.as_str()) || !v.is_string() {
                    return Err(format!(
                        "{at}: `ornament.{k}`: an ornament's name for one of {}",
                        ORNAMENT_SLOTS.join(", ")
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(r)
}

/// The recipe `top` (the text of a `theme.json`, read from `at`) with
/// everything it inherits merged in: its parent's resolved recipe, then
/// each mix-in in order, then its own keys. `extends` and `with` are gone
/// from the result. A theme that extends itself through any chain is an
/// error naming the chain.
pub fn resolve(top: &str, at: &str, read: &mut Read) -> Result<Map<String, Value>, String> {
    let own = match serde_json::from_str(top).map_err(|e| format!("{at}: {e}"))? {
        Value::Object(m) => m,
        _ => return Err(format!("{at}: not an object")),
    };
    resolve_with(own, at, read, &mut Vec::new())
}

fn resolve_with(
    own: Map<String, Value>,
    at: &str,
    read: &mut Read,
    chain: &mut Vec<String>,
) -> Result<Map<String, Value>, String> {
    let mut own = checked(own, at)?;
    let mut out = Value::Object(Map::new());
    if let Some(Value::String(parent)) = own.remove("extends") {
        let name = theme_name(&parent).to_owned();
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
            let mix = checked(load(read, &src)?, &path)?;
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
    /// The pack.json the dimensions compile to, without art.
    pub pack: Map<String, Value>,
    /// The art spec (`templates/tools/theme-art.py`'s input): the palette,
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
    /// only the colours the face was painted from. Each art slot's ink is
    /// moved until it holds 4.5:1 on its pieces as drawn, and every ink
    /// role is solved again over those grounds. Returns what still misses
    /// its ratio, for the cook to report.
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
        let mut grounds: [Vec<String>; 2] = [Vec::new(), Vec::new()];
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
            let light =
                drawn.iter().map(|f| luminance(f)).sum::<f32>() / drawn.len() as f32 >= 0.18;
            grounds[usize::from(!light)].extend(drawn);
        }
        let mut solved = Map::new();
        let seeds = seed_css(&self.pack_seeds());
        solve_inks(&seeds, &grounds, &mut solved, false);
        let mut more = Map::new();
        solve_inks(&seeds, &grounds, &mut more, true);
        set_more(&mut self.pack, more);
        // The ink solve moved by the drawn grounds wins over the one by
        // the painted faces: those grounds are what the text stands on.
        let Some(colors) = self
            .pack
            .get_mut("theme")
            .and_then(|t| t.get_mut("colors"))
            .and_then(Value::as_object_mut)
        else {
            return missed;
        };
        for (role, pair) in solved {
            colors.insert(role, pair);
        }
        // A fill the solve moved (`accent`) takes its own ink along.
        let seeds = seed_css(&self.pack_seeds());
        if let Some(colors) = self
            .pack
            .get_mut("theme")
            .and_then(|t| t.get_mut("colors"))
            .and_then(Value::as_object_mut)
        {
            solve_pairs(&seeds, colors);
        }
        missed
    }

    /// The pack's seeds alone, as [`seed_css`] reads them.
    fn pack_seeds(&self) -> Map<String, Value> {
        self.pack
            .get("seeds")
            .map(|s| Map::from_iter([("seeds".to_owned(), s.clone())]))
            .unwrap_or_default()
    }

    /// The layer, with `images` (the drawn art's pack entries, by name)
    /// under the recipe's own pack: what the cook lays over the default.
    pub fn layer(&self, images: Map<String, Value>) -> Result<Layer, String> {
        let mut pack = Value::Object(self.pack.clone());
        if !images.is_empty() {
            deep(&mut pack, &json!({ "images": images }));
        }
        deep(&mut pack, &Value::Object(self.over.clone()));
        Layer::parse(&pack.to_string()).map_err(|e| format!("the compiled theme: {e}"))
    }
}

mod color;
mod compile;
pub use color::{INKS, Lch, contrast, hex, lch, solve};
use color::{luminance, seed_css, set_more, solve_inks, solve_pairs};
use compile::ART_SLOTS;
pub use compile::compile;
