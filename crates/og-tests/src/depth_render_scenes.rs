//! The depth wave's render-lod scenes (docs/depth.md §6): mesh levels of
//! detail picked by projected error (`lod-levels`), a closed room whose
//! chrome sphere reflects a reflection probe (`probe-interior`), and
//! screen-space reflections on a wet floor (`ssr-wet-floor`).
//!
//! - `lod-levels`: the fx-showcase tree (3,160 triangles), cooked with the
//!   default `lods 1 0.5 0.25`, at 6, 30 and 70 m along a row seen from
//!   eye height. Each copy draws the level `og_render::lod::select` picks
//!   for its error projected to the 720 px view, tinted as `debugView lod`
//!   tints it: green the full mesh (near), yellow level 1, orange level 2.
//! - `probe-interior`: a closed 10 × 4 × 10 m room (red back wall, blue
//!   left, cream right and front, white ceiling, a dark floor) under the
//!   procedural sky, lit by two point lights, with a chrome sphere and a
//!   green block inside and a reflection probe filling the room. The
//!   sphere shows the room (the red wall behind the camera's view of it
//!   and the floor), not the sky: without the probe it reflects the sky's
//!   blue.
//! - `ssr-wet-floor` (the full tier): a mirror-smooth dark floor under a
//!   black background, a red sphere and a lit white panel standing on it,
//!   PostEffects.ssr on. The floor below each shows its reflection; with
//!   the environment black and no probe, nothing else puts it there.

use glam::{Mat4, Quat, Vec3};
use og_cook::runtime::Bundle;
use og_render::{
    Background, Camera, DirectionalLight, Exposure, InstanceFx, Light, LightKind, ProbeMode,
    ReflectionProbe, RenderScene, ScreenSpaceReflections, StandardMaterial, lod, shapes,
};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The project holding the tree (`assets/tree.glb`), cooked once per process.
fn cooked() -> Result<&'static PathBuf, String> {
    static ROOT: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("og-tests-lod-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let models = root.join("assets/models");
        std::fs::create_dir_all(&models).map_err(|e| format!("{}: {e}", models.display()))?;
        std::fs::write(models.join("tree.glb"), crate::scenes::asset("tree.glb")?)
            .map_err(|e| format!("tree.glb: {e}"))?;
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default())
            .map_err(|e| format!("cook: {e}"))?;
        if !report.ok() {
            return Err(format!("cook: {:?}", report.errors()));
        }
        Ok(root)
    })
    .as_ref()
    .map_err(Clone::clone)
}

/// The trees' distances along the row (m) and the camera of `lod-levels`.
pub const LOD_DISTANCES: [f32; 3] = [6.0, 30.0, 70.0];

