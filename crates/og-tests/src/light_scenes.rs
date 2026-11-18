//! The depth wave's light scenes (docs/design.md §10.7): area lights shaded
//! with linearly transformed cosines (`rect-lights`), spot and point light
//! cookies (`light-cookies`), fog lit as a medium by the sun through the
//! slots of a wall and by a spot (`volumetric-fog`: a froxel grid, or the
//! raymarch where asked), and, natively, a Cornell box baked by
//! og-bake in the scene's build with a light volume lighting a ball inside
//! it (`baked-cornell`).

use glam::{Mat3, Mat4, Quat, Vec3};
use og_render::{
    Background, Camera, ColorSpace, Environment, Exposure, Light, LightCookie, LightKind,
    MaterialShader, MeshData, Mips, RenderScene, ShaderMode, StandardMaterial, TextureData,
    VolumetricFog, shapes,
};

/// A mesh with every vertex moved by `m` (positions, normals).
pub fn transformed(mut mesh: MeshData, m: Mat4) -> MeshData {
    let n = Mat3::from_mat4(m).inverse().transpose();
    for v in &mut mesh.vertices {
        v.position = m.transform_point3(Vec3::from(v.position)).to_array();
        v.normal = (n * Vec3::from(v.normal)).normalize_or_zero().to_array();
    }
    mesh
}

/// A quad `a b c d`, counter-clockwise seen from its front.
fn quad(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> MeshData {
    let mut mesh = shapes::plane(1.0, 1.0);
    let normal = (b - a).cross(d - a).normalize();
    // shapes::plane's corners: (-,-) (+,-) (+,+) (-,+) in x and z, wound
    // 0 2 1, 0 3 2 for +y; here a b c d with the winding a b c, a c d.
    for (v, p) in mesh.vertices.iter_mut().zip([a, b, c, d]) {
        v.position = p.to_array();
        v.normal = normal.to_array();
    }
    mesh.indices = vec![0, 1, 2, 0, 2, 3];
    mesh
}

fn matte(color: Vec3, roughness: f32) -> StandardMaterial {
    StandardMaterial {
        color,
        roughness,
        ..Default::default()
    }
}

fn glow(color: Vec3, strength: f32) -> StandardMaterial {
    StandardMaterial {
        color: Vec3::ZERO,
        emissive: color,
        emissiveIntensity: strength,
        ..Default::default()
    }
}

fn put(scene: &mut RenderScene, mesh: MeshData, m: StandardMaterial, t: Mat4) {
    let mesh = scene.add_mesh(mesh);
    let h = scene.add_standard_material(m);
    scene.add_instance(mesh, h, t);
}

fn dark(scene: &mut RenderScene, ambient: f32) {
    scene.lighting.light.intensity = 0.0;
    scene.lighting.light.cast_shadows = false;
    scene.lighting.environment = Environment {
        intensity: ambient,
        background: Background::Color(Vec3::new(0.004, 0.006, 0.015)),
        ..Default::default()
    };
}

/// A rect light's half extents from its centre's frame: `right` and `up`
/// span it, and it faces `right × up` reversed (the entity's -Z).
fn rect(
    position: Vec3,
    right: Vec3,
    up: Vec3,
    size: (f32, f32),
    color: Vec3,
    intensity: f32,
    two_sided: bool,
) -> Light {
    Light {
        kind: LightKind::Rect {
            half_width: right.normalize() * size.0 * 0.5,
            half_height: up.normalize() * size.1 * 0.5,
            two_sided,
        },
        position,
        color,
        intensity,
        range: 12.0,
        cast_shadows: false,
    }
}

/// The emissive slab drawn where a rect light is.
fn fixture(scene: &mut RenderScene, light: &Light) {
    let LightKind::Rect {
        half_width,
        half_height,
        ..
    } = light.kind
    else {
        return;
    };
    let (x, y) = (half_width.normalize(), half_height.normalize());
    let basis = Mat4::from_cols(
        x.extend(0.0),
        y.extend(0.0),
        x.cross(y).extend(0.0),
        light.position.extend(1.0),
    );
    let size = Vec3::new(half_width.length() * 2.0, half_height.length() * 2.0, 0.02);
    put(scene, shapes::cuboid(size), glow(light.color, 2.0), basis);
}

/// Area lights in a dark gallery: a warm strip overhead and two tall
/// panels, teal on the left and magenta on the right, over a polished
/// stone floor that draws their long reflections; a black glass ball, a
/// copper ball and a marble plinth with a white ball take their light,
/// each showing the rectangles' shapes in its highlights (linearly
/// transformed cosines). The panels are drawn where the lights are.
pub fn rect_lights_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    dark(&mut scene, 0.02);
    // Polished dark concrete.
    let floor = scene.add_standard_material(StandardMaterial {
        color: Vec3::splat(0.06),
        roughness: 0.22,
        ..Default::default()
    });
    let mesh = scene.add_mesh(crate::kit::slab(
        glam::Vec2::new(-10.0, -8.0),
        glam::Vec2::new(10.0, 10.0),
        0.0,
        2.0,
    ));
    scene.add_instance(mesh, floor, Mat4::IDENTITY);
    let wall = crate::kit::plaster(&mut scene, [0.11, 0.11, 0.12]);
    let mesh = scene.add_mesh(crate::kit::block(
        Vec3::new(-8.0, 0.0, -3.4),
        Vec3::new(8.0, 5.0, -3.0),
        2.0,
    ));
    scene.add_instance(mesh, wall, Mat4::IDENTITY);
    // A ball of radius `r` at `at` (shapes::sphere is 1 m across).
    let ball = |r: f32, at: Vec3| {
        Mat4::from_scale_rotation_translation(Vec3::splat(2.0 * r), Quat::IDENTITY, at)
    };
    put(
        &mut scene,
        shapes::sphere(48, 32),
        StandardMaterial {
            color: Vec3::splat(0.015),
            roughness: 0.1,
            ..Default::default()
        },
        ball(0.55, Vec3::new(-1.0, 0.55, 0.1)),
    );
    put(
        &mut scene,
        shapes::sphere(48, 32),
        StandardMaterial {
            color: Vec3::new(0.95, 0.64, 0.54),
            metalness: 1.0,
            roughness: 0.28,
            ..Default::default()
        },
        ball(0.38, Vec3::new(1.05, 0.38, 0.7)),
    );
    // The plinth in white marble: grey veins wandering through it.
    let mut veins = crate::scenes::texture(512, ColorSpace::Srgb, |u, v| {
        // Thin veins where a warped diagonal band crosses zero.
        let warp = crate::fx_scenes::tiling_fbm(u, v, 3, 0x6a) * 2.5;
        let band = ((u * 1.0 + v * 2.0 + warp) * std::f32::consts::PI)
            .sin()
            .abs();
        let vein = 1.0 - crate::fx_scenes::smooth(0.0, 0.08, band);
        let cloud = crate::fx_scenes::tiling_fbm(u * 2.0, v * 2.0, 6, 0x6b);
        let g = 0.92 - 0.55 * vein - 0.1 * cloud;
        [(g * 250.0) as u8, (g * 247.0) as u8, (g * 240.0) as u8, 255]
    });
    veins.mips = Mips::Generate;
    let veins = scene.add_texture(veins);
    let marble = scene.add_material(og_render::Material {
        base_color: [1.0; 4],
        base_color_texture: Some(veins),
        roughness: 0.28,
        ..Default::default()
    });
    let plinth = scene.add_mesh(shapes::cuboid(Vec3::new(0.7, 1.1, 0.7)));
    scene.add_instance(
        plinth,
        marble,
        Mat4::from_rotation_translation(Quat::from_rotation_y(0.4), Vec3::new(0.25, 0.55, -1.3)),
    );
    put(
        &mut scene,
        shapes::sphere(48, 32),
        matte(Vec3::splat(0.85), 0.3),
        ball(0.24, Vec3::new(0.25, 1.34, -1.3)),
    );
    scene.lighting.lights = vec![
        // Overhead, facing down: right × up = +y, so up is -z.
        rect(
            Vec3::new(0.0, 3.0, -0.5),
            Vec3::X,
            Vec3::NEG_Z,
            (3.6, 0.3),
            Vec3::new(1.0, 0.78, 0.52),
            8.0,
            false,
        ),
        // Left, facing +x: right × up = -x.
        rect(
            Vec3::new(-2.7, 1.3, -0.3),
            Vec3::Z,
            Vec3::Y,
            (0.3, 2.3),
            Vec3::new(0.25, 0.8, 1.0),
            14.0,
            false,
        ),
        // Right, facing -x: right × up = +x.
        rect(
            Vec3::new(2.7, 1.3, -0.3),
            Vec3::NEG_Z,
            Vec3::Y,
            (0.3, 2.3),
            Vec3::new(1.0, 0.25, 0.65),
            14.0,
            false,
        ),
    ];
    for l in scene.lighting.lights.clone() {
        fixture(&mut scene, &l);
    }
    scene.look.post.bloom = Some(og_render::Bloom::default());
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.2, 5.2),
        target: Vec3::new(0.0, 0.85, -0.6),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// A `size`-square sRGB RGBA8 texture of `f(x, y)`.
