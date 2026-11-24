//! Theme recipes (og-ui-next's CONVENTIONS.md §Theme recipes) cooked into
//! the style pack they make: a `theme.json` resolved and compiled
//! (`og_ui_next::theme`), its art drawn (`ui_art`, cached under the
//! project's `.og/ui-art/` by spec), and the pack written beside it as the
//! `pack.json` a style directory has, with the art under `art/`. The rest
//! of the cook then treats it as any pack: its classes are checked, its
//! assets bundled, and a look's `ui "<name>"` names it.
//!
//! A game's theme is `assets/styles/<name>/theme.json`; the engine's are
//! `templates/themes/<name>/theme.json`, bundled as `@themes/<name>/` only
//! when a look names one. The dimension values both read are the engine's
//! (`templates/themes/dimensions/`).

use og_ui_next::theme::{self, Source};
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};

/// A theme's recipe file name.
pub const RECIPE: &str = "theme.json";

/// The import settings each piece of drawn art cooks with: colour art.
const ART_SETTINGS: &str =
    "type import\nversion 1\n---\n# Generated theme art: sRGB colour.\ntexture {\n}\n";

/// Whether `rel` is a game theme's recipe (`styles/<name>/theme.json`).
pub fn is_recipe(rel: &str) -> bool {
    rel.strip_prefix("styles/")
        .and_then(|r| r.strip_suffix(&format!("/{RECIPE}")))
        .is_some_and(|name| !name.is_empty() && !name.contains('/'))
}

/// The engine's `templates/themes/`: `OG_THEMES_DIR`, else beside the
/// built-in styles (a checkout's `templates/`, the engine package's
/// `themes/` beside `styles/`), else the checkout this crate was built in.
pub fn themes_dir() -> Option<&'static Path> {
    static DIR: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        if let Some(d) = std::env::var_os("OG_THEMES_DIR") {
            let d = PathBuf::from(d);
            return d.is_dir().then_some(d);
        }
        let near = super::systems::look::styles_dir()
            .and_then(Path::parent)
            .map(|p| p.join("themes"));
        near.into_iter()
            .chain([Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/themes")])
            .find(|d| d.join("dimensions").is_dir())
    })
    .as_deref()
}

/// An engine theme by name, if there is one.
pub fn builtin_recipe(name: &str) -> Option<PathBuf> {
    let p = themes_dir()?.join(name).join(RECIPE);
    p.is_file().then_some(p)
}

/// Reads a recipe source from the engine's themes.
fn read_engine(src: &Source) -> Result<String, String> {
    let dir = themes_dir().ok_or("the engine's themes (templates/themes) were not found")?;
    let rel = src.path();
    let rel = rel.strip_prefix("themes/").unwrap_or(&rel);
    std::fs::read_to_string(dir.join(rel)).map_err(|e| format!("{}: {e}", src.path()))
}

/// The recipe `text` (at `at`) resolved and compiled against the engine's
/// themes and dimensions.
pub fn compile(text: &str, at: &str) -> Result<theme::Theme, String> {
    let recipe = theme::resolve(text, at, &mut read_engine)?;
    theme::compile(&recipe, &mut read_engine).map_err(|e| format!("{at}: {e}"))
}

/// The files the recipe `text` (at `at`) cooks into, relative to its
/// style directory: `pack.json`, and each piece of art and its settings
/// under `art/`. `cache` keeps drawn art by spec.
pub fn files(text: &str, at: &str, cache: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    Ok(cook(text, at, cache)?.0)
}

/// A theme's cooked files, relative to its style directory.
pub type Files = Vec<(String, Vec<u8>)>;

/// [`files`], and the inks that still miss their ratio on the drawn art.
pub fn cook(text: &str, at: &str, cache: &Path) -> Result<(Files, Vec<String>), String> {
    let mut t = compile(text, at)?;
    let mut out = Vec::new();
    let mut images = Map::new();
    let mut missed = Vec::new();
    if !t.art.is_null() {
        let dir = super::ui_art::draw_cached(&t.art, cache).map_err(|e| format!("{at}: {e}"))?;
        let read = |p: PathBuf| std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()));
        let drawn: Map<String, Value> = serde_json::from_slice(&read(dir.join("images.json"))?)
            .map_err(|e| format!("{at}: the drawn art's images.json: {e}"))?;
        // Ground registration: each piece's centre as drawn.
        let mut faces = std::collections::BTreeMap::new();
        for (name, entry) in &drawn {
            let file = dir.join(format!("art/{name}.png"));
            let img = image::open(&file)
                .map_err(|e| format!("{}: {e}", file.display()))?
                .to_rgba8();
            if let Some(f) = centre_faces(&img, entry) {
                faces.insert(name.clone(), f);
            }
        }
        missed = t.register(&faces);
        for (name, entry) in drawn {
            // A piece the recipe's pack replaces is not bundled.
            if t.over.get("images").and_then(|i| i.get(&name)).is_some() {
                continue;
            }
            let file = format!("art/{name}.png");
            out.push((file.clone(), read(dir.join(&file))?));
            out.push((format!("{file}.ogt"), ART_SETTINGS.as_bytes().to_vec()));
            images.insert(name, entry);
        }
    }
    let mut pack: Value =
        serde_json::from_str(&t.layer(images)?.to_json()).map_err(|e| format!("{at}: {e}"))?;
    pack["about"] = json!(format!(
        "Compiled by og cook from {at}; edit the recipe, not this."
    ));
    out.push((
        "pack.json".into(),
        serde_json::to_vec_pretty(&pack).map_err(|e| e.to_string())?,
    ));
    Ok((out, missed))
}