fn lod_camera() -> Camera {
    Camera {
        position: Vec3::new(0.0, 1.7, 0.0),
        target: Vec3::new(0.0, 1.5, -20.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    }
}

/// `lod-levels` (module docs), with the level each tree draws and the
/// triangles of the three trees at those levels and at level 0.
pub type LodScene = (RenderScene, [usize; 3], (usize, usize));

/// `lod-levels`: [`LodScene`].
pub fn lod_scene() -> Result<LodScene, String> {
    let root = cooked()?;
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let model = bundle
        .load_model("models/tree.glb")
        .map_err(|e| e.to_string())?;
    let mut scene = RenderScene::new();
    let sun = Vec3::new(-0.4, 0.7, 0.5);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.6;
    scene.camera = lod_camera();
    let ground = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.3, 0.32, 0.28),
        roughness: 1.0,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(200.0, 1.0));
    scene.add_instance(floor, ground, Mat4::IDENTITY);
    let base = scene.mesh_count();
    let parts = model.add_to(&mut scene, &[]).parts;
    let levels = model.add_lod_levels(&mut scene);
    let prim_of =
        |mesh| (0..model.primitives.len()).find(|&p| scene.mesh_handle(base + p) == Some(mesh));
    let part_levels: Vec<Vec<(og_render::MeshHandle, f32)>> = parts
        .iter()
        .map(|&(m, _, _)| prim_of(m).map(|p| levels[p].clone()).unwrap_or_default())
        .collect();
    // The model's error per level: the largest of its parts' (the parts of
    // the tree share one node, unscaled).
    let n = part_levels.iter().map(|l| l.len()).max().unwrap_or(0);
    let errors: Vec<f32> = std::iter::once(0.0)
        .chain((0..n).map(|k| {
            part_levels
                .iter()
                .filter_map(|l| l.get(k).or(l.last()))
                .map(|x| x.1)
                .fold(0.0, f32::max)
        }))
        .collect();
    // The model's bounding sphere, over its primitives' level-0 vertices.
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for p in &model.primitives {
        for v in &p.vertices {
            lo = lo.min(Vec3::from(v.position));
            hi = hi.max(Vec3::from(v.position));
        }
    }
    let (center, radius) = ((lo + hi) * 0.5, (hi - lo).length() * 0.5);
    let tris = |level: usize| -> usize {
        model
            .primitives
            .iter()
            .map(|p| p.triangle_count(level.min(p.lods.len() - 1)))
            .sum()
    };
    let mut chosen = [0; 3];
    let mut drawn = 0;
    for (i, d) in LOD_DISTANCES.iter().enumerate() {
        // Alternately left and right of the line of sight.
        let x = if i % 2 == 0 { -1.6 } else { 1.6 };
        let at = Mat4::from_rotation_translation(
            Quat::from_rotation_y(0.6 * i as f32),
            Vec3::new(x, 0.0, -d),
        );
        let per_unit =
            lod::pixels_per_unit(&scene.camera, at.transform_point3(center), radius, 720.0);
        let level = lod::select(&errors, per_unit, 1.0, None);
        chosen[i] = level;
        drawn += tris(level);
        for (k, &(mesh, material, local)) in parts.iter().enumerate() {
            let mesh = match part_levels.get(k) {
                Some(l) if level > 0 && !l.is_empty() => l[(level - 1).min(l.len() - 1)].0,
                _ => mesh,
            };
            scene.instances.push(og_render::Instance {
                mesh,
                material,
                transform: at * local,
                material_layer: None,
                fx: InstanceFx {
                    flash: 0.8,
                    flash_color: lod::debug_color(level),
                    ..InstanceFx::NONE
                },
                cast_shadows: true,
                receive_shadows: true,
            });
        }
    }
    Ok((scene, chosen, (drawn, 3 * tris(0))))
}

fn mat(color: Vec3, metalness: f32, roughness: f32) -> StandardMaterial {
    StandardMaterial {
        color,
        metalness,
        roughness,
        ..Default::default()
    }
}

fn boxed(scene: &mut RenderScene, m: StandardMaterial, at: Vec3, size: Vec3) {
    let mesh = scene.add_mesh(shapes::cuboid(size));
    let h = scene.add_standard_material(m);
    scene.add_instance(mesh, h, Mat4::from_translation(at));
}

/// The chrome sphere's centre and radius in `probe-interior`.
pub const CHROME: (Vec3, f32) = (Vec3::new(-0.8, 1.0, 0.0), 1.0);

