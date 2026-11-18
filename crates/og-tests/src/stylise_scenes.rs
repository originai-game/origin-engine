//! The stylisation scenes (`og-render/src/systems/stylise/mod.rs`, the IDL's `Stylise`
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
//!   silhouettes); `stylise-bands-tint`: five as a tint down a street.
//! - `stylise-retro`: vertex snapping, affine textures and 12-bit depth
//!   over the physical materials scene.
//! - `stylise-material`: the material style alone: a warm shadow tint, a
//!   rim, outlines that thin with distance, and flat facets, over the
//!   physical materials scene (no per-material stylised fields), and toon
//!   on every lit material without a physical lobe (the physical ones keep
//!   their coats, sheen, film, anisotropy and glass).
//! - `stylise-outline-voxels`: hull outlines on unwelded voxel meshes.
//! - `stylise-outline-ground`: the neon-vector look's outline (1.4 px,
//!   cyan, `outlineDistance` 120) from a chase camera 1.2 m over a 1,600 m
//!   ground with barriers down one side: the barriers lined, the ground
//!   lined only where it ends (meridian-loop #7: it was one cyan wash).
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
    Halftone, HalftoneMode, HalftoneRegion, Hatching, Ink, MaterialStyle, Paint, Palette,
    PixelScale, Quantize, RenderScene, Retro3d, StandardMaterial, StyliseSpace, shapes,
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
    let mut scene = toon_square()?;
    scene.look.stylise.pixel = Some(PixelScale {
        height: 180,
        upscale: 0,
        snap: true,
    });
    scene.look.stylise.quantize = Some(Quantize {
        posterize: 6,
        mode: Default::default(),
        palette: None,
        dither: DitherPattern::Bayer4,
        dither_strength: 1.0,
        dither_scale: 1,
        space: StyliseSpace::Screen,
    });
    Ok(scene)
}

pub fn palette_scene() -> Result<RenderScene, String> {
    let mut scene = toon_square()?;
    let palette = scene.add_palette(dusk_palette());
    scene.look.stylise.pixel = Some(PixelScale {
        height: 240,
        upscale: 0,
        snap: true,
    });
    scene.look.stylise.quantize = Some(Quantize {
        posterize: 0,
        mode: Default::default(),
        palette: Some(palette),
        dither: DitherPattern::BlueNoise,
        dither_strength: 1.0,
        dither_scale: 1,
        space: StyliseSpace::Screen,
    });
    Ok(scene)
}

pub fn print_scene() -> RenderScene {
    let mut scene = curio_stall();
    scene.look.stylise.hatching = Some(Hatching {
        strength: 0.9,
        scale: 6.0,
        levels: 4,
        angle_degrees: 45.0,
        color: [0.04, 0.035, 0.03],
        paper: [1.0; 3],
        space: StyliseSpace::Screen,
        depth: 0.0,
        keep_color: false,
    });
    scene.look.stylise.ink = Some(Ink {
        threshold: 0.12,
        width: 1.5,
        color: [0.02, 0.02, 0.03],
        depth: 0.0,
        ..Default::default()
    });
    scene
}

pub fn halftone_scene() -> Result<RenderScene, String> {
    let mut scene = toon_square()?;
    scene.look.stylise.halftone = Some(Halftone {
        strength: 1.0,
        size: 7.0,
        angle_degrees: 30.0,
        color: [0.05, 0.02, 0.08],
        space: StyliseSpace::Screen,
        mode: HalftoneMode::Ink,
        misregistration: 0.0,
        region: HalftoneRegion::All,
    });
    Ok(scene)
}

pub fn halftone_surface_scene() -> Result<RenderScene, String> {
    let mut scene = toon_square()?;
    scene.look.stylise.halftone = Some(Halftone {
        strength: 1.0,
        size: 9.0,
        angle_degrees: 30.0,
        color: [0.05, 0.02, 0.08],
        space: StyliseSpace::Surface,
        mode: HalftoneMode::Ink,
        misregistration: 0.0,
        region: HalftoneRegion::All,
    });
    scene.look.stylise.ink = Some(Ink {
        threshold: 0.4,
        width: 1.0,
        color: [0.02, 0.02, 0.03],
        depth: 1.0,
        ..Default::default()
    });
    Ok(scene)
}

pub fn paint_scene() -> RenderScene {
    let mut scene = curio_stall();
    scene.look.stylise.paint = Some(Paint {
        radius: 4.0,
        anisotropy: 0.6,
        edge_darkening: 0.6,
        bleed: 0.4,
        canvas: None,
        canvas_strength: 0.0,
        canvas_scale: 1.0,
        canvas_space: StyliseSpace::Screen,
        canvas_world_scale: 0.0,
        ..Paint::DEFAULT
    });
    scene
}

pub fn bands_scene() -> Result<RenderScene, String> {
    let mut scene = toon_square()?;
    scene.look.stylise.depth_bands = Some(DepthBands {
        count: 4,
        fit: false,
        near: 4.0,
        far: 30.0,
        near_color: [0.03, 0.02, 0.06],
        far_color: [0.6, 0.65, 0.8],
        flatten: 1.0,
    });
    Ok(scene)
}