fn texture(width: u32, height: u32, f: impl Fn(u32, u32) -> [u8; 3]) -> TextureData {
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let [r, g, b] = f(x, y);
            rgba.extend([r, g, b, 255]);
        }
    }
    TextureData {
        width,
        height,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    }
}

/// The crypt ([`crate::shadow_scenes::crypt_room`]) at night, lit by two
/// cookies: the moon through a rose window high in the south wall, a spot
/// whose stained-glass cookie (red, blue, amber and green panes in dark
/// leading, round a white boss) lays the window's colours across the floor
/// and the altar, and a pierced brass lantern on the floor, a point light with a cube cookie (six faces stacked +X, -X,
/// +Y, -Y, +Z, -Z, each a grid of round holes, closed above) dotting the
/// walls and the floor round it.
pub fn light_cookies_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::shadow_scenes::crypt_room(&mut scene);
    dark(&mut scene, 0.04);
    let window = |x: u32, y: u32| {
        let (dx, dy) = (x as f32 - 63.5, y as f32 - 63.5);
        let r = (dx * dx + dy * dy).sqrt();
        let a = dy.atan2(dx) / std::f32::consts::TAU + 0.5;
        if r > 60.0 {
            return [6, 5, 4];
        }
        if r < 12.0 {
            return if r > 10.0 {
                [10, 8, 6]
            } else {
                [250, 245, 225]
            };
        }
        let ring = if r < 36.0 { 0 } else { 1 };
        let sectors = if ring == 0 { 8.0 } else { 16.0 };
        let s = a * sectors;
        let lead = (s - s.round()).abs() * (r * std::f32::consts::TAU / sectors) < 1.2
            || (r - 36.0).abs() < 1.5
            || (r - 58.5).abs() < 1.5;
        if lead {
            return [10, 8, 6];
        }
        let palette = [[230, 40, 35], [40, 80, 235], [255, 185, 60], [60, 170, 80]];
        palette[(s.floor() as usize + ring * 2) % 4]
    };
    let glass = scene.add_texture(texture(128, 128, window));
    let cube = scene.add_texture(texture(32, 32 * 6, |x, y| {
        let (face, fy) = (y / 32, y % 32);
        let (cx, cy) = ((x % 8) as f32 - 3.5, (fy % 8) as f32 - 3.5);
        let hole = cx * cx + cy * cy < 4.5;
        match face {
            2 => [8, 8, 8],
            _ if hole => [255, 200, 120],
            _ => [8, 8, 8],
        }
    }));
    // The window: its glass glows in the wall, the spot just inside it.
    let pane = Vec3::new(0.2, 3.0, 3.98);
    put(
        &mut scene,
        quad(
            pane + Vec3::new(0.6, -0.6, 0.0),
            pane + Vec3::new(-0.6, -0.6, 0.0),
            pane + Vec3::new(-0.6, 0.6, 0.0),
            pane + Vec3::new(0.6, 0.6, 0.0),
        ),
        StandardMaterial {
            color: Vec3::ZERO,
            emissive: Vec3::ONE,
            emissiveIntensity: 1.5,
            emissiveMap: glass.slot(),
            ..Default::default()
        },
        Mat4::IDENTITY,
    );
    let lamp = Vec3::new(1.9, 0.45, 1.9);
    scene.lighting.lights = vec![
        Light {
            kind: LightKind::Spot {
                direction: Vec3::new(-0.45, -0.7, -0.75).normalize(),
                angle_degrees: 26.0,
                penumbra: 0.1,
            },
            position: pane - Vec3::Z * 0.1,
            color: Vec3::new(0.9, 0.92, 1.0),
            intensity: 90.0,
            range: 14.0,
            cast_shadows: false,
        },
        Light {
            kind: LightKind::Point,
            position: lamp,
            color: Vec3::new(1.0, 0.75, 0.45),
            intensity: 10.0,
            range: 8.0,
            cast_shadows: false,
        },
    ];
    scene.lighting.light_cookies = vec![
        LightCookie {
            light: 0,
            texture: glass,
        },
        LightCookie {
            light: 1,
            texture: cube,
        },
    ];
    // The lantern's brass body, its flame glowing through it.
    put(
        &mut scene,
        shapes::sphere(16, 8),
        glow(Vec3::new(1.0, 0.7, 0.35), 2.0),
        Mat4::from_scale_rotation_translation(Vec3::splat(0.1), Quat::IDENTITY, lamp),
    );
    scene.frame.camera = Camera {
        position: Vec3::new(3.3, 1.9, -3.2),
        target: Vec3::new(-2.0, 1.0, 1.4),
        exposure: Exposure::linear(1.2),
        ..Default::default()
    };
    scene
}

