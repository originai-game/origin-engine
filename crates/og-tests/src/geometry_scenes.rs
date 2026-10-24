//! The depth wave's geometry scenes (docs/depth.md §1): the three.js
//! primitives and a procedural buffer geometry (`geometry-procedural`),
//! lines (`lines`), point clouds (`points`), wireframe materials
//! (`wireframe`) and debug draw over the finished frame (`debug-overlay`).

use glam::{Mat4, Quat, Vec3};
use og_render::geometry::{self, BufferAttributes, primitives};
use og_render::{
    Camera, DirectionalLight, Exposure, LineLayer, LineMode, Lines, MeshData, Points, RenderScene,
    StandardMaterial, debug, shapes,
};
use std::f32::consts::{PI, TAU};

fn lit(scene: &mut RenderScene, eye: Vec3, target: Vec3) {
    let sun = Vec3::new(-0.5, 0.75, 0.45);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment.sky.sun_direction = sun;
    scene.environment.intensity = 0.7;
    scene.camera = Camera {
        position: eye,
        target,
        exposure: Exposure::linear(0.75),
        ..Default::default()
    };
    // Widths in CSS pixels at one per device pixel of the 720-line frame.
    scene.css_height = 720.0;
}

fn put(scene: &mut RenderScene, mesh: MeshData, m: StandardMaterial, t: Mat4) {
    let mesh = scene.add_mesh(mesh);
    let h = scene.add_standard_material(m);
    scene.add_instance(mesh, h, t);
}

fn colour(r: f32, g: f32, b: f32) -> StandardMaterial {
    StandardMaterial {
        color: Vec3::new(r, g, b),
        roughness: 0.5,
        ..Default::default()
    }
}

/// A terrain-like height field of `n`×`n` cells over `size` metres, as the
/// og.render.bufferGeometry a game makes: positions and indices only, its
/// normals computed (`phase` moves its waves, as the station's per-tick
/// update does).
pub fn terrain_patch(n: u32, size: f32, phase: f32) -> MeshData {
    let mut positions = Vec::with_capacity(((n + 1) * (n + 1) * 3) as usize);
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let (u, v) = (i as f32 / n as f32, j as f32 / n as f32);
            let (x, z) = ((u - 0.5) * size, (v - 0.5) * size);
            let y = 0.5 * (x * 1.6 + phase).sin() * (z * 1.3).cos()
                + 0.25 * (x * 0.7 - z * 2.1 + phase * 0.5).sin();
            positions.extend([x, y, z]);
            uvs.extend([u, v]);
            // Grass in the hollows, rock and snow on the tops.
            let t = ((y + 0.75) / 1.5).clamp(0.0, 1.0);
            colors.extend([0.25 + 0.7 * t * t, 0.5 + 0.4 * t * t, 0.2 + 0.75 * t * t]);
        }
    }
    let mut indices = Vec::new();
    let row = n + 1;
    for j in 0..n {
        for i in 0..n {
            let a = j * row + i;
            // Counter-clockwise seen from above (+y).
            indices.extend([a, a + row, a + 1, a + 1, a + row, a + row + 1]);
        }
    }
    geometry::buffer_geometry(&BufferAttributes {
        positions: &positions,
        uvs: Some(&uvs),
        colors: Some(&colors),
        indices: Some(&indices),
        ..Default::default()
    })
    .expect("a valid height field")
}

/// The points of the tube's spline: a rising loop.
pub fn tube_points() -> Vec<Vec3> {
    (0..7)
        .map(|k| {
            let a = k as f32 / 6.0 * PI * 1.5;
            Vec3::new(a.cos() * 1.2, 0.4 + k as f32 * 0.22, a.sin() * 1.2)
        })
        .collect()
}

