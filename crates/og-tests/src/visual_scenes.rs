//! Visual bodies (og-render's `VisualBody`, og-physics' SIMULATION_VISUAL):
//! cloth and soft volumes simulated on the GPU from their rest shapes, the
//! host handing in only those and the scene's colliders.
//!
//! The courtyard: three banners hung from a beam stream in a gusting wind,
//! a sheet dropped on a ball drapes over it onto the flagstones, a jelly
//! sags on its plinth and a water balloon hangs from a hook. Each body's
//! rest shape is what the tick would carry (a flat banner, a flat sheet in
//! the air, round volumes); everything else is the GPU's.

use glam::{Mat4, Quat, Vec3};
use og_render::{
    Background, Bloom, Camera, DeformedVertices, DirectionalLight, Environment, Exposure,
    GarmentDeformation, GarmentTopology, JointPalette, Material, MeshData, ProceduralSky,
    RenderScene, SceneCollider, SimulationSurface, SkinData, SkinnedInstance, StandardMaterial,
    Vertex, VisualBody, VisualGarment, day_night, shapes,
};
use std::sync::Arc;

/// The ticks the courtyard runs before its frame.
pub const TICKS: u32 = 90;

/// A `columns` x `rows` grid hanging from `origin` along `right` and down
/// `down`, `spacing` apart, with uvs over it: a cloth's rest shape.
pub fn grid(
    columns: u32,
    rows: u32,
    spacing: f32,
    origin: Vec3,
    right: Vec3,
    down: Vec3,
) -> MeshData {
    let mut vertices = Vec::new();
    for y in 0..rows {
        for x in 0..columns {
            vertices.push(Vertex {
                position: (origin + right * (x as f32 * spacing) + down * (y as f32 * spacing))
                    .to_array(),
                uv: [
                    x as f32 / (columns - 1) as f32,
                    y as f32 / (rows - 1) as f32,
                ],
                ..Default::default()
            });
        }
    }
    let mut indices = Vec::new();
    for y in 0..rows - 1 {
        for x in 0..columns - 1 {
            let i = y * columns + x;
            indices.extend([i, i + columns, i + 1, i + 1, i + columns, i + columns + 1]);
        }
    }
    MeshData { vertices, indices }
}

/// A welded icosphere of `radius` at `centre`, divided `detail` times,
/// wound outwards: a soft volume's rest shape.
pub fn icosphere(detail: u32, radius: f32, centre: Vec3) -> MeshData {
    let t = (1.0 + 5.0f32.sqrt()) * 0.5;
    let mut points: Vec<Vec3> = [
        (-1.0, t, 0.0),
        (1.0, t, 0.0),
        (-1.0, -t, 0.0),
        (1.0, -t, 0.0),
        (0.0, -1.0, t),
        (0.0, 1.0, t),
        (0.0, -1.0, -t),
        (0.0, 1.0, -t),
        (t, 0.0, -1.0),
        (t, 0.0, 1.0),
        (-t, 0.0, -1.0),
        (-t, 0.0, 1.0),
    ]
    .iter()
    .map(|&(x, y, z)| Vec3::new(x, y, z).normalize())
    .collect();
    let mut tris: Vec<[u32; 3]> = vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    for _ in 0..detail {
        let mut cache = std::collections::BTreeMap::new();
        let mut mid = |points: &mut Vec<Vec3>, a: u32, b: u32| -> u32 {
            *cache.entry((a.min(b), a.max(b))).or_insert_with(|| {
                points.push(((points[a as usize] + points[b as usize]) * 0.5).normalize());
                points.len() as u32 - 1
            })
        };
        tris = tris
            .iter()
            .flat_map(|&[a, b, c]| {
                let (ab, bc, ca) = (
                    mid(&mut points, a, b),
                    mid(&mut points, b, c),
                    mid(&mut points, c, a),
                );
                [[a, ab, ca], [b, bc, ab], [c, ca, bc], [ab, bc, ca]]
            })
            .collect();
    }
    MeshData {
        vertices: points
            .iter()
            .map(|p| Vertex {
                position: (centre + *p * radius).to_array(),
                uv: [p.x * 0.5 + 0.5, p.y * 0.5 + 0.5],
                ..Default::default()
            })
            .collect(),
        indices: tris.into_iter().flatten().collect(),
    }
}

