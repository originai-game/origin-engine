//! `og bake lightmaps [project]` (docs/design.md §2.6): the baked lighting
//! of every scene that has static models or light volumes.
//!
//! The project is cooked first; then, per such scene, its cooked entities
//! are placed in world space (Transform and parent), every model entity's
//! parts (the runtime's own `CookedModel::add_to_with`, so the parts and
//! their vertices are the ones it draws) become og-bake meshes, those of
//! entities with `static true` lightmapped; the scene's directional, point,
//! spot and rect lights that are not movable (`og_render_data::light`, as
//! the frame draws them) become the tracer's. xatlas charts the lightmapped
//! meshes in world space (one texel density across the scene), og-bake
//! traces the light arriving at each texel (bounces, sky, emissive and the
//! static lights' direct light; not a stationary light's direct term,
//! which the renderer draws) and at each LightVolume's
//! probes, and writes a checked file.lightmap TS declaration under content/
//! with binary KTX2 pages under assets/ in the scene's module. The project is
//! cooked again so its manifest explicitly associates the scene and lighting.
//!
//! The bake names the scene source's hash and each static model's geometry
//! hash (`og_cook::lightmap::geometry_hash`); a scene or a model's geometry
//! changed since reads as stale at run time (`og.render.bakedLighting`) and
//! in `og doctor --project` ([`status`]). A re-cook that leaves them alone
//! (a cooker revision, the raycast BVH, a texture option) keeps the bake.

use glam::{Mat4, Quat, Vec2, Vec3};
use og_bake::{Material, Mesh, PackSettings, Scene, TraceSettings, UnwrapMesh, VolumeDesc};
use og_cook::lightmap::{BakedModel, BakedPart, BakedVolume, LightmapFile};
use og_cook::runtime::{Bundle, ComponentTable, SceneDesc};
use og_render_data::light::{DirectionalLight, LightDesc, LightKind, LightMobility};
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

/// A light row's mobility (`og.LightMobility`, the cooked variant index).
fn mobility(t: &ComponentTable, e: usize) -> LightMobility {
    match num(t, e, "mobility", 1.0) as u32 {
        0 => LightMobility::Movable,
        2 => LightMobility::Static,
        _ => LightMobility::Stationary,
    }
}

/// Whether the scene runs og.DayNight (`resource DayNight { enabled true }`):
/// the cycle moves the directional light, so none is baked.
fn day_night(s: &SceneDesc) -> bool {
    s.resources.iter().any(|r| {
        r.name.trim_start_matches("og.") == "DayNight"
            && r.schema
                .fields
                .iter()
                .position(|f| f.name == "enabled")
                .is_some_and(|i| {
                    let range = og_core::reflect::Layout::of(&r.schema).range(i);
                    r.bytes.get(range).is_some_and(|b| b.first() == Some(&1))
                })
    })
}