/// `geometry-procedural`: the seven new primitives in a row, a tube along
/// a spline and a height field made from attribute arrays with computed
/// normals, on a lit floor.
pub fn procedural_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::scenes::ground(&mut scene, 40.0, 0.5, 0.85);
    let row = |k: f32| Vec3::new(-4.5 + k * 1.5, 0.6, -1.5);
    put(
        &mut scene,
        primitives::cylinder(&primitives::Cylinder {
            radius_top: 0.35,
            radius_bottom: 0.5,
            height: 1.2,
            ..Default::default()
        }),
        colour(0.85, 0.3, 0.2),
        Mat4::from_translation(row(0.0)),
    );
    put(
        &mut scene,
        primitives::cone(0.5, 1.2, 32, 1, false),
        colour(0.95, 0.65, 0.15),
        Mat4::from_translation(row(1.0)),
    );
    put(
        &mut scene,
        primitives::capsule(0.35, 0.5, 8, 24, 1),
        colour(0.3, 0.75, 0.35),
        Mat4::from_translation(row(2.0)),
    );
    put(
        &mut scene,
        primitives::torus(0.42, 0.16, 16, 48, TAU),
        colour(0.2, 0.55, 0.9),
        Mat4::from_translation(row(3.0) + Vec3::Y * 0.1) * Mat4::from_rotation_x(-0.6),
    );
    let flat = |r: f32, g: f32, b: f32| StandardMaterial {
        side: 2,
        ..colour(r, g, b)
    };
    put(
        &mut scene,
        primitives::ring(0.25, 0.55, 32, 2, 0.0, TAU),
        flat(0.6, 0.3, 0.85),
        Mat4::from_translation(row(4.0)) * Mat4::from_rotation_y(-0.3),
    );
    put(
        &mut scene,
        primitives::circle(0.55, 32, 0.0, PI * 1.5),
        flat(0.9, 0.85, 0.8),
        Mat4::from_translation(row(5.0)) * Mat4::from_rotation_y(-0.3),
    );
    put(
        &mut scene,
        primitives::tube(&tube_points(), 96, 0.14, 12, false),
        StandardMaterial {
            metalness: 0.6,
            roughness: 0.3,
            ..colour(0.95, 0.75, 0.35)
        },
        Mat4::from_translation(Vec3::new(3.6, 0.0, 1.2)),
    );
    put(
        &mut scene,
        terrain_patch(48, 3.6, 0.0),
        colour(1.0, 1.0, 1.0),
        Mat4::from_translation(Vec3::new(-2.4, 0.8, 1.2)),
    );
    lit(
        &mut scene,
        Vec3::new(0.0, 4.6, 7.0),
        Vec3::new(0.0, 0.4, -0.2),
    );
    scene
}

