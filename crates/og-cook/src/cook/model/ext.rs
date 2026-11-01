//! The glTF extensions the model blob carries beside its geometry, read
//! from the plain JSON ([`super::GltfSource::json`]):
//! `KHR_texture_transform` (baked into a primitive's uvs where every
//! texture it can be drawn with shares one transform, carried per slot
//! into the material and the shaders otherwise), `KHR_materials_unlit`,
//! `KHR_lights_punctual`, `EXT_mesh_gpu_instancing` and
//! `KHR_materials_variants`.

use super::GltfSource;
use glam::{Mat4, Quat, Vec3};
use og_render::UvTransform;
use serde_json::Value;

fn f32_at(v: &Value, key: &str, d: f32) -> f32 {
    v.get(key).and_then(Value::as_f64).map_or(d, |x| x as f32)
}

fn vec2_at(v: &Value, key: &str, d: [f32; 2]) -> glam::Vec2 {
    let a = v.get(key).and_then(Value::as_array);
    let at = |i: usize| {
        a.and_then(|a| a.get(i))
            .and_then(Value::as_f64)
            .map_or(d[i], |x| x as f32)
    };
    glam::Vec2::new(at(0), at(1))
}

/// A texture info's `KHR_texture_transform` (identity without one) and
/// the `texCoord` it overrides, if any.
pub fn transform(info: &Value) -> (UvTransform, Option<u64>) {
    let Some(t) = info.pointer("/extensions/KHR_texture_transform") else {
        return (UvTransform::IDENTITY, None);
    };
    (
        UvTransform {
            offset: vec2_at(t, "offset", [0.0, 0.0]),
            scale: vec2_at(t, "scale", [1.0, 1.0]),
            rotation: f32_at(t, "rotation", 0.0),
        },
        t.get("texCoord").and_then(Value::as_u64),
    )
}

/// The texture infos of a material's physical extensions (the textures
/// og-render packs into one array, sampled at one uv transform).
const PHYSICAL: &[(&str, &str)] = &[
    ("KHR_materials_clearcoat", "clearcoatTexture"),
    ("KHR_materials_clearcoat", "clearcoatRoughnessTexture"),
    ("KHR_materials_clearcoat", "clearcoatNormalTexture"),
    ("KHR_materials_sheen", "sheenColorTexture"),
    ("KHR_materials_sheen", "sheenRoughnessTexture"),
    ("KHR_materials_transmission", "transmissionTexture"),
    ("KHR_materials_volume", "thicknessTexture"),
    ("KHR_materials_iridescence", "iridescenceTexture"),
    ("KHR_materials_iridescence", "iridescenceThicknessTexture"),
    ("KHR_materials_anisotropy", "anisotropyTexture"),
];

/// One material's textured slots and their transforms.
#[derive(Clone, Debug, Default)]
pub struct MaterialUv {
    /// og-render's slots ([`UvTransform::MAP`] and on): the transform of
    /// the slot's texture, `None` without a texture there.
    pub slots: [Option<UvTransform>; 5],
    /// The physical extensions' textures' transforms.
    pub physical: Vec<UvTransform>,
}

impl MaterialUv {
    fn of(m: &Value) -> MaterialUv {
        let pbr = m.get("pbrMetallicRoughness");
        let info = |v: Option<&Value>| {
            v.filter(|v| v.get("index").is_some())
                .map(|v| transform(v).0)
        };
        let get = |k: &str| info(pbr.and_then(|p| p.get(k)));
        MaterialUv {
            slots: [
                get("baseColorTexture"),
                get("metallicRoughnessTexture"),
                info(m.get("normalTexture")),
                info(m.get("emissiveTexture")),
                info(m.get("occlusionTexture")),
            ],
            physical: PHYSICAL
                .iter()
                .filter_map(|(e, t)| info(m.pointer(&format!("/extensions/{e}/{t}"))))
                .collect(),
        }
    }

    /// Every texture's transform.
    fn all(&self) -> impl Iterator<Item = UvTransform> + '_ {
        self.slots.iter().flatten().chain(&self.physical).copied()
    }

    /// The transform every texture shares (`None`: no texture, or two
    /// that differ: [`MaterialUv::mixed`]).
    fn common(&self) -> Option<UvTransform> {
        let mut all = self.all();
        let first = all.next()?;
        all.all(|t| t == first).then_some(first)
    }

    fn textured(&self) -> bool {
        self.all().next().is_some()
    }

    /// The transforms the renderer draws the material with when they are
    /// not baked: each slot's, and a physical material's packed maps
    /// (metallic-roughness, occlusion and the physical ones, sampled at
    /// the metallic-roughness slot's) at the first of theirs; true when
    /// the packed maps disagree.
    fn carried(&self, physical: bool) -> ([UvTransform; 5], bool) {
        let mut out = self.slots.map(|t| t.unwrap_or(UvTransform::IDENTITY));
        if !physical {
            return (out, false);
        }
        let packed: Vec<UvTransform> = [self.slots[1], self.slots[4]]
            .into_iter()
            .flatten()
            .chain(self.physical.iter().copied())
            .collect();
        let Some(&first) = packed.first() else {
            return (out, false);
        };
        out[UvTransform::METALNESS_ROUGHNESS] = first;
        out[UvTransform::AO] = first;
        (out, packed.iter().any(|&t| t != first))
    }
}