/// Fog lit as a medium: a pine wood at sunrise, the sun low behind the
/// trees ahead throwing shafts between their trunks and crowns towards the
/// camera (anisotropy 0.6, forward scattering) and bright bars on the
/// forest floor; a lantern over the watchtower's door in the clearing
/// draws its warm cone in the fog (the clustered lights light the froxels,
/// and the raymarch's steps).
pub fn volumetric_fog_scene() -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let floor = kit::surface_tinted(&mut scene, "grass", [0.62, 0.7, 0.5]);
    let ground = scene.add_mesh(kit::slab(
        glam::Vec2::splat(-300.0),
        glam::Vec2::splat(300.0),
        0.0,
        6.0,
    ));
    scene.add_instance(ground, floor, Mat4::IDENTITY);
    kit::hills(&mut scene, None);
    let pine = kit::prop(&mut scene, "pine");
    // Pines all round, thickest ahead where the sun comes through; none in
    // the clearing before the camera.
    for k in 0..150 {
        let h = |n: i32| crate::fx_scenes::hash2(k, n, 0xf06);
        let (x, z) = (-60.0 + h(0) * 120.0, 18.0 - h(1) * 110.0);
        let clearing = (x - 1.0).powi(2) / 64.0 + (z + 4.0).powi(2) / 100.0 < 1.0;
        if clearing {
            continue;
        }
        pine.place(
            &mut scene,
            Vec3::new(x, 0.0, z),
            h(2) * 6.0,
            14.0 + h(3) * 10.0,
        );
    }
    let boulder = kit::prop(&mut scene, "boulder");
    boulder.place(&mut scene, Vec3::new(-4.5, -0.3, -3.0), 1.2, 1.6);
    boulder.place(&mut scene, Vec3::new(5.5, -0.4, 1.0), 4.0, 1.2);
    kit::prop(&mut scene, "watchtower").place(&mut scene, Vec3::new(11.0, 0.0, -17.0), 2.4, 16.0);
    let sun = Vec3::new(-0.12, 0.22, -1.0).normalize();
    crate::sky_scenes::physical_light_under(&mut scene, sun);
    scene.lighting.light.max_shadow_distance = 80.0;
    scene.lighting.lights = vec![Light {
        kind: LightKind::Spot {
            direction: Vec3::new(-0.35, -1.0, 0.25).normalize(),
            angle_degrees: 26.0,
            penumbra: 0.3,
        },
        position: Vec3::new(8.6, 4.0, -13.6),
        color: Vec3::new(1.0, 0.6, 0.3),
        intensity: 60.0,
        range: 10.0,
        cast_shadows: true,
    }];
    put(
        &mut scene,
        shapes::sphere(12, 8),
        glow(Vec3::new(1.0, 0.65, 0.3), 2.5),
        Mat4::from_scale_rotation_translation(
            Vec3::splat(0.14),
            Quat::IDENTITY,
            Vec3::new(8.6, 4.1, -13.6),
        ),
    );
    scene.atmosphere.volumetric_fog = Some(VolumetricFog {
        color: Vec3::new(0.9, 0.92, 0.96),
        // Morning mist in the wood: the fog shows the sun's and the
        // lantern's light, a little of the sky's.
        ambient: Vec3::new(0.02, 0.025, 0.03),
        density: 0.035,
        height: 0.0,
        height_density: 0.03,
        falloff: 0.15,
        scattering: 0.9,
        anisotropy: 0.6,
        distance: 60.0,
    });
    scene.frame.camera = Camera {
        position: Vec3::new(-2.5, 1.7, 8.0),
        target: Vec3::new(1.5, 3.6, -20.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 1.2 },
        ..Default::default()
    };
    scene
}

