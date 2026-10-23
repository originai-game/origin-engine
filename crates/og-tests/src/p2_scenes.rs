//! The phase 2 render round's scenes: many local lights (Forward+), the M6
//! material features, a displaced skinned mesh and the moon.

use glam::{Mat4, Vec3};
use og_render::{
    Background, Camera, Environment, Exposure, Light, LightKind, Material, RenderScene, shapes,
};

/// A small integer hash to [0, 1) (deterministic colours and placements).
fn hash01(i: u32, salt: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// A saturated colour of hue `h` (0..1).
fn hue(h: f32) -> Vec3 {
    let k = |n: f32| {
        let k = (n + h * 6.0) % 6.0;
        1.0 - k.min(4.0 - k).clamp(0.0, 1.0)
    };
    Vec3::new(k(5.0), k(3.0), k(1.0))
}

/// The lights of [`many_lights`]: `count` of them over a 48 m square, every
/// eighth a spot pointing down, the rest points, hues and heights hashed.
pub fn many_lights_list(count: u32) -> Vec<Light> {
    let side = (count as f32).sqrt().ceil() as u32;
    let spacing = 48.0 / side as f32;
    (0..count)
        .map(|i| {
            let (gx, gz) = (i % side, i / side);
            let x = -24.0 + (gx as f32 + 0.5 + (hash01(i, 1) - 0.5) * 0.6) * spacing;
            let z = -24.0 + (gz as f32 + 0.5 + (hash01(i, 2) - 0.5) * 0.6) * spacing;
            let color = hue(hash01(i, 3)).lerp(Vec3::ONE, 0.25);
            if i % 8 == 7 {
                Light {
                    kind: LightKind::Spot {
                        direction: Vec3::new(hash01(i, 4) - 0.5, -1.0, hash01(i, 5) - 0.5),
                        angle_degrees: 28.0,
                        penumbra: 0.4,
                    },
                    position: Vec3::new(x, 3.2, z),
                    color,
                    intensity: 14.0,
                    range: 6.0,
                    cast_shadows: false,
                }
            } else {
                Light {
                    position: Vec3::new(x, 0.35 + hash01(i, 6) * 0.9, z),
                    color,
                    intensity: 2.2 + hash01(i, 7) * 1.6,
                    range: 2.6 + hash01(i, 8) * 1.4,
                    ..Default::default()
                }
            }
        })
        .collect()
}

/// Night over a 60 m plaza of pillars and spheres lit by `count` point and
/// spot lights (no sun, a faint sky light): the Forward+ scene.
pub fn many_lights(count: u32) -> RenderScene {
    let mut scene = RenderScene::new();
    crate::scenes::ground(&mut scene, 60.0, 0.5, 0.6);
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(0.5, 2.4, 0.5)));
    let sphere = scene.add_mesh(shapes::sphere(24, 16));
    let stone = scene.add_material(Material {
        base_color: [0.8, 0.78, 0.74, 1.0],
        roughness: 0.55,
        ..Default::default()
    });
    let metal = scene.add_material(Material {
        base_color: [0.92, 0.9, 0.86, 1.0],
        metallic: 1.0,
        roughness: 0.25,
        ..Default::default()
    });
    for i in 0..144u32 {
        let (gx, gz) = (i % 12, i / 12);
        let p = Vec3::new(-22.0 + gx as f32 * 4.0, 0.0, -22.0 + gz as f32 * 4.0);
        if (gx + gz) % 2 == 0 {
            scene.add_instance(pillar, stone, Mat4::from_translation(p + Vec3::Y * 1.2));
        } else {
            scene.add_instance(
                sphere,
                metal,
                Mat4::from_translation(p + Vec3::Y * 0.5) * Mat4::from_scale(Vec3::splat(0.5)),
            );
        }
    }
    scene.lights = many_lights_list(count);
    scene.light.intensity = 0.0;
    scene.light.cast_shadows = false;
    scene.environment = Environment {
        intensity: 0.03,
        background: Background::Color(Vec3::new(0.003, 0.004, 0.01)),
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 26.0, 30.0),
        target: Vec3::new(0.0, 0.0, -2.0),
        far: 200.0,
        exposure: Exposure::linear(1.4),
        ..Default::default()
    };
    scene
}

