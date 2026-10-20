//! The physical material scene (`materials-physical`): every
//! `MeshPhysicalMaterial` lobe og-render draws (`og-render/src/physical.rs`),
//! with and without its maps, and transmissive objects in front of textured
//! geometry.

use glam::{Mat4, Vec2, Vec3};
use og_render::{
    Camera, ColorSpace, DirectionalLight, Exposure, Mips, RenderScene, StandardMaterial,
    TextureData, shapes,
};

/// A procedural texture of `size`² from a function of uv (0..1, v down)
/// to RGBA in 0..1.
fn texture(size: u32, color_space: ColorSpace, f: impl Fn(f32, f32) -> [f32; 4]) -> TextureData {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let c = f(
                (x as f32 + 0.5) / size as f32,
                (y as f32 + 0.5) / size as f32,
            );
            rgba.extend(c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8));
        }
    }
    TextureData {
        width: size,
        height: size,
        rgba,
        color_space,
        mips: Mips::Generate,
    }
}

/// `n` stripes along u (across v with `across`): white on, black off (a
/// mask in every channel).
pub fn stripes(size: u32, n: f32, across: bool) -> TextureData {
    texture(size, ColorSpace::Linear, |u, v| {
        let t = if across { v } else { u };
        let on = if (t * n).fract() < 0.5 { 1.0 } else { 0.0 };
        [on, on, on, 1.0]
    })
}

/// A ramp down the image (v) in every channel: 0 at the top, 1 at the bottom.
pub fn ramp(size: u32) -> TextureData {
    texture(size, ColorSpace::Linear, |_, v| [v, v, v, v])
}

/// An anisotropy map (`KHR_materials_anisotropy`): the direction turning
/// round the centre of the image (red and green, -1..1 in the tangent
/// frame), full strength in blue.
pub fn swirl(size: u32) -> TextureData {
    texture(size, ColorSpace::Linear, |u, v| {
        let d = Vec2::new(u - 0.5, 0.5 - v);
        let t = Vec2::new(-d.y, d.x).normalize_or(Vec2::X);
        [t.x * 0.5 + 0.5, t.y * 0.5 + 0.5, 1.0, 1.0]
    })
}

/// A sheen colour and roughness map: pink and pale gold diamonds (sRGB),
/// roughness rising down the image in alpha.
pub fn sheen_map(size: u32) -> TextureData {
    texture(size, ColorSpace::Srgb, |u, v| {
        let diamond = ((u * 6.0).fract() - 0.5).abs() + ((v * 3.0).fract() - 0.5).abs() < 0.5;
        let c = if diamond {
            [1.0, 0.55, 0.75]
        } else {
            [0.95, 0.85, 0.45]
        };
        [c[0], c[1], c[2], 0.3 + 0.7 * v]
    })
}

/// Bold coloured bars behind the glass: what refraction shows (sRGB).
pub fn bars(size: u32) -> TextureData {
    let colors = [
        [0.85, 0.12, 0.1],
        [0.95, 0.8, 0.15],
        [0.1, 0.55, 0.85],
        [0.95, 0.95, 0.92],
        [0.15, 0.6, 0.25],
        [0.1, 0.1, 0.12],
    ];
    texture(size, ColorSpace::Srgb, |u, v| {
        let c = colors[((u * 12.0) as usize + (v * 2.0) as usize * 3) % colors.len()];
        [c[0], c[1], c[2], 1.0]
    })
}