/// The scene's lights as the renderer's (`og_render_data::light`, read
/// from the cooked rows with the record's defaults), those whose light is
/// baked: no movable one. The directional lights are keyed as a frame
/// draws them (`og_render::key_light`: the brightest, two handing over);
/// under og.DayNight none is.
fn lights(s: &SceneDesc, world: &[Mat4]) -> (Vec<LightDesc>, Option<DirectionalLight>) {
    let mut out = Vec::new();
    let mut directional = Vec::new();
    let forward = |m: &Mat4| m.transform_vector3(Vec3::NEG_Z).normalize_or(Vec3::NEG_Y);
    let color = |t: &ComponentTable, e: usize| vec3(t, e, "color", Vec3::ONE);
    let defaults = LightDesc::default();
    for (e, m) in world.iter().enumerate() {
        let at = m.transform_point3(Vec3::ZERO);
        if let Some(t) = table(s, "DirectionalLight").filter(|t| t.get(e).is_some()) {
            directional.push(DirectionalLight {
                direction_to_light: -forward(m),
                color: color(t, e),
                intensity: num(t, e, "intensity", DirectionalLight::default().intensity),
                mobility: mobility(t, e),
                ..Default::default()
            });
        }
        for (name, spot) in [("PointLight", false), ("SpotLight", true)] {
            let Some(t) = table(s, name).filter(|t| t.get(e).is_some()) else {
                continue;
            };
            let kind = if spot {
                LightKind::Spot {
                    direction: forward(m),
                    angle_degrees: num(t, e, "angle", std::f32::consts::FRAC_PI_6).to_degrees(),
                    penumbra: num(t, e, "penumbra", 0.0),
                }
            } else {
                LightKind::Point
            };
            out.push(LightDesc {
                kind,
                position: at,
                color: color(t, e),
                intensity: num(t, e, "intensity", defaults.intensity),
                range: num(t, e, "distance", defaults.range),
                radius: num(t, e, "radius", 0.0),
                cast_shadows: num(t, e, "castShadow", 0.0) != 0.0,
                mobility: mobility(t, e),
            });
        }
        if let Some(t) = table(s, "RectLight").filter(|t| t.get(e).is_some()) {
            out.push(LightDesc::rect(
                at,
                m.transform_vector3(Vec3::X) * num(t, e, "width", 10.0).max(0.0) * 0.5,
                m.transform_vector3(Vec3::Y) * num(t, e, "height", 10.0).max(0.0) * 0.5,
                num(t, e, "twoSided", 0.0) != 0.0,
                color(t, e),
                num(t, e, "intensity", 1.0),
                num(t, e, "castShadow", 0.0) != 0.0,
                mobility(t, e),
            ));
        }
    }
    out.retain(|l| l.mobility.baked());
    let sun = og_render::key_light(&directional)
        .map(|k| k.light)
        .filter(|l| l.mobility.baked() && !day_night(s));
    (out, sun)
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
        let lm = bundle
            .manifest()
            .lightmaps
            .get(&id)
            .ok_or(())
            .and_then(|lm_id| bundle.load_lightmap(lm_id).map_err(|_| ()));
        let reason = match lm {
            Err(_) => Some(format!(
                "{id} has static models or light volumes and no bake"
            )),
            Ok(lm) => {
                if lm.file.scene_hash != scene_hash(&bundle, &id) {
                    Some(format!("{id} changed since its bake"))
                } else {
                    lm.file
                        .models
                        .iter()
                        .find(|m| bundle.model_geometry_hash(&m.asset).ok() != Some(m.hash))
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

/// The hash the bake keys the scene on: its source's (the text the author
/// edits), so a re-cook alone never stales it.
fn scene_hash(bundle: &Bundle, id: &str) -> u64 {
    bundle.asset(id).map(|a| a.source_hash).unwrap_or(0)
}

fn bake(o: &Options) -> Result<(), String> {
    let project =
        crate::project::Project::open(Path::new(&o.project)).map_err(|e| e.to_string())?;
    let root = project.root.clone();
    crate::cook::cook(&root, o.threads, false)?;
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
    let sources = BakeSources::open(&root)?;
    for (id, s) in &scenes {
        bake_scene(o, &root, &bundle, id, s, &sources)?;
    }
    crate::cook::cook(&root, o.threads, false)?;
    println!(
        "og bake lightmaps: {} scene(s) in {:.1} s",
        scenes.len(),
        started.elapsed().as_secs_f32()
    );
    Ok(())
}

/// A static model entity: its index, its asset, the asset's geometry hash
/// and per part its og-bake mesh and its vertex count.
type StaticModel = (usize, String, u64, Vec<(usize, usize)>);

fn bake_scene(
    o: &Options,
    root: &Path,
    bundle: &Bundle,
    id: &str,
    s: &SceneDesc,
    sources: &BakeSources,
) -> Result<(), String> {
    let started = Instant::now();
    let world = world_transforms(s);
    let statics = statics(s);
    let (lights, sun) = lights(s, &world);
    let mut scene = Scene {
        lights,
        sun,
        sky: o.sky,
        ..Default::default()
    };
    // Per static entity: (entity, asset, per part: scene mesh index, vertex count).
    let mut baked_models: Vec<StaticModel> = Vec::new();
    for (e, ent) in s.entities.iter().enumerate() {
        let Some(asset) = &ent.instance else { continue };
        let (rscene, parts) = model_parts(bundle, asset)?;
        let geometry = og_cook::lightmap::geometry_hash(
            parts.parts.iter().filter_map(|&(m, _, _)| rscene.mesh(m)),
        );
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
                ..Default::default()
            });
            entry.push((scene.meshes.len() - 1, data.vertices.len()));
        }
        if lightmapped {
            baked_models.push((e, asset.clone(), geometry, entry));
        }
    }
    let lit: Vec<usize> = baked_models
        .iter()
        .flat_map(|(_, _, _, p)| p.iter().map(|(m, _)| *m))
        .collect();
    let mut file = LightmapFile {
        scene: id.to_string(),
        scene_hash: scene_hash(bundle, id),
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
        // The scene's HLOD proxies, lit by what was just baked.
        let mut groups = vec![(atlas.width, atlas.height, lightmap.pages.clone())];
        let proxies = light_proxies(bundle, id, &scene, &lit, &atlas, &lightmap.pages)?;
        groups.extend(proxies.iter().map(|g| (g.width, g.height, g.pages.clone())));
        // Every page at one size: the largest group's, each padded.
        width = groups.iter().map(|g| g.0).max().unwrap_or(1);
        height = groups.iter().map(|g| g.1).max().unwrap_or(1);
        let scale = |w: u32, h: u32| [w as f32 / width as f32, h as f32 / height as f32];
        pages_out = groups
            .iter()
            .flat_map(|(w, h, pages)| pages.iter().map(|p| padded(p, *w, *h, width, height)))
            .map(|p| {
                p.iter()
                    .flat_map(|t| [t[0], t[1], t[2], 1.0].map(og_cook::lightmap::f32_to_f16))
                    .collect()
            })
            .collect();
        let model_scale = scale(atlas.width, atlas.height);
        let mut k = 0;
        for (e, asset, geometry, parts) in &baked_models {
            let mut m = BakedModel {
                entity: s.entities[*e].name.clone(),
                asset: asset.clone(),
                hash: *geometry,
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
                                v.uv[0] / atlas.width as f32 * model_scale[0],
                                v.uv[1] / atlas.height as f32 * model_scale[1],
                                v.page as f32,
                            ]
                        })
                        .collect(),
                });
            }
            file.models.push(m);
        }
        let mut first = lightmap.pages.len() as f32;
        for g in proxies {
            let [su, sv] = scale(g.width, g.height);
            for mut m in g.models {
                for part in &mut m.parts {
                    for uv in &mut part.uv {
                        *uv = [uv[0] * su, uv[1] * sv, uv[2] + first];
                    }
                }
                file.models.push(m);
            }
            first += g.pages.len() as f32;
        }
    }
    if let Some(t) = table(s, "LightVolume") {
        let shapes = table(s, "Volume");
        for (e, m) in world.iter().enumerate() {
            if t.get(e).is_none() {
                continue;
            }
            // Its box is the og.Volume on its entity, as the world's axes
            // bound it (the runtime's `present::probes`).
            let Some(shapes) = shapes.filter(|v| v.get(e).is_some()) else {
                eprintln!(
                    "og bake lightmaps: {id}: the LightVolume on '{}' has no og.Volume, so no box: nothing baked for it",
                    s.entities[e].name
                );
                continue;
            };
            let shape = og_env::volume::Volume {
                size: vec3(shapes, e, "size", Vec3::ONE),
                radius: num(shapes, e, "radius", 0.0),
                fade: num(shapes, e, "fade", 0.0),
            };
            let (scale, quaternion, position) = m.to_scale_rotation_translation();
            let (min, max) = shape.bounds(&og_env::volume::Placement {
                position,
                quaternion,
                scale,
            });
            let desc = VolumeDesc {
                min,
                max,
                spacing: num(t, e, "spacing", 1.0).max(0.1),
            };
            let v =
                og_bake::volume::bake(&scene, &desc, o.samples.max(64), o.bounces, 1, o.threads);
            file.volumes.push(BakedVolume {
                entity: s.entities[e].name.clone(),
                min: v.min.to_array(),
                max: v.max.to_array(),
                fade: shape.fade.max(0.0),
                counts: v.counts,
                probes: v.probes,
            });
        }
    }
    file.seconds = (started.elapsed().as_secs_f32() * 10.0).round() / 10.0;
    if pages_out.is_empty() {
        pages_out.push(vec![0; 4]);
    }
    let ktx2 = og_cook::lightmap::pages_to_ktx2(width, height, &pages_out);
    let text_path = write_source(root, bundle, &file, &ktx2, sources)?;
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

