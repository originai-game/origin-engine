//! The depth wave's light scenes (docs/depth.md §7): area lights shaded
//! with linearly transformed cosines (`rect-lights`), spot and point light
//! cookies (`light-cookies`), fog lit as a medium by the sun through the
//! slots of a wall and by a spot (`volumetric-fog`: a froxel grid on the
//! full tier, a raymarch on WebGL2), and, natively, a Cornell box baked by
//! og-bake in the scene's build with a light volume lighting a ball inside
//! it (`baked-cornell`).

use glam::{Mat3, Mat4, Quat, Vec3};
use og_render::{
    Background, Camera, ColorSpace, Environment, Exposure, Light, LightCookie, LightKind, MeshData,
    Mips, RenderScene, StandardMaterial, TextureData, VolumetricFog, shapes,
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
    scene.light.intensity = 0.0;
    scene.light.cast_shadows = false;
    scene.environment = Environment {
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

/// Three rect lights over a glossy floor before a matte wall at night: a
/// warm one-sided panel facing down (its highlight on the boards is its
/// rectangle stretched toward the camera), a cyan two-sided panel standing
/// upright (it lights the wall behind it and the floor before it), and a
/// magenta one-sided panel facing left (nothing lit to its right).
pub fn rect_lights_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    dark(&mut scene, 0.03);
    put(
        &mut scene,
        shapes::plane(14.0, 1.0),
        matte(Vec3::splat(0.45), 0.25),
        Mat4::IDENTITY,
    );
    put(
        &mut scene,
        quad(
            Vec3::new(-7.0, 0.0, -3.0),
            Vec3::new(7.0, 0.0, -3.0),
            Vec3::new(7.0, 5.0, -3.0),
            Vec3::new(-7.0, 5.0, -3.0),
        ),
        matte(Vec3::splat(0.6), 0.85),
        Mat4::IDENTITY,
    );
    put(
        &mut scene,
        shapes::sphere(32, 16),
        matte(Vec3::splat(0.7), 0.4),
        Mat4::from_scale_rotation_translation(
            Vec3::splat(0.5),
            Quat::IDENTITY,
            Vec3::new(-0.2, 0.5, -0.4),
        ),
    );
    scene.lights = vec![
        // Facing down: right × up = +y, so up is -z.
        rect(
            Vec3::new(-2.6, 2.4, -0.8),
            Vec3::X,
            Vec3::NEG_Z,
            (2.4, 0.5),
            Vec3::new(1.0, 0.75, 0.45),
            10.0,
            false,
        ),
        rect(
            Vec3::new(1.4, 1.1, -1.6),
            Vec3::X,
            Vec3::Y,
            (0.4, 2.0),
            Vec3::new(0.3, 0.85, 1.0),
            12.0,
            true,
        ),
        // Facing -x: right × up = +x.
        rect(
            Vec3::new(4.2, 1.0, -0.5),
            Vec3::NEG_Z,
            Vec3::Y,
            (1.2, 0.8),
            Vec3::new(1.0, 0.25, 0.8),
            12.0,
            false,
        ),
    ];
    for l in scene.lights.clone() {
        fixture(&mut scene, &l);
    }
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.6, 6.5),
        target: Vec3::new(0.0, 0.8, -1.0),
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

/// A spot with a stained-glass cookie (red, amber and blue panes in dark
/// leading) thrown on the floor and the back wall, and a point light with a
/// cube cookie (six faces stacked +X, -X, +Y, -Y, +Z, -Z, each a grid of
/// round holes, green on the sides, closed above) dotting the walls and
/// floor around it.
pub fn light_cookies_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    dark(&mut scene, 0.04);
    let wall = matte(Vec3::splat(0.7), 0.9);
    put(&mut scene, shapes::plane(14.0, 1.0), wall, Mat4::IDENTITY);
    put(
        &mut scene,
        quad(
            Vec3::new(-7.0, 0.0, -3.0),
            Vec3::new(7.0, 0.0, -3.0),
            Vec3::new(7.0, 5.0, -3.0),
            Vec3::new(-7.0, 5.0, -3.0),
        ),
        wall,
        Mat4::IDENTITY,
    );
    put(
        &mut scene,
        quad(
            Vec3::new(5.0, 0.0, -3.0),
            Vec3::new(5.0, 0.0, 4.0),
            Vec3::new(5.0, 5.0, 4.0),
            Vec3::new(5.0, 5.0, -3.0),
        ),
        wall,
        Mat4::IDENTITY,
    );
    let glass = scene.add_texture(texture(64, 64, |x, y| {
        let (cx, cy) = (x % 32, y % 32);
        if cx < 3 || cy < 3 {
            return [12, 10, 8];
        }
        match (x / 32, y / 32) {
            (0, 0) => [235, 40, 30],
            (1, 0) => [255, 190, 60],
            (0, 1) => [40, 90, 240],
            _ => [250, 250, 240],
        }
    }));
    let cube = scene.add_texture(texture(32, 32 * 6, |x, y| {
        let (face, fy) = (y / 32, y % 32);
        let (cx, cy) = ((x % 16) as f32 - 7.5, (fy % 16) as f32 - 7.5);
        let hole = cx * cx + cy * cy < 22.0;
        match face {
            2 => [8, 8, 8],
            _ if hole => [90, 255, 120],
            _ => [8, 8, 8],
        }
    }));
    scene.lights = vec![
        Light {
            kind: LightKind::Spot {
                direction: Vec3::new(0.15, -1.0, -0.55).normalize(),
                angle_degrees: 32.0,
                penumbra: 0.1,
            },
            position: Vec3::new(-2.2, 4.2, 1.6),
            color: Vec3::ONE,
            intensity: 60.0,
            range: 12.0,
            cast_shadows: false,
        },
        Light {
            kind: LightKind::Point,
            position: Vec3::new(3.0, 1.4, -1.0),
            color: Vec3::ONE,
            intensity: 12.0,
            range: 8.0,
            cast_shadows: false,
        },
    ];
    scene.light_cookies = vec![
        LightCookie {
            light: 0,
            texture: glass,
        },
        LightCookie {
            light: 1,
            texture: cube,
        },
    ];
    put(
        &mut scene,
        shapes::sphere(16, 8),
        glow(Vec3::new(0.4, 1.0, 0.5), 1.5),
        Mat4::from_scale_rotation_translation(
            Vec3::splat(0.12),
            Quat::IDENTITY,
            Vec3::new(3.0, 1.4, -1.0),
        ),
    );
    scene.camera = Camera {
        position: Vec3::new(-0.5, 3.0, 7.0),
        target: Vec3::new(0.6, 1.0, -1.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    scene
}

/// Fog lit as a medium: a low sun behind a wall shines through its three
/// slots, throwing shafts toward the camera (anisotropy 0.6, forward
/// scattering) and bright bars on the ground; a warm spot on the right
/// draws its cone in the fog (on the full tier: the clustered lights light
/// the froxels; WebGL2's raymarch takes the sun alone).
pub fn volumetric_fog_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let sun = Vec3::new(0.15, 0.32, -1.0).normalize();
    scene.light.direction_to_light = sun;
    scene.light.intensity = 4.0;
    scene.light.color = Vec3::new(1.0, 0.9, 0.75);
    scene.light.cast_shadows = true;
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.15;
    scene.environment.background = Background::Color(Vec3::new(0.05, 0.06, 0.09));
    let stone = matte(Vec3::new(0.55, 0.5, 0.45), 0.9);
    put(&mut scene, shapes::plane(40.0, 1.0), stone, Mat4::IDENTITY);
    // The wall: piers between three slots 0.5 m wide, 0.6 m thick.
    let piers = [
        (-6.0f32, -1.75f32),
        (-1.25, -0.25),
        (0.25, 1.25),
        (1.75, 6.0),
    ];
    for (x0, x1) in piers {
        put(
            &mut scene,
            shapes::cuboid(Vec3::new(x1 - x0, 5.0, 0.6)),
            stone,
            Mat4::from_translation(Vec3::new((x0 + x1) * 0.5, 2.5, -3.0)),
        );
    }
    put(
        &mut scene,
        shapes::cuboid(Vec3::new(12.0, 1.0, 0.6)),
        stone,
        Mat4::from_translation(Vec3::new(0.0, 5.5, -3.0)),
    );
    scene.lights = vec![Light {
        kind: LightKind::Spot {
            direction: Vec3::new(-0.2, -1.0, 0.1).normalize(),
            angle_degrees: 22.0,
            penumbra: 0.3,
        },
        position: Vec3::new(3.6, 4.2, 0.5),
        color: Vec3::new(1.0, 0.6, 0.3),
        intensity: 40.0,
        range: 9.0,
        cast_shadows: true,
    }];
    scene.volumetric_fog = Some(VolumetricFog {
        color: Vec3::new(0.9, 0.92, 0.96),
        density: 0.06,
        height: 0.0,
        height_density: 0.04,
        falloff: 0.4,
        scattering: 0.9,
        anisotropy: 0.6,
        distance: 30.0,
    });
    scene.camera = Camera {
        position: Vec3::new(-0.8, 1.5, 6.0),
        target: Vec3::new(0.3, 2.0, -3.0),
        exposure: Exposure::linear(0.9),
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
    let mut scene = RenderScene::new();
    dark(&mut scene, 0.35);
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.0, 3.7),
        target: Vec3::new(0.0, 0.95, 0.0),
        projection: og_render::Projection::Perspective {
            fov_y_degrees: 40.0,
        },
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    cornell_box(&mut scene, Vec3::ZERO, false, baked)?;
    Ok(scene)
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
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.0, 5.6),
        target: Vec3::new(0.0, 0.95, 0.0),
        projection: og_render::Projection::Perspective {
            fov_y_degrees: 40.0,
        },
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    cornell_box(&mut scene, Vec3::new(-1.2, 0.0, 0.0), false, true)?;
    cornell_box(&mut scene, Vec3::new(1.2, 0.0, 0.0), true, true)?;
    Ok(scene)
}

