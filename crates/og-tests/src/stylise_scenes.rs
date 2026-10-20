//! The stylisation scenes (`og-render/src/stylise.rs`, the IDL's `Stylise`
//! and `MaterialStyle`): each primitive group over a scene the references
//! already know, so a frame shows the primitive and nothing else new.
//!
//! - `stylise-pixel`: the toon scene on a 180-row grid scaled up by whole
//!   numbers, six tones per channel through a Bayer 4 dither.
//! - `stylise-palette`: the same at 240 rows, every pixel the nearest of a
//!   16-colour palette, blue-noise dithered.
//! - `stylise-print`: cross-hatching and ink lines over the physical
//!   materials scene (a pencil drawing).
//! - `stylise-halftone`: halftone dots over the toon scene.
//! - `stylise-halftone-surface`: the same dots laid on the surfaces (dot
//!   grids on the three world planes, larger where the tone is darker,
//!   foreshortened on the floor and wrapping round the forms), with ink on
//!   luminance and depth edges: a comic print given depth.
//! - `stylise-paint`: painterly smoothing, edge darkening and bleed over the
//!   physical materials scene.
//! - `stylise-bands`: four flat depth bands over the toon scene (layered
//!   silhouettes).
//! - `stylise-retro`: vertex snapping, affine textures and 12-bit depth
//!   over the physical materials scene.
//! - `stylise-material`: the material style alone: a warm shadow tint, a
//!   rim, outlines that thin with distance, and flat facets, over the
//!   physical materials scene (no per-material stylised fields), and toon
//!   on every lit material without a physical lobe (the physical ones keep
//!   their coats, sheen, film, anisotropy and glass).
//! - `stylise-outline-voxels`: hull outlines on unwelded voxel meshes.
//! - `stylise-toon-physical`: a toon style over a plain, a lacquered, a
//!   jelly and an unstyled sphere.
//! - `stylise-ink-depth`: ink with depth edges over flat, unlit shapes of
//!   one colour on a floor of the same colour, where luminance finds no
//!   edge: every line is a silhouette or a crease from the prepass.
//! - `stylise-surface`: a pencil drawing laid on the surfaces: hatching in
//!   surface space (strokes on the three world planes, wrapping round the
//!   spheres, keeping about their pixel spacing by octaves from the near
//!   box to the far sphere), a paper canvas on the surfaces (the grain
//!   foreshortened on the floor, the sky keeping the screen grain) and
//!   depth ink with its thresholds set.
//! - `stylise-object`: the same drawing in object space, with a box
//!   tilted on two axes and a long beam scaled 4 x 0.4 x 0.5 laid across
//!   the floor: each instance's strokes and grain run along its own faces
//!   (square on the beam despite its scale), where world space blends the
//!   world's planes across a turned face.

use glam::{Mat4, Quat, Vec3};
use og_render::{
    Background, Camera, ColorSpace, DepthBands, DirectionalLight, DitherPattern, Exposure,
    Halftone, Hatching, Ink, MaterialStyle, Paint, Palette, PixelScale, Quantize, RenderScene,
    Retro3d, StandardMaterial, StyliseSpace, shapes,
};

/// A 16-colour palette: a dusk ramp from near-black through purples and
/// oranges to cream (sRGB).
fn dusk_palette() -> Palette {
    Palette {
        colors: vec![
            [16, 12, 24],
            [40, 24, 56],
            [72, 36, 88],
            [112, 48, 104],
            [160, 64, 104],
            [200, 88, 96],
            [232, 128, 88],
            [248, 176, 96],
            [252, 216, 136],
            [255, 244, 200],
            [48, 72, 96],
            [64, 112, 128],
            [96, 160, 152],
            [152, 208, 176],
            [208, 240, 208],
            [255, 255, 255],
        ],
    }
}

pub fn pixel_scene() -> Result<RenderScene, String> {
    let mut scene = crate::fx_scenes::stylised_scene()?;
    scene.stylise.pixel = Some(PixelScale {
        height: 180,
        upscale: 0,
        snap: true,
    });
    scene.stylise.quantize = Some(Quantize {
        posterize: 6,
        palette: None,
        dither: DitherPattern::Bayer4,
        dither_strength: 1.0,
        dither_scale: 1,
    });
    Ok(scene)
}