/// Capture bindings once for the whole bake, not once per scene in a world.
struct BakeSources {
    project: og_cook::cook::source_project::Project,
    index: og_cook::cook::source_index::Index,
    ids: std::collections::BTreeMap<String, String>,
}

impl BakeSources {
    fn open(root: &Path) -> Result<Self, String> {
        use og_cook::cook::{source_index::Index, source_project::Project};
        let project = Project::open(root).map_err(|e| e.to_string())?;
        let index = Index::open(root, &project).map_err(|e| e.to_string())?;
        let mut ids: std::collections::BTreeMap<_, _> =
            og_cook::cook::project_assets(root, &project)
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|(id, output)| (output.asset, id))
                .collect();
        ids.extend(index.entries.keys().map(|id| (id.clone(), id.clone())));
        Ok(Self {
            project,
            index,
            ids,
        })
    }
}

/// Resolve physical scanned IDs through the same catalogue as cook; authored
/// IDs are already canonical. Never infer source IDs from filename extensions.
fn write_source(
    root: &Path,
    bundle: &Bundle,
    file: &LightmapFile,
    pages: &[u8],
    sources: &BakeSources,
) -> Result<std::path::PathBuf, String> {
    use serde_json::json;
    let BakeSources {
        project,
        index,
        ids,
    } = sources;
    let canonical = |id: &str| {
        ids.get(id)
            .cloned()
            .ok_or_else(|| format!("{id}: no canonical source binding"))
    };
    let scene = canonical(&file.scene)?;
    let (scene_module, stem) = scene.split_once('/').ok_or("scene has no source module")?;
    let existing = bundle
        .manifest()
        .lightmaps
        .get(&file.scene)
        .map(|id| {
            index
                .entries
                .get(id)
                .ok_or("lightmap source absent from index")
        })
        .transpose()?;
    let module = existing.map_or(scene_module, |e| e.module.as_str());
    let module_root = root.join(&project.modules[module].root);
    let pages_stem = if module == scene_module { stem } else { &scene };
    let pages_path = format!("assets/lightmaps/{pages_stem}.pages.ktx2");
    let data_path = format!("assets/lightmaps/{pages_stem}.bindings.bin");
    let path = if let Some(existing) = existing {
        root.join(&existing.file)
    } else {
        let path = module_root.join(format!("content/lightmaps/{stem}.ts"));
        if path.exists()
            || module_root.join(&pages_path).exists()
            || module_root.join(&data_path).exists()
        {
            return Err(format!(
                "{}: bake output already exists without a scene association",
                path.display()
            ));
        }
        path
    };
    let tree_id = og_cook::hlod::hlod_id(&file.scene);
    let proxies: std::collections::BTreeMap<_, _> = bundle
        .read(&tree_id)
        .ok()
        .and_then(|bytes| bundle.hlod(&tree_id, &bytes).ok())
        .map(|t| {
            (0..t.nodes.len())
                .map(|n| (og_cook::hlod::node_id(&file.scene, n), n))
                .collect()
        })
        .unwrap_or_default();
    file.validate(og_cook::lightmap::pages_from_ktx2(pages)?.2.len())?;
    // Dense buffers must not consume the TS source/node budgets of a large
    // region. Explicit little-endian reads keep the generated source portable.
    let mut data = Vec::new();
    let mut buffer = |kind: &str, bytes: Vec<u8>| {
        let offset = data.len();
        let count = bytes.len() / 4;
        data.extend(bytes);
        format!("{kind}({offset}, {count})")
    };
    let mut models = Vec::new();
    for model in &file.models {
        let asset = if model.entity.is_empty() {
            let node = proxies
                .get(&model.asset)
                .ok_or_else(|| format!("{}: not a generated scene proxy", model.asset))?;
            json!({"hlodProxy":node})
        } else {
            json!({"$ogRef":canonical(&model.asset)?})
        };
        let parts: Vec<_> = model
            .parts
            .iter()
            .map(|p| {
                let xref = buffer("u32", p.xref.iter().flat_map(|v| v.to_le_bytes()).collect());
                let indices = buffer(
                    "u32",
                    p.indices.iter().flat_map(|v| v.to_le_bytes()).collect(),
                );
                let uv = buffer(
                    "f32",
                    p.uv.iter()
                        .flatten()
                        .flat_map(|v| v.to_le_bytes())
                        .collect(),
                );
                format!(
                    "{{vertices:{},xref:{xref},indices:{indices},uv:{uv}}}",
                    p.vertices
                )
            })
            .collect();
        models.push(format!(
            "{{entity:{},asset:{asset},hash:\"{:016x}\",parts:[{}]}}",
            json!(model.entity),
            model.hash,
            parts.join(",\n")
        ));
    }
    let volumes: Vec<_> = file
        .volumes
        .iter()
        .map(|v| {
            let probes = buffer(
                "f32",
                v.probes
                    .iter()
                    .flatten()
                    .flatten()
                    .flat_map(|v| v.to_le_bytes())
                    .collect(),
            );
            format!(
                "{{entity:{},min:{},max:{},fade:{},counts:{},probes:{probes}}}",
                json!(v.entity),
                json!(v.min),
                json!(v.max),
                v.fade,
                json!(v.counts)
            )
        })
        .collect();
    let text = format!(
        "// Written by og bake lightmaps; bake again after scene or geometry edits.
import {{file, input}} from '@origingame/engine/author';
const bytes = input.bytes({});
const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
const u32 = (offset: number, count: number) => Uint32Array.from({{length:count}}, (_, i) => view.getUint32(offset + i * 4, true));
const f32 = (offset: number, count: number) => Float32Array.from({{length:count}}, (_, i) => view.getFloat32(offset + i * 4, true));
export default file.lightmap({{
  scene: {}, sceneHash: \"{:016x}\", pages: {}, samples: {}, seconds: {},
  models: [{}],
  volumes: [{}]
}});
",
        json!(data_path), json!({"$ogRef":scene}), file.scene_hash, json!(pages_path),
        file.samples, file.seconds, models.join(",\n"), volumes.join(",\n")
    );
    let binary = module_root.join(&pages_path);
    std::fs::create_dir_all(binary.parent().ok_or("pages have no directory")?)
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(path.parent().ok_or("lightmap has no directory")?)
        .map_err(|e| e.to_string())?;
    std::fs::write(module_root.join(data_path), data).map_err(|e| e.to_string())?;
    std::fs::write(binary, pages).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok(path)
}