/// `lines`: a polyline, segments, a closed loop and a dashed spiral around a
/// box, a thick world-width line passing behind it (depth tested) and one
/// drawn over it, per-vertex colours, and a dashed screen line.
pub fn lines_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::scenes::ground(&mut scene, 40.0, 0.5, 0.85);
    put(
        &mut scene,
        shapes::cuboid(Vec3::new(1.2, 1.2, 1.2)),
        colour(0.7, 0.7, 0.72),
        Mat4::from_translation(Vec3::new(0.0, 0.6, 0.0)),
    );
    // A zigzag strip with colours from red to blue.
    let zig: Vec<Vec3> = (0..9)
        .map(|k| Vec3::new(-4.0 + k as f32 * 0.35, 0.3 + (k % 2) as f32 * 0.8, 1.0))
        .collect();
    let n = zig.len() as f32;
    scene.lines.push(Lines {
        colors: (0..zig.len())
            .map(|k| {
                let t = k as f32 / (n - 1.0);
                [1.0 - t, 0.2, t, 1.0]
            })
            .collect(),
        points: zig,
        width: 3.0,
        ..Default::default()
    });
    // A closed loop on the ground around the box.
    scene.lines.push(Lines {
        points: (0..24)
            .map(|k| {
                let a = k as f32 / 24.0 * TAU;
                Vec3::new(a.cos() * 1.6, 0.02, a.sin() * 1.6)
            })
            .collect(),
        mode: LineMode::Loop,
        color: [1.0, 0.85, 0.2, 1.0],
        width: 2.0,
        ..Default::default()
    });
    // A dashed helix climbing around the box (dashes in world units).
    scene.lines.push(Lines {
        points: (0..120)
            .map(|k| {
                let a = k as f32 / 20.0 * TAU / 2.0;
                Vec3::new(a.cos() * 1.1, 0.1 + k as f32 * 0.02, a.sin() * 1.1)
            })
            .collect(),
        color: [0.3, 1.0, 0.6, 1.0],
        width: 2.0,
        dashed: Some((0.2, 0.12)),
        ..Default::default()
    });
    // Segments: a fan of rays from a point on the right.
    let hub = Vec3::new(3.2, 0.1, 0.5);
    scene.lines.push(Lines {
        points: (0..8)
            .flat_map(|k| {
                let a = k as f32 / 8.0 * PI;
                [hub, hub + Vec3::new(a.cos(), a.sin(), 0.0) * 1.3]
            })
            .collect(),
        mode: LineMode::Segments,
        color: [0.9, 0.4, 1.0, 1.0],
        width: 1.5,
        ..Default::default()
    });
    // Behind the box from the camera: hidden where the box is (depth tested)...
    scene.lines.push(Lines {
        points: vec![Vec3::new(-3.0, 0.9, -1.5), Vec3::new(3.0, 0.9, -1.5)],
        color: [1.0, 0.3, 0.2, 1.0],
        width: 0.12,
        world_width: true,
        ..Default::default()
    });
    // ...and over it (no depth test).
    scene.lines.push(Lines {
        points: vec![Vec3::new(-3.0, 0.45, -1.5), Vec3::new(3.0, 0.45, -1.5)],
        color: [0.2, 0.6, 1.0, 1.0],
        width: 0.12,
        world_width: true,
        depth_test: false,
        ..Default::default()
    });
    // A dashed screen line across the top (CSS pixels, y down).
    scene.lines.push(Lines {
        points: vec![Vec3::new(80.0, 60.0, 0.0), Vec3::new(1200.0, 60.0, 0.0)],
        color: [1.0, 1.0, 1.0, 1.0],
        width: 3.0,
        screen: true,
        dashed: Some((24.0, 12.0)),
        ..Default::default()
    });
    lit(
        &mut scene,
        Vec3::new(0.0, 3.2, 6.5),
        Vec3::new(0.0, 0.6, 0.0),
    );
    scene
}

/// `points`: a point cloud on a sphere's surface coloured by height with
/// world-sized points, and a flat grid of fixed-size points.
pub fn points_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::scenes::ground(&mut scene, 40.0, 0.5, 0.85);
    let n = 1500;
    let golden = PI * (3.0 - 5f32.sqrt());
    let mut positions = Vec::with_capacity(n);
    let mut colors = Vec::with_capacity(n);
    for k in 0..n {
        let y = 1.0 - 2.0 * (k as f32 + 0.5) / n as f32;
        let r = (1.0 - y * y).sqrt();
        let a = golden * k as f32;
        positions.push(Vec3::new(r * a.cos(), y, r * a.sin()) * 1.3 + Vec3::new(-1.6, 1.5, 0.0));
        let t = (y + 1.0) * 0.5;
        colors.push([0.2 + 0.8 * t, 0.4, 1.0 - t, 1.0]);
    }
    scene.points.push(Points {
        positions,
        colors,
        size: 0.05,
        size_attenuation: true,
        ..Default::default()
    });
    scene.points.push(Points {
        positions: (0..12)
            .flat_map(|i| {
                (0..12).map(move |j| Vec3::new(1.2 + i as f32 * 0.22, 0.05, -1.2 + j as f32 * 0.22))
            })
            .collect(),
        color: [1.0, 0.85, 0.3, 1.0],
        size: 5.0,
        size_attenuation: false,
        ..Default::default()
    });
    lit(
        &mut scene,
        Vec3::new(0.0, 3.0, 6.0),
        Vec3::new(0.0, 0.9, 0.0),
    );
    scene
}