pub fn retro_scene() -> RenderScene {
    let mut scene = curio_stall();
    scene.look.stylise.pixel = Some(PixelScale {
        height: 240,
        upscale: 0,
        snap: false,
    });
    scene.look.stylise.retro = Some(Retro3d {
        vertex_snap: 2.0,
        affine: 1.0,
        depth_bits: 12,
    });
    scene
}

pub fn material_scene() -> RenderScene {
    let mut scene = curio_stall();
    scene.look.stylise.material = MaterialStyle {
        toon: true,
        gradient_map: None,
        shadow_tint: [0.5, 0.2, 0.1],
        shadow_tint_amount: 0.8,
        shadow_darkness: 0.0,
        rim_color: [1.0, 0.95, 0.85],
        rim_intensity: 0.5,
        rim_power: 3.0,
        outline_color: [0.05, 0.03, 0.02],
        outline_width: 3.0,
        outline_distance: 8.0,
        flat_shading: true,
        ..Default::default()
    };
    scene
}

pub fn ink_depth_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    // A papercut of the ruins: one flat colour everywhere (unlit), the
    // ground and the sky the same, so the tone has no edge to ink.
    let paper = Vec3::new(0.85, 0.62, 0.38);
    let floor = scene.add_mesh(shapes::plane(400.0, 1.0));
    let flat = scene.add_standard_material(StandardMaterial::basic(paper));
    scene.add_instance(floor, flat, Mat4::IDENTITY);
    ruins(&mut scene);
    restyle_all(&mut scene, |m| *m = StandardMaterial::basic(paper));
    scene.lighting.environment.background = Background::Color(paper);
    scene.frame.camera = Camera {
        position: Vec3::new(1.2, 1.7, 6.5),
        target: Vec3::new(-0.3, 2.4, -8.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene.look.stylise.ink = Some(Ink {
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
    use crate::kit::prop;
    let mut scene = RenderScene::new();
    // A pencil study of a market corner: a row of houses, a stall's wares,
    // a column and a boulder by the camera, pines and the watchtower far
    // off, all of one clay (the drawing's tone is the light's).
    let floor = scene.add_mesh(shapes::plane(240.0, 1.0));
    let ground = scene.add_standard_material(StandardMaterial::default());
    scene.add_instance(floor, ground, Mat4::IDENTITY);
    let house = prop(&mut scene, "townhouse");
    for (k, x) in [-8.6f32, -0.8, 7.0].into_iter().enumerate() {
        house.place(
            &mut scene,
            Vec3::new(x, 0.0, -8.0),
            0.0,
            11.0 + 1.5 * k as f32,
        );
    }
    crate::lighting_scenes::wares(&mut scene, Vec3::new(1.6, 0.0, -2.0), -0.35);
    prop(&mut scene, "column").place(&mut scene, Vec3::new(-2.8, 0.0, 1.2), 0.3, 3.4);
    prop(&mut scene, "boulder").place(&mut scene, Vec3::new(-1.2, 0.0, 3.0), 0.8, 0.9);
    prop(&mut scene, "streetlamp").place(&mut scene, Vec3::new(4.6, 0.0, -0.5), 0.0, 4.2);
    let pine = prop(&mut scene, "pine");
    for k in 0..6 {
        let h = |n: i32| crate::fx_scenes::hash2(k, n, 0x3c9);
        let at = Vec3::new(
            -30.0 + 12.0 * k as f32 + h(0) * 4.0,
            0.0,
            -34.0 - h(1) * 8.0,
        );
        pine.place(&mut scene, at, h(2) * 6.0, 11.0 + h(3) * 5.0);
    }
    prop(&mut scene, "watchtower").place(&mut scene, Vec3::new(-16.0, 0.0, -48.0), 1.1, 22.0);
    restyle_all(&mut scene, clay);
    let paper = scene.add_texture(paper_grain());
    scene.lighting.light = DirectionalLight {
        direction_to_light: Vec3::new(0.5, 0.7, 0.45),
        intensity: 3.0,
        ..Default::default()
    };
    scene.lighting.environment.background = Background::Color(Vec3::new(0.86, 0.84, 0.78));
    scene.frame.camera = Camera {
        position: Vec3::new(0.6, 2.4, 8.5),
        target: Vec3::new(-0.2, 2.2, -4.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene.look.stylise.hatching = Some(Hatching {
        strength: 0.9,
        scale: 7.0,
        levels: 4,
        angle_degrees: 30.0,
        color: [0.05, 0.045, 0.04],
        paper: [1.0; 3],
        space: StyliseSpace::Surface,
        depth: 0.0,
        keep_color: false,
    });
    scene.look.stylise.paint = Some(Paint {
        radius: 0.0,
        anisotropy: 0.0,
        edge_darkening: 0.0,
        bleed: 0.0,
        canvas: Some(paper),
        canvas_strength: 0.7,
        canvas_scale: 1.0,
        canvas_space: StyliseSpace::Surface,
        canvas_world_scale: 96.0,
        ..Paint::DEFAULT
    });
    scene.look.stylise.ink = Some(Ink {
        threshold: 0.5,
        width: 1.5,
        color: [0.03, 0.025, 0.02],
        depth: 1.0,
        depth_threshold: 0.03,
        normal_threshold: 0.3,
    });
    scene
}

/// A drawing's clay: the material's own shape (its normals) in one pale
/// colour, no albedo map, no metal, no glow.
fn clay(m: &mut StandardMaterial) {
    m.color = Vec3::new(0.8, 0.78, 0.74);
    m.map = 0;
    m.metalness = 0.0;
    m.metalnessMap = 0;
    m.emissive = Vec3::ZERO;
}

/// `stylise-object`: [`surface_scene`] in object space with a tilted box
/// and a scaled beam.
pub fn object_scene() -> RenderScene {
    let mut scene = surface_scene();
    // A crate fallen off the stack, tilted on two axes against the
    // boulder, and a beam (a unit box scaled 4 x 0.4 x 0.5) laid across
    // the setts.
    let crate_ = prop_styled(&mut scene, "crate", clay);
    crate_.place_by(
        &mut scene,
        Mat4::from_scale_rotation_translation(
            Vec3::ONE,
            Quat::from_rotation_y(-0.5) * Quat::from_rotation_x(0.45) * Quat::from_rotation_z(0.3),
            Vec3::new(0.4, 0.45, 2.4),
        ) * crate_.transform(Vec3::new(0.0, -0.35, 0.0), 0.0, 0.7),
    );
    let beam = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.8, 0.78, 0.74),
        roughness: 0.9,
        ..Default::default()
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    scene.add_instance(
        cube,
        beam,
        Mat4::from_scale_rotation_translation(
            Vec3::new(4.0, 0.4, 0.5),
            Quat::from_rotation_y(-0.4),
            Vec3::new(2.8, 0.2, 3.0),
        ),
    );
    for space in [
        scene.look.stylise.hatching.as_mut().map(|h| &mut h.space),
        scene
            .look
            .stylise
            .paint
            .as_mut()
            .map(|p| &mut p.canvas_space),
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
    // The sea round the island.
    let sea = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.02, 0.17, 0.3),
        roughness: 0.5,
        ..Default::default()
    });
    let water = scene.add_mesh(shapes::plane(200.0, 1.0));
    scene.add_instance(water, sea, Mat4::from_translation(Vec3::Y * 0.3));
    // The island: columns of 0.4 m voxels in terraces one to four high,
    // sand at the shore, grass on top, earth down the steps.
    const V: f32 = 0.4;
    const ISLAND: [i32; 3] = [36, 4, 28];
    let height = |x: i32, z: i32| {
        let dx = (x as f32 + 0.5 - 18.0) / 18.0;
        let dz = (z as f32 + 0.5 - 14.0) / 14.0;
        let r = (dx * dx + dz * dz).sqrt() + (cell_hash(x / 2, 7, z / 2) - 0.5) * 0.12;
        match r {
            r if r < 0.42 => 4,
            r if r < 0.66 => 3,
            r if r < 0.84 => 2,
            r if r < 1.0 => 1,
            _ => 0,
        }
    };
    let island = scene.add_mesh(voxel_mesh(
        ISLAND,
        V,
        |x, y, z| y < height(x, z),
        |x, y, z| {
            let s = 0.85 + 0.15 * cell_hash(x, y, z);
            let h = height(x, z);
            if y + 1 < h {
                [0.42 * s, 0.27 * s, 0.15 * s, 1.0]
            } else if h == 1 {
                [0.8 * s, 0.66 * s, 0.4 * s, 1.0]
            } else {
                [0.22 * s, 0.52 * s, 0.16 * s, 1.0]
            }
        },
    ));
    scene.add_instance(island, lit, Mat4::IDENTITY);
    // Where a thing stands on the island at world (x, z).
    let on = |x: f32, z: f32| {
        let (cx, cz) = (
            (x / V + ISLAND[0] as f32 * 0.5).floor() as i32,
            (z / V + ISLAND[2] as f32 * 0.5).floor() as i32,
        );
        Vec3::new(x, height(cx, cz) as f32 * V, z)
    };
    // The trees: a trunk two voxels wide, a canopy of 9 x 7 x 9 rounded and
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
    for (x, z, yaw, s) in [
        (-2.2, -1.6, 0.4, 1.0),
        (-3.6, 0.6, 1.3, 0.8),
        (1.0, -2.8, 2.1, 1.15),
        (4.2, -1.0, 0.9, 0.85),
    ] {
        scene.add_instance(
            tree,
            lit,
            Mat4::from_translation(on(x, z))
                * Mat4::from_rotation_y(yaw)
                * Mat4::from_scale(Vec3::splat(s)),
        );
    }
    // Slime-like mounds of 0.15 m voxels.
    let slime = scene.add_mesh(voxel_mesh(
        [7, 5, 7],
        0.15,
        |x, y, z| {
            let (dx, dy, dz) = (x as f32 - 3.0, y as f32 + 0.5, z as f32 - 3.0);
            dx * dx + dz * dz + dy * dy * 1.6 < 13.0
        },
        |_, y, _| [0.35 + 0.05 * y as f32, 0.75, 0.42, 1.0],
    ));
    for (x, z, yaw) in [(0.2, 1.4, -0.3), (2.0, 2.4, 0.6)] {
        scene.add_instance(
            slime,
            lit,
            Mat4::from_translation(on(x, z)) * Mat4::from_rotation_y(yaw),
        );
    }
    // Stone steps of 0.4 m cubes, three high, up the terrace's face.
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
        Mat4::from_translation(on(3.4, 1.8)) * Mat4::from_rotation_y(-0.5),
    );
    // A smooth (welded) orb on the top terrace for comparison.
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let clay = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.8, 0.5, 0.35),
        roughness: 0.7,
        ..Default::default()
    });
    scene.add_instance(
        sphere,
        clay,
        Mat4::from_translation(on(-0.6, 0.2) + Vec3::Y * 0.45) * Mat4::from_scale(Vec3::splat(0.9)),
    );
    scene.look.stylise.material = MaterialStyle {
        toon: true,
        outline_color: [0.04, 0.03, 0.06],
        outline_width: 2.5,
        rim_power: 3.0,
        ..Default::default()
    };
    let sun = Vec3::new(-0.6, 0.7, 0.4);
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.lighting.environment.sky.sun_direction = sun;
    scene.lighting.environment.intensity = 0.8;
    scene.lighting.environment.background = Background::Color(Vec3::new(0.55, 0.68, 0.82));
    scene.frame.camera = Camera {
        position: Vec3::new(1.5, 7.8, 9.8),
        target: Vec3::new(0.0, 1.0, -0.2),
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
    // A fairground booth in the market square: a plank deck, two posts and
    // an awning, and a striped canvas behind the four prize balls.
    crate::lighting_scenes::market_square(&mut scene);
    crate::lighting_scenes::wares(&mut scene, Vec3::new(-6.0, 0.0, 0.5), 0.6);
    crate::kit::prop(&mut scene, "streetlamp").place(
        &mut scene,
        Vec3::new(-4.9, 0.0, -2.6),
        0.0,
        4.2,
    );
    let planks = crate::kit::surface(&mut scene, "planks");
    const DECK: f32 = 0.12;
    for (min, max) in [
        (Vec3::new(-3.8, 0.0, -1.6), Vec3::new(3.8, DECK, 1.4)),
        (Vec3::new(-3.6, DECK, -1.45), Vec3::new(-3.35, 3.1, -1.2)),
        (Vec3::new(3.35, DECK, -1.45), Vec3::new(3.6, 3.1, -1.2)),
        (Vec3::new(-3.8, 3.1, -1.7), Vec3::new(3.8, 3.3, 0.4)),
    ] {
        let mesh = scene.add_mesh(crate::kit::block(min, max, 1.0));
        scene.add_instance(mesh, planks, Mat4::IDENTITY);
    }
    // The canvas behind the jelly, red and cream stripes, so its
    // refraction shows.
    let stripes = scene.add_texture(crate::scenes::texture(128, ColorSpace::Srgb, |u, _| {
        if (u * 8.0).fract() < 0.5 {
            [196, 38, 34, 255]
        } else {
            [240, 228, 200, 255]
        }
    }));
    let panel_material = scene.add_standard_material(StandardMaterial {
        map: stripes.slot(),
        roughness: 0.8,
        ..Default::default()
    });
    let panel = scene.add_mesh(shapes::cuboid(Vec3::new(6.7, 2.98, 0.1)));
    scene.add_instance(
        panel,
        panel_material,
        Mat4::from_translation(Vec3::new(0.0, DECK + 1.49, -1.3)),
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
            Mat4::from_translation(Vec3::new(-2.4 + 1.6 * i as f32, DECK + 0.6, 0.0))
                * Mat4::from_scale(Vec3::splat(1.2)),
        );
    }
    scene.look.stylise.material = MaterialStyle {
        toon: true,
        shadow_tint: [0.18, 0.16, 0.42],
        shadow_tint_amount: 0.7,
        shadow_darkness: 0.0,
        rim_color: [1.0, 0.85, 0.7],
        rim_intensity: 0.4,
        rim_power: 3.0,
        outline_color: [0.04, 0.03, 0.06],
        outline_width: 2.0,
        ..Default::default()
    };
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(-0.5, 0.75, 0.45));
    scene.frame.camera = Camera {
        position: Vec3::new(0.4, 1.9, 6.2),
        target: Vec3::new(0.0, 1.2, 0.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
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
        let i = scene.sprite.sprites.len() as u32;
        scene.add_sprite(sprite);
        if !tone_mapped {
            scene.sprite.sprite_fx.push(SpriteFx {
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
    scene.sprite.sprite_arrays.push(SpriteArray {
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
    scene.sprite.sprite_arrays.push(SpriteArray {
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
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        color: Vec3::new(1.0, 0.7, 0.45),
        ..Default::default()
    };
    scene.lighting.environment.sky.sun_direction = sun;
    scene.lighting.environment.intensity = 0.8;
    scene.look.post.tonemap = Tonemap::Agx;
    scene.look.post.bloom = Some(Bloom {
        threshold: 1.0,
        knee: 0.5,
        intensity: 0.8,
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 2.2, 6.0),
        target: Vec3::new(0.0, 1.0, 0.0),
        exposure: Exposure::linear(1.8),
        ..Default::default()
    };
    scene
}

/// `stylise-bands-tint`: five depth bands as a tint (`flatten` 0) from
/// plum to gold over a lit three-quarter view down a street of coloured
/// blocks receding to 40 m: each layer keeps its shading, its shadows and
/// the blocks' own hues under its filter, and the layers step from plum
/// through rose and amber to gold, never grey (lantern-vale FINDINGS #19).
pub fn bands_tint_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    // The old town's high street on a summer morning, a stall at the
    // camera's feet, the houses and lamps receding to the watchtower.
    crate::lighting_scenes::high_street(&mut scene, 120.0);
    crate::lighting_scenes::wares(&mut scene, Vec3::new(-2.0, 0.0, 4.5), 0.4);
    crate::lighting_scenes::wares(&mut scene, Vec3::new(2.4, 0.0, -14.0), -0.3);
    scene.look.stylise.depth_bands = Some(DepthBands {
        count: 5,
        fit: false,
        near: 3.0,
        far: 40.0,
        near_color: [0.042, 0.012, 0.07],
        far_color: [1.0, 0.62, 0.24],
        flatten: 0.0,
    });
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(-0.6, 0.6, 0.5));
    scene.frame.camera = Camera {
        position: Vec3::new(0.8, 2.6, 10.0),
        target: Vec3::new(-0.6, 3.0, -30.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// `stylise-outline-ground` (meridian-loop #7): the neon-vector look's
/// outline from a chase camera 1.2 m over a 1,600 m ground (one mesh) seen
/// at a grazing angle, barriers down the right side only, on a dark night
/// background. `distance` is the style's `outlineDistance` (0: constant
/// width). The ground's left half is plane alone: only its far edge may be
/// lined there.
pub fn outline_ground_scene(width: f32, distance: f32) -> RenderScene {
    let mut scene = RenderScene::new();
    let asphalt = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.05, 0.05, 0.08),
        roughness: 0.85,
        ..Default::default()
    });
    let plane = scene.add_mesh(shapes::plane(1600.0, 1.0));
    scene.add_instance(plane, asphalt, Mat4::IDENTITY);
    let wall = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.12, 0.1, 0.16),
        roughness: 0.6,
        ..Default::default()
    });
    let barrier = scene.add_mesh(shapes::cuboid(Vec3::new(0.4, 0.9, 6.0)));
    for k in 0..12 {
        let z = -6.0 - 9.0 * k as f32;
        scene.add_instance(
            barrier,
            wall,
            Mat4::from_translation(Vec3::new(5.0, 0.45, z)),
        );
    }
    // The rival's car ahead in the right lane, its tail lamps lit.
    let car = crate::kit::prop(&mut scene, "car");
    car.place(
        &mut scene,
        Vec3::new(2.0, 0.0, -10.0),
        std::f32::consts::FRAC_PI_2,
        1.45,
    );
    let lamp = scene.add_mesh(shapes::cuboid(Vec3::new(0.35, 0.1, 0.04)));
    let tail = scene.add_standard_material(StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::new(1.0, 0.08, 0.2),
        emissiveIntensity: 6.0,
        ..Default::default()
    });
    for x in [1.45, 2.55] {
        scene.add_instance(
            lamp,
            tail,
            Mat4::from_translation(Vec3::new(x, 0.78, -8.44)),
        );
    }
    // Pylons past the barriers, and the city's towers beyond the right
    // side: the left half stays the ground alone to its far edge.
    let pylon = scene.add_mesh(shapes::cuboid(Vec3::new(0.3, 7.0, 0.3)));
    let head = scene.add_mesh(shapes::cuboid(Vec3::new(1.6, 0.12, 0.3)));
    let neon = scene.add_standard_material(StandardMaterial {
        color: Vec3::ZERO,
        emissive: Vec3::new(1.0, 0.25, 0.85),
        emissiveIntensity: 5.0,
        ..Default::default()
    });
    for k in 0..8 {
        let z = -10.0 - 16.0 * k as f32;
        scene.add_instance(pylon, wall, Mat4::from_translation(Vec3::new(6.4, 3.5, z)));
        scene.add_instance(head, neon, Mat4::from_translation(Vec3::new(5.8, 7.0, z)));
    }
    let tower = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    for k in 0..14 {
        let h = |n: i32| crate::fx_scenes::hash2(k, n, 0x9e1);
        let (w, tall) = (20.0 + h(0) * 30.0, 40.0 + h(1) * 140.0);
        scene.add_instance(
            tower,
            wall,
            Mat4::from_translation(Vec3::new(
                40.0 + 45.0 * k as f32 + h(2) * 20.0,
                tall * 0.5,
                -260.0 - h(3) * 300.0,
            )) * Mat4::from_scale(Vec3::new(w, tall, w)),
        );
    }
    scene.look.stylise.material = MaterialStyle {
        outline_color: [0.25, 0.94, 1.0],
        outline_width: width,
        outline_distance: distance,
        ..Default::default()
    };
    scene.lighting.environment.background = Background::Color(Vec3::new(0.01, 0.01, 0.03));
    scene.lighting.environment.intensity = 0.4;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 1.2, 0.0),
        target: Vec3::new(0.0, 0.7, -20.0),
        far: 2000.0,
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let sun = Vec3::new(-0.4, 0.6, -0.5);
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 2.0,
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// The sets the stylisation primitives are drawn over. Each is a place a
// stylised game would show; the primitive's settings are the scene's own.
// ---------------------------------------------------------------------------

/// Every material of `scene` so far, changed by `f` (a set given one look:
/// toon, one flat colour, clay).
fn restyle_all(scene: &mut RenderScene, f: impl Fn(&mut StandardMaterial)) {
    for h in scene.material_handles() {
        let mut m = *scene.standard_material(h).expect("a live material");
        f(&mut m);
        scene.set_standard_material(h, m);
    }
}

/// The kit prop `name`, its materials changed by `f`.
fn prop_styled(
    scene: &mut RenderScene,
    name: &str,
    f: impl Fn(&mut StandardMaterial),
) -> crate::kit::Prop {
    let p = crate::kit::prop(scene, name);
    for &(_, h) in p.parts() {
        let mut m = *scene.standard_material(h).expect("a kit material");
        f(&mut m);
        scene.set_standard_material(h, m);
    }
    p
}

/// `stylised` and the screen primitives' set (pixel, palette, halftone,
/// halftone-surface, bands): a cel-shaded corner of the old town's market
/// square on a bright afternoon. Every material is toon through a
/// four-band ramp with a rim and a hull outline; the trickster is half
/// gone through a glowing dissolve, as when it teleports, a barrel by the
/// stall flashes white from a hit, and a gold ring pickup turns over the
/// setts. A screen outline and bloom finish the look.
pub fn toon_square() -> Result<RenderScene, String> {
    use crate::lighting_scenes::{market_square, wares};
    let mut scene = RenderScene::new();
    market_square(&mut scene);
    wares(&mut scene, Vec3::new(4.2, 0.0, -1.5), -0.5);
    wares(&mut scene, Vec3::new(-9.0, 0.0, -11.0), 0.4);
    let lamp = crate::kit::prop(&mut scene, "streetlamp");
    lamp.place(&mut scene, Vec3::new(-4.6, 0.0, -2.5), 0.4, 4.2);
    lamp.place(&mut scene, Vec3::new(9.0, 0.0, -14.0), 0.0, 4.2);
    let barrel = crate::kit::prop(&mut scene, "winebarrel");
    barrel.place(&mut scene, Vec3::new(1.4, 0.0, 2.2), 0.7, 1.0);
    let ramp = crate::fx_scenes::ramp_texture(&mut scene);
    let outline = Vec3::new(0.05, 0.04, 0.08);
    restyle_all(&mut scene, |m| {
        m.toon = 1;
        m.gradientMap = ramp.slot();
        m.rimColor = Vec3::new(1.0, 0.95, 0.9);
        m.rimIntensity = 0.35;
        m.rimPower = 3.5;
        m.outlineColor = outline;
        m.outlineWidth = 2.0;
    });
    // The hit barrel: the same toon, flashed.
    let flashed = prop_styled(&mut scene, "winebarrel", |m| {
        m.toon = 1;
        m.gradientMap = ramp.slot();
        m.outlineColor = outline;
        m.outlineWidth = 3.0;
        m.flash = 0.65;
        m.flashColor = Vec3::ONE;
    });
    flashed.place(&mut scene, Vec3::new(2.5, 0.0, 1.4), -0.4, 1.0);
    // The ring pickup, upright and a little turned.
    let ring = scene.add_mesh(crate::fx_scenes::torus(0.45, 0.12, 64, 24));
    let gold = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(1.0, 0.75, 0.2),
        roughness: 0.4,
        toon: 1,
        gradientMap: ramp.slot(),
        rimColor: Vec3::new(1.0, 0.95, 0.9),
        rimIntensity: 0.6,
        rimPower: 3.5,
        emissive: Vec3::new(1.0, 0.7, 0.2),
        emissiveIntensity: 0.4,
        outlineColor: outline,
        outlineWidth: 3.0,
        ..Default::default()
    });
    scene.add_instance(
        ring,
        gold,
        Mat4::from_translation(Vec3::new(-2.6, 1.3, 2.4))
            * Mat4::from_rotation_y(0.5)
            * Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2),
    );
    // The trickster, half dissolved.
    let model = crate::scenes::load("trickster.glb")?;
    let parts = model.add_to(&mut scene);
    for &(_, mat) in &parts.parts {
        let mut m = *scene.standard_material(mat).expect("a material");
        m.toon = 1;
        m.gradientMap = ramp.slot();
        m.dissolve = 0.45;
        m.dissolveScale = 6.0;
        m.dissolveEdgeColor = Vec3::new(1.0, 3.0, 6.0);
        m.dissolveEdgeWidth = 0.08;
        m.rimIntensity = 0.4;
        m.outlineWidth = 2.0;
        m.outlineColor = outline;
        scene.set_standard_material(mat, m);
    }
    let scale = 1.8 / (model.max - model.min).max_element().max(1e-3);
    parts.instance(
        &mut scene,
        Mat4::from_translation(Vec3::new(-0.6, -model.min.y * scale, 1.2))
            * Mat4::from_rotation_y(crate::scenes::TRICKSTER_TO_Z + 0.35)
            * Mat4::from_scale(Vec3::splat(scale)),
    );
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(0.55, 0.7, 0.45));
    scene.look.post.outline = Some(og_render::Outline {
        width: 1.5,
        ..Default::default()
    });
    scene.look.post.bloom = Some(og_render::Bloom::default());
    scene.frame.camera = Camera {
        position: Vec3::new(1.2, 1.6, 6.5),
        target: Vec3::new(-0.6, 1.3, -4.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    Ok(scene)
}

/// The physical-materials set (print, paint, retro, material): a curio
/// seller's stall in the market square, close up. On a linen cloth
/// (sheen) stand clear, frosted and amber glass orbs, a cut gem of high
/// `ior` and a lantern of striped glass (`transmissionMap`), a black
/// pearl and a polished orb under a thin film (iridescence) on a pink
/// velvet cushion, and a brushed silver urn (anisotropy); behind it the
/// seller's lacquered car (clearcoat). The stall's planks, the crates and
/// the town round it are plain lit materials.
pub fn curio_stall() -> RenderScene {
    use crate::physical_scenes::{ramp, stripes};
    let mut scene = RenderScene::new();
    crate::lighting_scenes::market_square(&mut scene);
    crate::lighting_scenes::wares(&mut scene, Vec3::new(-3.6, 0.0, -1.0), 0.3);
    let planks = crate::kit::surface(&mut scene, "planks");
    let table = scene.add_mesh(crate::kit::block(
        Vec3::new(-1.5, 0.0, -0.55),
        Vec3::new(1.5, 0.88, 0.55),
        1.0,
    ));
    scene.add_instance(table, planks, Mat4::IDENTITY);
    let put = |scene: &mut RenderScene, mesh, m: StandardMaterial, t: Mat4| {
        let h = scene.add_standard_material(m);
        scene.add_instance(mesh, h, t);
    };
    // The cloth over the table and down its front.
    let cloth = StandardMaterial {
        color: Vec3::new(0.72, 0.64, 0.5),
        roughness: 0.9,
        sheen: 0.6,
        sheenColor: Vec3::new(1.0, 0.92, 0.8),
        sheenRoughness: 0.5,
        ..Default::default()
    };
    let top = scene.add_mesh(crate::kit::block(
        Vec3::new(-1.56, 0.88, -0.6),
        Vec3::new(1.56, 0.9, 0.6),
        1.0,
    ));
    put(&mut scene, top, cloth, Mat4::IDENTITY);
    let drape = scene.add_mesh(crate::kit::block(
        Vec3::new(-1.56, 0.3, 0.6),
        Vec3::new(1.56, 0.9, 0.62),
        1.0,
    ));
    put(&mut scene, drape, cloth, Mat4::IDENTITY);
    let sphere = scene.add_mesh(shapes::sphere(64, 48));
    let on = |x: f32, z: f32, r: f32| {
        Mat4::from_translation(Vec3::new(x, 0.9 + r, z)) * Mat4::from_scale(Vec3::splat(r * 2.0))
    };
    let glass = StandardMaterial {
        color: Vec3::ONE,
        roughness: 0.02,
        transmission: 1.0,
        thickness: 0.3,
        ior: 1.5,
        ..Default::default()
    };
    put(&mut scene, sphere, glass, on(-1.1, 0.05, 0.17));
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            roughness: 0.35,
            ..glass
        },
        on(-0.62, -0.2, 0.13),
    );
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            attenuationColor: Vec3::new(1.0, 0.55, 0.1),
            attenuationDistance: 0.15,
            ..glass
        },
        on(-0.5, 0.25, 0.12),
    );
    // The gem: a faceted (low-poly) sphere of high ior.
    let facets = scene.add_mesh(shapes::sphere(8, 5));
    put(
        &mut scene,
        facets,
        StandardMaterial {
            ior: 2.4,
            attenuationColor: Vec3::new(0.5, 0.75, 1.0),
            attenuationDistance: 0.3,
            ..glass
        },
        on(-0.1, 0.3, 0.07),
    );
    // The lantern of striped glass.
    let striped = scene.add_texture(stripes(256, 6.0, false));
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    put(
        &mut scene,
        cube,
        StandardMaterial {
            transmissionMap: striped.slot(),
            ..glass
        },
        Mat4::from_translation(Vec3::new(-0.05, 1.06, -0.25))
            * Mat4::from_rotation_y(0.5)
            * Mat4::from_scale(Vec3::new(0.24, 0.32, 0.24)),
    );
    // The cushion, the pearl and the filmed orb.
    let film = scene.add_texture(ramp(64));
    put(
        &mut scene,
        cube,
        StandardMaterial {
            color: Vec3::new(0.03, 0.04, 0.18),
            roughness: 0.9,
            sheen: 1.0,
            sheenColor: Vec3::new(1.0, 0.45, 0.7),
            sheenRoughness: 0.35,
            ..Default::default()
        },
        Mat4::from_translation(Vec3::new(0.55, 0.94, 0.0))
            * Mat4::from_rotation_y(-0.2)
            * Mat4::from_scale(Vec3::new(0.62, 0.08, 0.42)),
    );
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            color: Vec3::splat(0.02),
            roughness: 0.15,
            iridescence: 1.0,
            iridescenceIOR: 1.3,
            iridescenceThicknessRange: glam::Vec2::new(200.0, 1200.0),
            iridescenceThicknessMap: film.slot(),
            ..Default::default()
        },
        Mat4::from_translation(Vec3::new(0.38, 1.1, 0.02)) * Mat4::from_scale(Vec3::splat(0.2)),
    );
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            color: Vec3::new(0.8, 0.8, 0.85),
            metalness: 1.0,
            roughness: 0.25,
            iridescence: 1.0,
            iridescenceIOR: 1.8,
            iridescenceThicknessRange: glam::Vec2::new(250.0, 450.0),
            ..Default::default()
        },
        Mat4::from_translation(Vec3::new(0.7, 1.12, -0.05)) * Mat4::from_scale(Vec3::splat(0.24)),
    );
    // The urn: a tall brushed-silver orb on the cloth.
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            color: Vec3::new(0.9, 0.9, 0.92),
            metalness: 1.0,
            roughness: 0.35,
            anisotropy: 0.9,
            ..Default::default()
        },
        Mat4::from_translation(Vec3::new(1.15, 1.18, -0.15))
            * Mat4::from_scale(Vec3::new(0.36, 0.56, 0.36)),
    );
    // The seller's car, its paint lacquered.
    prop_styled(&mut scene, "car", |m| {
        m.clearcoat = 1.0;
        m.clearcoatRoughness = 0.05;
    })
    .place(&mut scene, Vec3::new(2.4, 0.0, -5.2), -0.45, 1.45);
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(-0.45, 0.5, 0.75));
    scene.frame.camera = Camera {
        position: Vec3::new(0.1, 2.1, 1.85),
        target: Vec3::new(0.0, 1.0, -0.6),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// The kit's ruins for a drawn set: an arch over a way between two
/// colonnades (some fallen), boulders by the camera, a stand of pines and
/// the watchtower behind.
fn ruins(scene: &mut RenderScene) {
    use crate::kit::prop;
    let arch = prop(scene, "arch");
    arch.place(scene, Vec3::new(0.0, 0.0, -7.0), 0.0, 5.5);
    let column = prop(scene, "column");
    let broken = prop(scene, "column_broken");
    for k in 0..5 {
        let z = -1.5 - 3.2 * k as f32;
        for side in [-1.0f32, 1.0] {
            let whole = (k + (side > 0.0) as i32) % 3 != 1;
            let p = if whole { &column } else { &broken };
            let h = if whole {
                4.2
            } else {
                1.6 + 0.6 * (k % 2) as f32
            };
            p.place(scene, Vec3::new(side * 4.0, 0.0, z), 0.4 * k as f32, h);
        }
    }
    let boulder = prop(scene, "boulder");
    boulder.place(scene, Vec3::new(-2.4, 0.0, 2.0), 0.6, 1.1);
    boulder.place(scene, Vec3::new(2.8, 0.0, -0.5), 2.1, 0.7);
    let pine = prop(scene, "pine");
    for k in 0..9 {
        let h = |n: i32| crate::fx_scenes::hash2(k, n, 0x71e);
        let at = Vec3::new(
            -18.0 + 4.5 * k as f32 + h(0) * 2.0,
            0.0,
            -24.0 - h(1) * 10.0,
        );
        pine.place(scene, at, h(2) * 6.0, 9.0 + h(3) * 6.0);
    }
    prop(scene, "watchtower").place(scene, Vec3::new(11.0, 0.0, -38.0), 2.2, 20.0);
}