/// Seven feature spheres on a plinth of coloured bars, five transmissive
/// objects in front of it on the checker ground, under a low sun and the
/// sky. Back, left to right: clearcoat with its three maps (banded
/// coat, its roughness, bumps in the coat only) over a rough red base;
/// pink sheen on dark blue velvet; mapped sheen (diamonds, roughness
/// down the sphere) on dark green; iridescence on black with the film's
/// thickness rising down the sphere; iridescence on polished metal;
/// anisotropic brushed metal; anisotropy swirled by its map on gold.
/// Front: clear glass, frosted glass, amber glass absorbing through
/// `attenuationColor`, a striped `transmissionMap` cube, and a
/// high-`ior` sphere.
pub fn physical_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::scenes::ground(&mut scene, 40.0, 0.5, 0.8);
    let sphere = scene.add_mesh(shapes::sphere(64, 48));
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(0.85)));
    let plinth = scene.add_mesh(shapes::cuboid(Vec3::new(9.0, 0.7, 0.8)));
    let stripes_u = scene.add_texture(stripes(256, 6.0, false));
    let bands = scene.add_texture(stripes(256, 5.0, true));
    let bumps = scene.add_texture(crate::p2_scenes::bump_normal_map(256, 8.0));
    let ramp = scene.add_texture(ramp(64));
    let swirl = scene.add_texture(swirl(256));
    let sheen = scene.add_texture(sheen_map(256));
    let bars = scene.add_texture(bars(512));
    let put = |scene: &mut RenderScene, mesh, m: StandardMaterial, t: Mat4| {
        let h = scene.add_standard_material(m);
        scene.add_instance(mesh, h, t);
    };
    put(
        &mut scene,
        plinth,
        StandardMaterial {
            map: bars.slot(),
            roughness: 0.7,
            ..Default::default()
        },
        Mat4::from_translation(Vec3::new(0.0, 0.35, -1.8)),
    );
    let back = |i: f32| Mat4::from_translation(Vec3::new(i * 1.2 - 3.6, 1.2, -1.8));
    let back_row = [
        StandardMaterial {
            color: Vec3::new(0.7, 0.06, 0.05),
            roughness: 0.8,
            clearcoat: 1.0,
            clearcoatRoughness: 0.08,
            clearcoatMap: bands.slot(),
            clearcoatRoughnessMap: ramp.slot(),
            clearcoatNormalMap: bumps.slot(),
            ..Default::default()
        },
        StandardMaterial {
            color: Vec3::new(0.03, 0.04, 0.18),
            roughness: 0.9,
            sheen: 1.0,
            sheenColor: Vec3::new(1.0, 0.45, 0.7),
            sheenRoughness: 0.35,
            ..Default::default()
        },
        StandardMaterial {
            color: Vec3::new(0.02, 0.12, 0.05),
            roughness: 0.9,
            sheen: 1.0,
            sheenColor: Vec3::ONE,
            sheenRoughness: 0.6,
            sheenColorMap: sheen.slot(),
            sheenRoughnessMap: sheen.slot(),
            ..Default::default()
        },
        StandardMaterial {
            color: Vec3::splat(0.02),
            roughness: 0.15,
            iridescence: 1.0,
            iridescenceIOR: 1.3,
            iridescenceThicknessRange: Vec2::new(200.0, 1200.0),
            iridescenceThicknessMap: ramp.slot(),
            ..Default::default()
        },
        StandardMaterial {
            color: Vec3::new(0.8, 0.8, 0.85),
            metalness: 1.0,
            roughness: 0.25,
            iridescence: 1.0,
            iridescenceIOR: 1.8,
            iridescenceThicknessRange: Vec2::new(250.0, 450.0),
            ..Default::default()
        },
        StandardMaterial {
            color: Vec3::new(0.9, 0.9, 0.92),
            metalness: 1.0,
            roughness: 0.35,
            anisotropy: 0.9,
            ..Default::default()
        },
        StandardMaterial {
            color: Vec3::new(1.0, 0.78, 0.34),
            metalness: 1.0,
            roughness: 0.4,
            anisotropy: 1.0,
            anisotropyRotation: 0.3,
            anisotropyMap: swirl.slot(),
            ..Default::default()
        },
    ];
    for (i, m) in back_row.into_iter().enumerate() {
        put(&mut scene, sphere, m, back(i as f32));
    }
    let glass = StandardMaterial {
        color: Vec3::ONE,
        roughness: 0.02,
        transmission: 1.0,
        thickness: 1.0,
        ..Default::default()
    };
    let front = |i: f32| Vec3::new(i * 1.55 - 3.1, 0.5, 0.9);
    put(
        &mut scene,
        sphere,
        glass,
        Mat4::from_translation(front(0.0)),
    );
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            roughness: 0.35,
            ..glass
        },
        Mat4::from_translation(front(1.0)),
    );
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            roughness: 0.05,
            attenuationColor: Vec3::new(0.95, 0.45, 0.08),
            attenuationDistance: 0.35,
            ..glass
        },
        Mat4::from_translation(front(2.0)),
    );
    put(
        &mut scene,
        cube,
        StandardMaterial {
            color: Vec3::new(0.85, 0.9, 1.0),
            roughness: 0.1,
            transmissionMap: stripes_u.slot(),
            thickness: 0.85,
            ..glass
        },
        Mat4::from_translation(front(3.0) - Vec3::Y * 0.075) * Mat4::from_rotation_y(0.5),
    );
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            color: Vec3::new(0.95, 0.97, 1.0),
            roughness: 0.02,
            ior: 2.33,
            ..glass
        },
        Mat4::from_translation(front(4.0)),
    );
    let sun = Vec3::new(-0.5, 0.75, 0.45);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.9;
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.3, 6.8),
        target: Vec3::new(0.0, 0.8, -0.6),
        exposure: Exposure::linear(0.7),
        ..Default::default()
    };
    scene
}