pub fn palette_scene() -> Result<RenderScene, String> {
    let mut scene = crate::fx_scenes::stylised_scene()?;
    let palette = scene.add_palette(dusk_palette());
    scene.stylise.pixel = Some(PixelScale {
        height: 240,
        upscale: 0,
        snap: true,
    });
    scene.stylise.quantize = Some(Quantize {
        posterize: 0,
        palette: Some(palette),
        dither: DitherPattern::BlueNoise,
        dither_strength: 1.0,
        dither_scale: 1,
    });
    Ok(scene)
}

pub fn print_scene() -> RenderScene {
    let mut scene = crate::physical_scenes::physical_scene();
    scene.stylise.hatching = Some(Hatching {
        strength: 0.9,
        scale: 6.0,
        levels: 4,
        angle_degrees: 45.0,
        color: [0.04, 0.035, 0.03],
        space: StyliseSpace::Screen,
        depth: 0.0,
    });
    scene.stylise.ink = Some(Ink {
        threshold: 0.12,
        width: 1.5,
        color: [0.02, 0.02, 0.03],
        depth: 0.0,
        ..Default::default()
    });
    scene
}

pub fn halftone_scene() -> Result<RenderScene, String> {
    let mut scene = crate::fx_scenes::stylised_scene()?;
    scene.stylise.halftone = Some(Halftone {
        strength: 1.0,
        size: 7.0,
        angle_degrees: 30.0,
        color: [0.05, 0.02, 0.08],
        space: StyliseSpace::Screen,
    });
    Ok(scene)
}

pub fn halftone_surface_scene() -> Result<RenderScene, String> {
    let mut scene = crate::fx_scenes::stylised_scene()?;
    scene.stylise.halftone = Some(Halftone {
        strength: 1.0,
        size: 9.0,
        angle_degrees: 30.0,
        color: [0.05, 0.02, 0.08],
        space: StyliseSpace::Surface,
    });
    scene.stylise.ink = Some(Ink {
        threshold: 0.4,
        width: 1.0,
        color: [0.02, 0.02, 0.03],
        depth: 1.0,
        ..Default::default()
    });
    Ok(scene)
}

pub fn paint_scene() -> RenderScene {
    let mut scene = crate::physical_scenes::physical_scene();
    scene.stylise.paint = Some(Paint {
        radius: 4.0,
        anisotropy: 0.6,
        edge_darkening: 0.6,
        bleed: 0.4,
        canvas: None,
        canvas_strength: 0.0,
        canvas_scale: 1.0,
        canvas_space: StyliseSpace::Screen,
        canvas_world_scale: 0.0,
    });
    scene
}

pub fn bands_scene() -> Result<RenderScene, String> {
    let mut scene = crate::fx_scenes::stylised_scene()?;
    scene.stylise.depth_bands = Some(DepthBands {
        count: 4,
        near: 4.0,
        far: 30.0,
        near_color: [0.03, 0.02, 0.06],
        far_color: [0.6, 0.65, 0.8],
        flatten: 1.0,
    });
    Ok(scene)
}

pub fn retro_scene() -> RenderScene {
    let mut scene = crate::physical_scenes::physical_scene();
    scene.stylise.pixel = Some(PixelScale {
        height: 240,
        upscale: 0,
        snap: false,
    });
    scene.stylise.retro = Some(Retro3d {
        vertex_snap: 2.0,
        affine: 1.0,
        depth_bits: 12,
    });
    scene
}

pub fn material_scene() -> RenderScene {
    let mut scene = crate::physical_scenes::physical_scene();
    scene.stylise.material = MaterialStyle {
        toon: true,
        gradient_map: None,
        shadow_tint: [0.5, 0.2, 0.1],
        shadow_tint_amount: 0.8,
        rim_color: [1.0, 0.95, 0.85],
        rim_intensity: 0.5,
        rim_power: 3.0,
        outline_color: [0.05, 0.03, 0.02],
        outline_width: 3.0,
        outline_distance: 8.0,
        flat_shading: true,
    };
    scene
}

