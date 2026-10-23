//! Looks and palettes (IDL `load.look`, `load.lookInfo`, `load.palette`,
//! `render.applyLook`, `render.look`, `GameDesc.look`): a cooked `type
//! look` blob (og-cook's `look` module: flattened and checked against the
//! records by the cook) is loaded with every asset it names, and applying
//! it writes the resources it names whole, through the same path as
//! `world.setResource` (so a system that applies one must declare those
//! writes), then configures the UI with its style.
//!
//! A look is data over existing resources: applying one changes nothing
//! the world could not change by itself, and rendering reads the resources
//! as always (`super::fx` maps `Stylise` and `MaterialStyle` into the
//! scene). A snapshot carries the resources; [`LookState::applied`] is only
//! what `og.render.look` answers.
//!
//! A palette or a look in a streaming group's pack is let go when the
//! group unloads (`super::stream`): the palette leaves the renderer and
//! Stylise.palette quantises to none until the group is resident again,
//! when it is added back ([`resolve`]); the look's values go and applying
//! it is refused until then. One the frame still uses (the stylisation's
//! palette, the look last applied) is kept, with a warning naming it.

use super::engine::Engine;
use og_cook::look::{ANIMATED_FIELDS, LookFile};
use og_core::ogt::Value as Ogt;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// One loaded look: its file and, per resource it sets, the fields as the
/// resource's JSON with the assets it names loaded.
pub(crate) struct LoadedLook {
    pub id: String,
    pub file: LookFile,
    pub values: Vec<(String, Map<String, Value>)>,
    /// The UI style's load index.
    pub style: Option<usize>,
    /// False once its streaming group unloaded it (its values are gone
    /// until the group is resident again).
    pub resident: bool,
}

/// The looks and palettes an engine loaded.
#[derive(Default)]
pub(crate) struct LookState {
    pub looks: Vec<LoadedLook>,
    pub by_path: BTreeMap<String, usize>,
    /// The look last applied.
    pub applied: Option<usize>,
    /// Palettes by load index (`og.load.palette`): the asset id and the
    /// renderer's palette (`None` while its streaming group is not resident).
    pub palettes: Vec<(String, Option<og_render::PaletteHandle>)>,
    pub palette_by_path: BTreeMap<String, usize>,
}

fn handle_value(i: usize) -> Value {
    json!(super::assets::handle(i) as f64)
}