/// A lightmap's texel on a proxy covers this many times its node's error:
/// indirect light changes slowly, and a proxy is drawn where its error is
/// under a pixel, so a texel is a few pixels across.
const PROXY_TEXEL_ERRORS: f32 = 4.0;

/// A proxy in the scene's space, with what its entry names.
struct ProxyMesh {
    id: String,
    hash: u64,
    vertices: usize,
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    indices: Vec<u32>,
}

/// One level's proxies lit: their pages and their entries (uvs on their
/// own pages, 0..1 of `width` × `height`, page from 0).
struct LitProxies {
    width: u32,
    height: u32,
    pages: Vec<Vec<[f32; 4]>>,
    models: Vec<BakedModel>,
}

/// Scene `id`'s HLOD proxies (docs/design.md §10.6) lit by its static
/// models' bake: per level of the tree its proxies are charted at
/// [`PROXY_TEXEL_ERRORS`] of the level's error a texel, and each texel
/// takes the irradiance the models' lightmap holds where the proxy stands
/// for them (og-bake's `transfer`, reading the pages as a base colour map
/// through the models' lightmap uvs). Each proxy is an entry keyed by its
/// asset (`<scene>#hlod-<n>`, no entity), with its geometry hash, which
/// og-runtime's HLOD reads.
fn light_proxies(
    bundle: &Bundle,
    id: &str,
    scene: &Scene,
    lit: &[usize],
    atlas: &og_bake::Atlas,
    pages: &[Vec<[f32; 4]>],
) -> Result<Vec<LitProxies>, String> {
    use og_bake::transfer::{Mips, Source, Surface, Target, TransferSettings, transfer};
    let tree_id = og_cook::hlod::hlod_id(id);
    let Ok(bytes) = bundle.read(&tree_id) else {
        return Ok(Vec::new());
    };
    let tree = bundle
        .hlod(&tree_id, &bytes)
        .map_err(|e| format!("{tree_id}: {e}"))?;
    // The models as the transfer reads them: per lit mesh and page, its
    // triangles with their lightmap uvs, a surface per page.
    let images: Vec<Mips> = pages
        .iter()
        .map(|p| {
            Mips::new(og_bake::Image {
                width: atlas.width,
                height: atlas.height,
                texels: p.iter().map(|t| [t[0], t[1], t[2], 1.0]).collect(),
            })
        })
        .collect();
    let surfaces: Vec<Surface> = (0..pages.len())
        .map(|p| Surface {
            base_map: Some(p),
            ..Default::default()
        })
        .collect();
    let mut sources: Vec<Mesh> = Vec::new();
    for (k, &m) in lit.iter().enumerate() {
        let (mesh, charted) = (&scene.meshes[m], &atlas.meshes[k]);
        for page in 0..pages.len() as u32 {
            let indices: Vec<u32> = charted
                .indices
                .as_chunks::<3>()
                .0
                .iter()
                .filter(|t| charted.vertices[t[0] as usize].page == page)
                .flatten()
                .copied()
                .collect();
            if indices.is_empty() {
                continue;
            }
            let v = &charted.vertices;
            sources.push(Mesh {
                positions: v.iter().map(|v| mesh.positions[v.xref as usize]).collect(),
                normals: v.iter().map(|v| mesh.normals[v.xref as usize]).collect(),
                uvs: v
                    .iter()
                    .map(|v| Vec2::new(v.uv[0] / atlas.width as f32, v.uv[1] / atlas.height as f32))
                    .collect(),
                indices,
                material: page as usize,
                ..Default::default()
            });
        }
    }
    let refs: Vec<&Mips> = images.iter().collect();
    let source = Source {
        meshes: &sources,
        surfaces: &surfaces,
        images: &refs,
    };
    let mut out = Vec::new();
    let levels = tree
        .nodes
        .iter()
        .map(|n| n.level)
        .max()
        .map_or(0, |l| l + 1);
    for level in 0..levels {
        // A node over cells stands for other scenes' models, which this
        // bake did not light: it draws unbaked.
        let nodes: Vec<usize> = (0..tree.nodes.len())
            .filter(|&n| tree.nodes[n].level == level && tree.cells_under(n).next().is_none())
            .collect();
        if nodes.is_empty() {
            continue;
        }
        // Each node's proxy in the scene's space, and what its entry needs.
        let mut meshes: Vec<ProxyMesh> = Vec::new();
        for &n in &nodes {
            let pid = og_cook::hlod::node_id(id, n);
            let (rscene, parts) = model_parts(bundle, &pid)?;
            let hash = og_cook::lightmap::geometry_hash(
                parts.parts.iter().filter_map(|&(m, _, _)| rscene.mesh(m)),
            );
            let &[(mesh, _, local)] = parts.parts.as_slice() else {
                return Err(format!("{pid}: a proxy has one part"));
            };
            let data = rscene.mesh(mesh).ok_or("a proxy without its mesh")?;
            let normal = local.inverse().transpose();
            meshes.push(ProxyMesh {
                id: pid,
                hash,
                vertices: data.vertices.len(),
                positions: data
                    .vertices
                    .iter()
                    .map(|v| local.transform_point3(Vec3::from(v.position)))
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
                indices: data.indices.clone(),
            });
        }
        let error = nodes
            .iter()
            .map(|&n| tree.nodes[n].error)
            .fold(0.0f32, f32::max)
            .max(tree.error * (1u32 << level.min(20)) as f32);
        let texel = error * PROXY_TEXEL_ERRORS;
        let charted = og_bake::unwrap(
            &meshes
                .iter()
                .map(|m| UnwrapMesh {
                    positions: m.positions.iter().map(|p| p.to_array()).collect(),
                    normals: m.normals.iter().map(|n| n.to_array()).collect(),
                    indices: m.indices.clone(),
                })
                .collect::<Vec<_>>(),
            &PackSettings {
                texels_per_unit: 1.0 / texel,
                max_size: 1024,
                ..Default::default()
            },
        )?;
        let mut lit_pages = Vec::new();
        for page in 0..charted.pages {
            // Every proxy's triangles on this page, as one target.
            let (mut positions, mut normals, mut uvs, mut indices) =
                (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            for (m, a) in meshes.iter().zip(&charted.meshes) {
                let base = positions.len() as u32;
                for v in &a.vertices {
                    positions.push(m.positions[v.xref as usize]);
                    normals.push(m.normals[v.xref as usize]);
                    uvs.push(Vec2::from(v.uv));
                }
                indices.extend(
                    a.indices
                        .as_chunks::<3>()
                        .0
                        .iter()
                        .filter(|t| a.vertices[t[0] as usize].page == page)
                        .flatten()
                        .map(|&i| i + base),
                );
            }
            let drawn = transfer(
                source,
                Target {
                    positions: &positions,
                    normals: &normals,
                    tangents: &[],
                    uvs: &uvs,
                    indices: &indices,
                },
                &TransferSettings {
                    width: charted.width,
                    height: charted.height,
                    reach: error * 1.25 + texel * 0.5,
                    samples: 2,
                    texel,
                },
            );
            lit_pages.push(drawn.base);
        }
        let models = meshes
            .iter()
            .zip(&charted.meshes)
            .map(|(m, a)| BakedModel {
                entity: String::new(),
                asset: m.id.clone(),
                hash: m.hash,
                parts: vec![BakedPart {
                    vertices: m.vertices as u32,
                    xref: a.vertices.iter().map(|v| v.xref).collect(),
                    indices: a.indices.clone(),
                    uv: a
                        .vertices
                        .iter()
                        .map(|v| {
                            [
                                v.uv[0] / charted.width as f32,
                                v.uv[1] / charted.height as f32,
                                v.page as f32,
                            ]
                        })
                        .collect(),
                }],
            })
            .collect();
        out.push(LitProxies {
            width: charted.width,
            height: charted.height,
            pages: lit_pages,
            models,
        });
    }
    Ok(out)
}

/// `page`, `w` × `h` texels, at the top left of a `width` × `height` page
/// (the rest black).
fn padded(page: &[[f32; 4]], w: u32, h: u32, width: u32, height: u32) -> Vec<[f32; 4]> {
    let mut out = vec![[0.0; 4]; (width * height) as usize];
    for y in 0..h as usize {
        let row = &page[y * w as usize..(y + 1) * w as usize];
        out[y * width as usize..y * width as usize + w as usize].copy_from_slice(row);
    }
    out
}