pub fn ink_depth_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    // One flat colour everywhere (unlit), so the tone has no edge to ink.
    let flat = scene.add_standard_material(StandardMaterial::basic(Vec3::new(0.85, 0.62, 0.38)));
    let floor = scene.add_mesh(shapes::plane(40.0, 1.0));
    scene.add_instance(floor, flat, Mat4::IDENTITY);
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(1.2)));
    // Silhouettes: a sphere in front of a larger one, both on the floor.
    scene.add_instance(
        sphere,
        flat,
        Mat4::from_translation(Vec3::new(-1.9, 0.8, -0.4)) * Mat4::from_scale(Vec3::splat(1.6)),
    );
    scene.add_instance(
        sphere,
        flat,
        Mat4::from_translation(Vec3::new(-1.0, 0.5, 1.1)),
    );
    // Creases: two turned boxes, one on the other.
    scene.add_instance(
        cube,
        flat,
        Mat4::from_translation(Vec3::new(1.6, 0.6, -0.2)) * Mat4::from_rotation_y(0.6),
    );
    scene.add_instance(
        cube,
        flat,
        Mat4::from_translation(Vec3::new(1.6, 1.62, -0.2))
            * Mat4::from_rotation_y(1.25)
            * Mat4::from_scale(Vec3::splat(0.7)),
    );
    scene.environment.background = Background::Color(Vec3::new(0.85, 0.62, 0.38));
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.0, 7.0),
        target: Vec3::new(0.0, 0.6, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene.stylise.ink = Some(Ink {
        threshold: 0.15,
        width: 1.5,
        color: [0.03, 0.02, 0.04],
        depth: 1.0,
        ..Default::default()
    });
    scene
}

/// A tileable 256² paper grain (grey, linear): three octaves of periodic
/// value noise from a fixed hash, so the surface canvas has something to
/// show without an asset.
fn paper_grain() -> og_render::TextureData {
    fn hash(x: i32, y: i32) -> f32 {
        let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
        h ^= h >> 13;
        h = h.wrapping_mul(0x5bd1_e995);
        h ^= h >> 15;
        (h & 0xffff) as f32 / 65535.0
    }
    // Value noise with `cells` cells across, wrapping at the tile's edge.
    let noise = |u: f32, v: f32, cells: i32| {
        let (x, y) = (u * cells as f32, v * cells as f32);
        let (x0, y0) = (x.floor() as i32, y.floor() as i32);
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        let s = |t: f32| t * t * (3.0 - 2.0 * t);
        let at = |i: i32, j: i32| hash(i.rem_euclid(cells), j.rem_euclid(cells));
        let a = at(x0, y0) + (at(x0 + 1, y0) - at(x0, y0)) * s(fx);
        let b = at(x0, y0 + 1) + (at(x0 + 1, y0 + 1) - at(x0, y0 + 1)) * s(fx);
        a + (b - a) * s(fy)
    };
    crate::scenes::texture(256, ColorSpace::Linear, |u, v| {
        let g = 0.5 * noise(u, v, 64) + 0.3 * noise(u, v, 16) + 0.2 * noise(u, v, 128);
        let b = ((0.35 + 0.65 * g) * 255.0).round() as u8;
        [b, b, b, 255]
    })
}