/// `wireframe`: a solid sphere beside the same mesh with a wireframe
/// material, a wireframe torus and a wireframe height field.
pub fn wireframe_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::scenes::ground(&mut scene, 40.0, 0.5, 0.85);
    let sphere = scene.add_mesh(shapes::sphere(24, 16));
    let solid = scene.add_standard_material(colour(0.8, 0.3, 0.2));
    let wire = scene.add_standard_material(StandardMaterial {
        wireframe: 1,
        ..colour(0.2, 0.9, 1.0)
    });
    scene.add_instance(
        sphere,
        solid,
        Mat4::from_translation(Vec3::new(-2.4, 0.8, 0.0)) * Mat4::from_scale(Vec3::splat(1.5)),
    );
    scene.add_instance(
        sphere,
        wire,
        Mat4::from_translation(Vec3::new(-0.6, 0.8, 0.0)) * Mat4::from_scale(Vec3::splat(1.5)),
    );
    put(
        &mut scene,
        primitives::torus(0.6, 0.22, 12, 36, TAU),
        StandardMaterial {
            wireframe: 1,
            emissive: Vec3::new(0.6, 0.2, 0.0),
            ..colour(1.0, 0.6, 0.1)
        },
        Mat4::from_translation(Vec3::new(1.3, 0.9, 0.0)) * Mat4::from_rotation_x(-0.8),
    );
    put(
        &mut scene,
        terrain_patch(16, 2.2, 0.0),
        StandardMaterial {
            wireframe: 1,
            ..colour(0.5, 1.0, 0.4)
        },
        Mat4::from_translation(Vec3::new(3.3, 0.5, 0.3)),
    );
    lit(
        &mut scene,
        Vec3::new(0.0, 2.8, 5.6),
        Vec3::new(0.3, 0.7, 0.0),
    );
    scene
}

/// A debug label as `og.debug.text` draws one (og-runtime `geometry.rs`
/// `texts`): the world point projected through the scene's camera to the
/// 1280×720 play surface, the label centred above it in the overlay's UI
/// layer, so it is neither bloomed nor tone mapped and no mesh hides it.
fn debug_label(
    scene: &mut RenderScene,
    font: og_render::FontHandle,
    at: Vec3,
    text: &str,
    size: f32,
    color: [f32; 4],
) {
    let (w, h) = (1280.0, 720.0);
    let cam = &scene.camera;
    let fov = match cam.projection {
        og_render::Projection::Perspective { fov_y_degrees } => fov_y_degrees,
        og_render::Projection::Orthographic { .. } => 50.0,
    };
    let vpm =
        glam::camera::rh::proj::directx::perspective(fov.to_radians(), w / h, cam.near, cam.far)
            * glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let clip = vpm * at.extend(1.0);
    let ndc = clip.truncate() / clip.w;
    let width = scene.font(font).map_or(0.0, |f| f.measure(text) * size);
    let x = (ndc.x * 0.5 + 0.5) * w - width * 0.5;
    let y = (0.5 - ndc.y * 0.5) * h - size * 0.6;
    let o = scene
        .overlay
        .get_or_insert_with(|| og_render::Overlay::new(w, h, 0.0));
    o.texts.push(og_render::Text {
        position: Vec3::new(x, y, 0.0),
        size,
        color,
        layer: og_render::screen_layer(og_render::UI_LAYER),
        ..og_render::Text::new(font, text)
    });
}

