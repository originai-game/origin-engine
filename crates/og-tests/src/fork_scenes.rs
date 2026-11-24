//! Scenes for the rendering fields the Grand View Garden fork merge added
//! (PR #2): water over a mesh, its normal-map ripples and flow, its F0,
//! reflection bias and glint, the directional light's fixed shadow box, the
//! hemisphere light, AO on direct light, a distance-scaled AO radius and the
//! screen AO with water in its depth, CAS sharpening, UnrealBloomPass bloom,
//! alpha to coverage, the hook functions (`h.eye`, `h.front`,
//! `planar_reflection`, `screen_ao`), the second planar reflection, and the
//! terrain's `visible` (the scatter's `cast_shadows` is
//! `terrain_scenes::evening_meadow_scene`'s). Each is registered
//! in [`crate::scenes::build`]; `tests/fork_fields.rs` renders each with and
//! without its field and asserts what the field changes. Everything is
//! generated here.

use crate::scenes::texture;
use glam::{Mat4, Vec2, Vec3};
use og_render::{
    AmbientOcclusion, AoMethod, Background, Bloom, Camera, ColorSpace, DirectionalLight,
    Environment, Exposure, Material, MaterialShader, MeshData, ParamKind, ProceduralSky,
    RenderScene, ScatterDraw, ShaderParam, StandardMaterial, Terrain, TerrainControl, TerrainData,
    TerrainLayer, TerrainScatter, Tonemap, Vertex, Water, shapes,
};
use std::sync::Arc;

/// A sunlit sky that also lights the scene.
fn sky(scene: &mut RenderScene, sun: Vec3, intensity: f32) {
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            ..Default::default()
        },
        intensity: 1.0,
        background: Background::Sky,
        ..Environment::default()
    };
}

/// A flat, untextured ground plane of `size` metres at height 0.
fn plain_ground(scene: &mut RenderScene, size: f32, color: Vec3) {
    let plane = scene.add_mesh(shapes::plane(size, 1.0));
    let m = scene.add_standard_material(StandardMaterial {
        color,
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(plane, m, Mat4::IDENTITY);
}

/// A flat grid mesh over `[x0, x1] × [z0, z1]` at height 0, facing up,
/// `n` quads along its longer side; its uv are its metres.
pub fn strip(x0: f32, x1: f32, z0: f32, z1: f32, n: u32) -> MeshData {
    let mut m = MeshData::default();
    let (nx, nz) = if (x1 - x0) >= (z1 - z0) {
        (n, ((n as f32 * (z1 - z0) / (x1 - x0)).ceil() as u32).max(1))
    } else {
        (((n as f32 * (x1 - x0) / (z1 - z0)).ceil() as u32).max(1), n)
    };
    for j in 0..=nz {
        for i in 0..=nx {
            let x = x0 + (x1 - x0) * i as f32 / nx as f32;
            let z = z0 + (z1 - z0) * j as f32 / nz as f32;
            m.vertices.push(Vertex {
                position: [x, 0.0, z],
                normal: [0.0, 1.0, 0.0],
                uv: [x, z],
                color: [1.0; 4],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }
    }
    for j in 0..nz {
        for i in 0..nx {
            let a = j * (nx + 1) + i;
            let b = a + nx + 1;
            m.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    m
}

// ---------------------------------------------------------------------------
// Water.
// ---------------------------------------------------------------------------

/// The water's level in the water scenes.
pub const WATER_LEVEL: f32 = 0.4;

/// A sand floor 0.4 m under a 24 m water surface's frame, and over it (with
/// `mesh`) a stream 3 m wide along x at z -4 to -1, drawn over that mesh
/// instead of the 24 m grid, the sky above. Without `mesh` the grid covers
/// the whole floor.
pub fn water_mesh_scene(mesh: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    sky(&mut scene, Vec3::new(0.3, 0.8, 0.4), 2.5);
    plain_ground(&mut scene, 40.0, Vec3::new(0.7, 0.42, 0.18));
    let stream = mesh.then(|| scene.add_mesh(strip(-12.0, 12.0, -4.0, -1.0, 48)));
    scene.world.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, WATER_LEVEL, 0.0)),
        size: Vec2::splat(24.0),
        segments: 1,
        foam_width: 0.0,
        mesh: stream,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 6.0, 7.0),
        target: Vec3::new(0.0, 0.0, -2.0),
        exposure: Exposure::linear(0.45),
        ..Default::default()
    };
    scene
}

/// A tiling tangent-space ripple map (linear; red +x, green +z, blue up):
/// the normals of `sin(2πu·3) · sin(2πv·2)` bumps.
pub fn ripple_map(scene: &mut RenderScene) -> og_render::TextureHandle {
    let tau = std::f32::consts::TAU;
    scene.add_texture(texture(128, ColorSpace::Linear, |u, v| {
        let (a, b) = (3.0 * tau, 2.0 * tau);
        let dx = a * (a * u).cos() * (b * v).sin() * 0.06;
        let dz = b * (a * u).sin() * (b * v).cos() * 0.06;
        let n = Vec3::new(-dx, -dz, 1.0).normalize();
        let e = |c: f32| ((c * 0.5 + 0.5) * 255.0).round() as u8;
        [e(n.x), e(n.y), e(n.z), 255]
    }))
}

/// A lake rippled by [`ripple_map`] (Water2's flow: 0.25 repeats a metre,
/// sliding 0.2 uv a second along x), seen at a slant under a sunlit sky, the
/// planar reflection off.
pub fn water_ripples_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    sky(&mut scene, Vec3::new(-0.4, 0.5, -0.6), 2.5);
    plain_ground(&mut scene, 60.0, Vec3::new(0.3, 0.28, 0.22));
    let map = ripple_map(&mut scene);
    scene.world.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 1.5, 0.0)),
        size: Vec2::splat(40.0),
        segments: 1,
        foam_width: 0.0,
        wave_strength: 1.0,
        normal_map: Some(map),
        normal_scale: 0.25,
        flow: Vec2::new(0.2, 0.0),
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 5.0, 9.0),
        target: Vec3::new(0.0, 1.5, -3.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene.frame.tick = 30;
    scene
}