/// A Cornell box (white floor, ceiling and back, a red wall on the left,
/// a green one on the right, two white blocks) lit by a spot under its
/// ceiling, every surface lightmapped by og-bake when the scene is built
/// (xatlas charts, 64 paths a texel, one thread, a fixed seed: the same
/// pages every build), and a ball inside lit by the box's light volume
/// (L1 harmonics, 0.3 m apart). The direct spot and its shadows are drawn
/// live; the bake carries the bounces: the white surfaces take red and
/// green from the walls beside them and the ceiling is lit by bounces
/// alone. `baked: false` builds it unbaked (the environment's ambient).
#[cfg(not(target_arch = "wasm32"))]
pub fn baked_cornell_scene(baked: bool) -> Result<RenderScene, String> {
    let mut scene = cornell_view();
    cornell_box(&mut scene, Vec3::ZERO, false, baked, false)?;
    Ok(scene)
}

/// The dark scene and the camera the one-box Cornell scenes share.
#[cfg(not(target_arch = "wasm32"))]
fn cornell_view() -> RenderScene {
    let mut scene = RenderScene::new();
    dark(&mut scene, 0.35);
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.0, 3.7),
        target: Vec3::new(0.0, 0.95, 0.0),
        projection: og_render::Projection::Perspective {
            fov_y_degrees: 40.0,
        },
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    scene
}

/// Two of [`baked_cornell_scene`]'s boxes side by side, 2.4 m apart, the
/// right one with its red and green walls swapped, each baked on its own:
/// its own pages and light volume, as two baked scenes loaded together hand
/// them over (`og.load.scene` twice, or one scene streamed in beside
/// another). The left box's short block is green on the side facing its
/// green wall, the right box's tall block green on the side facing its
/// green wall, only when each box draws its own pages: the other's are red
/// there.
#[cfg(not(target_arch = "wasm32"))]
pub fn baked_cornell_pair_scene() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    dark(&mut scene, 0.35);
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.0, 5.6),
        target: Vec3::new(0.0, 0.95, 0.0),
        projection: og_render::Projection::Perspective {
            fov_y_degrees: 40.0,
        },
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    cornell_box(&mut scene, Vec3::new(-1.2, 0.0, 0.0), false, true, false)?;
    cornell_box(&mut scene, Vec3::new(1.2, 0.0, 0.0), true, true, false)?;
    Ok(scene)
}

/// [`baked_cornell_scene`]'s box with its parts drawn by every kind of lit
/// batch (`baked-batches`): the ceiling a physical material, the back wall
/// a plain material from compact vertices, the tall block an array-map
/// material and the short block a material shader; floor and side walls
/// standard. Each part draws its bake, so the frame is `baked-cornell`'s
/// but for the physical ceiling's specular; a batch that ignores its
/// lightmap takes the probes or the sky's ambient instead (the ceiling,
/// outside the light volume, goes near black).
#[cfg(not(target_arch = "wasm32"))]
pub fn baked_batches_scene(baked: bool) -> Result<RenderScene, String> {
    let mut scene = cornell_view();
    cornell_box(&mut scene, Vec3::ZERO, false, baked, true)?;
    Ok(scene)
}