/// `debug-overlay`: a bloomed scene with debug draw over the finished frame
/// (never bloomed, tone mapped or hidden): a box around a sphere, the
/// sphere's outline, an arrow along the sun, a frame's axes, a box behind a
/// wall that shows through it, and two `og.debug.text` labels: one above
/// the sphere (over the bloom), one at the hidden box (through the wall).
pub fn debug_scene() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let font = crate::scenes::load_font(&mut scene)?;
    crate::scenes::ground(&mut scene, 40.0, 0.5, 0.85);
    put(
        &mut scene,
        shapes::sphere(32, 24),
        StandardMaterial {
            emissive: Vec3::new(1.0, 0.5, 0.15),
            emissiveIntensity: 4.0,
            ..colour(0.1, 0.1, 0.1)
        },
        Mat4::from_translation(Vec3::new(-1.5, 0.6, 0.0)) * Mat4::from_scale(Vec3::splat(1.2)),
    );
    put(
        &mut scene,
        shapes::cuboid(Vec3::new(2.0, 1.6, 0.2)),
        colour(0.6, 0.6, 0.65),
        Mat4::from_translation(Vec3::new(1.6, 0.8, 0.8)),
    );
    scene.post.bloom = Some(og_render::Bloom {
        threshold: 1.0,
        knee: 0.5,
        intensity: 0.8,
    });
    let green = [0.3, 1.0, 0.4, 1.0];
    scene.lines.push(debug::segments(
        debug::box_edges(
            Mat4::from_translation(Vec3::new(-1.5, 0.6, 0.0)),
            Vec3::splat(0.65),
        ),
        green,
        2.0,
    ));
    scene.lines.push(debug::segments(
        debug::sphere(Vec3::new(-1.5, 0.6, 0.0), 0.62, Quat::IDENTITY),
        [1.0, 1.0, 0.3, 1.0],
        1.0,
    ));
    // Behind the wall: debug draw shows through.
    scene.lines.push(debug::segments(
        debug::box_edges(
            Mat4::from_translation(Vec3::new(1.6, 0.5, -0.4)),
            Vec3::splat(0.4),
        ),
        [1.0, 0.3, 0.9, 1.0],
        2.0,
    ));
    scene.lines.push(debug::segments(
        debug::arrow(
            Vec3::new(0.0, 0.05, 1.5),
            Vec3::new(0.0, 0.05, 1.5) + Vec3::new(-0.5, 0.75, 0.45) * 1.8,
            0.3,
        ),
        [1.0, 0.6, 0.1, 1.0],
        2.0,
    ));
    for l in debug::axes(Mat4::from_translation(Vec3::new(-3.3, 0.02, 1.0)), 1.0, 3.0) {
        scene.lines.push(l);
    }
    scene.points.push(Points {
        positions: vec![Vec3::new(-1.5, 0.6, 0.0)],
        color: [1.0, 1.0, 1.0, 1.0],
        size: 8.0,
        size_attenuation: false,
        depth_test: false,
        layer: LineLayer::Overlay,
        ..Default::default()
    });
    lit(
        &mut scene,
        Vec3::new(0.0, 3.0, 6.2),
        Vec3::new(0.0, 0.6, 0.0),
    );
    // The labels: after the camera, since they are projected through it.
    debug_label(
        &mut scene,
        font,
        Vec3::new(-1.5, 1.5, 0.0),
        "sphere 1.2 m",
        22.0,
        [0.3, 1.0, 0.4, 1.0],
    );
    debug_label(
        &mut scene,
        font,
        Vec3::new(1.6, 1.1, -0.4),
        "hidden box",
        22.0,
        [1.0, 0.3, 0.9, 1.0],
    );
    Ok(scene)
}

// ── geometry-shapes: og-geom's generators (round 3, geom-1) ─────────────────

/// An og-geom mesh as the renderer holds one (as og-runtime converts it).
pub fn from_geom(m: og_geom::Mesh) -> MeshData {
    let vertices = (0..m.positions.len())
        .map(|i| og_render::Vertex {
            position: m.positions[i],
            normal: m.normals[i],
            uv: m.uvs[i],
            color: m.colors.get(i).copied().unwrap_or([1.0; 4]),
            tangent: [0.0; 4],
        })
        .collect();
    MeshData {
        vertices,
        indices: m.indices,
    }
}

fn to_geom(m: &MeshData) -> og_geom::Mesh {
    og_geom::Mesh {
        positions: m.vertices.iter().map(|v| v.position).collect(),
        normals: m.vertices.iter().map(|v| v.normal).collect(),
        uvs: m.vertices.iter().map(|v| v.uv).collect(),
        colors: Vec::new(),
        indices: m.indices.clone(),
    }
}

