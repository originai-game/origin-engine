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