/// `baked-cornell` and `baked-batches` as a shot of a game: a monastery's
/// closed cellar at night, 4 × 3 × 4 m, seen from just inside its front
/// wall, lit by one lantern hung under the middle
/// of its ceiling (a spot thrown down): a plank floor, a limewashed ceiling
/// and back wall, the left wall painted red and the right green, a
/// plastered pier standing out of the back wall on the left and a plank
/// chest at the front right, every one of them lightmapped by og-bake when the scene is built
/// (its parts in [`baked_cornell_scene`]'s order, so with `batches` each
/// is drawn by the same kind of batch as there), and wine barrels and
/// crates lit by the cellar's light volume. The lantern and its shadows
/// are drawn live; the bake carries the bounces: the pier's side and the
/// floor take red from the left wall, the chest green from the right,
/// and the ceiling is lit by bounces alone. [`baked_cornell_scene`] and
/// [`baked_batches_scene`] are the plain fixtures `tests/light.rs` reads.
#[cfg(not(target_arch = "wasm32"))]
pub fn baked_cellar_scene(batches: bool) -> Result<RenderScene, String> {
    use crate::kit;
    let mut scene = RenderScene::new();
    dark(&mut scene, 0.1);
    let [planks, planks_normal, _] = kit::surface_textures(&mut scene, "planks");
    let [_, stone_normal, _] = kit::surface_textures(&mut scene, "stone");
    let paint = |color: Vec3| StandardMaterial {
        color,
        normalMap: stone_normal.slot(),
        roughness: 0.9,
        ..Default::default()
    };
    let wood = |tint: Vec3| StandardMaterial {
        color: tint,
        map: planks.slot(),
        normalMap: planks_normal.slot(),
        roughness: 0.85,
        ..Default::default()
    };
    // The planks' mean linear albedo (their colour map averaged), which
    // og-bake bakes them with.
    let planks_albedo = Vec3::new(0.165, 0.143, 0.115);
    let (white, red, green) = (
        Vec3::new(0.78, 0.75, 0.68),
        Vec3::new(0.6, 0.07, 0.04),
        Vec3::new(0.09, 0.36, 0.11),
    );
    let block = |min: Vec3, max: Vec3| {
        let size = max - min;
        transformed(
            shapes::cuboid(size),
            Mat4::from_translation((min + max) * 0.5),
        )
    };
    // (mesh, the albedo it is baked with, how it draws)
    let parts: Vec<(MeshData, Vec3, StandardMaterial)> = vec![
        (
            quad(
                Vec3::new(-2.0, 0.0, 2.0),
                Vec3::new(2.0, 0.0, 2.0),
                Vec3::new(2.0, 0.0, -2.0),
                Vec3::new(-2.0, 0.0, -2.0),
            ),
            planks_albedo * 1.2,
            wood(Vec3::splat(1.2)),
        ),
        (
            quad(
                Vec3::new(-2.0, 3.0, -2.0),
                Vec3::new(2.0, 3.0, -2.0),
                Vec3::new(2.0, 3.0, 2.0),
                Vec3::new(-2.0, 3.0, 2.0),
            ),
            white,
            paint(white),
        ),
        (
            quad(
                Vec3::new(-2.0, 0.0, -2.0),
                Vec3::new(2.0, 0.0, -2.0),
                Vec3::new(2.0, 3.0, -2.0),
                Vec3::new(-2.0, 3.0, -2.0),
            ),
            white,
            paint(white),
        ),
        (
            quad(
                Vec3::new(-2.0, 0.0, 2.0),
                Vec3::new(-2.0, 0.0, -2.0),
                Vec3::new(-2.0, 3.0, -2.0),
                Vec3::new(-2.0, 3.0, 2.0),
            ),
            red,
            paint(red),
        ),
        (
            quad(
                Vec3::new(2.0, 0.0, -2.0),
                Vec3::new(2.0, 0.0, 2.0),
                Vec3::new(2.0, 3.0, 2.0),
                Vec3::new(2.0, 3.0, -2.0),
            ),
            green,
            paint(green),
        ),
        (
            block(Vec3::new(-1.5, 0.0, -2.0), Vec3::new(-0.7, 3.0, -1.45)),
            white,
            paint(white),
        ),
        (
            block(Vec3::new(0.7, 0.0, 0.2), Vec3::new(1.8, 0.6, 0.9)),
            planks_albedo * 1.2,
            wood(Vec3::splat(1.2)),
        ),
        // The front wall behind the camera: the cellar is closed, so what
        // faces the camera takes the light the front wall bounces back.
        (
            quad(
                Vec3::new(2.0, 0.0, 2.0),
                Vec3::new(-2.0, 0.0, 2.0),
                Vec3::new(-2.0, 3.0, 2.0),
                Vec3::new(2.0, 3.0, 2.0),
            ),
            white,
            paint(white),
        ),
    ];
    let lantern = Vec3::new(0.0, 2.75, 0.0);
    let spot = Light {
        kind: LightKind::Spot {
            direction: Vec3::NEG_Y,
            angle_degrees: 62.0,
            penumbra: 0.4,
        },
        position: lantern - Vec3::Y * 0.12,
        color: Vec3::new(1.0, 0.78, 0.5),
        intensity: 16.0,
        range: 8.0,
        cast_shadows: true,
    };
    scene.lighting.lights.push(spot);
    put(
        &mut scene,
        shapes::sphere(16, 12),
        glow(Vec3::new(1.0, 0.78, 0.5), 6.0),
        Mat4::from_scale_rotation_translation(Vec3::splat(0.16), Quat::IDENTITY, lantern),
    );
    // The lantern's chain and the cellar's stores, lit by its light volume.
    put(
        &mut scene,
        shapes::cuboid(Vec3::new(0.02, 0.17, 0.02)),
        matte(Vec3::splat(0.05), 0.5),
        Mat4::from_translation(lantern + Vec3::Y * 0.165),
    );
    let barrel = kit::prop(&mut scene, "winebarrel");
    barrel.place(&mut scene, Vec3::new(-1.5, 0.0, 0.25), 0.4, 0.95);
    barrel.place(&mut scene, Vec3::new(-1.45, 0.0, -0.6), 1.5, 0.95);
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(1.3, 0.0, -1.5), 0.2, 0.6);
    crate_.place(&mut scene, Vec3::new(1.35, 0.6, -1.55), -0.15, 0.55);
    let LightKind::Spot {
        direction,
        angle_degrees,
        penumbra,
    } = spot.kind
    else {
        unreachable!("the lantern's light is a spot");
    };
    let lights = [og_bake::Light::Spot {
        position: spot.position,
        direction,
        radiance: spot.color * spot.intensity,
        range: spot.range,
        cos_outer: angle_degrees.to_radians().cos(),
        cos_inner: (angle_degrees * (1.0 - penumbra)).to_radians().cos(),
    }];
    let (meshes, materials): (Vec<_>, Vec<_>) = parts
        .into_iter()
        .map(|(m, albedo, draw)| ((m, albedo), draw))
        .unzip();
    bake_parts(
        &mut scene,
        meshes,
        Some(BakeLight {
            sky: Vec3::splat(0.02),
            lights: &lights,
            volume: og_bake::VolumeDesc {
                min: Vec3::new(-1.9, 0.05, -1.9),
                max: Vec3::new(1.9, 2.9, 1.9),
                spacing: 0.4,
            },
        }),
        &mut |scene: &mut RenderScene, i, mesh, _| {
            if batches && i == 2 {
                scene.set_vertex_encoding(mesh, og_render::VertexEncoding::Compact);
            }
            let material = part_material(scene, i, materials[i], batches);
            scene.add_instance(mesh, material, Mat4::IDENTITY);
        },
    )?;
    // Inside, at the front wall.
    scene.frame.camera = Camera {
        position: Vec3::new(0.5, 1.55, 1.85),
        target: Vec3::new(-0.2, 1.1, -1.0),
        projection: og_render::Projection::Perspective {
            fov_y_degrees: 62.0,
        },
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    Ok(scene)
}