/// A gear of `teeth` trapezoid teeth with a round hole, as an og.Shape
/// flattens one.
pub fn gear_shape(teeth: usize) -> og_geom::ShapePoints {
    use glam::DVec2;
    use og_geom::trig::{TAU, sincos};
    let mut points = Vec::new();
    for i in 0..teeth {
        for (f, r) in [(0.0, 0.62), (0.18, 0.8), (0.42, 0.8), (0.6, 0.62)] {
            let (s, c) = sincos((i as f64 + f) / teeth as f64 * TAU);
            points.push(DVec2::new(r * c, r * s));
        }
    }
    let hole = |cx: f64, cy: f64, r: f64, n: usize| -> Vec<DVec2> {
        (0..n)
            .map(|k| {
                let (s, c) = sincos(-(k as f64) / n as f64 * TAU);
                DVec2::new(cx + r * c, cy + r * s)
            })
            .collect()
    };
    og_geom::ShapePoints {
        points,
        holes: vec![
            hole(0.0, 0.0, 0.16, 24),
            hole(0.36, 0.0, 0.07, 12),
            hole(-0.36, 0.0, 0.07, 12),
        ],
    }
}

/// A sphere as a lathe of a half circle (closed exactly, for CSG).
fn ball(r: f64, segments: u32, rows: u32) -> og_geom::Mesh {
    use og_geom::trig::{PI, sincos};
    let points = (0..=rows)
        .map(|j| {
            let (s, c) = sincos(j as f64 / rows as f64 * PI);
            let x = if j == 0 || j == rows { 0.0 } else { r * s };
            glam::DVec2::new(x, -r * c)
        })
        .collect();
    og_geom::lathe(&og_geom::Lathe {
        points,
        segments,
        ..Default::default()
    })
}

/// A die: a unit cube with its 21 pips subtracted, one sphere at a time;
/// the spheres are black, so the pits they leave are.
pub fn dice() -> og_geom::Mesh {
    use glam::DVec3;
    let mut body = to_geom(&shapes::cuboid(Vec3::ONE));
    let mut pip = ball(0.11, 20, 10);
    pip.colors = vec![[0.05, 0.05, 0.06, 1.0]; pip.positions.len()];
    let s = 0.25;
    // Pips on a face: (u, v) in the face's plane, by count.
    let layout = |n: usize| -> Vec<(f64, f64)> {
        match n {
            1 => vec![(0.0, 0.0)],
            2 => vec![(-s, -s), (s, s)],
            3 => vec![(-s, -s), (0.0, 0.0), (s, s)],
            4 => vec![(-s, -s), (s, -s), (-s, s), (s, s)],
            5 => vec![(-s, -s), (s, -s), (0.0, 0.0), (-s, s), (s, s)],
            _ => vec![(-s, -s), (-s, 0.0), (-s, s), (s, -s), (s, 0.0), (s, s)],
        }
    };
    // Each face: its outward normal, its u and v axes, its count (opposite faces sum to 7).
    let faces = [
        (DVec3::Z, DVec3::X, DVec3::Y, 1),
        (DVec3::NEG_Z, DVec3::X, DVec3::Y, 6),
        (DVec3::X, DVec3::Z, DVec3::Y, 3),
        (DVec3::NEG_X, DVec3::Z, DVec3::Y, 4),
        (DVec3::Y, DVec3::X, DVec3::Z, 2),
        (DVec3::NEG_Y, DVec3::X, DVec3::Z, 5),
    ];
    for (n, u, v, count) in faces {
        for (a, b) in layout(count) {
            let at = og_geom::Placement {
                position: n * 0.54 + u * a + v * b,
                ..Default::default()
            };
            body = og_geom::csg(
                &body,
                og_geom::Placement::default(),
                &pip,
                at,
                og_geom::CsgOp::Subtract,
            )
            .expect("a pip");
        }
    }
    body
}

