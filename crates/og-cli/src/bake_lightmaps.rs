//! `og bake lightmaps [project]` (docs/design.md §2.6): the baked lighting
//! of every scene that has static models or light volumes.
//!
//! The project is cooked first; then, per such scene, its cooked entities
//! are placed in world space (Transform and parent), every model entity's
//! parts (the runtime's own `CookedModel::add_to_with`, so the parts and
//! their vertices are the ones it draws) become og-bake meshes, those of
//! entities with `static true` lightmapped; the scene's directional, point,
//! spot and rect lights become the tracer's. xatlas charts the lightmapped
//! meshes in world space (one texel density across the scene), og-bake
//! traces the light arriving at each texel (bounces, sky, emissive; not the
//! lights' direct term, which the renderer draws) and at each LightVolume's
//! probes, and the result is written beside the scene source:
//! `<scene>.lightmap.ogt` and `<scene>.lightmap.ktx2`. The project is cooked
//! again so the bundle holds them.
//!
//! The bake names the cooked scene's and models' hashes; a scene or model
//! changed since reads as stale at run time (`og.render.bakedLighting`) and
//! in `og doctor --project` ([`status`]).

use glam::{Mat4, Quat, Vec2, Vec3};
use og_bake::{Light, Material, Mesh, PackSettings, Scene, TraceSettings, UnwrapMesh, VolumeDesc};
use og_cook::lightmap::{BakedModel, BakedPart, BakedVolume, LightmapFile};
use og_cook::runtime::{Bundle, ComponentTable, SceneDesc};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage: og bake lightmaps [project] [--samples N] [--bounces N] [--texels-per-metre N] [--max-size N] [--sky R,G,B] [--seconds S] [--threads N] [--scene <id>]";

struct Options {
    project: String,
    samples: u32,
    bounces: u32,
    texels: f32,
    max_size: u32,
    sky: Vec3,
    seconds: f32,
    threads: usize,
    scene: Option<String>,
}

pub fn run(args: &[&str]) -> ExitCode {
    let Some((&"lightmaps", rest)) = args.split_first() else {
        eprintln!("og bake: what to bake? {USAGE}");
        return ExitCode::from(2);
    };
    let mut o = Options {
        project: ".".into(),
        samples: 256,
        bounces: 3,
        texels: 16.0,
        max_size: 1024,
        sky: Vec3::new(0.25, 0.3, 0.38),
        seconds: 0.0,
        threads: 0,
        scene: None,
    };
    let mut i = 0;
    while i < rest.len() {
        let a = rest[i];
        let flag = a.starts_with("--");
        let value = if flag { rest.get(i + 1).copied() } else { None };
        if flag && value.is_none() {
            eprintln!("og bake lightmaps: {a} takes a value. {USAGE}");
            return ExitCode::from(2);
        }
        let v = value.unwrap_or("");
        let ok = match a {
            "--samples" => v.parse().map(|n: u32| o.samples = n.max(1)).is_ok(),
            "--bounces" => v.parse().map(|n| o.bounces = n).is_ok(),
            "--texels-per-metre" => v.parse().map(|n| o.texels = n).is_ok(),
            "--max-size" => v.parse().map(|n| o.max_size = n).is_ok(),
            "--seconds" => v.parse().map(|n| o.seconds = n).is_ok(),
            "--threads" => v.parse().map(|n| o.threads = n).is_ok(),
            "--scene" => {
                o.scene = Some(v.to_string());
                true
            }
            "--sky" => {
                let c: Vec<f32> = v.split(',').filter_map(|x| x.parse().ok()).collect();
                match c.as_slice() {
                    [r, g, b] => {
                        o.sky = Vec3::new(*r, *g, *b);
                        true
                    }
                    _ => false,
                }
            }
            _ if a.starts_with('-') => {
                eprintln!("og bake lightmaps: unknown option `{a}`. {USAGE}");
                return ExitCode::from(2);
            }
            _ => {
                o.project = a.to_string();
                true
            }
        };
        if !ok {
            eprintln!("og bake lightmaps: bad value `{v}` for {a}. {USAGE}");
            return ExitCode::from(2);
        }
        i += if flag { 2 } else { 1 };
    }
    match bake(&o) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("og bake lightmaps: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The scenes of a cooked bundle that have static models or light volumes.
fn bakeable_scenes(bundle: &Bundle) -> Vec<(String, SceneDesc)> {
    bundle
        .manifest()
        .outputs()
        .filter(|o| o.ty == "og.Scene")
        .filter_map(|o| Some((o.id.clone(), bundle.load_scene(&o.id).ok()?)))
        .filter(|(_, s)| !statics(s).is_empty() || table(s, "LightVolume").is_some())
        .collect()
}

fn table<'a>(s: &'a SceneDesc, name: &str) -> Option<&'a ComponentTable> {
    s.components
        .iter()
        .find(|t| t.name.trim_start_matches("og.") == name)
}