/// The material of part `i` of a baked room (0 floor, 1 ceiling, 2 back,
/// 3 and 4 the side walls, 5 the tall block, 6 the short one): `m`, or
/// with `batches` the batch kind [`baked_batches_scene`] names drawing it
/// (the back wall's compact vertices are set by the caller).
#[cfg(not(target_arch = "wasm32"))]
fn part_material(
    scene: &mut RenderScene,
    i: usize,
    m: StandardMaterial,
    batches: bool,
) -> og_render::MaterialHandle {
    use og_render::{MaterialShader, ParamKind, ShaderParam, TextureArrayData, TextureSource};
    if !batches {
        return scene.add_standard_material(m);
    }
    match i {
        // Physical: an ior other than 1.5 is a physical lobe.
        1 => scene.add_standard_material(StandardMaterial { ior: 1.45, ..m }),
        5 => {
            let white = TextureData {
                width: 4,
                height: 4,
                rgba: vec![255; 64],
                color_space: ColorSpace::Srgb,
                mips: Mips::Generate,
            };
            let array = scene
                .add_texture_array(TextureArrayData {
                    layers: vec![
                        TextureSource::Rgba(white.clone()),
                        TextureSource::Rgba(white),
                    ],
                })
                .expect("two white layers");
            scene.add_standard_material(StandardMaterial {
                mapArray: array.slot(),
                mapLayer: 1,
                ..m
            })
        }
        6 => {
            let shader = scene.add_shader(MaterialShader {
                name: "tint".into(),
                code: "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {\n  return vec4<f32>(c.rgb * p.gain, c.a);\n}\n".into(),
                params: vec![ShaderParam {
                    name: "gain".into(),
                    kind: ParamKind::Float,
                    default: [1.0, 0.0, 0.0, 0.0],
                }],
                mode: Default::default(),
            });
            let material = scene.add_standard_material(m);
            scene.set_material_shader(material, Some(shader));
            material
        }
        _ => scene.add_standard_material(m),
    }
}

/// One Cornell box at `at` (red wall on the left, green on the right, or
/// the other way round when `swapped`): its spot, its ball and its walls
/// and blocks (each part's material [`part_material`]'s), baked by og-bake
/// on their own when `baked` (its pages and its light volume added to the
/// scene's bake).
#[cfg(not(target_arch = "wasm32"))]
fn cornell_box(
    scene: &mut RenderScene,
    at: Vec3,
    swapped: bool,
    baked: bool,
    batches: bool,
) -> Result<(), String> {
    let (white, red, green) = (
        Vec3::new(0.73, 0.73, 0.73),
        Vec3::new(0.63, 0.065, 0.05),
        Vec3::new(0.14, 0.45, 0.09),
    );
    let (red, green) = if swapped { (green, red) } else { (red, green) };
    let p = |x: f32, y: f32, z: f32| at + Vec3::new(x, y, z);
    let block = |size: Vec3, at: Vec3, turn: f32| {
        transformed(
            shapes::cuboid(size),
            Mat4::from_rotation_translation(Quat::from_rotation_y(turn.to_radians()), at),
        )
    };
    let parts: Vec<(MeshData, Vec3)> = vec![
        (
            quad(
                p(-1., 0., 1.),
                p(1., 0., 1.),
                p(1., 0., -1.),
                p(-1., 0., -1.),
            ),
            white,
        ),
        (
            quad(
                p(-1., 2., -1.),
                p(1., 2., -1.),
                p(1., 2., 1.),
                p(-1., 2., 1.),
            ),
            white,
        ),
        (
            quad(
                p(-1., 0., -1.),
                p(1., 0., -1.),
                p(1., 2., -1.),
                p(-1., 2., -1.),
            ),
            white,
        ),
        (
            quad(
                p(-1., 0., 1.),
                p(-1., 0., -1.),
                p(-1., 2., -1.),
                p(-1., 2., 1.),
            ),
            red,
        ),
        (
            quad(p(1., 0., -1.), p(1., 0., 1.), p(1., 2., 1.), p(1., 2., -1.)),
            green,
        ),
        (
            block(Vec3::new(0.6, 1.2, 0.6), p(-0.35, 0.6, -0.3), 18.0),
            white,
        ),
        (block(Vec3::splat(0.6), p(0.38, 0.3, 0.3), -18.0), white),
    ];
    let spot = Light {
        kind: LightKind::Spot {
            direction: Vec3::NEG_Y,
            angle_degrees: 62.0,
            penumbra: 0.4,
        },
        position: p(0.0, 1.96, 0.0),
        color: Vec3::new(1.0, 0.95, 0.88),
        intensity: 7.0,
        range: 5.0,
        cast_shadows: true,
    };
    // Centre and radius: resting on the floor.
    let ball = (p(-0.55, 0.22, 0.55), 0.22);

    scene.lighting.lights.push(spot);
    put(
        scene,
        shapes::sphere(32, 16),
        matte(Vec3::splat(0.85), 0.7),
        // shapes::sphere has radius 0.5: scaled by the diameter.
        Mat4::from_scale_rotation_translation(Vec3::splat(2.0 * ball.1), Quat::IDENTITY, ball.0),
    );
    let place = |scene: &mut RenderScene, i: usize, mesh: og_render::MeshHandle, color: Vec3| {
        if batches && i == 2 {
            scene.set_vertex_encoding(mesh, og_render::VertexEncoding::Compact);
        }
        let material = part_material(scene, i, matte(color, 0.9), batches);
        scene.add_instance(mesh, material, Mat4::IDENTITY);
    };
    let LightKind::Spot {
        direction,
        angle_degrees,
        penumbra,
    } = spot.kind
    else {
        unreachable!("the box's light is a spot");
    };
    let lights = [og_bake::Light::Spot {
        position: spot.position,
        direction,
        radiance: spot.color * spot.intensity,
        range: spot.range,
        cos_outer: angle_degrees.to_radians().cos(),
        cos_inner: (angle_degrees * (1.0 - penumbra)).to_radians().cos(),
    }];
    let volume = og_bake::VolumeDesc {
        min: p(-0.95, 0.05, -0.95),
        max: p(0.95, 1.95, 0.95),
        spacing: 0.3,
    };
    bake_parts(
        scene,
        parts,
        baked.then_some(BakeLight {
            sky: Vec3::splat(0.05),
            lights: &lights,
            volume,
        }),
        &mut |scene: &mut RenderScene, i, mesh, color| place(scene, i, mesh, color),
    )
}