/// One Cornell box at `at` (red wall on the left, green on the right, or
/// the other way round when `swapped`): its spot, its ball and its walls
/// and blocks, baked by og-bake on their own when `baked` (its pages and
/// its light volume added to the scene's bake).
#[cfg(not(target_arch = "wasm32"))]
fn cornell_box(
    scene: &mut RenderScene,
    at: Vec3,
    swapped: bool,
    baked: bool,
) -> Result<(), String> {
    use og_render::{BakedMesh, LightmapPages, LightmapUvs, ProbeVolume};
    use std::sync::Arc;

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
    let ball = (p(-0.55, 0.22, 0.55), 0.22);

    scene.lights.push(spot);
    put(
        scene,
        shapes::sphere(32, 16),
        matte(Vec3::splat(0.85), 0.7),
        Mat4::from_scale_rotation_translation(Vec3::splat(ball.1), Quat::IDENTITY, ball.0),
    );
    if !baked {
        for (mesh, color) in parts {
            put(scene, mesh, matte(color, 0.9), Mat4::IDENTITY);
        }
        return Ok(());
    }

    let mut bake = og_bake::Scene {
        sky: Vec3::splat(0.05),
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
    let LightKind::Spot {
        direction,
        angle_degrees,
        penumbra,
    } = spot.kind
    else {
        unreachable!("the box's light is a spot");
    };
    bake.lights.push(og_bake::Light::Spot {
        position: spot.position,
        direction,
        radiance: spot.color * spot.intensity,
        range: spot.range,
        cos_outer: angle_degrees.to_radians().cos(),
        cos_inner: (angle_degrees * (1.0 - penumbra)).to_radians().cos(),
    });
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
    let volume = og_bake::volume::bake(
        &bake,
        &og_bake::VolumeDesc {
            min: p(-0.95, 0.05, -0.95),
            max: p(0.95, 1.95, 0.95),
            spacing: 0.3,
        },
        64,
        3,
        1,
        1,
    );
    let pages = Arc::new(LightmapPages {
        width: lightmap.width,
        height: lightmap.height,
        pages: lightmap.pages,
    });
    for ((mesh, color), a) in parts.into_iter().zip(&atlas.meshes) {
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
        scene.baked.meshes.insert(
            handle.index(),
            BakedMesh {
                pages: pages.clone(),
                uvs: Arc::new(LightmapUvs { uvs }),
            },
        );
        let material = scene.add_standard_material(matte(color, 0.9));
        scene.add_instance(handle, material, Mat4::IDENTITY);
    }
    scene.baked.volumes.push(ProbeVolume {
        min: volume.min,
        max: volume.max,
        counts: volume.counts,
        probes: Arc::new(volume.probes),
    });
    Ok(())
}