/// How the model's texture transforms are cooked: per glTF material the
/// transforms its row carries (identity where they are baked), and per
/// glTF primitive the transform baked into its uvs.
#[derive(Clone, Debug, Default)]
pub struct UvPlan {
    pub materials: Vec<[UvTransform; 5]>,
    /// By (mesh, primitive) index.
    pub bake: std::collections::BTreeMap<(usize, usize), UvTransform>,
    pub notes: Vec<String>,
}

/// The materials a primitive can be drawn with: its own and its variants'
/// (`None`: glTF's default material, which has no texture).
fn primitive_materials(prim: &Value) -> Vec<Option<usize>> {
    let own = prim
        .get("material")
        .and_then(Value::as_u64)
        .map(|m| m as usize);
    let mut out = vec![own];
    for m in mappings(prim) {
        out.push(Some(m.0));
    }
    out
}

/// The plan: a material is baked when it has textures that all share one
/// transform and every primitive drawing it (with any variant) bakes; a
/// primitive bakes when every textured material it can be drawn with is
/// baked with the same transform. Materials stop being baked until both
/// hold.
pub fn uv_plan(src: &GltfSource) -> UvPlan {
    let json = &src.json;
    let mats: Vec<MaterialUv> = json
        .get("materials")
        .and_then(Value::as_array)
        .map(|a| a.iter().map(MaterialUv::of).collect())
        .unwrap_or_default();
    let mut baked: Vec<Option<UvTransform>> = mats.iter().map(MaterialUv::common).collect();
    let mut prims = Vec::new();
    for (mi, mesh) in json
        .get("meshes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        for (pi, prim) in mesh
            .get("primitives")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let has_uv = prim.pointer("/attributes/TEXCOORD_0").is_some();
            let textured: Vec<usize> = primitive_materials(prim)
                .into_iter()
                .flatten()
                .filter(|&m| mats.get(m).is_some_and(MaterialUv::textured))
                .collect();
            prims.push(((mi, pi), has_uv, textured));
        }
    }
    loop {
        let mut changed = false;
        for (_, has_uv, textured) in &prims {
            let first = textured.first().and_then(|&m| baked[m]);
            let bakes = *has_uv && first.is_some() && textured.iter().all(|&m| baked[m] == first);
            if !bakes {
                for &m in textured {
                    changed |= baked[m].take().is_some();
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut plan = UvPlan::default();
    for ((mi, pi), _, textured) in &prims {
        if let Some(t) = textured.first().and_then(|&m| baked[m])
            && !t.is_identity()
        {
            plan.bake.insert((*mi, *pi), t);
        }
    }
    let (mut n_baked, mut n_carried) = (0, 0);
    for (i, m) in mats.iter().enumerate() {
        let physical = json
            .pointer(&format!("/materials/{i}/extensions"))
            .and_then(Value::as_object)
            .is_some_and(|e| PHYSICAL.iter().any(|(x, _)| e.contains_key(*x)));
        if baked[i].is_some() {
            plan.materials.push([UvTransform::IDENTITY; 5]);
            n_baked += baked[i].is_some_and(|t| !t.is_identity()) as usize;
            continue;
        }
        let (t, mixed) = m.carried(physical);
        if t.iter().any(|t| !t.is_identity()) {
            n_carried += 1;
        }
        if mixed {
            let name = json
                .pointer(&format!("/materials/{i}/name"))
                .and_then(Value::as_str)
                .unwrap_or("");
            plan.notes.push(format!(
                "material '{name}': its metallic-roughness, occlusion and physical maps carry different KHR_texture_transforms; the renderer packs them into one texture and samples it at the first's"
            ));
        }
        plan.materials.push(t);
    }
    if n_baked + n_carried > 0 {
        plan.notes.push(format!(
            "KHR_texture_transform: {n_baked} material(s) baked into their primitives' uvs, {n_carried} carried to the renderer per texture slot"
        ));
    }
    // texCoord overrides: only TEXCOORD_0 is cooked.
    for (i, m) in json
        .get("materials")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let mut found = false;
        let mut visit = |v: &Value| {
            if let Some(o) = v.pointer("/extensions/KHR_texture_transform/texCoord")
                && o.as_u64() != Some(0)
            {
                found = true;
            }
        };
        walk_infos(m, &mut visit);
        if found {
            plan.notes.push(format!(
                "material {i}: a KHR_texture_transform names another texCoord; only TEXCOORD_0 is cooked"
            ));
        }
    }
    plan
}

/// Every texture info under a material (an object with an `index`).
fn walk_infos(v: &Value, f: &mut impl FnMut(&Value)) {
    match v {
        Value::Object(o) => {
            if o.contains_key("index") {
                f(v);
            }
            for x in o.values() {
                walk_infos(x, f);
            }
        }
        Value::Array(a) => a.iter().for_each(|x| walk_infos(x, f)),
        _ => {}
    }
}

/// Whether glTF material `i` has `KHR_materials_unlit`.
pub fn unlit(json: &Value, i: usize) -> bool {
    json.pointer(&format!("/materials/{i}/extensions/KHR_materials_unlit"))
        .is_some()
}

/// A `KHR_lights_punctual` light: its kind ([`crate::format::light`]),
/// colour, intensity, range (0: infinite) and cone, by glTF node.
pub struct Light {
    pub node: usize,
    pub name: String,
    pub kind: u32,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub cone: [f32; 2],
}

pub fn lights(json: &Value, notes: &mut Vec<String>) -> Vec<Light> {
    let defs = json
        .pointer("/extensions/KHR_lights_punctual/lights")
        .and_then(Value::as_array);
    let mut out = Vec::new();
    for (n, node) in json
        .get("nodes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let Some(l) = node
            .pointer("/extensions/KHR_lights_punctual/light")
            .and_then(Value::as_u64)
        else {
            continue;
        };
        let Some(d) = defs.and_then(|d| d.get(l as usize)) else {
            notes.push(format!(
                "node {n}: KHR_lights_punctual light {l} does not exist; skipped"
            ));
            continue;
        };
        let kind = match d.get("type").and_then(Value::as_str) {
            Some("directional") => crate::format::light::DIRECTIONAL,
            Some("point") => crate::format::light::POINT,
            Some("spot") => crate::format::light::SPOT,
            t => {
                notes.push(format!(
                    "node {n}: a light of type {t:?} is not punctual; skipped"
                ));
                continue;
            }
        };
        let c = d.get("color").and_then(Value::as_array);
        let ch = |i: usize| {
            c.and_then(|c| c.get(i))
                .and_then(Value::as_f64)
                .map_or(1.0, |x| x as f32)
        };
        let spot = d.get("spot");
        out.push(Light {
            node: n,
            name: d
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            kind,
            color: [ch(0), ch(1), ch(2)],
            intensity: f32_at(d, "intensity", 1.0),
            range: f32_at(d, "range", 0.0).max(0.0),
            cone: [
                spot.map_or(0.0, |s| f32_at(s, "innerConeAngle", 0.0)),
                spot.map_or(std::f32::consts::FRAC_PI_4, |s| {
                    f32_at(s, "outerConeAngle", std::f32::consts::FRAC_PI_4)
                }),
            ],
        });
    }
    out
}

/// `EXT_mesh_gpu_instancing`: per glTF node with the extension, its
/// instances' transforms in its space.
pub fn instances(src: &GltfSource) -> Result<std::collections::BTreeMap<usize, Vec<Mat4>>, String> {
    let json = &src.json;
    let mut out = std::collections::BTreeMap::new();
    for (n, node) in json
        .get("nodes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let Some(attrs) = node
            .pointer("/extensions/EXT_mesh_gpu_instancing/attributes")
            .and_then(Value::as_object)
        else {
            continue;
        };
        let read = |name: &str| -> Result<Option<(Vec<f32>, usize)>, String> {
            match attrs.get(name).and_then(Value::as_u64) {
                Some(a) => super::plain::read_accessor(json, &src.buffers, a as usize)
                    .map(Some)
                    .map_err(|e| format!("node {n}: EXT_mesh_gpu_instancing {name}: {e}")),
                None => Ok(None),
            }
        };
        let (t, r, s) = (read("TRANSLATION")?, read("ROTATION")?, read("SCALE")?);
        let count = [&t, &r, &s]
            .iter()
            .filter_map(|x| x.as_ref().map(|(v, k)| v.len() / k))
            .max()
            .unwrap_or(0);
        let at = |x: &Option<(Vec<f32>, usize)>, i: usize, d: &[f32]| -> Vec<f32> {
            match x {
                Some((v, k)) if v.len() >= (i + 1) * k => v[i * k..(i + 1) * k].to_vec(),
                _ => d.to_vec(),
            }
        };
        let mats = (0..count)
            .map(|i| {
                let t = at(&t, i, &[0.0; 3]);
                let r = at(&r, i, &[0.0, 0.0, 0.0, 1.0]);
                let s = at(&s, i, &[1.0; 3]);
                Mat4::from_scale_rotation_translation(
                    Vec3::new(s[0], s[1], s[2]),
                    Quat::from_xyzw(r[0], r[1], r[2], r[3]).normalize(),
                    Vec3::new(t[0], t[1], t[2]),
                )
            })
            .collect();
        out.insert(n, mats);
    }
    Ok(out)
}

/// `KHR_materials_variants`: the variants' names.
pub fn variants(json: &Value) -> Vec<String> {
    json.pointer("/extensions/KHR_materials_variants/variants")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|v| {
            v.get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        })
        .collect()
}

/// A primitive's variant mappings: (material, variant) pairs.
pub fn mappings(prim: &Value) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for m in prim
        .pointer("/extensions/KHR_materials_variants/mappings")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(material) = m.get("material").and_then(Value::as_u64) else {
            continue;
        };
        for v in m
            .get("variants")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_u64)
        {
            out.push((material as usize, v as usize));
        }
    }
    out
}
