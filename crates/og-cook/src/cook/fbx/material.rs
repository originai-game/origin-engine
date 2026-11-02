//! ufbx materials as glTF metallic-roughness materials, their textures,
//! images and samplers (the module docs of `fbx.rs` say what maps to what).

use super::*;

/// An image the glb holds or names.
enum Source {
    /// A file beside the model, named by its path relative to the model's folder.
    Uri(String),
    /// Bytes packed into the glb (an embedded texture, or one packed at cook time).
    Bytes(Vec<u8>),
}

#[derive(Default)]
pub(super) struct Materials {
    mats: Vec<Value>,
    mat_of: HashMap<u32, usize>,
    images: Vec<(String, Source)>,
    image_of: HashMap<String, usize>,
    textures: Vec<Value>,
    texture_of: HashMap<(usize, usize, String), usize>,
    samplers: Vec<Value>,
    sampler_of: HashMap<(u32, u32), usize>,
}

/// A texture's pixels, decoded for packing.
struct Pixels {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// The finished lists: materials, images, textures, samplers and the files read.
type Finished = (
    Vec<Value>,
    Vec<Value>,
    Vec<Value>,
    Vec<Value>,
    Vec<(String, Vec<u8>)>,
);

/// A map's value as one number.
fn real(m: &ufbx::MaterialMap) -> f64 {
    m.value_vec4.x
}

/// A map's value as a colour.
fn rgb3(m: &ufbx::MaterialMap) -> DVec3 {
    DVec3::new(m.value_vec4.x, m.value_vec4.y, m.value_vec4.z)
}

fn wrap(w: ufbx::WrapMode) -> u32 {
    match w {
        ufbx::WrapMode::Clamp => 33071,
        _ => 10497,
    }
}

/// The file texture a material map samples (a layered texture's first layer).
fn file_texture<'a>(
    map: &'a ufbx::MaterialMap,
    what: &str,
    notes: &mut Vec<String>,
) -> Option<&'a ufbx::Texture> {
    let t = map.texture.as_ref()?;
    if !map.texture_enabled && map.feature_disabled {
        return None;
    }
    if t.type_ == ufbx::TextureType::File {
        return Some(t);
    }
    let first = t.file_textures.first()?;
    if t.file_textures.len() > 1 {
        notes.push(format!(
            "{what}: texture '{}' blends {} layers; its first, '{}', is kept",
            t.element.name,
            t.file_textures.len(),
            first.element.name
        ));
    }
    Some(first)
}

/// `KHR_texture_transform` of a texture's UV transform, in glTF's uv
/// space (v down): the affine `F M F` with `F(u, v) = (u, 1 - v)`,
/// decomposed into offset, rotation and scale.
fn uv_transform(t: &ufbx::Texture) -> Option<Value> {
    if !t.has_uv_transform {
        return None;
    }
    let m = &t.uv_to_texture;
    // M: u' = a u + b v + tx, v' = c u + d v + ty (FBX, v up).
    let (a, b, tx, c, d, ty) = (m.m00, m.m01, m.m03, m.m10, m.m11, m.m13);
    // F M F: u'' = a u + b (1 - v) + tx; v'' = 1 - (c u + d (1 - v) + ty).
    let (a2, b2, tx2) = (a, -b, b + tx);
    let (c2, d2, ty2) = (-c, d, 1.0 - d - ty);
    let sx = (a2 * a2 + c2 * c2).sqrt();
    if sx < 1e-12 {
        return None;
    }
    let (cos, sin) = (a2 / sx, -c2 / sx);
    let sy = b2 * sin + d2 * cos;
    let r = sin.atan2(cos);
    Some(json!({
        "offset": [tx2 as f32, ty2 as f32],
        "rotation": r as f32,
        "scale": [sx as f32, sy as f32],
    }))
}