/// The ground a nine-slice piece draws under text: the 10th and 90th
/// percentile by luminance of the opaque pixels inside its slice (the
/// stretched centre), as `#rrggbb`. `None` when the centre is empty or
/// see-through.
fn centre_faces(img: &image::RgbaImage, entry: &Value) -> Option<Vec<String>> {
    let s = &entry["slice"];
    let inset = |i: usize| s.get(i).or(Some(s)).and_then(Value::as_u64).unwrap_or(0) as u32;
    let (t, r, b, l) = (inset(0), inset(1), inset(2), inset(3));
    let (w, h) = img.dimensions();
    if l + r >= w || t + b >= h {
        return None;
    }
    let mut px: Vec<[u8; 3]> = Vec::new();
    for y in t..h - b {
        for x in l..w - r {
            let p = img.get_pixel(x, y).0;
            if p[3] >= 200 {
                px.push([p[0], p[1], p[2]]);
            }
        }
    }
    if px.is_empty() {
        return None;
    }
    let lum = |p: &[u8; 3]| {
        0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2])
    };
    px.sort_by(|a, b| lum(a).total_cmp(&lum(b)));
    let at = |q: f32| px[((px.len() - 1) as f32 * q).round() as usize];
    let hex = |p: [u8; 3]| format!("#{:02x}{:02x}{:02x}", p[0], p[1], p[2]);
    Some(vec![hex(at(0.1)), hex(at(0.9))])
}

/// The engine theme `name`'s files by bundle id (`@themes/<name>/…`),
/// drawn once per process.
pub fn builtin_files(
    name: &str,
    cache: &Path,
) -> Result<Vec<(&'static str, &'static [u8])>, String> {
    type Done = std::collections::HashMap<String, Vec<(&'static str, &'static [u8])>>;
    static DONE: std::sync::Mutex<Option<Done>> = std::sync::Mutex::new(None);
    let path = builtin_recipe(name).ok_or_else(|| format!("no engine theme {name}"))?;
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let key = format!("{name}\n{text}");
    let mut done = DONE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let done = done.get_or_insert_with(Done::new);
    if let Some(f) = done.get(&key) {
        return Ok(f.clone());
    }
    let at = format!("@themes/{name}/{RECIPE}");
    let f: Vec<(&'static str, &'static [u8])> = files(&text, &at, cache)?
        .into_iter()
        .map(|(rel, bytes)| {
            let id: &'static str = Box::leak(format!("@themes/{name}/{rel}").into_boxed_str());
            (id, &*Box::leak(bytes.into_boxed_slice()))
        })
        .collect();
    done.insert(key, f.clone());
    Ok(f)
}

/// Where a project's drawn art is kept: `<project>/.og/ui-art/`.
pub fn cache_dir(assets_dir: &Path) -> PathBuf {
    assets_dir
        .parent()
        .unwrap_or(assets_dir)
        .join(".og")
        .join("ui-art")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every engine theme (`templates/themes/<name>/theme.json`) resolves,
    /// compiles and cooks into a pack with its art, and every art slot's ink
    /// holds 4.5:1 on its pieces as drawn.
    ///
    /// Ways this could fail: a recipe naming a dimension value that does not
    /// exist, an `extends` to a missing theme, a compiled pack the layer
    /// parser refuses, a material and shape pair the generator cannot draw.
    #[test]
    fn every_engine_theme_cooks() {
        let dir = themes_dir().unwrap();
        let cache = std::env::temp_dir().join(format!("og-ui-themes-all-{}", std::process::id()));
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| {
                let p = e.ok()?.path();
                p.join(RECIPE)
                    .is_file()
                    .then(|| p.file_name()?.to_str().map(str::to_owned))?
            })
            .collect();
        names.sort();
        assert!(names.len() >= 10, "{names:?}");
        let mut bad = Vec::new();
        for name in &names {
            let at = format!("@themes/{name}/{RECIPE}");
            let text = std::fs::read_to_string(dir.join(name).join(RECIPE)).unwrap();
            match cook(&text, &at, &cache) {
                // Every ink holds on the art as drawn (ground registration).
                Ok((f, missed)) if f.iter().any(|(n, _)| n == "art/panel.png") => {
                    bad.extend(missed.into_iter().map(|m| format!("{at}: {m}")));
                }
                Ok(_) => bad.push(format!("{at}: no art")),
                Err(e) => bad.push(e),
            }
        }
        let _ = std::fs::remove_dir_all(&cache);
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }
}