/// Deep, dark, still water seen from 6 m (straight down at the bottom of
/// the frame, at a slant at the top) under a bright sky, with three.js
/// Water.js's F0 (0.3) and reflection bias (0.1).
pub fn water_fresnel_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    sky(&mut scene, Vec3::new(0.3, 0.8, 0.4), 2.5);
    plain_ground(&mut scene, 60.0, Vec3::new(0.2, 0.18, 0.15));
    scene.world.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 4.0, 0.0)),
        size: Vec2::splat(60.0),
        segments: 1,
        foam_width: 0.0,
        wave_strength: 0.0,
        deep: Vec3::new(0.01, 0.03, 0.04),
        opacity: 1.0,
        depth_fade: 0.5,
        f0: 0.3,
        reflection_bias: 0.1,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 10.0, 4.0),
        target: Vec3::new(0.0, 4.0, -1.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

/// Where [`water_glint_scene`]'s sun is mirrored to its camera: the sun's
/// direction, the eye and the point on the surface.
pub const GLINT_SUN: Vec3 = Vec3::new(0.0, 0.35, -1.0);
pub const GLINT_EYE: Vec3 = Vec3::new(0.0, 3.0, 6.0);

/// Still water under a low light ahead of the camera (the sky's sun disk
/// elsewhere), with three.js Water's glint (2): the light's mirror image on
/// the surface.
pub fn water_glint_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    sky(&mut scene, GLINT_SUN, 1.0);
    // The sky's own sun disk high behind the camera: the glint is the
    // light's mirror image, not the disk's reflection.
    scene.lighting.environment.sky.sun_direction = Vec3::new(0.2, 0.8, 0.6);
    plain_ground(&mut scene, 200.0, Vec3::new(0.2, 0.18, 0.15));
    scene.world.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        size: Vec2::splat(200.0),
        segments: 1,
        foam_width: 0.0,
        wave_strength: 0.0,
        opacity: 1.0,
        glint: 2.0,
        ..Default::default()
    });
    scene.look.post.tonemap = Tonemap::None;
    scene.frame.camera = Camera {
        position: GLINT_EYE,
        target: glint_point(),
        exposure: Exposure::linear(0.5),
        ..Default::default()
    };
    scene
}

/// The surface point (at y 1) where the glint scene's sun is mirrored to
/// its eye.
pub fn glint_point() -> Vec3 {
    let s = GLINT_SUN.normalize();
    // The ray from the eye down to the plane y = 1 that leaves it towards the sun.
    let h = GLINT_EYE.y - 1.0;
    let down = Vec3::new(s.x, -s.y, s.z);
    GLINT_EYE + down * (h / s.y)
}

// ---------------------------------------------------------------------------
// The directional light's shadow box.
// ---------------------------------------------------------------------------

/// The shadow-box scene's sun (from -z, 31 degrees up: shadows 5 m long
/// fall towards the camera).
pub const BOX_SUN: Vec3 = Vec3::new(0.0, 0.6, -1.0);
/// Its camera, 6 m up looking along -z.
pub const BOX_EYE: Vec3 = Vec3::new(0.0, 6.0, 12.0);
/// Its two 3 m posts: one 12 m ahead of the camera (inside an 8 m box
/// focused 12 m ahead on the ground under the eye), one 42 m ahead.
pub const BOX_NEAR_POST: Vec3 = Vec3::new(-1.5, 1.5, 0.0);
pub const BOX_FAR_POST: Vec3 = Vec3::new(5.0, 1.5, -30.0);
/// The leaf cards' centre: a striped leaf card (alpha-tested, 2 cm stripes)
/// facing the sun, and a white card [`LEAF_GAP`] behind it along the light.
pub const BOX_LEAF: Vec3 = Vec3::new(2.0, 1.5, 3.0);
pub const LEAF_GAP: f32 = 0.02;

