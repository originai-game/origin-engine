//! A glTF material and its textures read onto a part.

use super::*;

impl Doc<'_> {
    /// Material `mi` (`None`: glTF's default material) as the file means
    /// it, and the factors it leaves unset. What is read past (a map on a
    /// second UV set, a texture transform, a sampler that does not
    /// repeat) is noted in `skipped`.
    pub(crate) fn material(
        &self,
        name: &str,
        mi: Option<usize>,
        images: &[ImageInfo],
        bytes: &[Option<Vec<u8>>],
        skipped: &mut Vec<String>,
    ) -> (Material, Vec<String>) {
        let m = match mi {
            Some(mi) => self.json.get("materials").at(mi),
            None => &Value::Null,
        };
        let pbr = m.get("pbrMetallicRoughness");
        let mut defaults = Vec::new();
        let mut num = |v: &Value, key: &str, default: f64| -> f64 {
            match v.get(key).num() {
                Some(x) if x.is_finite() => x,
                _ => {
                    defaults.push(key.to_string());
                    default
                }
            }
        };
        let metalness = num(pbr, "metallicFactor", 1.0);
        let roughness = num(pbr, "roughnessFactor", 1.0);
        let base = pbr
            .get("baseColorFactor")
            .nums()
            .filter(|v| v.len() == 4 && v.iter().all(|x| x.is_finite()))
            .unwrap_or_else(|| {
                defaults.push("baseColorFactor".into());
                vec![1.0; 4]
            });
        let emissive = m
            .get("emissiveFactor")
            .nums()
            .filter(|v| v.len() == 3 && v.iter().all(|x| x.is_finite()))
            .map_or([0.0; 3], |v| [v[0], v[1], v[2]]);
        let ext = m.get("extensions");
        let strength = ext
            .get("KHR_materials_emissive_strength")
            .get("emissiveStrength")
            .num()
            .filter(|x| x.is_finite() && *x != 1.0);
        let factor = |extension: &str, field: &str, default: f64| {
            ext.get(extension)
                .get(field)
                .num()
                .filter(|x| x.is_finite())
                .map_or(default, |x| x.clamp(0.0, 1.0))
        };
        let mode = m.get("alphaMode").str().unwrap_or("OPAQUE");
        let cutoff = m.get("alphaCutoff").num().filter(|x| x.is_finite());
        let mut tex =
            |info: &Value, what: &str| self.texture(name, info, what, images, bytes, skipped);
        let material = Material {
            name: match mi {
                Some(mi) => m
                    .get("name")
                    .str()
                    .map_or_else(|| format!("material{mi}"), str::to_string),
                None => String::new(),
            },
            color: Some([base[0], base[1], base[2]]),
            metalness,
            roughness: Some(roughness),
            emissive,
            opacity: Some(base[3]),
            map: tex(pbr.get("baseColorTexture"), "base-colour"),
            color_space: ColorSpace::Linear,
            emissive_intensity: strength,
            transparent: Some(mode == "BLEND"),
            alpha_test: (mode == "MASK").then(|| cutoff.unwrap_or(0.5)),
            double_sided: m.get("doubleSided").bool().unwrap_or(false),
            metalness_map: tex(pbr.get("metallicRoughnessTexture"), "metallic-roughness"),
            normal_map: tex(m.get("normalTexture"), "normal"),
            normal_scale: m
                .get("normalTexture")
                .get("scale")
                .num()
                .filter(|x| x.is_finite() && *x != 1.0),
            emissive_map: tex(m.get("emissiveTexture"), "emissive"),
            ao_map: tex(m.get("occlusionTexture"), "occlusion"),
            ao_map_intensity: m
                .get("occlusionTexture")
                .get("strength")
                .num()
                .filter(|x| x.is_finite() && *x != 1.0),
            clearcoat: factor("KHR_materials_clearcoat", "clearcoatFactor", 0.0),
            clearcoat_roughness: factor("KHR_materials_clearcoat", "clearcoatRoughnessFactor", 0.0),
            sheen: ext
                .get("KHR_materials_sheen")
                .get("sheenColorFactor")
                .nums()
                .filter(|v| v.len() == 3 && v.iter().all(|x| x.is_finite()))
                .map_or([0.0; 3], |v| [v[0], v[1], v[2]].map(|c| c.clamp(0.0, 1.0))),
            sheen_roughness: factor("KHR_materials_sheen", "sheenRoughnessFactor", 0.0),
            transmission: factor("KHR_materials_transmission", "transmissionFactor", 0.0),
            iridescence: factor("KHR_materials_iridescence", "iridescenceFactor", 0.0),
            surface: None,
            painted_from: None,
        };
        // The factors of these are read; a map inside one is not.
        const READ: [&str; 5] = [
            "KHR_materials_emissive_strength",
            "KHR_materials_clearcoat",
            "KHR_materials_sheen",
            "KHR_materials_transmission",
            "KHR_materials_iridescence",
        ];
        for (k, body) in ext.entries() {
            if !READ.contains(&k.as_str()) {
                skipped.push(format!("{name}: its material's extension {k} is not read"));
                continue;
            }
            for (field, _) in body.entries() {
                if field.ends_with("Texture") {
                    skipped.push(format!(
                        "{name}: its material's {field} is not read (the factor is)"
                    ));
                }
            }
        }
        (material, defaults)
    }

    /// The texture a material's texture info names, if its image could be
    /// read. What is read past is noted in `skipped`.
    pub(crate) fn texture(
        &self,
        name: &str,
        info: &Value,
        what: &str,
        images: &[ImageInfo],
        bytes: &[Option<Vec<u8>>],
        skipped: &mut Vec<String>,
    ) -> Option<Texture> {
        let ti = info.get("index").index()?;
        if info.get("texCoord").index().unwrap_or(0) != 0 {
            skipped.push(format!(
                "{name}: its {what} texture uses a second UV set, which is not read"
            ));
            return None;
        }
        if !info
            .get("extensions")
            .get("KHR_texture_transform")
            .is_null()
        {
            skipped.push(format!(
                "{name}: its {what} texture's KHR_texture_transform is not applied to the UVs"
            ));
        }
        let tex = self.json.get("textures").at(ti);
        let sampler = self
            .json
            .get("samplers")
            .at(tex.get("sampler").index().unwrap_or(usize::MAX));
        for key in ["wrapS", "wrapT"] {
            if let Some(w) = sampler.get(key).index().filter(|&w| w != 10497) {
                skipped.push(format!(
                    "{name}: its {what} texture's {key} {w} is written as repeat"
                ));
            }
        }
        let ext = tex.get("extensions");
        let source = tex
            .get("source")
            .index()
            .or(ext.get("EXT_texture_webp").get("source").index())
            .or(ext.get("KHR_texture_basisu").get("source").index())?;
        let Some(b) = bytes.get(source).and_then(Option::as_ref) else {
            skipped.push(format!(
                "{name}: its {what} texture's image could not be read"
            ));
            return None;
        };
        let n = images.get(source).map_or("", |i| i.name.as_str());
        Some(Texture::from_bytes(n, b.clone()))
    }
}
