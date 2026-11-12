use crate::material::PhysicalExtensions;

/// What a texture a glTF material samples holds: how it is decoded (and cooked).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureKind {
    /// sRGB colour.
    Color,
    /// A tangent-space normal map.
    Normal,
    /// Linear data (masks, factors).
    Linear,
}

/// The material's physical extensions (`KHR_materials_clearcoat`,
/// `_sheen`, `_transmission`, `_volume`, `_ior`, `_iridescence`,
/// `_anisotropy`), each texture as `texture` makes it from the glTF
/// texture and what it holds. `present` is 0 for a material with none.
pub fn physical_extensions<T>(
    doc: &gltf::Document,
    m: &gltf::Material,
    mut texture: impl FnMut(gltf::Texture, TextureKind) -> T,
) -> PhysicalExtensions<T> {
    use gltf::json::Value;
    type Ext<T> = PhysicalExtensions<T>;
    let mut e = Ext::default();
    let num = |v: &Value, k: &str, fallback: f32| {
        v.get(k)
            .and_then(Value::as_f64)
            .map_or(fallback, |x| x as f32)
    };
    let rgb = |v: &Value, k: &str, fallback: [f32; 3]| {
        let mut c = fallback;
        if let Some(a) = v.get(k).and_then(Value::as_array) {
            for (o, x) in c.iter_mut().zip(a) {
                *o = x.as_f64().map_or(*o, |x| x as f32);
            }
        }
        c
    };
    let mut tex = |v: &Value, k: &str, kind: TextureKind| {
        v.get(k)
            .and_then(|t| t.get("index"))
            .and_then(Value::as_u64)
            .and_then(|i| doc.textures().nth(i as usize))
            .map(|t| texture(t, kind))
    };
    use TextureKind::{Color, Linear, Normal};
    if let Some(v) = m.extension_value("KHR_materials_clearcoat") {
        e.present |= Ext::<T>::CLEARCOAT;
        e.clearcoat = num(v, "clearcoatFactor", 0.0);
        e.clearcoat_texture = tex(v, "clearcoatTexture", Linear);
        e.clearcoat_roughness = num(v, "clearcoatRoughnessFactor", 0.0);
        e.clearcoat_roughness_texture = tex(v, "clearcoatRoughnessTexture", Linear);
        e.clearcoat_normal_texture = tex(v, "clearcoatNormalTexture", Normal);
        e.clearcoat_normal_scale = v
            .get("clearcoatNormalTexture")
            .map_or(1.0, |t| num(t, "scale", 1.0));
    }
    if let Some(v) = m.extension_value("KHR_materials_sheen") {
        e.present |= Ext::<T>::SHEEN;
        e.sheen_color = rgb(v, "sheenColorFactor", [0.0; 3]);
        e.sheen_color_texture = tex(v, "sheenColorTexture", Color);
        e.sheen_roughness = num(v, "sheenRoughnessFactor", 0.0);
        e.sheen_roughness_texture = tex(v, "sheenRoughnessTexture", Linear);
    }
    if let Some(v) = m.extension_value("KHR_materials_transmission") {
        e.present |= Ext::<T>::TRANSMISSION;
        e.transmission = num(v, "transmissionFactor", 0.0);
        e.transmission_texture = tex(v, "transmissionTexture", Linear);
    }
    if let Some(v) = m.extension_value("KHR_materials_volume") {
        e.present |= Ext::<T>::VOLUME;
        e.thickness = num(v, "thicknessFactor", 0.0);
        e.thickness_texture = tex(v, "thicknessTexture", Linear);
        // Absent is +infinity: no absorption.
        e.attenuation_distance = num(v, "attenuationDistance", 0.0);
        e.attenuation_color = rgb(v, "attenuationColor", [1.0; 3]);
    }
    if let Some(v) = m.extension_value("KHR_materials_ior") {
        e.present |= Ext::<T>::IOR;
        e.ior = num(v, "ior", 1.5);
    }
    if let Some(v) = m.extension_value("KHR_materials_iridescence") {
        e.present |= Ext::<T>::IRIDESCENCE;
        e.iridescence = num(v, "iridescenceFactor", 0.0);
        e.iridescence_texture = tex(v, "iridescenceTexture", Linear);
        e.iridescence_ior = num(v, "iridescenceIor", 1.3);
        e.iridescence_thickness = [
            num(v, "iridescenceThicknessMinimum", 100.0),
            num(v, "iridescenceThicknessMaximum", 400.0),
        ];
        e.iridescence_thickness_texture = tex(v, "iridescenceThicknessTexture", Linear);
    }
    if let Some(v) = m.extension_value("KHR_materials_anisotropy") {
        e.present |= Ext::<T>::ANISOTROPY;
        e.anisotropy = num(v, "anisotropyStrength", 0.0);
        e.anisotropy_rotation = num(v, "anisotropyRotation", 0.0);
        e.anisotropy_texture = tex(v, "anisotropyTexture", Linear);
    }
    e
}