pub fn surface_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let clay = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.8, 0.78, 0.74),
        roughness: 0.9,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(60.0, 1.0));
    scene.add_instance(floor, clay, Mat4::IDENTITY);
    let sphere = scene.add_mesh(shapes::sphere(64, 48));
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    // A large sphere, a turned box near the camera, a small sphere on a
    // block, and a far sphere: strokes on round and flat forms at several
    // distances.
    scene.add_instance(
        sphere,
        clay,
        Mat4::from_translation(Vec3::new(-1.6, 1.3, -0.6)) * Mat4::from_scale(Vec3::splat(2.6)),
    );
    scene.add_instance(
        cube,
        clay,
        Mat4::from_translation(Vec3::new(1.7, 0.6, 1.4))
            * Mat4::from_rotation_y(0.55)
            * Mat4::from_scale(Vec3::splat(1.2)),
    );
    scene.add_instance(
        cube,
        clay,
        Mat4::from_translation(Vec3::new(2.4, 0.5, -2.2))
            * Mat4::from_scale(Vec3::new(1.4, 1.0, 1.4)),
    );
    scene.add_instance(
        sphere,
        clay,
        Mat4::from_translation(Vec3::new(2.4, 1.55, -2.2)),
    );
    scene.add_instance(
        sphere,
        clay,
        Mat4::from_translation(Vec3::new(-5.0, 1.5, -12.0)) * Mat4::from_scale(Vec3::splat(3.0)),
    );
    let paper = scene.add_texture(paper_grain());
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(0.5, 0.7, 0.45),
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.background = Background::Color(Vec3::new(0.86, 0.84, 0.78));
    scene.camera = Camera {
        position: Vec3::new(0.5, 3.2, 7.5),
        target: Vec3::new(0.0, 0.9, -1.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene.stylise.hatching = Some(Hatching {
        strength: 0.9,
        scale: 7.0,
        levels: 4,
        angle_degrees: 30.0,
        color: [0.05, 0.045, 0.04],
        space: StyliseSpace::Surface,
        depth: 0.0,
    });
    scene.stylise.paint = Some(Paint {
        radius: 0.0,
        anisotropy: 0.0,
        edge_darkening: 0.0,
        bleed: 0.0,
        canvas: Some(paper),
        canvas_strength: 0.7,
        canvas_scale: 1.0,
        canvas_space: StyliseSpace::Surface,
        canvas_world_scale: 96.0,
    });
    scene.stylise.ink = Some(Ink {
        threshold: 0.5,
        width: 1.5,
        color: [0.03, 0.025, 0.02],
        depth: 1.0,
        depth_threshold: 0.03,
        normal_threshold: 0.3,
    });
    scene
}

/// `stylise-object`: [`surface_scene`] in object space with a tilted box
/// and a scaled beam.
pub fn object_scene() -> RenderScene {
    let mut scene = surface_scene();
    let clay = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.8, 0.78, 0.74),
        roughness: 0.9,
        ..Default::default()
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    scene.add_instance(
        cube,
        clay,
        Mat4::from_scale_rotation_translation(
            Vec3::splat(1.1),
            Quat::from_rotation_y(-0.5) * Quat::from_rotation_x(0.45) * Quat::from_rotation_z(0.3),
            Vec3::new(-0.6, 0.85, 2.6),
        ),
    );
    scene.add_instance(
        cube,
        clay,
        Mat4::from_scale_rotation_translation(
            Vec3::new(4.0, 0.4, 0.5),
            Quat::from_rotation_y(0.35),
            Vec3::new(-3.6, 0.2, 1.4),
        ),
    );
    for space in [
        scene.stylise.hatching.as_mut().map(|h| &mut h.space),
        scene.stylise.paint.as_mut().map(|p| &mut p.canvas_space),
    ]
    .into_iter()
    .flatten()
    {
        *space = StyliseSpace::Object;
    }
    scene
}

/// Voxels meshed as a MagicaVoxel import is (`.vox` through the cook):
/// one quad per exposed voxel face, its four vertices its own, so every
/// face carries its own normal (unwelded, flat-shaded), coloured per
/// voxel. `solid(x, y, z)` says which cells of `size` are filled; `voxel`
/// is the cell's edge in metres, the cluster centred on x and z and
/// standing on y 0.
fn voxel_mesh(
    size: [i32; 3],
    voxel: f32,
    solid: impl Fn(i32, i32, i32) -> bool,
    color: impl Fn(i32, i32, i32) -> [f32; 4],
) -> og_render::MeshData {
    let filled = |x: i32, y: i32, z: i32| {
        (0..size[0]).contains(&x)
            && (0..size[1]).contains(&y)
            && (0..size[2]).contains(&z)
            && solid(x, y, z)
    };
    let origin = Vec3::new(size[0] as f32 * 0.5, 0.0, size[2] as f32 * 0.5);
    // (normal, u, v) per face: corners at the cell's centre + (n ± u ± v) / 2.
    let faces = [
        (Vec3::X, Vec3::NEG_Z, Vec3::Y),
        (Vec3::NEG_X, Vec3::Z, Vec3::Y),
        (Vec3::Y, Vec3::X, Vec3::NEG_Z),
        (Vec3::NEG_Y, Vec3::X, Vec3::Z),
        (Vec3::Z, Vec3::X, Vec3::Y),
        (Vec3::NEG_Z, Vec3::NEG_X, Vec3::Y),
    ];
    let mut mesh = og_render::MeshData::default();
    for x in 0..size[0] {
        for y in 0..size[1] {
            for z in 0..size[2] {
                if !filled(x, y, z) {
                    continue;
                }
                let centre = Vec3::new(x as f32, y as f32, z as f32) + Vec3::splat(0.5) - origin;
                for (n, u, v) in faces {
                    let (nx, ny, nz) = (n.x as i32, n.y as i32, n.z as i32);
                    if filled(x + nx, y + ny, z + nz) {
                        continue;
                    }
                    let base = mesh.vertices.len() as u32;
                    for (cu, cv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                        let p = (centre + (n + u * cu + v * cv) * 0.5) * voxel;
                        mesh.vertices.push(og_render::Vertex {
                            position: p.to_array(),
                            normal: n.to_array(),
                            uv: [(cu + 1.0) * 0.5, (1.0 - cv) * 0.5],
                            color: color(x, y, z),
                            tangent: [0.0; 4],
                        });
                    }
                    mesh.indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
            }
        }
    }
    mesh
}

/// A small hash of a cell (0..1), for the voxel scenes' ragged shapes and
/// shades.
fn cell_hash(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ (z as u32).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0xffff) as f32 / 65535.0
}