/// A field of `entity`'s row as numbers (`None`: no row or no such field).
fn field(t: &ComponentTable, entity: usize, name: &str) -> Option<Vec<f32>> {
    use og_core::FieldType as F;
    let i = t.schema.fields.iter().position(|f| f.name == name)?;
    let ty = t.schema.fields[i].ty;
    let bytes = t.get(entity)?;
    let r = og_core::reflect::Layout::of(&t.schema).range(i);
    let b = bytes.get(r)?;
    let words = || b.as_chunks::<4>().0.iter().copied();
    Some(match ty {
        F::Bool => vec![b[0] as f32],
        F::U32 | F::Enum => words().map(|w| u32::from_le_bytes(w) as f32).collect(),
        F::I32 => words().map(|w| i32::from_le_bytes(w) as f32).collect(),
        F::F32 | F::Vec2 | F::Vec3 | F::Vec4 | F::Quat | F::Mat4 => {
            words().map(f32::from_le_bytes).collect()
        }
        F::F64 => b
            .as_chunks::<8>()
            .0
            .iter()
            .map(|w| f64::from_le_bytes(*w) as f32)
            .collect(),
        _ => return None,
    })
}

fn num(t: &ComponentTable, entity: usize, name: &str, default: f32) -> f32 {
    field(t, entity, name)
        .and_then(|v| v.first().copied())
        .unwrap_or(default)
}

fn vec3(t: &ComponentTable, entity: usize, name: &str, default: Vec3) -> Vec3 {
    match field(t, entity, name).as_deref() {
        Some([x, y, z, ..]) => Vec3::new(*x, *y, *z),
        _ => default,
    }
}

/// Entities whose model is marked `static true`.
fn statics(s: &SceneDesc) -> Vec<usize> {
    let Some(t) = table(s, "Model") else {
        return Vec::new();
    };
    (0..s.entities.len())
        .filter(|&e| s.entities[e].instance.is_some() && num(t, e, "static", 0.0) != 0.0)
        .collect()
}

/// Every entity's world transform (parents precede children).
fn world_transforms(s: &SceneDesc) -> Vec<Mat4> {
    let t = table(s, "Transform");
    let mut out: Vec<Mat4> = Vec::with_capacity(s.entities.len());
    for (i, e) in s.entities.iter().enumerate() {
        let local = match t {
            Some(t) if t.get(i).is_some() => {
                let q = field(t, i, "quaternion").unwrap_or(vec![0.0, 0.0, 0.0, 1.0]);
                let q = Quat::from_xyzw(q[0], q[1], q[2], q[3]).normalize();
                Mat4::from_scale_rotation_translation(
                    vec3(t, i, "scale", Vec3::ONE),
                    if q.is_finite() { q } else { Quat::IDENTITY },
                    vec3(t, i, "position", Vec3::ZERO),
                )
            }
            _ => Mat4::IDENTITY,
        };
        let parent = e.parent.and_then(|p| out.get(p).copied());
        out.push(parent.map_or(local, |p| p * local));
    }
    out
}