fn cloth_material(scene: &mut RenderScene, color: Vec3, sheen: f32) -> og_render::MaterialHandle {
    scene.add_standard_material(StandardMaterial {
        color,
        roughness: 0.78,
        sheen,
        sheenColor: color * 0.6 + Vec3::splat(0.4),
        sheenRoughness: 0.6,
        side: 2,
        ..Default::default()
    })
}

/// Adds a body: its rest mesh as a simulation surface made visual.
fn body(
    scene: &mut RenderScene,
    mesh: MeshData,
    kind: SimulationSurface,
    body: VisualBody,
    material: og_render::MaterialHandle,
) -> og_render::MeshHandle {
    let handle = scene.add_mesh(MeshData::default());
    assert!(scene.set_simulation_surface(handle, mesh, kind));
    assert!(scene.set_visual_body(handle, Some(Arc::new(body))));
    scene.add_instance(handle, material, Mat4::IDENTITY);
    handle
}

/// An empty scene in the courtyard's afternoon light, at 60 ticks a second.
fn daylight() -> RenderScene {
    let mut scene = RenderScene::new();
    let look = day_night(15.5, 35.0, -35.0);
    scene.light = DirectionalLight {
        direction_to_light: look.light_direction,
        color: look.light_color,
        intensity: look.light_intensity,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: look.sun_direction,
            sun_color: look.sun_color,
            look: Some(look.sky),
        },
        intensity: look.ambient,
        background: Background::Procedural {
            sun_direction: look.sun_direction,
            sun_color: look.sun_color,
        },
        ..Environment::default()
    };
    scene.post.bloom = Some(Bloom::default());
    scene.tick_rate = 60.0;
    scene
}