/// `stylise-outline-voxels`: the material style's outline (2.5 px, toon)
/// over unwelded, flat-shaded voxel models, three-quarters from above as a
/// top-down game sees them: a tree (a ragged canopy of 0.2 m voxels on a
/// trunk), a slime-like mound and a stepped block of cubes, beside a
/// smooth (welded) sphere. Each is outlined at its silhouette and where one
/// part passes in front of another; the edges between its faces, every
/// voxel step of the canopy, stay clean (lantern-vale FINDINGS #15).
pub fn outline_voxels_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let lit = scene.add_standard_material(StandardMaterial {
        roughness: 0.85,
        ..Default::default()
    });
    let ground = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.32, 0.46, 0.28),
        roughness: 1.0,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(40.0, 1.0));
    scene.add_instance(floor, ground, Mat4::IDENTITY);
    // The tree: a trunk two voxels wide, a canopy of 9 x 7 x 9 rounded and
    // ragged by the hash.
    let tree = scene.add_mesh(voxel_mesh(
        [9, 14, 9],
        0.2,
        |x, y, z| {
            let (dx, dz) = (x as f32 - 4.0, z as f32 - 4.0);
            if y < 6 {
                return (3..5).contains(&x) && (3..5).contains(&z);
            }
            let dy = y as f32 - 9.5;
            let r = (dx * dx + dz * dz + dy * dy * 1.4).sqrt();
            r < 4.3 - cell_hash(x, y, z) * 1.3
        },
        |x, y, z| {
            if y < 6 {
                [0.36, 0.2, 0.1, 1.0]
            } else {
                let s = 0.75 + 0.25 * cell_hash(z, x, y);
                [0.2 * s, 0.5 * s, 0.18 * s, 1.0]
            }
        },
    ));
    scene.add_instance(
        tree,
        lit,
        Mat4::from_translation(Vec3::new(-1.6, 0.0, -0.4)) * Mat4::from_rotation_y(0.4),
    );
    // A slime-like mound of 0.15 m voxels.
    let slime = scene.add_mesh(voxel_mesh(
        [7, 5, 7],
        0.15,
        |x, y, z| {
            let (dx, dy, dz) = (x as f32 - 3.0, y as f32 + 0.5, z as f32 - 3.0);
            dx * dx + dz * dz + dy * dy * 1.6 < 13.0
        },
        |_, y, _| [0.35 + 0.05 * y as f32, 0.75, 0.42, 1.0],
    ));
    scene.add_instance(
        slime,
        lit,
        Mat4::from_translation(Vec3::new(0.2, 0.0, 1.2)) * Mat4::from_rotation_y(-0.3),
    );
    // A stepped block of 0.4 m cubes: a staircase three high.
    let steps = scene.add_mesh(voxel_mesh(
        [3, 3, 2],
        0.4,
        |x, y, _| y <= 2 - x,
        |x, y, z| {
            let s = 0.8 + 0.2 * cell_hash(x, y, z);
            [0.75 * s, 0.68 * s, 0.6 * s, 1.0]
        },
    ));
    scene.add_instance(
        steps,
        lit,
        Mat4::from_translation(Vec3::new(1.9, 0.0, -0.6)) * Mat4::from_rotation_y(-0.5),
    );
    // A smooth sphere for comparison.
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let clay = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.8, 0.5, 0.35),
        roughness: 0.7,
        ..Default::default()
    });
    scene.add_instance(
        sphere,
        clay,
        Mat4::from_translation(Vec3::new(1.4, 0.45, 1.5)) * Mat4::from_scale(Vec3::splat(0.9)),
    );
    scene.stylise.material = MaterialStyle {
        toon: true,
        outline_color: [0.04, 0.03, 0.06],
        outline_width: 2.5,
        rim_power: 3.0,
        ..Default::default()
    };
    let sun = Vec3::new(-0.6, 0.7, 0.4);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.8;
    scene.environment.background = Background::Color(Vec3::new(0.55, 0.68, 0.82));
    scene.camera = Camera {
        position: Vec3::new(0.0, 4.2, 5.6),
        target: Vec3::new(0.0, 0.8, 0.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    scene
}

/// `stylise-toon-physical`: a look's toon (with a shadow tint, a rim and a
/// 2 px outline) over four spheres of one red: a plain one, which turns
/// toon; a lacquered one (`clearcoat` 1), which keeps its physical shading
/// and its coat's sharp highlight; a green jelly (`transmission`, a
/// thickness, ior 1.4) in front of a striped panel, which keeps refracting
/// it; and one with `styled` 0, drawn exactly as declared (smooth PBR, no
/// rim, outline or tinted shadow of its own). The first three take the
/// rim, the opaque two the outline (a transmissive material is not in the
/// depth prepass) (lantern-vale FINDINGS #16).
pub fn toon_physical_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let floor_material = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.55, 0.55, 0.6),
        roughness: 0.9,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(40.0, 1.0));
    scene.add_instance(floor, floor_material, Mat4::IDENTITY);
    // A striped panel behind the jelly, so its refraction shows.
    let stripes = scene.add_texture(crate::physical_scenes::stripes(128, 8.0, false));
    let panel_material = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.9, 0.85, 0.7),
        map: stripes.slot(),
        roughness: 0.8,
        ..Default::default()
    });
    let panel = scene.add_mesh(shapes::cuboid(Vec3::new(6.4, 1.6, 0.1)));
    scene.add_instance(
        panel,
        panel_material,
        Mat4::from_translation(Vec3::new(0.0, 0.8, -1.2)),
    );
    let sphere = scene.add_mesh(shapes::sphere(64, 48));
    let red = Vec3::new(0.7, 0.08, 0.06);
    let spheres = [
        StandardMaterial {
            color: red,
            roughness: 0.55,
            ..Default::default()
        },
        StandardMaterial {
            color: red,
            roughness: 0.55,
            clearcoat: 1.0,
            clearcoatRoughness: 0.06,
            ..Default::default()
        },
        StandardMaterial {
            color: Vec3::new(0.55, 0.95, 0.5),
            roughness: 0.15,
            transmission: 0.9,
            thickness: 0.8,
            attenuationColor: Vec3::new(0.3, 0.85, 0.3),
            attenuationDistance: 1.2,
            ior: 1.4,
            ..Default::default()
        },
        StandardMaterial {
            color: red,
            roughness: 0.55,
            styled: 0,
            ..Default::default()
        },
    ];
    for (i, m) in spheres.into_iter().enumerate() {
        let material = scene.add_standard_material(m);
        scene.add_instance(
            sphere,
            material,
            Mat4::from_translation(Vec3::new(-2.4 + 1.6 * i as f32, 0.6, 0.0))
                * Mat4::from_scale(Vec3::splat(1.2)),
        );
    }
    scene.stylise.material = MaterialStyle {
        toon: true,
        shadow_tint: [0.18, 0.16, 0.42],
        shadow_tint_amount: 0.7,
        rim_color: [1.0, 0.85, 0.7],
        rim_intensity: 0.4,
        rim_power: 3.0,
        outline_color: [0.04, 0.03, 0.06],
        outline_width: 2.0,
        ..Default::default()
    };
    let sun = Vec3::new(-0.5, 0.75, 0.45);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.9;
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.0, 5.4),
        target: Vec3::new(0.0, 0.6, 0.0),
        exposure: Exposure::linear(0.7),
        ..Default::default()
    };
    scene
}

