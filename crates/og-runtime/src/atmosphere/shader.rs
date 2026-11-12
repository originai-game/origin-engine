//! Material shaders: `og.load.shader`, `og.render.shaderMaterial`,
//! `og.render.setMaterialShader` and `og.render.setShaderParam`.

use crate::engine::engine;
use crate::parse;
use serde_json::{Value, json};

/// `og.load.shader`: a cooked shader, added to the scene once per path.
pub fn og_load_shader(world: u32, path: String) -> Result<serde_json::Value, String> {
    (|| -> Result<Value, String> {
        let eng = engine(world)?;
        Ok(json!(crate::assets::handle(load_shader(&eng, &path)?)))
    })()
}

/// A cooked shader's load index, added to the scene once per path (an
/// effect material's hook loads it too, `crate::effect`).
pub(crate) fn load_shader(eng: &crate::engine::Engine, path: &str) -> Result<usize, String> {
    let id = og_bundle::runtime::asset_id(path).to_string();
    if let Some(&i) = eng.render.borrow().atmo.shader_paths.get(&id) {
        return Ok(i);
    }
    let shader = {
        let assets = eng.assets.borrow();
        let (bytes, bundle) = assets.blob(&id)?;
        bundle
            .shader(&id, bytes.bytes())
            .map_err(|e| e.to_string())?
    };
    let mut rs = eng.render.borrow_mut();
    let h = rs.scene.add_shader(shader);
    rs.atmo.shaders.push(h);
    let i = rs.atmo.shaders.len() - 1;
    rs.atmo.shader_paths.insert(id, i);
    Ok(i)
}

pub fn og_render_shader_material(
    world: u32,
    shader: f64,
    desc: Option<String>,
) -> Result<serde_json::Value, String> {
    (|| -> Result<Value, String> {
        let eng = engine(world)?;
        let desc = parse(desc.as_deref())?;
        let mut rs = eng.render.borrow_mut();
        let s = crate::assets::index_of(shader)
            .and_then(|i| rs.atmo.shaders.get(i).copied())
            .ok_or_else(|| format!("shader {shader} is not loaded in this world"))?;
        let m = crate::api::material(
            og_render::StandardMaterial::default(),
            &desc,
            &rs.textures,
            &rs.texture_arrays,
        )?;
        let handle = rs.scene.add_standard_material(m);
        rs.scene.set_material_shader(handle, Some(s));
        rs.materials.push(handle);
        Ok(json!(crate::api::material_handle(rs.materials.len() - 1)))
    })()
}

/// og.render.setMaterialShader: `shader` is the JSON of a shader handle or null.
pub fn og_render_set_material_shader(
    world: u32,
    material: f64,
    shader: String,
) -> Result<(), String> {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let shader: Option<u64> = crate::wire::arg(Some(&shader), "shader")?;
        let mut rs = eng.render.borrow_mut();
        let h = crate::assets::index_of(material)
            .and_then(|i| rs.materials.get(i).copied())
            .ok_or_else(|| format!("material {material} is not alive"))?;
        let s = match shader {
            None => None,
            Some(handle) => Some(
                crate::assets::index_of(handle as f64)
                    .and_then(|i| rs.atmo.shaders.get(i).copied())
                    .ok_or_else(|| format!("shader {handle} is not loaded in this world"))?,
            ),
        };
        if !rs.scene.set_material_shader(h, s) {
            return Err(format!("material {material} is not a standard material"));
        }
        Ok(())
    })();
    match r {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn og_render_set_shader_param(
    world: u32,
    material: f64,
    name: String,
    value: String,
) -> Result<(), String> {
    let r = (|| -> Result<(), String> {
        let eng = engine(world)?;
        let value: Vec<f32> = match serde_json::from_str::<Value>(&value) {
            // Hand read: a bare number, which the IDL's List.f32 does not hold.
            Ok(Value::Number(n)) => vec![n.as_f64().unwrap_or(0.0) as f32],
            Ok(v @ Value::Array(_)) => crate::wire::value(&v, "value")?,
            _ => return Err("value is a list of numbers".into()),
        };
        let mut rs = eng.render.borrow_mut();
        let h = crate::assets::index_of(material)
            .and_then(|i| rs.materials.get(i).copied())
            .ok_or_else(|| format!("material {material} is not alive"))?;
        if rs.scene.material_shader(h).is_none() {
            return Err(format!(
                "material {material} has no shader (make it with og.render.shaderMaterial or give it one with og.render.setMaterialShader)"
            ));
        }
        if !rs.scene.set_shader_param(h, &name, &value) {
            return Err(format!(
                "the material's shader declares no parameter '{name}'"
            ));
        }
        Ok(())
    })();
    match r {
        Ok(()) => Ok(()),
        Err(e) => Err(e),
    }
}