/// A 200 m ground with two posts and a pair of leaf cards under a sun from -z,
/// shadowed by one 8 m box focused 12 m ahead of the camera on the ground
/// 6 m below it, with 4 cm of depth and normal bias.
pub fn shadow_box_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    sky(&mut scene, BOX_SUN, 3.0);
    scene.lighting.environment.intensity = 0.4;
    plain_ground(&mut scene, 200.0, Vec3::splat(0.7));
    let post = scene.add_mesh(shapes::cuboid(Vec3::new(1.0, 3.0, 1.0)));
    let wood = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.5, 0.35, 0.2),
        roughness: 0.8,
        ..Default::default()
    });
    for at in [BOX_NEAR_POST, BOX_FAR_POST] {
        scene.add_instance(post, wood, Mat4::from_translation(at));
    }
    // The leaf cards, facing the sun.
    let l = BOX_SUN.normalize();
    let face = glam::Quat::from_rotation_arc(Vec3::Z, l);
    let stripes = scene.add_texture(texture(64, ColorSpace::Srgb, |u, _| {
        [
            255,
            255,
            255,
            if (u * 16.0).fract() < 0.5 { 255 } else { 0 },
        ]
    }));
    let leaf = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.35, 0.6, 0.2),
        roughness: 0.7,
        side: 2,
        map: stripes.slot(),
        alphaTest: 0.5,
        ..Default::default()
    });
    let backing = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.8),
        roughness: 0.7,
        side: 2,
        ..Default::default()
    });
    let small = scene.add_mesh(shapes::plane_xy(0.5, 0.5));
    let large = scene.add_mesh(shapes::plane_xy(1.0, 1.0));
    scene.add_instance(small, leaf, Mat4::from_rotation_translation(face, BOX_LEAF));
    scene.add_instance(
        large,
        backing,
        Mat4::from_rotation_translation(face, BOX_LEAF - l * LEAF_GAP),
    );
    scene.lighting.light.shadow_box = 8.0;
    scene.lighting.light.shadow_focus = 12.0;
    scene.lighting.light.shadow_focus_drop = 6.0;
    scene.lighting.light.shadow_bias = 0.04;
    scene.lighting.light.shadow_normal_bias = 0.04;
    scene.frame.camera = Camera {
        position: BOX_EYE,
        target: Vec3::new(0.0, 0.0, -6.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// The hemisphere light.
// ---------------------------------------------------------------------------

/// The hemisphere scene's floor tile of mirror metal, and its ceiling's height.
pub const HEMI_METAL: Vec3 = Vec3::new(1.2, 0.01, 3.0);
pub const HEMI_CEILING: f32 = 2.4;

/// A white floor and, 2.4 m up, a white ceiling facing down, seen from
/// between them with no sun, a mirror-metal tile on the floor; the
/// hemisphere light's sky red and its ground blue, at intensity 2.
pub fn hemisphere_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    sky(&mut scene, Vec3::new(0.3, 0.8, 0.4), 0.0);
    scene.lighting.light.cast_shadows = false;
    plain_ground(&mut scene, 40.0, Vec3::splat(0.8));
    let ceiling = scene.add_mesh(shapes::plane(40.0, 1.0));
    let white = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.8),
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(
        ceiling,
        white,
        Mat4::from_translation(Vec3::new(0.0, HEMI_CEILING, -20.0))
            * Mat4::from_rotation_x(std::f32::consts::PI),
    );
    let tile = scene.add_mesh(shapes::plane(1.2, 1.0));
    let metal = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.9),
        metalness: 1.0,
        roughness: 0.25,
        ..Default::default()
    });
    scene.add_instance(tile, metal, Mat4::from_translation(HEMI_METAL));
    scene.lighting.hemisphere = ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0], 2.0);
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.2, 6.0),
        target: Vec3::new(0.0, 1.2, -4.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// Post effects.
// ---------------------------------------------------------------------------

/// The AO scene ([`crate::fx_scenes::ao_scene`], GTAO) with its occlusion
/// also on the direct light (`direct` 1).
pub fn ao_direct_scene() -> RenderScene {
    let mut scene = crate::fx_scenes::ao_scene(AoMethod::Gtao);
    if let Some(ao) = &mut scene.look.post.ao {
        ao.direct = 1.0;
    }
    scene
}

/// The screen-AO pool's water level and its post (on the bed at y 0).
pub const POOL_LEVEL: f32 = 1.0;
pub const POOL_POST: Vec3 = Vec3::new(0.0, 1.5, 0.0);
/// A 0.5 m block on the pool's bed, 0.5 m under the water.
pub const POOL_BLOCK: Vec3 = Vec3::new(1.6, 0.25, 0.0);

/// A flooded cistern in a ruined courtyard on a summer afternoon: a 3 m
/// column standing on its stone bed and a 0.5 m block sunk beside it, the
/// water 1 m deep round them inside a rubble rim, the yard's setts, crates
/// and a barrel above, with the screen AO (three.js-era post SSAO) on the
/// direct light too. With `water` the surface is at 1 m; without, the same
/// surface 2 m under the bed, where the prepass hides it.
pub fn ao_screen_water_scene(water: bool) -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let bed = kit::surface_tinted(&mut scene, "stone", [0.7, 0.66, 0.6]);
    let floor = scene.add_mesh(kit::slab(Vec2::splat(-4.0), Vec2::splat(4.0), 0.0, 2.0));
    scene.add_instance(floor, bed, Mat4::IDENTITY);
    // The rim: four rubble walls from the bed to the yard, 1.4 m high.
    let stone = kit::surface_tinted(&mut scene, "stone", [0.95, 0.9, 0.84]);
    for (min, max) in [
        (Vec3::new(-4.6, 0.0, -4.6), Vec3::new(4.6, 1.4, -4.0)),
        (Vec3::new(-4.6, 0.0, 4.0), Vec3::new(4.6, 1.4, 4.6)),
        (Vec3::new(-4.6, 0.0, -4.0), Vec3::new(-4.0, 1.4, 4.0)),
        (Vec3::new(4.0, 0.0, -4.0), Vec3::new(4.6, 1.4, 4.0)),
    ] {
        let m = scene.add_mesh(kit::block(min, max, 2.4));
        scene.add_instance(m, stone, Mat4::IDENTITY);
    }
    // The yard round it, cobbled, and the country beyond.
    let setts = kit::surface_tinted(&mut scene, "cobbles", [0.9, 0.88, 0.84]);
    for (min, max) in [
        (Vec2::new(-30.0, -30.0), Vec2::new(30.0, -4.6)),
        (Vec2::new(-30.0, 4.6), Vec2::new(30.0, 30.0)),
        (Vec2::new(-30.0, -4.6), Vec2::new(-4.6, 4.6)),
        (Vec2::new(4.6, -4.6), Vec2::new(30.0, 4.6)),
    ] {
        let m = scene.add_mesh(kit::slab(min, max, 1.4, 2.0));
        scene.add_instance(m, setts, Mat4::IDENTITY);
    }
    let grass = kit::surface_tinted(&mut scene, "grass", [0.86, 1.0, 0.72]);
    for (min, max) in [
        (Vec2::new(-260.0, -260.0), Vec2::new(-30.0, 260.0)),
        (Vec2::new(30.0, -260.0), Vec2::new(260.0, 260.0)),
        (Vec2::new(-30.0, -260.0), Vec2::new(30.0, -30.0)),
        (Vec2::new(-30.0, 30.0), Vec2::new(30.0, 260.0)),
    ] {
        let m = scene.add_mesh(kit::slab(min, max, 1.4, 7.0));
        scene.add_instance(m, grass, Mat4::IDENTITY);
    }
    kit::hills(&mut scene, None);
    // The column standing in the water and the block sunk beside it.
    let column = kit::prop(&mut scene, "column");
    column.place(&mut scene, POOL_POST - Vec3::Y * 1.5, 0.3, 3.0);
    let block = scene.add_mesh(kit::block(
        POOL_BLOCK - Vec3::splat(0.25),
        POOL_BLOCK + Vec3::splat(0.25),
        1.0,
    ));
    scene.add_instance(block, stone, Mat4::IDENTITY);
    // The yard's clutter on the rim.
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-3.2, 1.4, -5.6), 0.2, 0.8);
    crate_.place(&mut scene, Vec3::new(-2.3, 1.4, -5.5), -0.3, 0.7);
    kit::prop(&mut scene, "winebarrel").place(&mut scene, Vec3::new(-1.2, 1.4, -5.4), 0.5, 1.0);
    kit::prop(&mut scene, "column_broken").place(&mut scene, Vec3::new(2.6, 1.4, -5.6), 1.1, 1.3);
    kit::prop(&mut scene, "boulder").place(&mut scene, Vec3::new(-5.6, 1.3, -1.0), 0.7, 0.9);
    // What is left of the colonnade round the yard, and the town behind.
    for x in [-6.5, -2.2, 2.2, 6.5] {
        column.place(&mut scene, Vec3::new(x, 1.4, -8.0), 0.2 * x, 4.2);
    }
    column.place(&mut scene, Vec3::new(-6.5, 1.4, -3.0), 0.9, 4.2);
    let house = kit::prop(&mut scene, "townhouse");
    for (x, z, turn) in [
        (-12.0, -15.0, 0.0),
        (-1.0, -16.0, 0.1),
        (10.0, -15.0, -0.1),
        (-16.0, -4.0, 1.57),
    ] {
        house.place(&mut scene, Vec3::new(x, 1.4, z), turn, 9.0);
    }
    let sun = Vec3::new(0.4, 0.8, 0.6);
    crate::sky_scenes::physical_light_under(&mut scene, sun);
    scene.world.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(
            0.0,
            if water { POOL_LEVEL } else { -2.0 },
            0.0,
        )),
        size: Vec2::splat(8.0),
        segments: 1,
        foam_width: 0.0,
        wave_strength: 0.0,
        ..Default::default()
    });
    scene.look.post.ao = Some(AmbientOcclusion {
        method: AoMethod::Screen,
        intensity: 1.0,
        direct: 1.0,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(3.0, 4.6, 6.4),
        target: Vec3::new(0.0, 1.0, -0.6),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// The bloom scene (`bloom`) with UnrealBloomPass at radius 0.5
/// and contrast 1, strength 0.2 (its halo squares the strength and spans the
/// frame at the classic bloom's 0.8).
pub fn bloom_unreal(mut scene: RenderScene) -> RenderScene {
    scene.look.post.bloom = Some(Bloom {
        threshold: 1.0,
        knee: 0.5,
        intensity: 0.2,
    });
    scene.look.post.bloom_unreal = Some(0.5);
    scene.look.post.bloom_contrast = 1.0;
    scene
}

// ---------------------------------------------------------------------------
// Alpha to coverage.
// ---------------------------------------------------------------------------

/// A green card 4 m square straight ahead on black, its alpha a radial
/// ramp (1 at the centre to 0 at the edge), cut at 0.5 with alpha to
/// coverage (unlit; the desc asks for 4x MSAA in `scenes::build`).
pub fn alpha_to_coverage_scene(a2c: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.lighting.light.intensity = 0.0;
    scene.look.post.tonemap = Tonemap::None;
    let ramp = scene.add_texture(texture(256, ColorSpace::Srgb, |u, v| {
        let r = (Vec2::new(u, v) - 0.5).length() * 2.0;
        [
            255,
            255,
            255,
            ((1.0 - r).clamp(0.0, 1.0) * 255.0).round() as u8,
        ]
    }));
    let card = scene.add_mesh(shapes::plane_xy(4.0, 4.0));
    let m = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.0, 1.0, 0.0),
        map: ramp.slot(),
        alphaTest: 0.5,
        alphaToCoverage: a2c as u32,
        unlit: 1,
        ..Default::default()
    });
    scene.add_instance(card, m, Mat4::IDENTITY);
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 0.0, 5.0),
        target: Vec3::ZERO,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// Material shader hooks: h.eye, h.front, planar_reflection, screen_ao.