/// Reads a cooked palette and adds it to the renderer.
fn add_palette(eng: &Engine, id: &str) -> Result<og_render::PaletteHandle, String> {
    let colors = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(id)?;
        bundle
            .palette(id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    Ok(eng
        .render
        .borrow_mut()
        .scene
        .add_palette(og_render::Palette { colors }))
}

/// `og.load.palette`: a cooked `.hex` or `.gpl` palette added to the
/// renderer once.
pub(crate) fn load_palette(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().fx.look.palette_by_path.get(&id) {
        return Ok(i);
    }
    let h = add_palette(eng, &id)?;
    let mut rs = eng.render.borrow_mut();
    let state = &mut rs.fx.look;
    state.palettes.push((id.clone(), Some(h)));
    let i = state.palettes.len() - 1;
    state.palette_by_path.insert(id, i);
    Ok(i)
}

/// The asset a handle field names, loaded by its column's loader.
fn load_handle(eng: &Engine, column: &str, path: &str) -> Result<Value, String> {
    let i = match column {
        "Texture" => super::assets::load_texture(eng, path)?,
        "Lut" => super::fx::load_lut(eng, path)?,
        "Palette" => load_palette(eng, path)?,
        "EnvironmentMap" => super::environment::load(eng, path)?,
        other => return Err(format!("a {other} is not loaded by a look")),
    };
    Ok(handle_value(i))
}

fn numbers(v: &[Ogt]) -> Vec<f64> {
    v.iter()
        .filter_map(|x| x.as_number().map(|n| n.as_f64()))
        .collect()
}

/// One field's values as the resource's JSON (the cook checked them).
fn field_value(eng: &Engine, decl: &og_api::host::FieldDecl, v: &[Ogt]) -> Result<Value, String> {
    if !decl.variants.is_empty() {
        return Ok(json!(v.first().and_then(Ogt::as_word).unwrap_or_default()));
    }
    let n = numbers(v);
    let at = |i: usize| n.get(i).copied().unwrap_or(0.0);
    Ok(match decl.column {
        "bool" => json!(v.first().and_then(Ogt::as_word) == Some("true")),
        "f32" | "u32" | "i32" => json!(at(0)),
        "Color" => json!({ "r": at(0), "g": at(1), "b": at(2) }),
        "Vector2" => json!({ "x": at(0), "y": at(1) }),
        "Vector3" => json!({ "x": at(0), "y": at(1), "z": at(2) }),
        column => match v.first().and_then(Ogt::as_str) {
            Some(path) => load_handle(eng, column, path)?,
            None => return Err(format!("{} takes an asset id", decl.name)),
        },
    })
}

/// What a look's blob holds, read with the assets it names loaded.
type LookParts = (LookFile, Vec<(String, Map<String, Value>)>, Option<usize>);

fn read(eng: &Engine, id: &str) -> Result<LookParts, String> {
    let id = id.to_string();
    let file = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle.look(&id, bytes.bytes()).map_err(|e| e.to_string())?
    };
    let mut values = Vec::new();
    for block in &file.blocks {
        let schema = format!("og.{}", block.resource);
        let record = og_api::host::RECORDS
            .iter()
            .find(|r| r.schema == schema && r.kind == "resource")
            .ok_or_else(|| format!("{id}: {} is not a resource of this engine", block.resource))?;
        let mut map = Map::new();
        for (field, v, _) in &block.fields {
            let decl = record
                .fields
                .iter()
                .find(|f| f.name == field)
                .ok_or_else(|| {
                    format!(
                        "{id}: {} has no field '{field}' (cook it again)",
                        block.resource
                    )
                })?;
            let value =
                field_value(eng, decl, v).map_err(|e| format!("{id}: {}.{e}", block.resource))?;
            map.insert(field.clone(), value);
        }
        values.push((block.resource.clone(), map));
    }
    let style = file
        .ui
        .as_deref()
        .map(|s| super::modules::load_style(eng, s))
        .transpose()
        .map_err(|e| format!("{id}: ui style: {e}"))?;
    Ok((file, values, style))
}

/// `og.load.look`: the look's load index, the assets it names loaded
/// (once per path).
pub(crate) fn load(eng: &Engine, path: &str) -> Result<usize, String> {
    let id = og_cook::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().fx.look.by_path.get(&id) {
        return Ok(i);
    }
    let (file, values, style) = read(eng, &id)?;
    let mut rs = eng.render.borrow_mut();
    let state = &mut rs.fx.look;
    state.looks.push(LoadedLook {
        id: id.clone(),
        file,
        values,
        style,
        resident: true,
    });
    let i = state.looks.len() - 1;
    state.by_path.insert(id, i);
    Ok(i)
}

/// At a tick boundary: adds back the palettes and reads again the looks
/// whose streaming group is resident again after an unload let them go.
pub(crate) fn resolve(eng: &Engine) {
    let palettes: Vec<(usize, String)> = {
        let rs = eng.render.borrow();
        let palettes = rs.fx.look.palettes.iter().enumerate();
        palettes
            .filter(|(_, (_, h))| h.is_none())
            .map(|(i, (id, _))| (i, id.clone()))
            .collect()
    };
    let looks: Vec<(usize, String)> = {
        let rs = eng.render.borrow();
        let looks = rs.fx.look.looks.iter().enumerate();
        looks
            .filter(|(_, l)| !l.resident)
            .map(|(i, l)| (i, l.id.clone()))
            .collect()
    };
    for (i, id) in palettes {
        if let Ok(h) = add_palette(eng, &id) {
            eng.render.borrow_mut().fx.look.palettes[i].1 = Some(h);
        }
    }
    for (i, id) in looks {
        if let Ok((file, values, style)) = read(eng, &id) {
            let mut rs = eng.render.borrow_mut();
            let look = &mut rs.fx.look.looks[i];
            look.file = file;
            look.values = values;
            look.style = style;
            look.resident = true;
        }
    }
}

