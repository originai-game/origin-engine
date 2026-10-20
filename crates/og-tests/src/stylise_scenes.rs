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
//! - `stylise-material`: the material style alone: every lit material toon
//!   with a warm shadow tint, a rim, outlines that thin with distance, and
//!   flat facets, over the physical materials scene (no per-material
//!   stylised fields).
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