/// The courtyard's bodies and colliders; `visual` false leaves every body
/// at its rest shape (what a device without compute draws).
pub fn visual_bodies_scene(visual: bool) -> RenderScene {
    let mut scene = daylight();
    let mut bodies = Vec::new();
    scene.camera = Camera {
        position: Vec3::new(0.4, 2.1, 6.4),
        target: Vec3::new(0.0, 1.15, 0.0),
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    // Flagstones, a beam on two posts, a plinth, a ball and a hook arm.
    let stone = scene.add_material(Material {
        base_color: [0.56, 0.52, 0.47, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    let wood = scene.add_material(Material {
        base_color: [0.33, 0.22, 0.14, 1.0],
        roughness: 0.7,
        ..Default::default()
    });
    let brass = scene.add_material(Material {
        base_color: [0.78, 0.6, 0.3, 1.0],
        metallic: 1.0,
        roughness: 0.35,
        ..Default::default()
    });
    let rope = scene.add_material(Material {
        base_color: [0.32, 0.26, 0.2, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    let ground = scene.add_mesh(shapes::plane(30.0, 10.0));
    scene.add_instance(ground, stone, Mat4::IDENTITY);
    scene.colliders.push(SceneCollider::Plane {
        normal: Vec3::Y,
        offset: 0.0,
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let solid_box = |scene: &mut RenderScene, at: Vec3, size: Vec3, material| {
        let transform = Mat4::from_scale_rotation_translation(size, Quat::IDENTITY, at);
        scene.add_instance(cube, material, transform);
        scene.colliders.push(SceneCollider::Box {
            transform,
            half_extents: Vec3::splat(0.5),
        });
    };
    // The beam the banners hang from, on its posts (left of centre).
    let beam_y = 2.9;
    solid_box(
        &mut scene,
        Vec3::new(-1.6, beam_y + 0.06, -0.6),
        Vec3::new(3.4, 0.12, 0.12),
        wood,
    );
    for x in [-3.2, 0.0] {
        solid_box(
            &mut scene,
            Vec3::new(x, beam_y * 0.5, -0.6),
            Vec3::new(0.12, beam_y, 0.12),
            wood,
        );
    }
    // The plinth the jelly sits on, and the ball the sheet falls on.
    solid_box(
        &mut scene,
        Vec3::new(1.55, 0.35, 0.6),
        Vec3::new(0.9, 0.7, 0.9),
        stone,
    );
    let ball = scene.add_mesh(shapes::sphere(32, 20));
    let (ball_at, ball_r) = (Vec3::new(-0.9, 0.42, 1.3), 0.42);
    scene.add_instance(
        ball,
        brass,
        Mat4::from_scale_rotation_translation(Vec3::splat(ball_r * 2.0), Quat::IDENTITY, ball_at),
    );
    scene.colliders.push(SceneCollider::Sphere {
        center: ball_at,
        radius: ball_r,
    });
    // The hook's arm over the balloon.
    solid_box(
        &mut scene,
        Vec3::new(2.85, 1.5, -0.4),
        Vec3::new(0.1, 3.0, 0.1),
        wood,
    );
    solid_box(
        &mut scene,
        Vec3::new(2.55, 2.95, -0.4),
        Vec3::new(0.7, 0.08, 0.08),
        wood,
    );

    let cloth = |pinned: Vec<u32>, wind: Vec3| VisualBody {
        pinned: pinned.into(),
        substeps: 16,
        damping: 0.985,
        wind,
        gust: 0.7,
        radius: 0.03,
        friction: 0.4,
        stiffness: 1.0,
        bending: 0.2,
        ..VisualBody::default()
    };
    // Three banners, 27 x 43 points 4 cm apart, from the beam's front.
    let (columns, rows, spacing) = (27u32, 43u32, 0.04);
    for (k, color) in [
        Vec3::new(0.62, 0.06, 0.07),
        Vec3::new(0.08, 0.16, 0.45),
        Vec3::new(0.72, 0.5, 0.12),
    ]
    .into_iter()
    .enumerate()
    {
        let x = -3.0 + k as f32 * 1.12;
        let mesh = grid(
            columns,
            rows,
            spacing,
            Vec3::new(x, beam_y, -0.5),
            Vec3::X,
            -Vec3::Y,
        );
        let mut pinned: Vec<u32> = (0..columns).collect();
        if k == 2 {
            // Curtain rings: every eighth point of the top row.
            pinned.retain(|x| x % 8 == 0 || x + 1 == columns);
        }
        let material = cloth_material(&mut scene, color, 0.5);
        bodies.push(body(
            &mut scene,
            mesh,
            SimulationSurface::Cloth { columns, rows },
            VisualBody {
                bending: 0.04,
                ..cloth(pinned, Vec3::new(6.5, 0.0, 2.0))
            },
            material,
        ));
    }
    // A sheet 24 x 24 points 6 cm apart, flat in the air over the ball.
    let n = 24;
    let sheet = grid(
        n,
        n,
        0.06,
        Vec3::new(-0.9 - 0.69, 1.25, 1.3 - 0.69),
        Vec3::X,
        Vec3::Z,
    );
    let linen = cloth_material(&mut scene, Vec3::new(0.86, 0.84, 0.78), 0.3);
    bodies.push(body(
        &mut scene,
        sheet,
        SimulationSurface::Cloth {
            columns: n,
            rows: n,
        },
        VisualBody {
            thickness: 0.05,
            ..cloth(Vec::new(), Vec3::ZERO)
        },
        linen,
    ));
    // The jelly on its plinth.
    let jelly = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.25, 0.72, 0.3),
        roughness: 0.18,
        clearcoat: 0.6,
        side: 2,
        ..Default::default()
    });
    let soft = |pinned: Vec<u32>, stiffness: f32| VisualBody {
        pinned: pinned.into(),
        substeps: 32,
        damping: 0.995,
        radius: 0.02,
        friction: 0.3,
        stiffness,
        pressure: 1.0,
        volume_stiffness: 1.0,
        ..VisualBody::default()
    };
    bodies.push(body(
        &mut scene,
        icosphere(3, 0.36, Vec3::new(1.55, 0.7 + 0.36 + 0.03, 0.6)),
        SimulationSurface::SoftBody,
        soft(Vec::new(), 0.25),
        jelly,
    ));
    // The balloon on a cord from the hook: its top pinned to the cord.
    let (cord_top, cord_end) = (2.91, 2.36);
    scene.add_instance(
        cube,
        rope,
        Mat4::from_scale_rotation_translation(
            Vec3::new(0.014, cord_top - cord_end + 0.02, 0.014),
            Quat::IDENTITY,
            Vec3::new(2.3, (cord_top + cord_end) * 0.5, -0.4),
        ),
    );
    let balloon = icosphere(3, 0.26, Vec3::new(2.3, cord_end - 0.26, -0.4));
    let top = balloon
        .vertices
        .iter()
        .fold(f32::MIN, |m, v| m.max(v.position[1]));
    let pins: Vec<u32> = (0..balloon.vertices.len() as u32)
        .filter(|&i| balloon.vertices[i as usize].position[1] >= top - 0.03)
        .collect();
    let rubber = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.85, 0.2, 0.42),
        roughness: 0.3,
        clearcoat: 0.8,
        side: 2,
        ..Default::default()
    });
    bodies.push(body(
        &mut scene,
        balloon,
        SimulationSurface::SoftBody,
        soft(pins, 0.6),
        rubber,
    ));
    if !visual {
        for handle in bodies {
            scene.set_visual_body(handle, None);
        }
    }
    scene
}

/// A frame's update: the next tick.
pub fn next_tick(scene: &mut RenderScene, frame: u32) {
    scene.tick = u64::from(frame) + 1;
}

/// The ticks the garment runs before its frame.
pub const GARMENT_TICKS: u32 = 120;

/// A skirt on a mannequin in a gusting wind (og.Garment's
/// SIMULATION_VISUAL): hung from its waist (the top ring held), each ring
/// below free to stray further from the skin's pose, held off the legs and
/// hips by their colliders and the backstop. `visual` false draws the
/// skin's pose (what a device without compute draws).
pub fn visual_garment_scene(visual: bool) -> RenderScene {
    let mut scene = daylight();
    scene.camera = Camera {
        position: Vec3::new(0.3, 0.9, 1.95),
        target: Vec3::new(0.0, 0.66, 0.0),
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    let stone = scene.add_material(Material {
        base_color: [0.56, 0.52, 0.47, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    let ground = scene.add_mesh(shapes::plane(30.0, 10.0));
    scene.add_instance(ground, stone, Mat4::IDENTITY);
    scene.colliders.push(SceneCollider::Plane {
        normal: Vec3::Y,
        offset: 0.0,
    });
    // The mannequin: legs, hips, torso and head, its legs and hips solid.
    let form = scene.add_material(Material {
        base_color: [0.78, 0.72, 0.64, 1.0],
        roughness: 0.55,
        ..Default::default()
    });
    let ball = scene.add_mesh(shapes::sphere(32, 20));
    let part = |scene: &mut RenderScene, at: Vec3, size: Vec3| {
        let t = Mat4::from_scale_rotation_translation(size, Quat::IDENTITY, at);
        scene.add_instance(ball, form, t);
    };
    for x in [-0.1f32, 0.1] {
        part(
            &mut scene,
            Vec3::new(x, 0.5, 0.0),
            Vec3::new(0.15, 1.0, 0.15),
        );
        scene.colliders.push(SceneCollider::Capsule {
            a: Vec3::new(x, 0.1, 0.0),
            b: Vec3::new(x, 0.9, 0.0),
            radius: 0.07,
        });
    }
    part(
        &mut scene,
        Vec3::new(0.0, 0.98, 0.0),
        Vec3::new(0.38, 0.3, 0.27),
    );
    part(
        &mut scene,
        Vec3::new(0.0, 1.35, 0.0),
        Vec3::new(0.34, 0.62, 0.22),
    );
    part(
        &mut scene,
        Vec3::new(0.0, 1.82, 0.0),
        Vec3::new(0.2, 0.25, 0.22),
    );
    scene.colliders.push(SceneCollider::Capsule {
        a: Vec3::new(-0.06, 0.98, 0.0),
        b: Vec3::new(0.06, 0.98, 0.0),
        radius: 0.13,
    });
    // The skirt: rings from the waist (r 0.2 at 1.03 m) flaring to the hem
    // (r 0.46 at 0.3 m), welded around; the drawn seam repeats a column.
    let (rings, sides) = (24u32, 48u32);
    let mut rest = Vec::new();
    let mut normals = Vec::new();
    for j in 0..rings {
        let t = j as f32 / (rings - 1) as f32;
        let (y, r) = (1.03 - 0.73 * t, 0.2 + 0.26 * t.powf(0.8));
        let slope = 0.26 * 0.8 * t.max(0.02).powf(-0.2) / 0.73;
        for i in 0..sides {
            let a = std::f32::consts::TAU * i as f32 / sides as f32;
            let (sin, cos) = a.sin_cos();
            rest.push(Vec3::new(r * cos, y, r * sin));
            normals.push(Vec3::new(cos, slope, sin).normalize());
        }
    }
    let point = |j: u32, i: u32| j * sides + i % sides;
    let mut triangles = Vec::new();
    for j in 0..rings - 1 {
        for i in 0..sides {
            let (a, b, c, d) = (
                point(j, i),
                point(j, i + 1),
                point(j + 1, i),
                point(j + 1, i + 1),
            );
            triangles.extend([[a, c, b], [b, c, d]]);
        }
    }
    let mut vertices = Vec::new();
    let mut map = Vec::new();
    for j in 0..rings {
        for i in 0..=sides {
            let p = point(j, i);
            map.push((p, 1.0));
            vertices.push(Vertex {
                position: rest[p as usize].to_array(),
                normal: normals[p as usize].to_array(),
                uv: [i as f32 / sides as f32 * 4.0, j as f32 / (rings - 1) as f32],
                ..Default::default()
            });
        }
    }
    let mut indices = Vec::new();
    for j in 0..rings - 1 {
        for i in 0..sides {
            let v = |j: u32, i: u32| j * (sides + 1) + i;
            indices.extend([
                v(j, i),
                v(j + 1, i),
                v(j, i + 1),
                v(j, i + 1),
                v(j + 1, i),
                v(j + 1, i + 1),
            ]);
        }
    }
    let n = vertices.len();
    let topology = Arc::new(GarmentTopology {
        rest: rest.clone(),
        normals: normals.clone(),
        skin: vec![([0; 4], [1.0, 0.0, 0.0, 0.0]); rest.len()],
        triangles,
        vertices: map,
    });
    let reach: Vec<f32> = (0..rings)
        .flat_map(|j| {
            let t = j as f32 / (rings - 1) as f32;
            std::iter::repeat_n(if j == 0 { 0.0 } else { 0.45 * t }, sides as usize)
        })
        .collect();
    let body = VisualBody {
        pinned: (0..sides).collect(),
        reach: reach.into(),
        // Deep enough that the legs and hips, not the skin's cone, hold the
        // windward side.
        backstop: 0.3,
        substeps: 24,
        damping: 0.985,
        wind: Vec3::new(4.5, 0.0, 1.2),
        gust: 0.7,
        radius: 0.015,
        friction: 0.3,
        stiffness: 1.0,
        bending: 0.06,
        garment: Some(VisualGarment {
            rest: Arc::new(rest.clone()),
            normals: Arc::new(normals),
        }),
        ..VisualBody::default()
    };
    let silk = cloth_material(&mut scene, Vec3::new(0.5, 0.05, 0.12), 0.7);
    let mesh = scene.add_skinned_mesh(
        MeshData { vertices, indices },
        SkinData {
            joints: vec![[0; 4]; n],
            weights: vec![[0.0; 4]; n],
        },
    );
    scene
        .joint_palettes
        .push(JointPalette::from(vec![Mat4::IDENTITY]));
    let points = Arc::new(rest);
    scene.deformed_vertices.push(DeformedVertices {
        garment: Some(GarmentDeformation {
            key: (1, 0),
            topology,
            palette: Arc::new(vec![Mat4::IDENTITY]),
            points: points.clone(),
            targets: points,
            world: Mat4::IDENTITY,
            output_transform: Mat4::IDENTITY,
            visual: visual.then(|| Arc::new(body)),
        }),
        ..Default::default()
    });
    let mut skirt = SkinnedInstance::new(mesh, silk, Mat4::IDENTITY, 0);
    skirt.deformed = Some(0);
    scene.skinned_instances.push(skirt);
    scene
}

/// The ticks the fluid runs before its frame.
pub const FLUID_TICKS: u32 = 150;

/// A dam break in a stone basin (og.Fluid3D's SIMULATION_VISUAL): a block
/// of 60 000 particles at the basin's left end falls, runs round a brass
/// ball and up the far wall, while a spout pours more in from the right.
/// `visual` false draws the block where it spawned (what a device without
/// compute draws).
pub fn visual_fluid_scene(visual: bool) -> RenderScene {
    let mut scene = daylight();
    scene.camera = Camera {
        position: Vec3::new(0.6, 2.6, 4.4),
        target: Vec3::new(0.0, 0.35, 0.0),
        exposure: Exposure::linear(0.85),
        ..Default::default()
    };
    let stone = scene.add_material(Material {
        base_color: [0.56, 0.52, 0.47, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    let brass = scene.add_material(Material {
        base_color: [0.78, 0.6, 0.3, 1.0],
        metallic: 1.0,
        roughness: 0.35,
        ..Default::default()
    });
    let ground = scene.add_mesh(shapes::plane(30.0, 10.0));
    scene.add_instance(ground, stone, Mat4::IDENTITY);
    scene.colliders.push(SceneCollider::Plane {
        normal: Vec3::Y,
        offset: 0.0,
    });
    // The basin's walls, 3 x 2 m inside, 0.9 m high.
    let (inside, high, thick) = (Vec3::new(3.0, 0.9, 2.0), 0.9, 0.12);
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    for (at, size) in [
        (
            Vec3::new(-(inside.x + thick) * 0.5, high * 0.5, 0.0),
            Vec3::new(thick, high, inside.z + 2.0 * thick),
        ),
        (
            Vec3::new((inside.x + thick) * 0.5, high * 0.5, 0.0),
            Vec3::new(thick, high, inside.z + 2.0 * thick),
        ),
        (
            Vec3::new(0.0, high * 0.5, -(inside.z + thick) * 0.5),
            Vec3::new(inside.x, high, thick),
        ),
        (
            Vec3::new(0.0, high * 0.5, (inside.z + thick) * 0.5),
            Vec3::new(inside.x, high * 0.35, thick),
        ),
    ] {
        let at = if size.y < high {
            at - Vec3::Y * (high - size.y) * 0.5
        } else {
            at
        };
        let transform = Mat4::from_scale_rotation_translation(size, Quat::IDENTITY, at);
        scene.add_instance(cube, stone, transform);
        scene.colliders.push(SceneCollider::Box {
            transform,
            half_extents: Vec3::splat(0.5),
        });
    }
    let ball = scene.add_mesh(shapes::sphere(32, 20));
    let (ball_at, ball_r) = (Vec3::new(0.2, 0.3, -0.1), 0.3);
    scene.add_instance(
        ball,
        brass,
        Mat4::from_scale_rotation_translation(Vec3::splat(ball_r * 2.0), Quat::IDENTITY, ball_at),
    );
    scene.colliders.push(SceneCollider::Sphere {
        center: ball_at,
        radius: ball_r,
    });
    // A brass spout over the basin's right end, pouring from its mouth.
    let spout = Mat4::from_scale_rotation_translation(
        Vec3::new(1.2, 0.17, 0.17),
        Quat::IDENTITY,
        Vec3::new(1.64, 1.78, 0.3),
    );
    scene.add_instance(cube, brass, spout);
    let fluid = og_render::VisualFluid {
        key: 1,
        count: 60_000,
        centre: Vec3::new(-1.0, 0.45, 0.0),
        size: Vec3::new(1.0, 0.9, 2.0),
        // The basin, from the block's centre, with room above it.
        bounds: Vec3::new(5.0, 3.0, 2.0),
        cell: 0.05,
        max_count: 110_000,
        substeps: 2,
        seed: 7,
        // The spout's mouth: 1500 particles a second leave it at 1.6 m/s
        // about 16 cm across and land at over 5 m/s, the stream arcing
        // out and narrowing as it falls.
        emit_rate: 1500.0,
        emit_at: Vec3::new(1.02, 1.76, 0.3),
        emit_radius: 0.07,
        emit_velocity: Vec3::new(-1.0, -1.2, 0.0),
        ..og_render::VisualFluid::default()
    };
    let volume = 1.0 * 0.9 * 2.0 / 60_000.0;
    let mut drawn = og_render::Fluid3d {
        // Basin water: clear where shallow, deep blue-green where it is
        // thick, the spacing (3 cm) covered twice over so thin sheets stay
        // whole.
        color: Vec3::new(0.06, 0.32, 0.42),
        opacity: 0.93,
        radius: 0.065,
        smoothing: 1.3,
        volume,
        ..og_render::Fluid3d::default()
    };
    if visual {
        drawn.visual = Some(Arc::new(fluid));
    } else {
        drawn.particles = Arc::from(og_render::visual_fluid_at_rest(&fluid));
    }
    scene.fluids_3d.push(drawn);
    scene
}

/// The ticks the spark spray runs before its frame.
pub const SPARK_TICKS: u32 = 90;

/// A grinder's sparks in a dark workshop (compute particles meeting the
/// scene's colliders, `CollisionTarget::Colliders`): a spray of hot
/// sparks off a wheel ricochets from a steel block's top, skips down a
/// sloped plate, glances off a brass ball and dies out on the floor,
/// every surface a collider and none read from the depth prepass.
pub fn visual_particles_scene() -> RenderScene {
    use og_render::{
        Background, CollisionTarget, ColorCurve, Curve, EmitterShape, Light, ParticleAlign,
        ParticleBlend, ParticleCollision, ParticleEmitter, ParticleSystem,
    };
    let mut scene = RenderScene::new();
    let moon = Vec3::new(-0.4, 0.7, -0.5);
    scene.light = DirectionalLight {
        direction_to_light: moon,
        color: Vec3::new(0.55, 0.65, 1.0),
        intensity: 0.15,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: moon,
            sun_color: Vec3::new(0.3, 0.35, 0.5),
            look: None,
        },
        intensity: 0.05,
        background: Background::Color(Vec3::new(0.006, 0.007, 0.012)),
        ..Environment::default()
    };
    scene.tick_rate = 60.0;
    scene.camera = Camera {
        position: Vec3::new(-0.7, 2.1, 3.3),
        target: Vec3::new(-0.2, 0.4, 0.0),
        exposure: Exposure::linear(1.6),
        ..Default::default()
    };
    let concrete = scene.add_material(Material {
        base_color: [0.32, 0.31, 0.3, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    let steel = scene.add_material(Material {
        base_color: [0.55, 0.56, 0.58, 1.0],
        metallic: 1.0,
        roughness: 0.38,
        ..Default::default()
    });
    let dark = scene.add_material(Material {
        base_color: [0.08, 0.08, 0.09, 1.0],
        metallic: 0.6,
        roughness: 0.5,
        ..Default::default()
    });
    // The grinding wheel's abrasive: matte grey grit.
    let grit = scene.add_material(Material {
        base_color: [0.12, 0.115, 0.11, 1.0],
        roughness: 0.95,
        ..Default::default()
    });
    let brick = scene.add_material(Material {
        base_color: [0.3, 0.2, 0.16, 1.0],
        roughness: 0.9,
        ..Default::default()
    });
    let brass = scene.add_material(Material {
        base_color: [0.78, 0.6, 0.3, 1.0],
        metallic: 1.0,
        roughness: 0.3,
        ..Default::default()
    });
    let ground = scene.add_mesh(shapes::plane(30.0, 10.0));
    scene.add_instance(ground, concrete, Mat4::IDENTITY);
    // The workshop's back wall.
    let wall = scene.add_mesh(shapes::cuboid(Vec3::new(14.0, 5.0, 0.3)));
    scene.add_instance(
        wall,
        brick,
        Mat4::from_translation(Vec3::new(0.0, 2.5, -2.2)),
    );
    scene.colliders.push(SceneCollider::Plane {
        normal: Vec3::Y,
        offset: 0.0,
    });
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let solid = |scene: &mut RenderScene, material, transform: Mat4| {
        scene.add_instance(cube, material, transform);
        scene.colliders.push(SceneCollider::Box {
            transform,
            half_extents: Vec3::splat(0.5),
        });
    };
    // The steel block the wheel grinds at its right end.
    solid(
        &mut scene,
        steel,
        Mat4::from_scale_rotation_translation(
            Vec3::new(1.1, 0.5, 0.55),
            Quat::IDENTITY,
            Vec3::new(0.0, 0.25, 0.0),
        ),
    );
    // A plate leaning off its left end, down to the floor.
    solid(
        &mut scene,
        steel,
        Mat4::from_scale_rotation_translation(
            Vec3::new(0.9, 0.025, 0.7),
            Quat::from_rotation_z(0.42),
            Vec3::new(-0.92, 0.2, 0.05),
        ),
    );
    // The grinder: its wheel standing over the block's right end, on a guard.
    let wheel_at = Vec3::new(0.62, 0.66, 0.0);
    let ball = scene.add_mesh(shapes::sphere(32, 20));
    scene.add_instance(
        ball,
        grit,
        Mat4::from_scale_rotation_translation(
            Vec3::new(0.34, 0.34, 0.06),
            Quat::IDENTITY,
            wheel_at,
        ),
    );
    scene.add_instance(
        cube,
        dark,
        Mat4::from_scale_rotation_translation(
            Vec3::new(0.5, 0.12, 0.12),
            Quat::from_rotation_z(0.3),
            wheel_at + Vec3::new(0.32, 0.16, 0.0),
        ),
    );
    // A brass ball in the spray's path on the floor.
    let (ball_at, ball_r) = (Vec3::new(-1.15, 0.16, 0.45), 0.16);
    scene.add_instance(
        ball,
        brass,
        Mat4::from_scale_rotation_translation(Vec3::splat(ball_r * 2.0), Quat::IDENTITY, ball_at),
    );
    scene.colliders.push(SceneCollider::Sphere {
        center: ball_at,
        radius: ball_r,
    });
    let spark = crate::fx_scenes::spark_texture(&mut scene);
    let sparks = scene.add_emitter(ParticleEmitter {
        max_particles: 2048,
        shape: EmitterShape::Cone {
            angle_degrees: 16.0,
            radius: 0.01,
        },
        rate: 900.0,
        lifetime: [0.7, 1.5],
        speed: [3.5, 6.5],
        size: [0.012, 0.022],
        size_over_life: Curve::new(&[(0.0, 1.0), (0.7, 0.8), (1.0, 0.3)]),
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 0.95, 0.8, 1.0]),
            (0.25, [1.0, 0.7, 0.3, 1.0]),
            (0.7, [1.0, 0.35, 0.08, 0.9]),
            (1.0, [0.6, 0.1, 0.02, 0.0]),
        ]),
        color: [7.0, 4.6, 2.0, 1.0],
        gravity: Vec3::new(0.0, -9.8, 0.0),
        drag: 0.35,
        texture: Some(spark),
        blend: ParticleBlend::Additive,
        align: ParticleAlign::Velocity { stretch: 0.035 },
        collision: Some(ParticleCollision {
            bounce: 0.45,
            friction: 0.25,
            against: CollisionTarget::Colliders,
        }),
        ..Default::default()
    });
    // Off the wheel's bottom-left, across the block and down the plate.
    let grind = wheel_at + Vec3::new(-0.2, -0.17, 0.0);
    let towards = Vec3::new(-1.0, 0.12, 0.08).normalize();
    scene.particles.push(ParticleSystem::new(
        1,
        sparks,
        Mat4::from_rotation_translation(Quat::from_rotation_arc(Vec3::Y, towards), grind),
        11,
        0,
    ));
    // The sparks' glow: above and in front of the grind, so the wheel is
    // not lit flat; no shadows (it would throw the wheel across the wall).
    scene.lights = vec![Light {
        position: grind + Vec3::new(-0.35, 0.3, 0.7),
        color: Vec3::new(1.0, 0.55, 0.22),
        intensity: 1.6,
        range: 5.0,
        ..Default::default()
    }];
    scene.post.bloom = Some(Bloom {
        threshold: 1.0,
        knee: 0.6,
        intensity: 0.45,
    });
    scene
}

/// Steps a scene's clock and its particle systems' to `frame + 1`.
pub fn next_particle_tick(scene: &mut RenderScene, frame: u32) {
    crate::fx_scenes::tick_all(scene, u64::from(frame) + 1);
}