/// Where [`moon_scene`] puts the moon: up and to the right of the
/// colonnade camera's view.
pub fn moon_direction() -> Vec3 {
    let cam = crate::atmo_scenes::colonnade_camera();
    let forward = (cam.target - cam.position).normalize();
    let right = forward.cross(Vec3::Y).normalize();
    (forward + Vec3::Y * 0.24 + right * 0.18).normalize()
}

/// The colonnade at 23:00 (the day-night look) with a moon of `size`
/// degrees and `phase` in view, lighting the scene from where it hangs.
pub fn moon_scene(phase: f32, size: f32) -> RenderScene {
    let mut scene = crate::atmo_scenes::day_night_scene(23.0, false);
    let moon = moon_direction();
    if let Some(look) = scene.environment.sky.look.as_mut() {
        look.moon_size = size;
        look.moon_phase = phase;
        look.moon_direction = moon;
    }
    scene.light.direction_to_light = moon;
    scene
}

/// A tangent-space normal map of round bumps (linear RGB), `cells` across.
pub fn bump_normal_map(size: u32, cells: f32) -> og_render::TextureData {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let u = ((x as f32 + 0.5) / size as f32 * cells).fract() - 0.5;
            let v = ((y as f32 + 0.5) / size as f32 * cells).fract() - 0.5;
            // A dome per cell: the slope of sqrt(r^2 - d^2) inside radius r;
            // glTF's convention, green up the image (rows run down it).
            let r2 = 0.16f32;
            let d2 = u * u + v * v;
            let n = if d2 < r2 {
                let h = (r2 - d2).sqrt().max(0.05);
                Vec3::new(u / h, -v / h, 1.0).normalize()
            } else {
                Vec3::Z
            };
            let enc = |c: f32| ((c * 0.5 + 0.5) * 255.0).round() as u8;
            rgba.extend_from_slice(&[enc(n.x), enc(n.y), enc(n.z), 255]);
        }
    }
    og_render::TextureData {
        width: size,
        height: size,
        rgba,
        color_space: og_render::ColorSpace::Linear,
        mips: og_render::Mips::Generate,
    }
}

/// A grille: opaque bars with holes of alpha 0 between them (sRGB).
pub fn grille(size: u32, bars: u32) -> og_render::TextureData {
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let cell = size / bars;
            let bar = x % cell < cell / 3 || y % cell < cell / 3;
            rgba.extend_from_slice(if bar {
                &[230, 200, 120, 255]
            } else {
                &[0, 0, 0, 0]
            });
        }
    }
    og_render::TextureData {
        width: size,
        height: size,
        rgba,
        color_space: og_render::ColorSpace::Srgb,
        mips: og_render::Mips::None,
    }
}

/// An open half cylinder (no caps), its inside facing +z: double-sided
/// materials show both faces.
pub fn half_shell(segments: u32) -> og_render::MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for i in 0..=segments {
        let a = std::f32::consts::PI * i as f32 / segments as f32;
        let (s, c) = a.sin_cos();
        for (y, v) in [(-0.5f32, 1.0f32), (0.5, 0.0)] {
            vertices.push(og_render::Vertex {
                position: [c * 0.5, y, -s * 0.5],
                normal: [c, 0.0, -s],
                uv: [i as f32 / segments as f32, v],
                ..Default::default()
            });
        }
    }
    for i in 0..segments {
        let a = i * 2;
        indices.extend_from_slice(&[a, a + 2, a + 1, a + 1, a + 2, a + 3]);
    }
    og_render::MeshData { vertices, indices }
}