impl Materials {
    /// The glTF material of `m`, written once.
    pub(super) fn material(
        &mut self,
        m: &ufbx::Material,
        dir: &Path,
        what: &str,
        read: &mut Vec<(String, Vec<u8>)>,
        notes: &mut Vec<String>,
    ) -> Result<usize, String> {
        if let Some(&i) = self.mat_of.get(&m.element.typed_id) {
            return Ok(i);
        }
        let pbr = &m.pbr;
        let f = &m.features;
        let name = m.element.name.to_string();
        let at = format!("{what}: material '{name}'");
        let mut out = json!({ "name": name, "pbrMetallicRoughness": {} });

        // Base colour and opacity.
        let base_tex = file_texture(&pbr.base_color, &at, notes);
        let opacity_tex = file_texture(&pbr.opacity, &at, notes);
        let factor = if pbr.base_factor.has_value {
            real(&pbr.base_factor)
        } else {
            1.0
        };
        let rgb = if base_tex.is_some() {
            DVec3::splat(factor)
        } else {
            rgb3(&pbr.base_color) * factor
        };
        let opacity = if opacity_tex.is_some() || !pbr.opacity.has_value {
            1.0
        } else {
            real(&pbr.opacity).clamp(0.0, 1.0)
        };
        out["pbrMetallicRoughness"]["baseColorFactor"] =
            json!([rgb.x as f32, rgb.y as f32, rgb.z as f32, opacity as f32]);
        let same = |a: &ufbx::Texture, b: &ufbx::Texture| {
            a.element.typed_id == b.element.typed_id
                || (!a.filename.is_empty() && *a.filename == *b.filename)
        };
        let base_index = match (base_tex, opacity_tex) {
            (Some(b), Some(o)) if !same(b, o) => {
                let rgb_px = self.pixels(b, dir, &at, read)?;
                let a_px = self.pixels(o, dir, &at, read)?;
                let packed = pack(&rgb_px, |p, i| {
                    let a = sample(&a_px, p.width, p.height, i)[0];
                    let mut px = [p.rgba[i * 4], p.rgba[i * 4 + 1], p.rgba[i * 4 + 2], a];
                    px[3] = a;
                    px
                });
                Some(self.texture_packed(
                    format!("packed:{}+{}", b.element.typed_id, o.element.typed_id),
                    packed,
                    b,
                ))
            }
            (None, Some(o)) => {
                // Opacity alone: white with the map's red as alpha.
                let a_px = self.pixels(o, dir, &at, read)?;
                let packed = pack(&a_px, |p, i| [255, 255, 255, p.rgba[i * 4]]);
                Some(self.texture_packed(format!("alpha:{}", o.element.typed_id), packed, o))
            }
            (Some(b), _) => Some(self.texture(b, dir, &at, read)?),
            (None, None) => None,
        };
        if let Some(t) = base_index {
            out["pbrMetallicRoughness"]["baseColorTexture"] = t;
        }
        if opacity < 1.0 - 1e-6 || opacity_tex.is_some() {
            out["alphaMode"] = "BLEND".into();
        }

        // Roughness (glossiness inverted) and metalness.
        let mut gloss = f.roughness_as_glossiness.enabled;
        let mut rough_tex = file_texture(&pbr.roughness, &at, notes);
        let mut rough_value = real(&pbr.roughness);
        if rough_tex.is_none()
            && !pbr.roughness.has_value
            && (pbr.glossiness.has_value || pbr.glossiness.texture.is_some())
        {
            rough_tex = file_texture(&pbr.glossiness, &at, notes);
            rough_value = real(&pbr.glossiness);
            gloss = true;
        }
        if gloss {
            rough_value = 1.0 - rough_value;
        }
        let metal_tex = file_texture(&pbr.metalness, &at, notes);
        let metal_value = if f.metalness.enabled || pbr.metalness.has_value {
            real(&pbr.metalness)
        } else {
            0.0
        };
        out["pbrMetallicRoughness"]["roughnessFactor"] = (if rough_tex.is_some() {
            1.0
        } else {
            rough_value.clamp(0.0, 1.0) as f32
        })
        .into();
        out["pbrMetallicRoughness"]["metallicFactor"] = (if metal_tex.is_some() {
            1.0
        } else {
            metal_value.clamp(0.0, 1.0) as f32
        })
        .into();
        let mr = match (rough_tex, metal_tex) {
            (Some(r), Some(mt)) if same(r, mt) && !gloss => Some(self.texture(r, dir, &at, read)?),
            (None, None) => None,
            (r, mt) => {
                let r_px = r.map(|t| self.pixels(t, dir, &at, read)).transpose()?;
                let m_px = mt.map(|t| self.pixels(t, dir, &at, read)).transpose()?;
                let size = r_px.as_ref().or(m_px.as_ref()).unwrap();
                let (w, h) = (size.width, size.height);
                let shape = Pixels {
                    width: w,
                    height: h,
                    rgba: vec![0; (w * h * 4) as usize],
                };
                let packed = pack(&shape, |_, i| {
                    let g = r_px.as_ref().map_or(255, |p| {
                        let v = sample(p, w, h, i)[0];
                        if gloss { 255 - v } else { v }
                    });
                    let b = m_px.as_ref().map_or(255, |p| sample(p, w, h, i)[0]);
                    [255, g, b, 255]
                });
                let key = format!(
                    "mr:{}:{}:{gloss}",
                    r.map_or(0, |t| t.element.typed_id + 1),
                    mt.map_or(0, |t| t.element.typed_id + 1)
                );
                Some(self.texture_packed(key, packed, r.or(mt).unwrap()))
            }
        };
        if let Some(t) = mr {
            out["pbrMetallicRoughness"]["metallicRoughnessTexture"] = t;
        }

        // Normal map; a bump (height) map is not one.
        if let Some(t) = file_texture(&pbr.normal_map, &at, notes) {
            out["normalTexture"] = self.texture(t, dir, &at, read)?;
        } else if m.fbx.bump.texture.is_some() {
            notes.push(format!(
                "{at}: its bump map is a height map, which the engine does not draw; dropped (bake it to a normal map and connect that as the normal map)"
            ));
        }

        // Emission.
        let emission_tex = file_texture(&pbr.emission_color, &at, notes);
        let ef = if pbr.emission_factor.has_value {
            real(&pbr.emission_factor)
        } else {
            1.0
        };
        let e = if emission_tex.is_some() {
            DVec3::splat(ef)
        } else {
            rgb3(&pbr.emission_color) * ef
        };
        let peak = e.max_element();
        if peak > 1e-6 {
            let (e, strength) = if peak > 1.0 {
                (e / peak, peak)
            } else {
                (e, 1.0)
            };
            out["emissiveFactor"] = json!([e.x as f32, e.y as f32, e.z as f32]);
            if strength > 1.0 {
                out["extensions"]["KHR_materials_emissive_strength"] =
                    json!({ "emissiveStrength": strength as f32 });
            }
            if let Some(t) = emission_tex {
                out["emissiveTexture"] = self.texture(t, dir, &at, read)?;
            }
        }

        // Ambient occlusion.
        if let Some(t) = file_texture(&pbr.ambient_occlusion, &at, notes) {
            out["occlusionTexture"] = self.texture(t, dir, &at, read)?;
        }
        if f.double_sided.enabled {
            out["doubleSided"] = true.into();
        }
        if f.unlit.enabled {
            out["extensions"]["KHR_materials_unlit"] = json!({});
        }
        self.mats.push(out);
        self.mat_of.insert(m.element.typed_id, self.mats.len() - 1);
        Ok(self.mats.len() - 1)
    }

