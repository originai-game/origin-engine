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
    let t = compile(text, at)?;
    let mut out = Vec::new();
    let mut images = Map::new();
    if !t.art.is_null() {
        let dir = super::ui_art::draw_cached(&t.art, cache).map_err(|e| format!("{at}: {e}"))?;
        let read = |p: PathBuf| std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()));
        let drawn: Map<String, Value> = serde_json::from_slice(&read(dir.join("images.json"))?)
            .map_err(|e| format!("{at}: the drawn art's images.json: {e}"))?;
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
    Ok(out)
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

    #[test]
    fn a_recipe_cooks_into_a_pack_and_its_art() {
        let cache = std::env::temp_dir().join(format!("og-ui-themes-{}", std::process::id()));
        let text = r##"{
          "color": { "source": "#d9a531", "accent": "#e8c25a", "neutral": "#4c5643" },
          "material": "metal", "shape": "cut", "type": "condensed-stencil", "motion": "snappy",
          "ornament": { "panel": "brackets" },
          "pack": { "images": { "chip": { "src": "art/hand-chip.png", "slice": 12 } } }
        }"##;
        let f = files(text, "styles/game/theme.json", &cache).unwrap();
        let names: Vec<&str> = f.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"art/panel.png") && names.contains(&"art/panel.png.ogt"));
        assert!(names.contains(&"art/button-primary-pressed.png"));
        // the recipe's own chip replaces the drawn one, which is not bundled
        assert!(!names.contains(&"art/chip.png"));
        let pack = &f.iter().find(|(n, _)| n == "pack.json").unwrap().1;
        let layer = og_ui_next::pack::Layer::parse(std::str::from_utf8(pack).unwrap()).unwrap();
        assert_eq!(layer.images[1]["panel"].src, "art/panel.png");
        assert_eq!(layer.images[1]["chip"].src, "art/hand-chip.png");
        assert!(layer.variants["Panel.frame"].contains("nine-panel"));
        // drawn once: the second cook reads the cache
        let again = files(text, "styles/game/theme.json", &cache).unwrap();
        assert_eq!(f, again);
        let _ = std::fs::remove_dir_all(&cache);
    }

    #[test]
    fn only_a_styles_directorys_recipe_is_one() {
        assert!(is_recipe("styles/game/theme.json"));
        assert!(!is_recipe("styles/theme.json"));
        assert!(!is_recipe("styles/a/b/theme.json"));
        assert!(!is_recipe("levels/game/theme.json"));
    }
}