/// `probe-interior` (module docs); `probe` false leaves the probe out.
pub fn probe_scene(probe: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    // The sun is off: the room is closed, its light the lamps'.
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(-0.4, 0.7, 0.5),
        intensity: 0.0,
        cast_shadows: false,
        ..Default::default()
    };
    scene.environment.intensity = 1.0;
    scene.camera = Camera {
        position: Vec3::new(0.3, 1.6, 4.2),
        target: Vec3::new(-0.6, 1.0, 0.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    let wall = |c: Vec3| mat(c, 0.0, 0.8);
    boxed(
        &mut scene,
        wall(Vec3::new(0.75, 0.15, 0.1)),
        Vec3::new(0.0, 2.0, -5.0),
        Vec3::new(10.0, 4.0, 0.2),
    );
    boxed(
        &mut scene,
        wall(Vec3::new(0.12, 0.3, 0.75)),
        Vec3::new(-5.0, 2.0, 0.0),
        Vec3::new(0.2, 4.0, 10.0),
    );
    boxed(
        &mut scene,
        wall(Vec3::new(0.85, 0.8, 0.65)),
        Vec3::new(5.0, 2.0, 0.0),
        Vec3::new(0.2, 4.0, 10.0),
    );
    boxed(
        &mut scene,
        wall(Vec3::new(0.85, 0.8, 0.65)),
        Vec3::new(0.0, 2.0, 5.0),
        Vec3::new(10.0, 4.0, 0.2),
    );
    boxed(
        &mut scene,
        wall(Vec3::new(0.9, 0.9, 0.88)),
        Vec3::new(0.0, 4.1, 0.0),
        Vec3::new(10.2, 0.2, 10.2),
    );
    boxed(
        &mut scene,
        wall(Vec3::new(0.25, 0.22, 0.2)),
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(10.0, 0.2, 10.0),
    );
    boxed(
        &mut scene,
        mat(Vec3::new(0.2, 0.65, 0.3), 0.0, 0.5),
        Vec3::new(2.2, 0.6, -2.0),
        Vec3::splat(1.2),
    );
    let sphere = scene.add_mesh(shapes::sphere(64, 32));
    let chrome = scene.add_standard_material(mat(Vec3::splat(0.95), 1.0, 0.02));
    scene.add_instance(
        sphere,
        chrome,
        Mat4::from_scale_rotation_translation(Vec3::splat(CHROME.1), Quat::IDENTITY, CHROME.0),
    );
    for (at, c) in [
        (Vec3::new(-3.0, 3.2, -3.0), Vec3::new(1.0, 0.75, 0.45)),
        (Vec3::new(3.0, 3.2, 2.0), Vec3::new(0.6, 0.75, 1.0)),
    ] {
        scene.lights.push(Light {
            kind: LightKind::Point,
            position: at,
            color: c,
            intensity: 30.0,
            range: 12.0,
            cast_shadows: false,
        });
    }
    if probe {
        let mut p = ReflectionProbe::new(1, Vec3::new(0.0, 1.6, 0.0), Vec3::new(10.0, 4.2, 10.0));
        p.mode = ProbeMode::Baked;
        p.blend_distance = 0.5;
        scene.reflection_probes.push(p);
    }
    scene
}

/// `ssr-wet-floor` (module docs); `ssr` false turns the reflections off.
pub fn ssr_scene(ssr: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.environment.background = Background::Color(Vec3::ZERO);
    scene.environment.intensity = 0.0;
    let key = Vec3::new(0.3, 0.8, 0.6);
    scene.light = DirectionalLight {
        direction_to_light: key,
        intensity: 2.5,
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.4, 5.0),
        target: Vec3::new(0.0, 0.6, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    boxed(
        &mut scene,
        mat(Vec3::splat(0.04), 0.0, 0.04),
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(14.0, 0.2, 14.0),
    );
    let sphere = scene.add_mesh(shapes::sphere(48, 24));
    let red = scene.add_standard_material(mat(Vec3::new(0.85, 0.08, 0.05), 0.0, 0.4));
    scene.add_instance(
        sphere,
        red,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(0.8),
            Quat::IDENTITY,
            Vec3::new(-1.0, 0.8, 0.0),
        ),
    );
    let panel = scene.add_standard_material(StandardMaterial {
        color: Vec3::ONE,
        emissive: Vec3::new(1.0, 0.95, 0.8),
        emissiveIntensity: 2.0,
        ..Default::default()
    });
    let slab = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 2.0, 0.2)));
    scene.add_instance(
        slab,
        panel,
        Mat4::from_translation(Vec3::new(1.4, 1.0, -0.8)),
    );
    if ssr {
        scene.post.ssr = Some(ScreenSpaceReflections {
            max_distance: 10.0,
            steps: 48,
            ..Default::default()
        });
    }
    scene
}