/// Every M6 material feature on one board, under a low sun and the sky:
/// a metalness row, a roughness row, then emissive, a normal map on a
/// sphere and a box, clearcoat (glossy and rough) over a rough red base,
/// and a flattened ellipsoid (the normal matrix); then alpha mask (a
/// grille), blended glass, a double-sided shell, unlit.
pub fn materials_scene() -> RenderScene {
    use og_render::{Mips, StandardMaterial};
    let mut scene = RenderScene::new();
    crate::scenes::ground(&mut scene, 40.0, 0.5, 0.8);
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let cube = scene.add_mesh(shapes::cuboid(Vec3::splat(0.8)));
    let quad = scene.add_mesh(shapes::plane(1.0, 1.0));
    let shell = scene.add_mesh(half_shell(32));
    let bumps = scene.add_texture(bump_normal_map(256, 6.0));
    let grille = scene.add_texture(grille(128, 4));
    let mut checker = shapes::checker(64, 4);
    checker.mips = Mips::Generate;
    let checker = scene.add_texture(checker);
    let at = |x: f32, row: f32| Vec3::new(x * 1.3 - 2.6, 0.5, -row * 1.6 + 1.6);
    let put = |scene: &mut RenderScene, mesh, m: StandardMaterial, t: Mat4| {
        let h = scene.add_standard_material(m);
        scene.add_instance(mesh, h, t);
    };
    let gold = Vec3::new(1.0, 0.78, 0.34);
    for i in 0..5 {
        let f = i as f32 / 4.0;
        // Metalness 0 to 1.
        put(
            &mut scene,
            sphere,
            StandardMaterial {
                color: gold,
                metalness: f,
                roughness: 0.3,
                ..Default::default()
            },
            Mat4::from_translation(at(i as f32, 0.0)),
        );
        // Roughness 0.05 to 1 on a blue dielectric.
        put(
            &mut scene,
            sphere,
            StandardMaterial {
                color: Vec3::new(0.12, 0.3, 0.75),
                roughness: 0.05 + 0.95 * f,
                ..Default::default()
            },
            Mat4::from_translation(at(i as f32, 1.0)),
        );
    }
    // Row 3: emissive, normal map (sphere, box), clearcoat glossy and rough.
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            color: Vec3::splat(0.05),
            emissive: Vec3::new(1.0, 0.45, 0.1),
            emissiveIntensity: 3.0,
            ..Default::default()
        },
        Mat4::from_translation(at(0.0, 2.0)),
    );
    let bumpy = StandardMaterial {
        color: Vec3::new(0.75, 0.72, 0.68),
        roughness: 0.45,
        normalMap: bumps.slot(),
        ..Default::default()
    };
    put(
        &mut scene,
        sphere,
        bumpy,
        Mat4::from_translation(at(1.0, 2.0)),
    );
    put(
        &mut scene,
        cube,
        bumpy,
        Mat4::from_translation(at(2.0, 2.0) - Vec3::Y * 0.1) * Mat4::from_rotation_y(0.6),
    );
    for (i, coat_rough) in [(3.0, 0.03), (4.0, 0.35)] {
        put(
            &mut scene,
            sphere,
            StandardMaterial {
                color: Vec3::new(0.7, 0.06, 0.05),
                roughness: 0.75,
                clearcoat: 1.0,
                clearcoatRoughness: coat_rough,
                ..Default::default()
            },
            Mat4::from_translation(at(i, 2.0)),
        );
    }
    // Row 4: alpha mask, blended glass, double-sided, unlit, the ellipsoid.
    put(
        &mut scene,
        quad,
        StandardMaterial {
            map: grille.slot(),
            alphaTest: 0.5,
            side: 2,
            roughness: 0.6,
            ..Default::default()
        },
        Mat4::from_translation(at(0.0, 3.0) + Vec3::Y * 0.1)
            * Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2)
            * Mat4::from_scale(Vec3::splat(1.0)),
    );
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            color: Vec3::new(0.6, 0.9, 0.8),
            opacity: 0.35,
            transparent: 1,
            roughness: 0.05,
            ..Default::default()
        },
        Mat4::from_translation(at(1.0, 3.0)),
    );
    put(
        &mut scene,
        shell,
        StandardMaterial {
            map: checker.slot(),
            color: Vec3::new(1.0, 0.85, 0.6),
            side: 2,
            roughness: 0.7,
            ..Default::default()
        },
        Mat4::from_translation(at(2.0, 3.0)) * Mat4::from_rotation_y(-0.5),
    );
    put(
        &mut scene,
        cube,
        StandardMaterial::basic(Vec3::new(0.95, 0.3, 0.7)),
        Mat4::from_translation(at(3.0, 3.0) - Vec3::Y * 0.1) * Mat4::from_rotation_y(0.4),
    );
    put(
        &mut scene,
        sphere,
        StandardMaterial {
            color: Vec3::new(0.85, 0.85, 0.82),
            roughness: 0.2,
            ..Default::default()
        },
        Mat4::from_translation(at(4.0, 3.0) - Vec3::Y * 0.25)
            * Mat4::from_scale(Vec3::new(1.15, 0.45, 1.15)),
    );
    let sun = Vec3::new(-0.55, 0.6, 0.5);
    scene.light = og_render::DirectionalLight {
        direction_to_light: sun,
        intensity: 3.2,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.8;
    scene.camera = Camera {
        position: Vec3::new(0.0, 7.4, 5.4),
        target: Vec3::new(0.0, 0.0, -1.3),
        exposure: Exposure::linear(0.7),
        ..Default::default()
    };
    scene
}