    /// A texture's image: embedded bytes, or the file beside the model.
    fn source(t: &ufbx::Texture, dir: &Path, at: &str) -> Result<(String, Source), String> {
        if !t.content.is_empty() {
            return Ok((
                format!(
                    "embedded:{}:{:016x}",
                    t.element.typed_id,
                    super::super::hash_bytes(&t.content)
                ),
                Source::Bytes(t.content.to_vec()),
            ));
        }
        let norm = |s: &str| s.replace('\\', "/");
        let rel = norm(&t.relative_filename);
        let abs = norm(&t.filename);
        let base = |s: &str| s.rsplit('/').next().unwrap_or("").to_string();
        let mut tried = Vec::new();
        for cand in [rel.clone(), base(&rel), base(&abs)] {
            if cand.is_empty() || tried.contains(&cand) {
                continue;
            }
            if dir.join(&cand).is_file() {
                return Ok((format!("file:{cand}"), Source::Uri(cand)));
            }
            tried.push(cand);
        }
        let name = if rel.is_empty() {
            abs.clone()
        } else {
            rel.clone()
        };
        Err(format!(
            "{at}: texture '{name}' is neither embedded nor beside the model (looked for {}); put the image there, or export the FBX with Embed Media on",
            tried.join(", ")
        ))
    }