/// A ribbed horn: a ten-lobed section swept along a curve, tapering from 1
/// to 0.08 and twisting a turn and a half.
pub fn horn() -> og_geom::Mesh {
    use glam::{DVec2, DVec3};
    use og_geom::trig::{TAU, sincos};
    let section: Vec<DVec2> = (0..40)
        .map(|k| {
            let (s, c) = sincos(k as f64 / 40.0 * TAU);
            let r = 0.22 * (1.0 + 0.12 * sincos(k as f64 / 40.0 * TAU * 10.0).1);
            DVec2::new(r * c, r * s)
        })
        .collect();
    // A quadratic Bézier rising and curling to the right.
    let (p0, p1, p2) = (
        DVec3::new(0.0, 0.0, 0.0),
        DVec3::new(-0.1, 1.5, 0.0),
        DVec3::new(0.9, 1.9, 0.3),
    );
    let sample = move |t: f64| {
        let k = 1.0 - t;
        let p = p0 * (k * k) + p1 * (2.0 * k * t) + p2 * (t * t);
        let d = (p1 - p0) * (2.0 * k) + (p2 - p1) * (2.0 * t);
        (p, d.normalize())
    };
    og_geom::sweep(
        &og_geom::Sweep {
            section,
            steps: 64,
            scale: vec![
                og_geom::Key { t: 0.0, value: 1.0 },
                og_geom::Key {
                    t: 1.0,
                    value: 0.08,
                },
            ],
            twist: vec![
                og_geom::Key { t: 0.0, value: 0.0 },
                og_geom::Key { t: 1.0, value: 3.0 },
            ],
            caps: true,
        },
        &og_geom::extrude::ExtrudePath {
            sample: &sample,
            closed: false,
        },
    )
    .expect("a horn")
}

/// A vase: a profile from the axis at its foot out to a lip, turned.
pub fn vase() -> og_geom::Mesh {
    use glam::DVec2;
    let profile = [
        (0.0, 0.0),
        (0.3, 0.0),
        (0.34, 0.05),
        (0.42, 0.3),
        (0.46, 0.55),
        (0.4, 0.8),
        (0.26, 1.05),
        (0.2, 1.25),
        (0.24, 1.4),
        (0.3, 1.48),
    ];
    og_geom::lathe(&og_geom::Lathe {
        points: profile.iter().map(|&(x, y)| DVec2::new(x, y)).collect(),
        segments: 48,
        ..Default::default()
    })
}

/// `geometry-shapes`: og-geom's generators in a row on a lit floor: an
/// extruded, bevelled gear with three holes standing on edge, a lathed vase
/// (double-sided: its mouth shows its inside), a swept horn tapering and
/// twisting along a curve, and a dice made by CSG, a cube less 21 black
/// spheres.
pub fn shapes_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    crate::scenes::ground(&mut scene, 40.0, 0.5, 0.85);
    let gear = og_geom::extrude(
        &[gear_shape(14)],
        &og_geom::Extrude {
            depth: 0.22,
            bevel_thickness: 0.05,
            bevel_size: 0.04,
            bevel_segments: 3,
            ..Default::default()
        },
        None,
    );
    put(
        &mut scene,
        from_geom(gear),
        StandardMaterial {
            metalness: 0.7,
            roughness: 0.35,
            ..colour(0.95, 0.55, 0.15)
        },
        Mat4::from_translation(Vec3::new(-3.0, 0.88, 0.0)) * Mat4::from_rotation_y(0.35),
    );
    put(
        &mut scene,
        from_geom(vase()),
        StandardMaterial {
            side: 2,
            roughness: 0.25,
            ..colour(0.2, 0.45, 0.85)
        },
        Mat4::from_translation(Vec3::new(-1.0, 0.0, 0.0)),
    );
    put(
        &mut scene,
        from_geom(horn()),
        colour(0.92, 0.86, 0.72),
        Mat4::from_translation(Vec3::new(0.7, 0.0, 0.0)),
    );
    put(
        &mut scene,
        from_geom(dice()),
        StandardMaterial {
            roughness: 0.4,
            ..colour(0.95, 0.95, 0.93)
        },
        Mat4::from_translation(Vec3::new(3.1, 0.5, 0.0)) * Mat4::from_rotation_y(0.6),
    );
    lit(
        &mut scene,
        Vec3::new(0.0, 3.4, 6.4),
        Vec3::new(0.0, 0.75, 0.0),
    );
    scene
}