// ---------------------------------------------------------------------------

fn hook_shader(name: &str, code: &str, params: Vec<ShaderParam>) -> MaterialShader {
    MaterialShader {
        name: name.into(),
        code: code.into(),
        params,
        mode: Default::default(),
    }
}

/// The eye-and-facing hook: red where `h.eye` is the camera (`p.eye`),
/// green on a front face, blue on a back face.
pub fn eye_front_shader() -> MaterialShader {
    hook_shader(
        "eye-front",
        "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  let at_eye = 1.0 - clamp(length(h.eye - p.eye) * 4.0, 0.0, 1.0);
  return vec4<f32>(at_eye, select(0.0, 1.0, h.front > 0.0), select(0.0, 1.0, h.front < 0.0), 1.0);
}
",
        vec![ShaderParam {
            name: "eye".into(),
            kind: ParamKind::Vec3,
            default: [0.0; 4],
        }],
    )
}

/// The hooks scene's camera.
pub const HOOK_EYE: Vec3 = Vec3::new(0.0, 1.0, 5.0);

/// Two double-sided unlit cards drawn by [`eye_front_shader`]: the left
/// one facing the camera, the right one turned away from it (its back
/// seen).
pub fn hooks_eye_front_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    let shader = scene.add_shader(eye_front_shader());
    let m = scene.add_standard_material(StandardMaterial {
        unlit: 1,
        side: 2,
        ..Default::default()
    });
    scene.set_material_shader(m, Some(shader));
    scene.set_shader_param(m, "eye", &HOOK_EYE.to_array());
    let card = scene.add_mesh(shapes::plane_xy(1.6, 1.6));
    scene.add_instance(card, m, Mat4::from_translation(Vec3::new(-1.0, 1.0, 0.0)));
    scene.add_instance(
        card,
        m,
        Mat4::from_translation(Vec3::new(1.0, 1.0, 0.0)) * Mat4::from_rotation_y(2.8),
    );
    scene.frame.camera = Camera {
        position: HOOK_EYE,
        target: Vec3::new(0.0, 1.0, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// The planar-reflection hook: the frame's planar reflection at the
/// fragment (alpha 1 when there is one), or magenta where there is none.
pub fn planar_hook_shader() -> MaterialShader {
    hook_shader(
        "planar",
        "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  let r = planar_reflection(h.world, vec2<f32>(0.0));
  return vec4<f32>(select(vec3<f32>(1.0, 0.0, 1.0), r.rgb, r.a > 0.5), 1.0);
}
",
        Vec::new(),
    )
}

/// The planar hook scene's red emissive block, standing on the water's
/// far side (its centre).
pub const PLANAR_BLOCK: Vec3 = Vec3::new(0.0, 2.0, -12.0);

/// Still water at y 1 drawn with a planar reflection, a red emissive block
/// standing in it 16 m ahead, and lying on the water before it a 6 m card
/// whose unlit material shows [`planar_hook_shader`]: what the water's
/// mirror sees there (the block's mirror image on its far half).
pub fn hooks_planar_scene(planar: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    sky(&mut scene, Vec3::new(0.3, 0.8, 0.4), 2.0);
    plain_ground(&mut scene, 60.0, Vec3::new(0.3, 0.28, 0.22));
    scene.world.water.push(Water {
        transform: Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        size: Vec2::splat(40.0),
        segments: 1,
        foam_width: 0.0,
        wave_strength: 0.0,
        planar_reflection: if planar { 0.5 } else { 0.0 },
        ..Default::default()
    });
    let block = scene.add_mesh(shapes::cuboid(Vec3::new(4.0, 2.0, 1.0)));
    let red = scene.add_standard_material(StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::new(4.0, 0.0, 0.0),
        ..Default::default()
    });
    scene.add_instance(block, red, Mat4::from_translation(PLANAR_BLOCK));
    let shader = scene.add_shader(planar_hook_shader());
    let m = scene.add_standard_material(StandardMaterial {
        unlit: 1,
        ..Default::default()
    });
    scene.set_material_shader(m, Some(shader));
    let card = scene.add_mesh(shapes::plane(6.0, 1.0));
    scene.add_instance(card, m, Mat4::from_translation(Vec3::new(0.0, 1.02, -8.0)));
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 4.0, 4.0),
        target: Vec3::new(0.0, 1.0, -6.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

/// The screen-AO hook: `4 (1 - screen_ao)` in red, unlit.
pub fn screen_ao_hook_shader() -> MaterialShader {
    hook_shader(
        "screen-ao",
        "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(clamp((1.0 - screen_ao(h.world)) * 4.0, 0.0, 1.0), 0.0, 0.0, 1.0);
}
",
        Vec::new(),
    )
}