    fn image(&mut self, key: String, src: Source) -> usize {
        if let Some(&i) = self.image_of.get(&key) {
            return i;
        }
        self.images.push((key.clone(), src));
        self.image_of.insert(key, self.images.len() - 1);
        self.images.len() - 1
    }

    fn texture_json(&mut self, image: usize, t: &ufbx::Texture) -> Value {
        let s = (wrap(t.wrap_u), wrap(t.wrap_v));
        let sampler = *self.sampler_of.entry(s).or_insert_with(|| {
            self.samplers.push(json!({ "wrapS": s.0, "wrapT": s.1 }));
            self.samplers.len() - 1
        });
        let tf = uv_transform(t);
        let key = (image, sampler, String::new());
        let index = *self.texture_of.entry(key).or_insert_with(|| {
            self.textures
                .push(json!({ "source": image, "sampler": sampler }));
            self.textures.len() - 1
        });
        let mut info = json!({ "index": index });
        if let Some(tf) = tf {
            info["extensions"] = json!({ "KHR_texture_transform": tf });
        }
        info
    }

    fn texture(
        &mut self,
        t: &ufbx::Texture,
        dir: &Path,
        at: &str,
        _read: &mut Vec<(String, Vec<u8>)>,
    ) -> Result<Value, String> {
        let (key, src) = Self::source(t, dir, at)?;
        let image = self.image(key, src);
        Ok(self.texture_json(image, t))
    }

    /// A texture packed at cook time, sampled as `like` is.
    fn texture_packed(&mut self, key: String, png: Vec<u8>, like: &ufbx::Texture) -> Value {
        let image = self.image(key, Source::Bytes(png));
        self.texture_json(image, like)
    }

    /// A texture's pixels, its file recorded as read.
    fn pixels(
        &mut self,
        t: &ufbx::Texture,
        dir: &Path,
        at: &str,
        read: &mut Vec<(String, Vec<u8>)>,
    ) -> Result<Pixels, String> {
        let bytes = match Self::source(t, dir, at)?.1 {
            Source::Bytes(b) => b,
            Source::Uri(rel) => {
                let b = std::fs::read(dir.join(&rel)).map_err(|e| format!("{at}: {rel}: {e}"))?;
                read.push((rel, b.clone()));
                b
            }
        };
        let img = super::super::texture::decode(&bytes)
            .map_err(|e| format!("{at}: texture '{}': {e}", t.relative_filename))?;
        Ok(Pixels {
            width: img.width,
            height: img.height,
            rgba: img.rgba,
        })
    }

    /// Writes the embedded and packed images into the glb's buffer.
    pub(super) fn finish(
        self,
        bin: &mut Bin,
        read: Vec<(String, Vec<u8>)>,
    ) -> Result<Finished, String> {
        let mut images = Vec::new();
        for (_, src) in self.images {
            images.push(match src {
                Source::Uri(rel) => json!({ "uri": rel.replace('%', "%25") }),
                Source::Bytes(bytes) => {
                    let mime = if bytes.starts_with(&[0xFF, 0xD8]) {
                        "image/jpeg"
                    } else {
                        "image/png"
                    };
                    json!({ "bufferView": bin.view(&bytes), "mimeType": mime })
                }
            });
        }
        Ok((self.mats, images, self.textures, self.samplers, read))
    }
}

/// The pixel of `p` under pixel `i` of a `w` x `h` image (nearest, when sizes differ).
fn sample(p: &Pixels, w: u32, h: u32, i: usize) -> [u8; 4] {
    let (x, y) = ((i as u32) % w, (i as u32) / w);
    let sx = ((x as u64 * p.width as u64) / w as u64) as u32;
    let sy = ((y as u64 * p.height as u64) / h as u64) as u32;
    let j = ((sy * p.width + sx) * 4) as usize;
    [p.rgba[j], p.rgba[j + 1], p.rgba[j + 2], p.rgba[j + 3]]
}

/// A PNG of `shape`'s size, each pixel from `f(shape, index)`.
fn pack(shape: &Pixels, f: impl Fn(&Pixels, usize) -> [u8; 4]) -> Vec<u8> {
    let n = (shape.width * shape.height) as usize;
    let mut rgba = Vec::with_capacity(n * 4);
    for i in 0..n {
        rgba.extend(f(shape, i));
    }
    let img = image::RgbaImage::from_raw(shape.width, shape.height, rgba).expect("sized");
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encodes");
    out
}