/// A 16² pixel speech bubble: a cream body with a dark rim, a tail at
/// the bottom left, a red "!" (sRGB, no mips; drawn nearest).
fn bubble_texture() -> og_render::TextureData {
    let mut t = crate::scenes::texture(16, ColorSpace::Srgb, |u, v| {
        let (x, y) = ((u * 16.0) as i32, (v * 16.0) as i32);
        let cream = [242, 228, 196, 255];
        let rim = [52, 38, 58, 255];
        let red = [200, 40, 40, 255];
        match (x, y) {
            (7..=8, 3..=7) | (7..=8, 9..=10) => red,
            (2..=13, 2..=11) => cream,
            (1 | 14, 2..=11) | (2..=13, 1 | 12) => rim,
            (3..=4, 13) | (3, 14) => rim,
            _ => [0, 0, 0, 0],
        }
    });
    t.mips = og_render::Mips::None;
    t
}

/// `sprites-untonemapped`: cream pixel speech bubbles over a bright dusk
/// set (exposure 1.8, AgX, bloom from 1.0, TAA): the left bubble (from a
/// texture array, a layer below) tone mapped as before, blown towards
/// white and blooming; the next one
/// `SpriteFx::tone_mapped` false, its cream and red as authored and no
/// glow round it, a tone-mapped blue marker drawn over its corner (a
/// higher layer) still curved; a third untonemapped bubble half behind
/// the crate (still depth tested); a fourth drawn from a texture array,
/// untonemapped (lantern-vale FINDINGS #17).
pub fn untonemapped_sprites_scene() -> RenderScene {
    use og_render::{
        Bloom, SpriteArray, SpriteFx, TextureArrayData, TextureFilter, TextureSource, Tonemap,
    };
    let mut scene = RenderScene::new();
    let ground = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.5, 0.55, 0.42),
        roughness: 1.0,
        ..Default::default()
    });
    let floor = scene.add_mesh(shapes::plane(40.0, 1.0));
    scene.add_instance(floor, ground, Mat4::IDENTITY);
    let wood = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.6, 0.4, 0.22),
        roughness: 0.8,
        ..Default::default()
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(1.0)));
    scene.add_instance(
        cube,
        wood,
        Mat4::from_translation(Vec3::new(1.9, 0.5, 0.6)) * Mat4::from_rotation_y(0.3),
    );
    let bubble = scene.add_texture(bubble_texture());
    scene.set_texture_filter(bubble, TextureFilter::Nearest);
    let bubbles = scene
        .add_texture_array(TextureArrayData {
            layers: vec![TextureSource::Rgba(bubble_texture())],
        })
        .expect("one layer");
    scene.set_texture_array_filter(bubbles, TextureFilter::Nearest);
    let at = |x: f32, y: f32, z: f32| og_render::Sprite {
        texture: Some(bubble),
        position: Vec3::new(x, y, z),
        size: glam::Vec2::splat(1.2),
        ..Default::default()
    };
    let add = |scene: &mut RenderScene, sprite: og_render::Sprite, tone_mapped: bool| {
        let i = scene.sprites.len() as u32;
        scene.add_sprite(sprite);
        if !tone_mapped {
            scene.sprite_fx.push(SpriteFx {
                sprite: i,
                tone_mapped,
                ..Default::default()
            });
        }
        i
    };
    // The tone-mapped bubble from the array, a layer below the rest: its
    // array group is bound when the cover draws before the first
    // untonemapped run.
    let left = add(
        &mut scene,
        og_render::Sprite {
            texture: None,
            layer: -1,
            ..at(-2.2, 1.6, 0.0)
        },
        true,
    );
    scene.sprite_arrays.push(SpriteArray {
        sprite: left,
        array: bubbles,
        layer: 0,
    });
    add(&mut scene, at(-0.6, 1.6, 0.0), false);
    // Behind the crate: its lower right is hidden.
    add(&mut scene, at(2.3, 0.9, -0.4), false);
    let a = add(
        &mut scene,
        og_render::Sprite {
            texture: None,
            ..at(0.9, 2.3, -0.6)
        },
        false,
    );
    scene.sprite_arrays.push(SpriteArray {
        sprite: a,
        array: bubbles,
        layer: 0,
    });
    // A tone-mapped marker over the untonemapped bubble's corner.
    add(
        &mut scene,
        og_render::Sprite {
            position: Vec3::new(-0.1, 2.1, 0.05),
            size: glam::Vec2::splat(0.35),
            color: [0.2, 0.45, 1.6, 1.0],
            layer: 1,
            ..Default::default()
        },
        true,
    );
    let sun = Vec3::new(-0.7, 0.35, 0.6);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        color: Vec3::new(1.0, 0.7, 0.45),
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.8;
    scene.post.tonemap = Tonemap::Agx;
    scene.post.bloom = Some(Bloom {
        threshold: 1.0,
        knee: 0.5,
        intensity: 0.8,
    });
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.2, 6.0),
        target: Vec3::new(0.0, 1.0, 0.0),
        exposure: Exposure::linear(1.8),
        ..Default::default()
    };
    scene
}