/// The screen-AO hook scene's block (1 m, on the ground at the origin).
pub const AO_HOOK_BLOCK: Vec3 = Vec3::new(0.0, 0.5, 0.0);

/// A ground whose unlit material shows [`screen_ao_hook_shader`] around a
/// lit 1 m block standing on it, GTAO on.
pub fn hooks_screen_ao_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    sky(&mut scene, Vec3::new(0.3, 0.8, 0.4), 2.0);
    let ground = scene.add_mesh(shapes::plane(30.0, 1.0));
    let shader = scene.add_shader(screen_ao_hook_shader());
    let m = scene.add_standard_material(StandardMaterial {
        unlit: 1,
        ..Default::default()
    });
    scene.set_material_shader(m, Some(shader));
    scene.add_instance(ground, m, Mat4::IDENTITY);
    let block = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let grey = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.6),
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(block, grey, Mat4::from_translation(AO_HOOK_BLOCK));
    scene.look.post.ao = Some(AmbientOcclusion {
        radius: 0.8,
        intensity: 1.5,
        method: AoMethod::Gtao,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 3.0, 4.0),
        target: Vec3::new(0.0, 0.3, 0.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// The second planar reflection.
// ---------------------------------------------------------------------------

/// The two pools' centres (their water's height in y), and the emissive
/// slabs standing behind each.
pub const POOL_A: Vec3 = Vec3::new(-3.5, 0.3, -2.0);
pub const POOL_B: Vec3 = Vec3::new(3.5, 1.5, -2.0);
pub const SLAB_A: Vec3 = Vec3::new(-3.5, 1.5, -7.5);
pub const SLAB_B: Vec3 = Vec3::new(3.5, 3.0, -7.5);

/// Two still pools side by side at different heights (0.3 and 1.5 m), each
/// with a planar reflection, a red slab standing behind pool A and a blue
/// one behind B: each pool mirrors its own slab in its own plane.
pub fn water_planar_two_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    sky(&mut scene, Vec3::new(0.3, 0.8, 0.4), 2.0);
    plain_ground(&mut scene, 60.0, Vec3::new(0.3, 0.28, 0.22));
    for c in [POOL_A, POOL_B] {
        scene.world.water.push(Water {
            transform: Mat4::from_translation(c),
            size: Vec2::new(6.0, 8.0),
            segments: 1,
            foam_width: 0.0,
            wave_strength: 0.0,
            depth_fade: 0.5,
            opacity: 1.0,
            planar_reflection: 0.5,
            ..Default::default()
        });
    }
    // Pool B's basin: a block under its water.
    let basin = scene.add_mesh(shapes::cuboid(Vec3::new(6.0, 1.0, 8.0)));
    let stone = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.4),
        roughness: 0.9,
        ..Default::default()
    });
    scene.add_instance(
        basin,
        stone,
        Mat4::from_translation(Vec3::new(POOL_B.x, 0.5, POOL_B.z)),
    );
    let slab = scene.add_mesh(shapes::cuboid(Vec3::new(4.0, 2.0, 0.5)));
    for (at, rgb) in [
        (SLAB_A, Vec3::new(4.0, 0.0, 0.0)),
        (SLAB_B, Vec3::new(0.0, 0.0, 4.0)),
    ] {
        let m = scene.add_standard_material(StandardMaterial {
            color: Vec3::ZERO,
            emissive: rgb,
            ..Default::default()
        });
        scene.add_instance(slab, m, Mat4::from_translation(at));
    }
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 6.0, 6.0),
        target: Vec3::new(0.0, 0.5, -2.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// The terrain's visibility and its scatter's shadows.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// `terrain-hidden`: an overgrown viaduct whose plants stand on a hidden
// terrain.
// ---------------------------------------------------------------------------

/// The viaduct: its deck's top and half its width, the pitch of its piers,
/// its arches' radius, where it starts and where it has fallen (x), and its
/// stone's repeat (m).
const DECK_TOP: f32 = 30.0;
const DECK_HALF: f32 = 4.5;
const SPAN: f32 = 16.0;
const ARCH: f32 = 6.0;
const DECK: [f32; 2] = [-120.0, 56.0];
const MASONRY: f32 = 5.0;

/// A quad of `corners` facing `normal`, each corner with its surface uv.
fn quad(m: &mut MeshData, corners: [(Vec3, [f32; 2]); 4], normal: Vec3) {
    let first = m.vertices.len() as u32;
    for (p, uv) in corners {
        m.vertices.push(Vertex {
            position: p.into(),
            normal: normal.into(),
            uv,
            color: [1.0; 4],
            tangent: [0.0; 4],
        });
    }
    let [a, b, c, _] = corners.map(|(p, _)| p);
    let order: [u32; 6] = if (b - a).cross(c - a).dot(normal) > 0.0 {
        [0, 1, 2, 0, 2, 3]
    } else {
        [0, 2, 1, 0, 3, 2]
    };
    m.indices.extend(order.map(|i| first + i));
}

/// One span of the viaduct between two piers, its opening's middle at
/// x = 0: both faces from the arch up to the deck's top, and the arch's
/// underside.
fn viaduct_span() -> MeshData {
    const STEPS: u32 = 20;
    let spring = DECK_TOP - 2.0 - ARCH;
    let mut m = MeshData::default();
    let on_arch = |i: u32| {
        let a = i as f32 / STEPS as f32 * std::f32::consts::PI;
        (Vec3::new(-ARCH * a.cos(), spring + ARCH * a.sin(), 0.0), a)
    };
    let face = |v: Vec3| [v.x / MASONRY, v.y / MASONRY];
    for i in 0..STEPS {
        let ((p, a), (q, b)) = (on_arch(i), on_arch(i + 1));
        for side in [-1.0f32, 1.0] {
            let z = Vec3::Z * (side * DECK_HALF);
            let corners = [
                p + z,
                q + z,
                Vec3::new(q.x, DECK_TOP, 0.0) + z,
                Vec3::new(p.x, DECK_TOP, 0.0) + z,
            ];
            quad(&mut m, corners.map(|v| (v, face(v))), Vec3::Z * side);
        }
        // The underside, facing the arch's centre.
        let mid = (a + b) * 0.5;
        let z = Vec3::Z * DECK_HALF;
        let (u, v) = (a * ARCH / MASONRY, b * ARCH / MASONRY);
        let w = DECK_HALF / MASONRY;
        quad(
            &mut m,
            [
                (p - z, [u, -w]),
                (q - z, [v, -w]),
                (q + z, [v, w]),
                (p + z, [u, w]),
            ],
            Vec3::new(mid.cos(), -mid.sin(), 0.0),
        );
    }
    m
}

/// `terrain-hidden`: a ruined viaduct across a valley in the early sun, its
/// deck overgrown with grass, heather, gold flowers and young pines that
/// og.TerrainScatter places on a terrain 256 by 64 m lying level at the
/// deck's top, its second layer (by its control map) the deck. With
/// `visible` false (the scene) the terrain is not drawn: the masonry is the
/// game's own meshes, the valley shows beside the deck and through the
/// arches, and the plants still stand on the terrain's heights. With
/// `visible` true it is a grey sheet 64 m wide through the viaduct at the
/// deck's top.
pub fn viaduct_scene(visible: bool) -> RenderScene {
    use crate::fx_scenes::{hash2, noise2, smooth};
    use crate::kit;
    use crate::terrain_scenes::{afternoon, camera_at, fair_sky, flower_clump, grass_tuft};
    let mut scene = RenderScene::new();
    afternoon(&mut scene);
    scene.lighting.light.direction_to_light = Vec3::new(-0.5, 0.8, 0.62);
    scene.lighting.light.max_shadow_distance = 320.0;
    fair_sky(&mut scene, 9.2);
    // The clouds where their shadows leave the valley in the sun.
    scene.frame.tick = 0;
    // The valley: its floor, and the hills round it.
    let floor = -2.0;
    let meadow = kit::surface_tinted(&mut scene, "grass", [0.78, 1.0, 0.62]);
    let land = scene.add_mesh(kit::slab(
        Vec2::splat(-160.0),
        Vec2::splat(160.0),
        floor,
        6.0,
    ));
    scene.add_instance(land, meadow, Mat4::IDENTITY);
    kit::hills(&mut scene, None);

    // The masonry: piers to the deck's top with an impost where the arches
    // spring and a plinth, the spans between them, a cornice, the deck's
    // setts and what is left of its parapets.
    let stone = kit::surface_tinted(&mut scene, "cobbles", [1.0, 0.93, 0.8]);
    let setts = kit::surface_tinted(&mut scene, "cobbles", [0.8, 0.84, 0.66]);
    let spring = DECK_TOP - 2.0 - ARCH;
    let pier_half = SPAN * 0.5 - ARCH;
    let side = Vec3::new(pier_half, 0.0, DECK_HALF);
    let ledge = Vec3::new(0.3, 0.0, 0.3);
    let pier = scene.add_mesh(kit::block(
        Vec3::new(0.0, floor - 2.0, 0.0) - side,
        Vec3::new(0.0, DECK_TOP, 0.0) + side,
        MASONRY,
    ));
    let impost = scene.add_mesh(kit::block(
        Vec3::new(0.0, spring - 0.7, 0.0) - side - ledge,
        Vec3::new(0.0, spring, 0.0) + side + ledge,
        MASONRY,
    ));
    let plinth = scene.add_mesh(kit::block(
        Vec3::new(0.0, floor - 2.0, 0.0) - side - ledge * 2.0,
        Vec3::new(0.0, floor + 2.2, 0.0) + side + ledge * 2.0,
        MASONRY,
    ));
    let span = scene.add_mesh(viaduct_span());
    let spans = ((DECK[1] - DECK[0]) / SPAN) as i32;
    for k in 0..=spans {
        let at = Mat4::from_translation(Vec3::X * (DECK[0] + k as f32 * SPAN));
        for mesh in [pier, impost, plinth] {
            scene.add_instance(mesh, stone, at);
        }
        if k < spans {
            let middle = at * Mat4::from_translation(Vec3::X * (SPAN * 0.5));
            scene.add_instance(span, stone, middle);
        }
    }
    let lip = Vec3::new(0.0, 0.0, 0.35);
    let cornice = scene.add_mesh(kit::block(
        Vec3::new(DECK[0] - pier_half, DECK_TOP - 0.6, -DECK_HALF) - lip,
        Vec3::new(DECK[1] + pier_half, DECK_TOP - 0.02, DECK_HALF) + lip,
        MASONRY,
    ));
    scene.add_instance(cornice, stone, Mat4::IDENTITY);
    let deck = scene.add_mesh(kit::slab(
        Vec2::new(DECK[0] - pier_half, -DECK_HALF),
        Vec2::new(DECK[1] + pier_half, DECK_HALF),
        DECK_TOP,
        2.5,
    ));
    scene.add_instance(deck, setts, Mat4::IDENTITY);
    // The parapets in lengths of 4 m, one in four fallen.
    for k in 0..((DECK[1] - DECK[0]) / 4.0) as i32 {
        for (s, z) in [(0, -DECK_HALF), (1, DECK_HALF - 0.55)] {
            if hash2(k, s, 0x9a11) < 0.25 {
                continue;
            }
            let x = DECK[0] + k as f32 * 4.0;
            let tall = 0.75 + 0.5 * hash2(k, s, 0x9a12);
            let wall = scene.add_mesh(kit::block(
                Vec3::new(x + 0.06, DECK_TOP, z),
                Vec3::new(x + 3.94, DECK_TOP + tall, z + 0.55),
                2.5,
            ));
            scene.add_instance(wall, stone, Mat4::IDENTITY);
        }
    }
    // Past the fall: a pier still near its height, one broken low, and the
    // stones of the spans between them in the grass.
    for (x, top) in [
        (DECK[1] + SPAN, DECK_TOP - 3.5),
        (DECK[1] + 2.0 * SPAN, 13.0),
    ] {
        let stump = scene.add_mesh(kit::block(
            Vec3::new(x, floor - 2.0, 0.0) - side,
            Vec3::new(x, top, 0.0) + side,
            MASONRY,
        ));
        scene.add_instance(stump, stone, Mat4::IDENTITY);
        scene.add_instance(plinth, stone, Mat4::from_translation(Vec3::X * x));
    }
    let boulder = kit::prop(&mut scene, "boulder");
    for k in 0..46 {
        let h = |i: i32| hash2(k, i, 0x4b1d);
        let height = 0.8 + 3.4 * h(2) * h(2);
        let at = Vec3::new(
            DECK[1] + 4.0 + 44.0 * h(0),
            floor - 0.3 * height,
            (h(1) - 0.5) * 30.0,
        );
        boulder.place(&mut scene, at, 6.3 * h(3), height);
    }
    // The valley's floor: a road of setts under the ninth arch, and pines
    // in stands either side of it.
    let road_x = DECK[0] + 8.5 * SPAN;
    let road = scene.add_mesh(kit::slab(
        Vec2::new(road_x - 2.6, -150.0),
        Vec2::new(road_x + 2.6, 150.0),
        floor + 0.04,
        3.0,
    ));
    scene.add_instance(road, setts, Mat4::IDENTITY);
    let pine = kit::prop(&mut scene, "pine");
    for k in 0..900 {
        let h = |i: i32| hash2(k, i, 0x71de);
        let (x, z) = (-150.0 + 300.0 * h(0), -150.0 + 300.0 * h(1));
        let stand = noise2(x / 38.0 + 3.0, z / 38.0 + 7.0, 0x33);
        if stand < 0.44 || (x - road_x).abs() < 7.0 || z.abs() < 9.0 {
            continue;
        }
        let at = Vec3::new(x, floor - 0.3, z);
        pine.place(&mut scene, at, 6.3 * h(2), 9.0 + 9.0 * h(3));
    }

    // The terrain: level at the deck's top, its control map (0.25 m a
    // texel) the deck between its parapets as layer 1, ragged at the
    // edges, and layer 0 everywhere else.
    let size = Vec2::new(256.0, 64.0);
    let (cw, ch) = (1024u32, 256u32);
    let mut rgba = Vec::with_capacity((cw * ch * 4) as usize);
    for y in 0..ch {
        for x in 0..cw {
            let at = (Vec2::new(x as f32, y as f32) + 0.5) / Vec2::new(cw as f32, ch as f32) * size
                - size * 0.5;
            let ragged = 0.7 * noise2(at.x / 2.3, at.y / 2.3, 0x51);
            let across = 1.0 - smooth(DECK_HALF - 1.9, DECK_HALF - 0.9, at.y.abs() + ragged);
            let along =
                smooth(DECK[0], DECK[0] + 2.0, at.x) * (1.0 - smooth(DECK[1], DECK[1] + 1.5, at.x));
            let grown = (across * along * 255.0) as u8;
            rgba.extend_from_slice(&[255 - grown, grown, 0, 0]);
        }
    }
    let (w, h) = (129u32, 33u32);
    let handle = scene.add_terrain(TerrainData {
        heights: Arc::from(vec![DECK_TOP; (w * h) as usize]),
        control: Some(TerrainControl {
            width: cw,
            height: ch,
            rgba: Arc::from(rgba),
        }),
        ..TerrainData::flat(size, w, h)
    });
    let layer = TerrainLayer {
        color: Vec3::splat(0.75),
        ..Default::default()
    };
    scene.world.terrains.push(Terrain {
        layers: vec![layer, layer],
        fade_in: false,
        visible,
        ..Terrain::new(handle)
    });

    // What grows on the deck.
    let plant = scene.add_material(Material {
        base_color: [1.0; 4],
        roughness: 0.85,
        ..Default::default()
    });
    let green = scene.add_material(Material {
        base_color: [0.3, 0.42, 0.12, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    let tuft = scene.add_mesh(grass_tuft());
    let heather = scene.add_mesh(flower_clump([0.42, 0.12, 0.52], true));
    let gold = scene.add_mesh(flower_clump([0.95, 0.66, 0.05], false));
    // Young pines, a metre tall at scale 1.
    let (sapling, bark) = pine.rooted(&mut scene, 1.0);
    let grown = |scatter: TerrainScatter| TerrainScatter {
        layer: 1,
        distance: 220.0,
        fade: 0.2,
        ..scatter
    };
    for (mesh, material, scatter) in [
        (
            tuft,
            green,
            TerrainScatter {
                density: 16.0,
                scale: [0.7, 1.7],
                seed: 3,
                sway: 0.6,
                noise_scale: 4.0,
                noise_threshold: 0.3,
                ..Default::default()
            },
        ),
        (
            heather,
            plant,
            TerrainScatter {
                density: 1.6,
                scale: [0.8, 1.5],
                seed: 31,
                sway: 0.6,
                noise_scale: 6.0,
                noise_threshold: 0.45,
                ..Default::default()
            },
        ),
        (
            gold,
            plant,
            TerrainScatter {
                density: 1.6,
                scale: [0.8, 1.5],
                seed: 37,
                sway: 0.6,
                noise_scale: 6.0,
                noise_threshold: 0.45,
                ..Default::default()
            },
        ),
        (
            sapling,
            bark,
            TerrainScatter {
                density: 0.02,
                scale: [1.6, 4.6],
                seed: 11,
                sway: 0.2,
                footprint: 1.4,
                ..Default::default()
            },
        ),
    ] {
        scene.world.scatters.push(ScatterDraw {
            id: 0,
            terrain: 0,
            mesh,
            material,
            lod_materials: [None; 2],
            scatter: grown(scatter),
            cast_shadows: true,
            impostor: None,
            impostor_distance: 0.0,
            with: None,
            lods: [None; 2],
        });
    }
    scene.frame.camera = camera_at(
        Vec3::new(-96.0, DECK_TOP + 5.5, 13.0),
        Vec3::new(4.0, DECK_TOP - 6.0, -6.0),
    );
    scene
}