/// `og.render.applyLook`: every resource the look names replaced whole
/// (its fields from the look, the rest at their defaults, the animated ones
/// kept), then its style given to the UI with the last UiDesc.
pub(crate) fn apply(eng: &Engine, index: usize) -> Result<(), String> {
    let (values, style) = {
        let rs = eng.render.borrow();
        let look = rs
            .fx
            .look
            .looks
            .get(index)
            .ok_or("that look is not loaded in this world")?;
        if !look.resident {
            return Err(format!(
                "{}: its streaming group is not resident (the look loads with it)",
                look.id
            ));
        }
        (look.values.clone(), look.style)
    };
    for (resource, fields) in &values {
        let schema = format!("og.{resource}");
        let layout = eng
            .state
            .borrow()
            .layouts
            .get(&schema)
            .cloned()
            .ok_or_else(|| format!("{resource} is not registered with this world"))?;
        let mut whole = Map::new();
        for f in layout.fields.iter() {
            if ANIMATED_FIELDS.contains(&(resource.as_str(), f.name.as_str())) {
                continue;
            }
            let v = fields
                .get(&f.name)
                .cloned()
                .unwrap_or_else(|| f.default.clone());
            whole.insert(f.name.clone(), v);
        }
        super::engine::write_resource_value(eng.id, &schema, &Value::Object(whole))
            .map_err(|e| format!("{resource}: {e}"))?;
    }
    if let Some(style) = style {
        // Every other field keeps the last UiDesc's value (og.ui.configure merges).
        let desc = json!({ "style": handle_value(style) });
        let answer = super::modules::og_ui_configure(eng.id, Some(desc.to_string()));
        if let Ok(Value::Object(o)) = serde_json::from_str::<Value>(&answer)
            && o.get("ok") == Some(&Value::Bool(false))
        {
            return Err(format!(
                "the ui style: {}",
                o.get("reason").and_then(Value::as_str).unwrap_or_default()
            ));
        }
    }
    eng.render.borrow_mut().fx.look.applied = Some(index);
    Ok(())
}

/// `og.load.lookInfo`.
pub(crate) fn info(eng: &Engine, index: usize) -> Result<Value, String> {
    let rs = eng.render.borrow();
    let look = rs
        .fx
        .look
        .looks
        .get(index)
        .ok_or("that look is not loaded in this world")?;
    let name = look.file.name.clone().unwrap_or_else(|| {
        let file = look.id.rsplit('/').next().unwrap_or(&look.id);
        file.trim_end_matches(".ogt").to_string()
    });
    Ok(json!({
        "look": handle_value(index),
        "path": look.id,
        "name": name,
        "extends": look.file.extends,
        "resources": look.file.resources(),
        "style": look.style.map(handle_value),
        "antialias": look.file.antialias,
    }))
}

/// GameDesc.look's antialias as the renderer's, when GameDesc.render
/// leaves it out: read before the host attaches the renderer, from the
/// look's blob alone. Loading the look itself loads the textures, style and
/// environment map it names, which the web host can only decode once its
/// renderer is attached (they go to the renderer's encodings), so a boot
/// that loaded the look left the web's renderer on the tier's method:
/// Lantern Vale's TAA was 4x MSAA on WebGL2 (and TAA on WebGPU only
/// because that is the full tier's method), its ink and outlines unblended
/// (web findings #3). A look whose blob is missing is left to `apply`
/// before start, which reports it.
pub(crate) fn boot_antialias(eng: &Engine, path: &str) {
    if eng.render.borrow().options.get("antialias").is_some() {
        return;
    }
    let id = og_cook::runtime::asset_id(path).to_string();
    let file = {
        let assets = eng.assets.borrow();
        let Ok((bytes, bundle)) = assets.blob(&id) else {
            return;
        };
        let Ok(file) = bundle.look(&id, bytes.bytes()) else {
            return;
        };
        file
    };
    let Some(aa) = file.antialias else {
        return;
    };
    let mut rs = eng.render.borrow_mut();
    if !rs.options.is_object() {
        rs.options = json!({});
    }
    rs.options["antialias"] = json!(aa);
}

/// The handle's load index, refused for a value that is not a handle.
pub(crate) fn index(look: f64) -> Result<usize, String> {
    super::assets::index_of(look).ok_or_else(|| format!("{look} is not a look"))
}
