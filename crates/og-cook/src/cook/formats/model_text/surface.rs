//! One glTF material writer for native model and surface declarations.
use super::{DIFFUSE_TRANSMISSION, foliage_extras};
use crate::cook::source_schemas::world::materials::Material;
use glam::DVec3;

pub(crate) fn material(
    name: &str,
    d: &Material,
    images: &mut Vec<String>,
    first: usize,
) -> Result<serde_json::Value, String> {
    let c = d.color.unwrap_or([1.0; 3]);
    let opacity = d.opacity.unwrap_or(1.0);
    let e = DVec3::from_array(d.emissive.unwrap_or([0.0; 3])) * d.emissive_intensity.unwrap_or(1.0);
    let mut m = serde_json::json!({"name":name,"pbrMetallicRoughness":{
        "baseColorFactor":[c[0],c[1],c[2],opacity],"metallicFactor":d.metalness.unwrap_or(0.0),"roughnessFactor":d.roughness.unwrap_or(1.0)
    },"doubleSided":d.double_sided.unwrap_or(false)});
    if e != DVec3::ZERO {
        let peak = e.max_element();
        m["emissiveFactor"] = serde_json::json!(if peak > 1.0 {
            (e / peak).to_array()
        } else {
            e.to_array()
        });
        if peak > 1.0 {
            m["extensions"] =
                serde_json::json!({"KHR_materials_emissive_strength":{"emissiveStrength":peak}});
        }
    }
    if let Some(cut) = d.alpha_test {
        m["alphaMode"] = "MASK".into();
        m["alphaCutoff"] = cut.into();
    } else if opacity < 1.0 {
        m["alphaMode"] = "BLEND".into();
    }
    let mut texture = |file: &Option<String>| -> Result<Option<usize>, String> {
        let Some(file) = file else {
            return Ok(None);
        };
        if file.is_empty() || file.starts_with('/') || file.contains('\\') {
            return Err(format!("material '{name}': names a relative image input"));
        }
        let i = match images.iter().position(|u| u == file) {
            Some(i) => i,
            None => {
                images.push(file.clone());
                images.len() - 1
            }
        };
        Ok(Some(first + i))
    };
    if let Some(t) = texture(&d.map)? {
        m["pbrMetallicRoughness"]["baseColorTexture"] = serde_json::json!({"index":t});
    }
    if let Some(t) = texture(&d.normal_map)? {
        m["normalTexture"] = serde_json::json!({"index":t,"scale":d.normal_scale.unwrap_or(1.0)});
    }
    if let Some(t) = texture(&d.metalness_map)? {
        m["pbrMetallicRoughness"]["metallicRoughnessTexture"] = serde_json::json!({"index":t});
    }
    if let Some(t) = texture(&d.ao_map)? {
        m["occlusionTexture"] = serde_json::json!({"index":t});
    }
    let mut ext = serde_json::Map::new();
    let transmission = d.transmission.unwrap_or(0.0);
    if transmission > 0.0 {
        ext.insert(
            "KHR_materials_transmission".into(),
            serde_json::json!({"transmissionFactor":transmission}),
        );
        let thickness = d.thickness.unwrap_or(0.0);
        if thickness > 0.0 {
            ext.insert(
                "KHR_materials_volume".into(),
                serde_json::json!({"thicknessFactor":thickness}),
            );
        }
    }
    if let Some(ior) = d.ior {
        ext.insert("KHR_materials_ior".into(), serde_json::json!({"ior":ior}));
    }
    let translucency = d.translucency.unwrap_or(0.0);
    if translucency > 0.0 {
        ext.insert(
            DIFFUSE_TRANSMISSION.into(),
            serde_json::json!({"diffuseTransmissionFactor":translucency}),
        );
    }
    let default = og_render_data::foliage::LeafSeason::EVERGREEN;
    let season = og_render_data::foliage::LeafSeason {
        season: d.season.unwrap_or(default.season as f64) as f32,
        autumn_color: d.autumn_color.unwrap_or(default.autumn_color),
        autumn_color2: d.autumn_color2.unwrap_or(default.autumn_color2),
        shift: d.season_shift.unwrap_or(default.shift),
    };
    if let Some(extras) = foliage_extras(d.dome_normals.unwrap_or(0.0), &season) {
        m["extras"] = extras;
    }
    let grime = d.grime.unwrap_or(0.0);
    let stains = d.stains.unwrap_or(0.0);
    let stochastic = d.stochastic.unwrap_or(0.0);
    if grime > 0.0 || stains > 0.0 || stochastic > 0.0 {
        if !m["extras"].is_object() {
            m["extras"] = serde_json::json!({});
        }
        m["extras"]["ogVariation"] = serde_json::json!({"grime":grime,"grimeColor":d.grime_color.unwrap_or([0.0;3]),"grimeHeight":d.grime_height.unwrap_or(0.0),"stains":stains,"stochastic":stochastic});
    }
    let coat = d.clearcoat.unwrap_or(0.0);
    if coat > 0.0 {
        ext.insert("KHR_materials_clearcoat".into(),serde_json::json!({"clearcoatFactor":coat,"clearcoatRoughnessFactor":d.clearcoat_roughness.unwrap_or(0.0)}));
    }
    if !ext.is_empty() {
        let mut all = m["extensions"]
            .as_object_mut()
            .map(std::mem::take)
            .unwrap_or_default();
        all.extend(ext);
        m["extensions"] = all.into();
    }
    Ok(m)
}