/// What [`bake_parts`] bakes its parts under: the sky's radiance, the
/// lights and the light volume's box.
#[cfg(not(target_arch = "wasm32"))]
struct BakeLight<'a> {
    sky: Vec3,
    lights: &'a [og_bake::Light],
    volume: og_bake::VolumeDesc,
}

/// Adds `parts` (each a mesh and the linear albedo og-bake bakes it with)
/// to `scene`, each drawn by `place` (its index, its mesh, its albedo),
/// and with `bake_light` lightmaps them by og-bake when the scene is built
/// (xatlas charts 16 texels a metre in pages up to 256, 64 paths a texel,
/// one thread: the same pages every build) and bakes the light volume
/// over its `volume` (L1 harmonics), its pages and volume added to the
/// scene's bake. Without, the parts are drawn unbaked.
#[cfg(not(target_arch = "wasm32"))]
fn bake_parts(
    scene: &mut RenderScene,
    parts: Vec<(MeshData, Vec3)>,
    bake_light: Option<BakeLight>,
    place: &mut dyn FnMut(&mut RenderScene, usize, og_render::MeshHandle, Vec3),
) -> Result<(), String> {
    use og_render::{BakedMesh, LightmapPages, LightmapUvs, ProbeVolume};
    use std::sync::Arc;

    let Some(light) = bake_light else {
        for (i, (mesh, color)) in parts.into_iter().enumerate() {
            let mesh = scene.add_mesh(mesh);
            place(scene, i, mesh, color);
        }
        return Ok(());
    };
    let mut bake = og_bake::Scene {
        sky: light.sky,
        ..Default::default()
    };
    for (i, (mesh, color)) in parts.iter().enumerate() {
        bake.materials.push(og_bake::Material {
            base_color: *color,
            ..Default::default()
        });
        bake.meshes.push(og_bake::Mesh {
            positions: mesh
                .vertices
                .iter()
                .map(|v| Vec3::from(v.position))
                .collect(),
            normals: mesh.vertices.iter().map(|v| Vec3::from(v.normal)).collect(),
            indices: mesh.indices.clone(),
            material: i,
            lightmapped: true,
            ..Default::default()
        });
    }
    bake.lights.extend_from_slice(light.lights);
    let unwrap: Vec<og_bake::UnwrapMesh> = parts
        .iter()
        .map(|(m, _)| og_bake::UnwrapMesh {
            positions: m.vertices.iter().map(|v| v.position).collect(),
            normals: m.vertices.iter().map(|v| v.normal).collect(),
            indices: m.indices.clone(),
        })
        .collect();
    let atlas = og_bake::unwrap(
        &unwrap,
        &og_bake::PackSettings {
            texels_per_unit: 16.0,
            max_size: 256,
            ..Default::default()
        },
    )?;
    let lit: Vec<usize> = (0..parts.len()).collect();
    let lightmap = og_bake::lightmap::bake(
        &bake,
        &atlas,
        &lit,
        &og_bake::TraceSettings {
            samples: 64,
            threads: 1,
            ..Default::default()
        },
        &mut |_, _, _| {},
    );
    let volume = og_bake::volume::bake(&bake, &light.volume, 64, 3, 1, 1);
    let pages = Arc::new(LightmapPages {
        width: lightmap.width,
        height: lightmap.height,
        pages: lightmap.pages,
    });
    for (i, ((mesh, color), a)) in parts.into_iter().zip(&atlas.meshes).enumerate() {
        let lit = MeshData {
            vertices: a
                .vertices
                .iter()
                .map(|v| mesh.vertices[v.xref as usize])
                .collect(),
            indices: a.indices.clone(),
        };
        let uvs = a
            .vertices
            .iter()
            .map(|v| {
                [
                    v.uv[0] / atlas.width as f32,
                    v.uv[1] / atlas.height as f32,
                    v.page as f32,
                ]
            })
            .collect();
        let handle = scene.add_mesh(lit);
        scene.lighting.baked.meshes.insert(
            handle.index(),
            BakedMesh {
                pages: pages.clone(),
                uvs: Arc::new(LightmapUvs { uvs }),
            },
        );
        place(scene, i, handle, color);
    }
    scene.lighting.baked.volumes.push(ProbeVolume {
        min: volume.min,
        max: volume.max,
        fade: 0.0,
        counts: volume.counts,
        probes: Arc::new(volume.probes),
    });
    Ok(())
}