/// A material shader that moves a mesh by `offset` (world units), ripples
/// it along its normal by `bulge` travelling up it, and paints bands of
/// `stripe` up its height: on skinned meshes, with shadows and the prepass.
pub fn ripple_shader() -> og_render::MaterialShader {
    use og_render::{MaterialShader, ParamKind, ShaderParam};
    let param = |name: &str, kind, default| ShaderParam {
        name: name.into(),
        kind,
        default,
    };
    MaterialShader {
        name: "ripple".into(),
        code: "fn vertex_offset(h: Hook, p: Params) -> vec3<f32> {
  let wave = sin(h.local.y * 9.0 - h.time * 4.0) * p.bulge;
  return h.normal * wave + p.offset;
}
fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  let band = step(0.5, fract(h.local.y * 2.5));
  return vec4<f32>(mix(c.rgb, p.stripe, band * 0.7), c.a);
}
"
        .into(),
        params: vec![
            param("offset", ParamKind::Vec3, [0.0; 4]),
            param("bulge", ParamKind::Float, [0.0; 4]),
            param("stripe", ParamKind::Vec3, [0.95, 0.85, 0.2, 0.0]),
        ],
        mode: Default::default(),
    }
}

/// The skinned cylinders of the `skinning` scene drawn with
/// [`ripple_shader`]: rippling, banded and lifted 0.4 m off the ground by
/// their material shader, casting their displaced shadows; ambient
/// occlusion and outlines read their displaced depth.
pub fn skinned_hooks_scene() -> RenderScene {
    let mut scene = crate::fx_scenes::skinning_scene();
    let ripple = scene.add_shader(ripple_shader());
    let m = scene.add_standard_material(og_render::StandardMaterial {
        color: Vec3::new(0.75, 0.3, 0.55),
        roughness: 0.45,
        outlineWidth: 2.0,
        outlineColor: Vec3::new(0.05, 0.02, 0.04),
        ..Default::default()
    });
    scene.set_material_shader(m, Some(ripple));
    scene.set_shader_param(m, "offset", &[0.0, 0.4, 0.0]);
    scene.set_shader_param(m, "bulge", &[0.05]);
    for s in &mut scene.skinned_instances {
        s.material = m;
    }
    scene.post.ao = Some(og_render::AmbientOcclusion {
        radius: 0.6,
        intensity: 1.0,
        method: og_render::AoMethod::Gtao,
    });
    scene.tick = 30;
    scene
}

/// The 3D fluid's pool (owed row 29): `spacing` apart on a jittered
/// lattice filling a basin's floor to a gently waving surface, as
/// og-physics' og.Fluid3D leaves particles at rest (about half its radius
/// apart).
pub fn fluid_3d_pool(spacing: f32) -> Vec<Vec3> {
    let mut out = Vec::new();
    let (nx, nz) = ((3.0 / spacing) as i32, (1.8 / spacing) as i32);
    let mut i = 0;
    for gx in 0..=nx {
        for gz in 0..=nz {
            let x = -1.5 + gx as f32 * spacing;
            let z = -0.9 + gz as f32 * spacing;
            let top = 0.42 + 0.06 * (x * 2.3).sin() * (z * 3.1).cos();
            let mut y = 0.06;
            while y <= top {
                let j = |salt| (hash01(i, salt) - 0.5) * spacing * 0.3;
                out.push(Vec3::new(x + j(1), y + j(2), z + j(3)));
                y += spacing;
                i += 1;
            }
        }
    }
    out
}

