//! Checked theme declarations compiled to native layers and captured art.
use super::{generator, source_schemas};
use og_ui_next::theme::{self, Source};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

pub const RECIPE: &str = "theme.ts";

pub fn themes_dir() -> Option<&'static Path> {
    static DIR: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        if let Some(d) = std::env::var_os("OG_THEMES_DIR") {
            let d = PathBuf::from(d);
            return d.is_dir().then_some(d);
        }
        super::systems::look::styles_dir()
            .and_then(Path::parent)
            .map(|p| p.join("themes"))
            .into_iter()
            .chain([Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/themes")])
            .find(|d| d.join("dimensions").is_dir())
    })
    .as_deref()
}

fn slug(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

pub fn builtin_recipe(name: &str) -> Option<PathBuf> {
    if !slug(name) {
        return None;
    }
    let path = themes_dir()?.join(name).join(RECIPE);
    path.is_file().then_some(path)
}

pub fn builtin_slugs() -> Vec<String> {
    let Some(dir) = themes_dir() else {
        return Vec::new();
    };
    let mut out: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| {
            e.path()
                .join(RECIPE)
                .is_file()
                .then(|| e.file_name().to_string_lossy().into_owned())
        })
        .collect();
    out.sort();
    out
}

/// Engine recipes use the same file.theme envelope, schema and closed evaluator
/// as project recipes. Dimension tables are engine-owned compilation data.
pub(crate) fn read_engine(
    src: &Source,
    inputs: &mut Vec<PathBuf>,
) -> Result<Map<String, Value>, String> {
    let dir = themes_dir().ok_or("engine themes were not found")?;
    let path = match src {
        Source::Theme(id) => {
            let name = id
                .strip_prefix("@themes/")
                .filter(|n| slug(n))
                .ok_or_else(|| format!("{id}: not an explicit engine theme ID"))?;
            format!("{name}/{RECIPE}")
        }
        Source::Mixin(name) if slug(name) => format!("mixins/{name}.ts"),
        Source::Dimension(d, v) if slug(d) && slug(v) => {
            let path = dir.join("dimensions").join(d).join(format!("{v}.json"));
            inputs.push(path.clone());
            return serde_json::from_slice(
                &std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
            )
            .map_err(|e| format!("{}: {e}", path.display()));
        }
        _ => {
            return Err(format!(
                "{}: invalid engine dimension or mix-in name",
                src.path()
            ));
        }
    };
    let program = generator::Program::prepare_value(dir, &path)?;
    inputs.extend(program.dependencies.iter().map(|p| dir.join(p)));
    let file: source_schemas::File =
        serde_json::from_value(program.run()?).map_err(|e| format!("{path}: {e}"))?;
    file.check(None).map_err(|e| format!("{path}: {e}"))?;
    if file.kind != "theme" {
        return Err(format!("{path}: expected file.theme"));
    }
    Ok(file.value.as_object().expect("checked theme").clone())
}

/// Fields have already passed the author schema. Only canonical reference
/// tokens in the pack are lowered to IDs for the native theme compiler.
pub(crate) fn compile(
    value: Map<String, Value>,
    at: &str,
    read: &mut theme::Read,
) -> Result<theme::Theme, String> {
    let mut recipe = theme::resolve(value, at, read)?;
    fn ids(value: &mut Value) {
        match value {
            Value::Object(m) if m.len() == 1 && m.get("$ogRef").is_some_and(Value::is_string) => {
                *value = m.remove("$ogRef").expect("canonical reference");
            }
            Value::Object(m) => m.values_mut().for_each(ids),
            Value::Array(a) => a.iter_mut().for_each(ids),
            _ => {}
        }
    }
    if let Some(pack) = recipe.get_mut("pack") {
        ids(pack);
    }
    theme::compile(&recipe, read).map_err(|e| format!("{at}: {e}"))
}

/// One style's native fields, optional recipe diagnostics, and generated art.
pub struct Style {
    pub layer: og_ui_next::pack::Layer,
    pub theme: Option<theme::Theme>,
    pub(crate) art: Vec<(String, Vec<u8>)>,
    pub(crate) notes: Vec<String>,
    pub(crate) inputs: Vec<PathBuf>,
}

/// Draw the same art as before; name each actual generated image as a
/// subresource of its producer. No synthetic pack.json is ever written.
pub(crate) fn cook(mut t: theme::Theme, id: &str, cache: &Path) -> Result<Style, String> {
    let mut art = Vec::new();
    let mut images = Map::new();
    let mut notes = Vec::new();
    if !t.art.is_null() {
        let dir = super::ui_art::draw_cached(&t.art, cache).map_err(|e| format!("{id}: {e}"))?;
        let read = |p: PathBuf| std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()));
        let drawn: Map<String, Value> = serde_json::from_slice(&read(dir.join("images.json"))?)
            .map_err(|e| format!("{id}: {e}"))?;
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
        notes = t.register(&faces);
        for (name, mut entry) in drawn {
            if t.over.get("images").and_then(|i| i.get(&name)).is_some() {
                continue;
            }
            let resource = format!("art/{name}");
            entry["src"] = Value::String(format!("{id}#{resource}"));
            art.push((resource, read(dir.join(format!("art/{name}.png")))?));
            images.insert(name, entry);
        }
    }
    Ok(Style {
        layer: t.layer(images)?,
        theme: Some(t),
        art,
        notes,
        inputs: Vec::new(),
    })
}

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

pub fn cache_dir(assets_dir: &Path) -> PathBuf {
    assets_dir.parent().unwrap_or(assets_dir).join(".og/ui-art")
}