/// The scene's lights as the tracer's.
fn lights(s: &SceneDesc, world: &[Mat4]) -> Vec<Light> {
    let mut out = Vec::new();
    let forward = |m: &Mat4| m.transform_vector3(Vec3::NEG_Z).normalize_or(Vec3::NEG_Y);
    let radiance = |t: &ComponentTable, e: usize, intensity: f32| {
        vec3(t, e, "color", Vec3::ONE) * num(t, e, "intensity", intensity)
    };
    for (e, m) in world.iter().enumerate() {
        let at = m.transform_point3(Vec3::ZERO);
        if let Some(t) = table(s, "DirectionalLight").filter(|t| t.get(e).is_some()) {
            out.push(Light::Directional {
                towards: -forward(m),
                radiance: radiance(t, e, 3.0),
            });
        }
        if let Some(t) = table(s, "PointLight").filter(|t| t.get(e).is_some()) {
            out.push(Light::Point {
                position: at,
                radiance: radiance(t, e, 10.0),
                range: num(t, e, "distance", 10.0),
            });
        }
        if let Some(t) = table(s, "SpotLight").filter(|t| t.get(e).is_some()) {
            let angle = num(t, e, "angle", std::f32::consts::FRAC_PI_6);
            let penumbra = num(t, e, "penumbra", 0.0).clamp(0.0, 1.0);
            out.push(Light::Spot {
                position: at,
                direction: forward(m),
                radiance: radiance(t, e, 10.0),
                range: num(t, e, "distance", 10.0),
                cos_outer: angle.cos(),
                cos_inner: (angle * (1.0 - penumbra)).cos(),
            });
        }
        if let Some(t) = table(s, "RectLight").filter(|t| t.get(e).is_some()) {
            out.push(Light::Rect {
                position: at,
                half_x: m.transform_vector3(Vec3::X) * num(t, e, "width", 10.0) * 0.5,
                half_y: m.transform_vector3(Vec3::Y) * num(t, e, "height", 10.0) * 0.5,
                radiance: radiance(t, e, 1.0),
                two_sided: num(t, e, "twoSided", 0.0) != 0.0,
            });
        }
    }
    out
}

/// A model's parts as the runtime makes them, and their meshes.
fn model_parts(
    bundle: &Bundle,
    asset: &str,
) -> Result<(og_render::RenderScene, og_cook::runtime::ModelParts), String> {
    let (model, textures) = bundle
        .load_model_with_textures(asset)
        .map_err(|e| format!("{asset}: {e}"))?;
    let mut scene = og_render::RenderScene::new();
    let sources: Vec<og_render::TextureSource> = textures
        .into_iter()
        .map(og_render::TextureSource::Rgba)
        .collect();
    let parts = model.add_to_with(&mut scene, &sources);
    Ok((scene, parts))
}