/// A falling drop of radius `r` at `centre`, `spacing` apart.
pub fn fluid_3d_drop(centre: Vec3, r: f32, spacing: f32) -> Vec<Vec3> {
    let n = (r / spacing).ceil() as i32;
    let mut out = Vec::new();
    for x in -n..=n {
        for y in -n..=n {
            for z in -n..=n {
                let p = Vec3::new(x as f32, y as f32, z as f32) * spacing;
                if p.length() <= r {
                    out.push(centre + p);
                }
            }
        }
    }
    out
}

/// Two 3D fluids in a stone basin: a clear blue pool over coloured tiles
/// (refracted and tinted through it) and an orange drop falling into it
/// (denser, its particles shading from orange to red), lit by the sun, the
/// sky and an orange point light, the basin's walls cutting the surface.
pub fn fluid_3d_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let stone = scene.add_material(Material {
        base_color: [0.62, 0.6, 0.56, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    let slab = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let block = |scene: &mut RenderScene, at: Vec3, size: Vec3, m| {
        scene.add_instance(slab, m, Mat4::from_translation(at) * Mat4::from_scale(size));
    };
    block(
        &mut scene,
        Vec3::new(0.0, -0.1, 0.0),
        Vec3::new(12.0, 0.2, 9.0),
        stone,
    );
    for (at, size) in [
        (Vec3::new(0.0, 0.45, -1.1), Vec3::new(3.6, 0.9, 0.2)),
        (Vec3::new(-1.7, 0.45, 0.0), Vec3::new(0.2, 0.9, 2.4)),
        (Vec3::new(1.7, 0.45, 0.0), Vec3::new(0.2, 0.9, 2.4)),
        (Vec3::new(0.0, 0.15, 1.1), Vec3::new(3.6, 0.3, 0.2)),
    ] {
        block(&mut scene, at, size, stone);
    }
    let tiles: Vec<_> = (0..6)
        .map(|i| {
            let c = hue(i as f32 / 6.0).lerp(Vec3::ONE, 0.2);
            scene.add_material(Material {
                base_color: [c.x, c.y, c.z, 1.0],
                roughness: 0.6,
                ..Default::default()
            })
        })
        .collect();
    for gx in 0..6 {
        for gz in 0..4 {
            let at = Vec3::new(-1.25 + gx as f32 * 0.5, 0.01, -0.75 + gz as f32 * 0.5);
            block(
                &mut scene,
                at,
                Vec3::new(0.42, 0.02, 0.42),
                tiles[(gx + gz) % 6],
            );
        }
    }
    let spacing = 0.1;
    scene.fluids_3d.push(og_render::Fluid3d {
        particles: fluid_3d_pool(spacing).into(),
        color: Vec3::new(0.2, 0.55, 0.9),
        opacity: 0.85,
        radius: 0.12,
        volume: spacing.powi(3),
        ..Default::default()
    });
    let drop = fluid_3d_drop(Vec3::new(0.35, 1.25, 0.1), 0.3, spacing);
    let colors: Vec<Vec3> = drop
        .iter()
        .map(|p| {
            Vec3::new(0.95, 0.5, 0.08).lerp(Vec3::new(0.85, 0.1, 0.05), (1.25 - p.y) / 0.6 + 0.5)
        })
        .collect();
    scene.fluids_3d.push(og_render::Fluid3d {
        particles: drop.into(),
        colors: Some(colors.into()),
        color: Vec3::new(0.95, 0.45, 0.1),
        opacity: 0.95,
        radius: 0.12,
        volume: spacing.powi(3),
        ..Default::default()
    });
    scene.lights.push(Light {
        position: Vec3::new(-1.0, 1.0, 0.9),
        color: Vec3::new(1.0, 0.6, 0.3),
        intensity: 6.0,
        range: 4.0,
        ..Default::default()
    });
    let sun = Vec3::new(-0.45, 0.7, 0.4);
    scene.light = og_render::DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.8;
    scene.environment.background = Background::Sky;
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.4, 3.9),
        target: Vec3::new(0.0, 0.35, 0.0),
        exposure: Exposure::linear(0.55),
        ..Default::default()
    };
    scene
}