/// The island's meadow at night (`grass` with its scattered grass, else the
/// bare terrain) lit by one upright one-sided rect light 10 m ahead of the
/// camera, its face turned to the camera's left. With linearly transformed
/// cosines the meadow left of the panel is lit and the meadow right of it,
/// behind its back, stays dark; a point at its centre (how terrain and
/// scatter lit rect lights before) lights both sides alike. Also the two
/// ground points, left and right of the panel, 2.5 m from it.
pub fn rect_meadow_scene(grass: bool) -> (RenderScene, [Vec3; 2]) {
    use crate::terrain_scenes::{grass_scene, island_height, island_view};
    let mut scene = if grass {
        grass_scene()
    } else {
        island_view("near")
    };
    scene.lighting.light.intensity = 0.0;
    scene.lighting.light.cast_shadows = false;
    scene.lighting.environment.intensity = 0.02;
    scene.lighting.environment.background = Background::Color(Vec3::new(0.004, 0.006, 0.015));
    scene.atmosphere.fog = None;
    let eye = scene.frame.camera.position;
    let ahead = (scene.frame.camera.target - eye) * Vec3::new(1.0, 0.0, 1.0);
    let forward = ahead.normalize();
    let side = forward.cross(Vec3::Y).normalize();
    let foot = eye + forward * 10.0;
    let ground = |p: Vec3| Vec3::new(p.x, island_height(p.x, p.z), p.z);
    let centre = ground(foot) + Vec3::Y * 0.9;
    // Spanned by forward and up, so it faces -(forward × up): the left.
    let panel = rect(
        centre,
        forward,
        Vec3::Y,
        (1.6, 1.4),
        Vec3::new(1.0, 0.85, 0.6),
        6.0,
        false,
    );
    fixture(&mut scene, &panel);
    scene.lighting.lights = vec![panel];
    (
        scene,
        [ground(foot - side * 2.5), ground(foot + side * 2.5)],
    )
}

/// A shadowed rect light: a 1.6 m square panel facing down 3 m over a
/// matte floor and 1.2 m to the left of a 2 m post standing on it, the
/// panel's emissive slab drawn where it is. With `cast` the post throws a
/// shadow to the right, sharp at its foot and widening into penumbra
/// toward its tip (x 2.4); without, the floor is lit through it. Returns
/// the scene and floor points along the shadow's axis: by the post's foot
/// and near the tip.
pub fn rect_shadow_scene(cast: bool) -> (RenderScene, [Vec3; 2]) {
    let mut scene = RenderScene::new();
    dark(&mut scene, 0.02);
    put(
        &mut scene,
        shapes::plane(14.0, 1.0),
        matte(Vec3::splat(0.6), 0.9),
        Mat4::IDENTITY,
    );
    put(
        &mut scene,
        shapes::cuboid(Vec3::new(0.3, 2.0, 0.3)),
        matte(Vec3::new(0.7, 0.3, 0.2), 0.8),
        Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0)),
    );
    // Facing down: right × up = +y, so up is -z.
    let mut panel = rect(
        Vec3::new(-1.2, 3.0, 0.0),
        Vec3::X,
        Vec3::NEG_Z,
        (1.6, 1.6),
        Vec3::new(1.0, 0.9, 0.75),
        4.0,
        false,
    );
    panel.cast_shadows = cast;
    fixture(&mut scene, &panel);
    scene.lighting.lights = vec![panel];
    scene.frame.camera = Camera {
        position: Vec3::new(0.6, 4.5, 5.5),
        target: Vec3::new(0.6, 0.3, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    (scene, [Vec3::new(0.45, 0.0, 0.0), Vec3::new(2.1, 0.0, 0.0)])
}

/// [`rect_shadow_scene`] with the floor and the post drawn by a full
/// custom lit shader that keeps the standard entries (`custom_lit.wgsl`):
/// lit as the standard material is, the rect light as an area with its
/// soft shadow. Lit as a point at the panel's centre without its shadow,
/// as before the debt round 2, the floor reads otherwise and the post
/// throws nothing.
pub fn rect_shadow_custom_scene(cast: bool) -> (RenderScene, [Vec3; 2]) {
    let (mut scene, points) = rect_shadow_scene(cast);
    let shader = scene.add_shader(MaterialShader {
        name: "standard-full".into(),
        code: String::new(),
        params: Vec::new(),
        mode: ShaderMode::Full { lit: true },
    });
    let drawn: Vec<_> = scene.objects.instances[..2]
        .iter()
        .map(|i| i.material)
        .collect();
    for m in drawn {
        assert!(scene.set_material_shader(m, Some(shader)));
    }
    (scene, points)
}