/// Why each bakeable scene of the cooked project at `root` draws unbaked
/// (`og doctor --project`): no bake, or one whose scene or static models
/// changed since. Scenes whose bake matches are left out.
pub fn status(root: &Path) -> Vec<(String, String)> {
    let Ok(bundle) = Bundle::open(&root.join("cooked")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (id, _) in bakeable_scenes(&bundle) {
        let lm_id = og_cook::lightmap::lightmap_id(&id);
        let reason = match bundle.load_lightmap(&lm_id) {
            Err(_) => Some(format!(
                "{id} has static models or light volumes and no bake"
            )),
            Ok(lm) => {
                let hash = |a: &str| bundle.output(a).map(|o| o.hash).unwrap_or(0);
                if lm.file.scene_hash != hash(&id) {
                    Some(format!("{id} changed since its bake"))
                } else {
                    lm.file
                        .models
                        .iter()
                        .find(|m| m.hash != hash(&m.asset))
                        .map(|m| {
                            format!("{id}: its static model {} changed since its bake", m.asset)
                        })
                }
            }
        };
        if let Some(r) = reason {
            out.push((id, r));
        }
    }
    out
}

fn bake(o: &Options) -> Result<(), String> {
    let project =
        crate::project::Project::open(Path::new(&o.project)).map_err(|e| e.to_string())?;
    let root = project.root.clone();
    crate::cook::cook(&root, o.threads)?;
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let scenes: Vec<_> = bakeable_scenes(&bundle)
        .into_iter()
        .filter(|(id, _)| o.scene.as_ref().is_none_or(|s| s == id))
        .collect();
    if scenes.is_empty() {
        println!(
            "og bake lightmaps: no scene of {} has a static model or a LightVolume",
            root.display()
        );
        return Ok(());
    }
    let started = Instant::now();
    for (id, s) in &scenes {
        bake_scene(o, &root, &bundle, id, s)?;
    }
    crate::cook::cook(&root, o.threads)?;
    println!(
        "og bake lightmaps: {} scene(s) in {:.1} s",
        scenes.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(())
}

/// A static model entity: its index, its asset and per part its og-bake
/// mesh and its vertex count.
type StaticModel = (usize, String, Vec<(usize, usize)>);

fn bake_scene(
    o: &Options,
    root: &Path,
    bundle: &Bundle,
    id: &str,
    s: &SceneDesc,
) -> Result<(), String> {
    let started = Instant::now();
    let world = world_transforms(s);
    let statics = statics(s);
    let mut scene = Scene {
        lights: lights(s, &world),
        sky: o.sky,
        ..Default::default()
    };
    // Per static entity: (entity, asset, per part: scene mesh index, vertex count).
    let mut baked_models: Vec<StaticModel> = Vec::new();
    for (e, ent) in s.entities.iter().enumerate() {
        let Some(asset) = &ent.instance else { continue };
        let (rscene, parts) = model_parts(bundle, asset)?;
        let lightmapped = statics.contains(&e);
        let mut entry = Vec::new();
        for &(mesh, material, local) in &parts.parts {
            let data = rscene.mesh(mesh).ok_or("a model part without its mesh")?;
            let m = world[e] * local;
            let mat = rscene.standard_material(material);
            scene.materials.push(match mat {
                Some(sm) => Material {
                    base_color: sm.color,
                    metallic: sm.metalness,
                    emissive: sm.emissive * sm.emissiveIntensity,
                    double_sided: sm.side == 2,
                    ..Default::default()
                },
                None => Material::default(),
            });
            let normal = m.inverse().transpose();
            scene.meshes.push(Mesh {
                positions: data
                    .vertices
                    .iter()
                    .map(|v| m.transform_point3(Vec3::from(v.position)))
                    .collect(),
                normals: data
                    .vertices
                    .iter()
                    .map(|v| {
                        normal
                            .transform_vector3(Vec3::from(v.normal))
                            .normalize_or(Vec3::Y)
                    })
                    .collect(),
                uvs: data.vertices.iter().map(|v| Vec2::from(v.uv)).collect(),
                colors: data
                    .vertices
                    .iter()
                    .map(|v| Vec3::new(v.color[0], v.color[1], v.color[2]))
                    .collect(),
                indices: data.indices.clone(),
                material: scene.materials.len() - 1,
                lightmapped,
            });
            entry.push((scene.meshes.len() - 1, data.vertices.len()));
        }
        if lightmapped {
            baked_models.push((e, asset.clone(), entry));
        }
    }
    let lit: Vec<usize> = baked_models
        .iter()
        .flat_map(|(_, _, p)| p.iter().map(|(m, _)| *m))
        .collect();
    let mut file = LightmapFile {
        scene: id.to_string(),
        scene_hash: bundle.output(id).map(|o| o.hash).unwrap_or(0),
        pages: og_cook::lightmap::pages_name(id),
        ..Default::default()
    };
    let mut pages_out: Vec<Vec<u16>> = Vec::new();
    let (mut width, mut height) = (1, 1);
    if !lit.is_empty() {
        let unwrap: Vec<UnwrapMesh> = lit
            .iter()
            .map(|&m| UnwrapMesh {
                positions: scene.meshes[m]
                    .positions
                    .iter()
                    .map(|p| p.to_array())
                    .collect(),
                normals: scene.meshes[m]
                    .normals
                    .iter()
                    .map(|n| n.to_array())
                    .collect(),
                indices: scene.meshes[m].indices.clone(),
            })
            .collect();
        let atlas = og_bake::unwrap(
            &unwrap,
            &PackSettings {
                texels_per_unit: o.texels,
                max_size: o.max_size,
                ..Default::default()
            },
        )?;
        let settings = TraceSettings {
            samples: o.samples,
            bounces: o.bounces,
            seconds: o.seconds,
            threads: o.threads,
            ..Default::default()
        };
        let lightmap =
            og_bake::lightmap::bake(&scene, &atlas, &lit, &settings, &mut |done, total, secs| {
                eprint!("\r{id}: {done}/{total} samples, {secs:.1} s   ");
            });
        eprintln!();
        file.samples = lightmap.samples;
        width = lightmap.width;
        height = lightmap.height;
        pages_out = lightmap
            .pages
            .iter()
            .map(|p| {
                p.iter()
                    .flat_map(|t| [t[0], t[1], t[2], 1.0].map(og_cook::lightmap::f32_to_f16))
                    .collect()
            })
            .collect();
        let mut k = 0;
        for (e, asset, parts) in &baked_models {
            let mut m = BakedModel {
                entity: s.entities[*e].name.clone(),
                asset: asset.clone(),
                hash: bundle.output(asset).map(|o| o.hash).unwrap_or(0),
                parts: Vec::new(),
            };
            if m.entity.is_empty() {
                return Err(format!(
                    "{id}: a static model entity needs a name (entity \"<name>\" {{ ... }}) so its lightmap finds it"
                ));
            }
            for &(_, vertices) in parts {
                let a = &atlas.meshes[k];
                k += 1;
                m.parts.push(BakedPart {
                    vertices: vertices as u32,
                    xref: a.vertices.iter().map(|v| v.xref).collect(),
                    indices: a.indices.clone(),
                    uv: a
                        .vertices
                        .iter()
                        .map(|v| {
                            [
                                v.uv[0] / atlas.width as f32,
                                v.uv[1] / atlas.height as f32,
                                v.page as f32,
                            ]
                        })
                        .collect(),
                });
            }
            file.models.push(m);
        }
    }
    if let Some(t) = table(s, "LightVolume") {
        for (e, m) in world.iter().enumerate() {
            if t.get(e).is_none() {
                continue;
            }
            let center = m.transform_point3(Vec3::ZERO);
            let half = vec3(t, e, "size", Vec3::splat(4.0)).abs() * 0.5;
            let desc = VolumeDesc {
                min: center - half,
                max: center + half,
                spacing: num(t, e, "spacing", 1.0).max(0.1),
            };
            let v =
                og_bake::volume::bake(&scene, &desc, o.samples.max(64), o.bounces, 1, o.threads);
            file.volumes.push(BakedVolume {
                entity: s.entities[e].name.clone(),
                min: v.min.to_array(),
                max: v.max.to_array(),
                counts: v.counts,
                probes: v.probes,
            });
        }
    }
    file.seconds = (started.elapsed().as_secs_f32() * 10.0).round() / 10.0;
    // Beside the scene's source.
    let source = root.join("assets").join(id);
    let dir = source.parent().ok_or("a scene without a folder")?;
    if pages_out.is_empty() {
        pages_out.push(vec![0; 4]);
    }
    let ktx2 = og_cook::lightmap::pages_to_ktx2(width, height, &pages_out);
    std::fs::write(dir.join(&file.pages), ktx2).map_err(|e| e.to_string())?;
    let text_path = root.join("assets").join(og_cook::lightmap::lightmap_id(id));
    std::fs::write(&text_path, file.to_text()).map_err(|e| e.to_string())?;
    println!(
        "{id}: {} static model(s), {} page(s) of {width}x{height}, {} volume(s), {} samples, {:.1} s -> {}",
        file.models.len(),
        if lit.is_empty() { 0 } else { pages_out.len() },
        file.volumes.len(),
        file.samples,
        file.seconds,
        text_path.display()
    );
    Ok(())
}
