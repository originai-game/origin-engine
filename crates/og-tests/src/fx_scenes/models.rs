//! Stylised materials, skinned models and per-instance looks.

use super::*;

// ---------------------------------------------------------------------------
// The stylised look: toon, outlines, rim light, dissolve, flash.
// ---------------------------------------------------------------------------

fn torus(major: f32, minor: f32, segments: u32, sides: u32) -> MeshData {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for i in 0..=segments {
        let u = i as f32 / segments as f32 * std::f32::consts::TAU;
        for j in 0..=sides {
            let v = j as f32 / sides as f32 * std::f32::consts::TAU;
            let n = Vec3::new(u.cos() * v.cos(), v.sin(), u.sin() * v.cos());
            let c = Vec3::new(u.cos(), 0.0, u.sin()) * major;
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: (c + n * minor).to_array(),
                normal: n.to_array(),
                uv: [i as f32 / segments as f32, j as f32 / sides as f32],
                color: [1.0; 4],
            });
        }
    }
    for i in 0..segments {
        for j in 0..sides {
            let a = i * (sides + 1) + j;
            let b = a + sides + 1;
            indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    MeshData { vertices, indices }
}

fn ramp_texture(scene: &mut RenderScene) -> TextureHandle {
    // Four hard bands, cool shadow to warm light.
    let rgba: Vec<u8> = (0..8)
        .flat_map(|i| {
            let v = [40u8, 40, 110, 110, 190, 190, 255, 255][i];
            [v, v, v, 255]
        })
        .collect();
    scene.add_texture(TextureData {
        width: 8,
        height: 1,
        rgba,
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    })
}

/// A cel-shaded toy box: toon spheres and a torus with outlines and rim
/// light, a trickster dissolving with a glowing edge, one hit-flashed.
pub fn stylised_scene() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let ramp = ramp_texture(&mut scene);
    let floor = scene.add_mesh(shapes::plane(40.0, 1.0));
    let floor_m = scene.add_standard_material(StandardMaterial {
        color: Vec3::new(0.55, 0.75, 0.55),
        toon: 1,
        gradientMap: ramp.slot(),
        ..Default::default()
    });
    scene.add_instance(floor, floor_m, Mat4::IDENTITY);
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    let ring = scene.add_mesh(torus(0.7, 0.25, 64, 24));
    let toon = |color: Vec3, outline: f32| StandardMaterial {
        color,
        roughness: 0.4,
        toon: 1,
        gradientMap: ramp.slot(),
        rimColor: Vec3::new(1.0, 0.95, 0.9),
        rimIntensity: 0.6,
        rimPower: 3.5,
        outlineColor: Vec3::new(0.05, 0.04, 0.08),
        outlineWidth: outline,
        ..Default::default()
    };
    let red = scene.add_standard_material(toon(Vec3::new(0.9, 0.25, 0.2), 3.0));
    let blue = scene.add_standard_material(toon(Vec3::new(0.2, 0.45, 0.95), 2.0));
    let gold = scene.add_standard_material(StandardMaterial {
        metalness: 0.0,
        ..toon(Vec3::new(1.0, 0.75, 0.2), 4.0)
    });
    let flashed = scene.add_standard_material(StandardMaterial {
        flash: 0.65,
        flashColor: Vec3::new(1.0, 1.0, 1.0),
        ..toon(Vec3::new(0.3, 0.8, 0.4), 3.0)
    });
    scene.add_instance(
        sphere,
        red,
        Mat4::from_translation(Vec3::new(-2.0, 0.6, 0.0)) * Mat4::from_scale(Vec3::splat(1.2)),
    );
    scene.add_instance(
        sphere,
        blue,
        Mat4::from_translation(Vec3::new(2.1, 0.5, 0.6)),
    );
    scene.add_instance(
        sphere,
        flashed,
        Mat4::from_translation(Vec3::new(0.9, 0.35, 2.0)) * Mat4::from_scale(Vec3::splat(0.7)),
    );
    scene.add_instance(
        ring,
        gold,
        Mat4::from_translation(Vec3::new(0.0, 1.0, -1.2)) * Mat4::from_rotation_x(1.1),
    );
    // A trickster half dissolved.
    let model = load("trickster.glb")?;
    let parts = model.add_to(&mut scene);
    for &(_, mat) in &parts.parts {
        let mut m = *scene.standard_material(mat).expect("a material");
        m.dissolve = 0.45;
        m.dissolveScale = 6.0;
        m.dissolveEdgeColor = Vec3::new(1.0, 3.0, 6.0);
        m.dissolveEdgeWidth = 0.08;
        m.rimIntensity = 0.4;
        m.outlineWidth = 2.0;
        m.outlineColor = Vec3::new(0.05, 0.04, 0.08);
        scene.set_standard_material(mat, m);
    }
    let scale = 2.2 / (model.max - model.min).max_element().max(1e-3);
    parts.instance(
        &mut scene,
        Mat4::from_translation(Vec3::new(-0.4, -model.min.y * scale, 0.6))
            * Mat4::from_rotation_y(crate::scenes::TRICKSTER_TO_Z + 0.5)
            * Mat4::from_scale(Vec3::splat(scale)),
    );
    let sun = Vec3::new(0.5, 0.8, 0.4);
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 2.6,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::ONE,
            look: None,
        },
        intensity: 0.3,
        background: Background::Color(Vec3::new(0.35, 0.55, 0.85)),
        ..Environment::default()
    };
    scene.look.post.outline = Some(Outline {
        width: 1.5,
        ..Default::default()
    });
    scene.look.post.bloom = Some(Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.2, 7.0),
        target: Vec3::new(0.0, 0.7, 0.0),
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    Ok(scene)
}

// ---------------------------------------------------------------------------
// GPU skinning.
// ---------------------------------------------------------------------------

/// Bones of the skinned cylinder.
pub const CYLINDER_BONES: usize = 4;
/// Its height (bones split it evenly along +y).
pub const CYLINDER_HEIGHT: f32 = 2.4;

/// A capped cylinder along +y with `rings` rings, skinned to
/// [`CYLINDER_BONES`] bones: each vertex blends the two bones nearest its
/// height.
pub fn skinned_cylinder(radius: f32, rings: u32, sides: u32) -> (MeshData, SkinData) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let seg = CYLINDER_HEIGHT / CYLINDER_BONES as f32;
    let skin_at = |y: f32| {
        let f = (y / seg - 0.5).clamp(0.0, (CYLINDER_BONES - 1) as f32);
        let a = (f.floor() as u16).min(CYLINDER_BONES as u16 - 1);
        let b = (a + 1).min(CYLINDER_BONES as u16 - 1);
        let w = f - a as f32;
        ([a, b, 0, 0], [1.0 - w, w, 0.0, 0.0])
    };
    for r in 0..=rings {
        let y = r as f32 / rings as f32 * CYLINDER_HEIGHT;
        for s in 0..=sides {
            let a = s as f32 / sides as f32 * std::f32::consts::TAU;
            let n = Vec3::new(a.cos(), 0.0, a.sin());
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: [n.x * radius, y, n.z * radius],
                normal: n.to_array(),
                uv: [s as f32 / sides as f32, 1.0 - r as f32 / rings as f32],
                color: [1.0; 4],
            });
            let (j, w) = skin_at(y);
            joints.push(j);
            weights.push(w);
        }
    }
    for r in 0..rings {
        for s in 0..sides {
            let a = r * (sides + 1) + s;
            let b = a + sides + 1;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    // Caps.
    for (y, up) in [(0.0f32, -1.0f32), (CYLINDER_HEIGHT, 1.0)] {
        let center = vertices.len() as u32;
        vertices.push(Vertex {
            tangent: [0.0; 4],
            position: [0.0, y, 0.0],
            normal: [0.0, up, 0.0],
            uv: [0.5, 0.5],
            color: [1.0; 4],
        });
        let (j, w) = skin_at(y);
        joints.push(j);
        weights.push(w);
        for s in 0..=sides {
            let a = s as f32 / sides as f32 * std::f32::consts::TAU;
            vertices.push(Vertex {
                tangent: [0.0; 4],
                position: [a.cos() * radius, y, a.sin() * radius],
                normal: [0.0, up, 0.0],
                uv: [0.5 + a.cos() * 0.5, 0.5 + a.sin() * 0.5],
                color: [1.0; 4],
            });
            joints.push(j);
            weights.push(w);
        }
        for s in 0..sides {
            let (p, q) = (center + 1 + s, center + 2 + s);
            if up > 0.0 {
                indices.extend_from_slice(&[center, q, p]);
            } else {
                indices.extend_from_slice(&[center, p, q]);
            }
        }
    }
    (
        vertices_mesh(vertices, indices),
        SkinData { joints, weights },
    )
}

fn vertices_mesh(vertices: Vec<Vertex>, indices: Vec<u32>) -> MeshData {
    MeshData { vertices, indices }
}

/// The palette that bends the cylinder: each bone turns by `bend` radians
/// about z (and `twist` about y) relative to its parent, at its base.
pub fn bend_palette(bend: f32, twist: f32) -> JointPalette {
    let seg = CYLINDER_HEIGHT / CYLINDER_BONES as f32;
    let mut matrices = Vec::with_capacity(CYLINDER_BONES);
    let mut global = Mat4::IDENTITY;
    for b in 0..CYLINDER_BONES {
        // Bone b's rest frame sits at height b * seg.
        let local = if b == 0 {
            Mat4::from_rotation_y(twist)
        } else {
            Mat4::from_translation(Vec3::Y * seg)
                * Mat4::from_rotation_z(bend)
                * Mat4::from_rotation_y(twist)
        };
        global *= local;
        let inverse_bind = Mat4::from_translation(-Vec3::Y * seg * b as f32);
        matrices.push(global * inverse_bind);
    }
    JointPalette { matrices }
}

/// Five skinned cylinders (tentacles) on one mesh and material, each posed
/// by its own palette, casting shadows.
pub fn skinning_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 40.0);
    let (mesh, skin) = skinned_cylinder(0.22, 24, 24);
    let cylinder = scene.add_skinned_mesh(mesh, skin);
    let flesh = scene.add_material(Material {
        base_color: [0.75, 0.3, 0.55, 1.0],
        roughness: 0.45,
        ..Default::default()
    });
    for i in 0..5 {
        let bend = -0.45 + i as f32 * 0.22;
        scene
            .joint_palettes
            .push(bend_palette(bend, i as f32 * 0.2));
        scene.skinned_instances.push(SkinnedInstance::new(
            cylinder,
            flesh,
            Mat4::from_translation(Vec3::new(-3.0 + i as f32 * 1.5, 0.0, 0.0)),
            i,
        ));
    }
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.6, 7.0),
        target: Vec3::new(0.0, 1.2, 0.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

/// Skinned culling (og-render's cull_gpu.rs `SkinnedCull`): a crowd of
/// 7 × 5 cylinders playing one baked clip at their own frames, its back
/// three rows behind a grey wall, and in front a posed cylinder whose
/// palette carries it 12 m from where its bind pose stands (out of view
/// there): culled by its posed box, it is drawn. The wall is 30 m wide and
/// 10 m tall, past the frame's top and sides: the occlusion test reads up
/// to 2 × 2 texels of the depth pyramid around a box's rectangle (texels
/// as large as the rectangle), so a wall whose top edge ran just above the
/// crowd would leave sky in them and hide nothing. In the frame: the grey
/// wall filling everything above the ground, the two front rows of
/// bending pink cylinders before it, and the moved cylinder, orange, at
/// the centre in front.
pub fn crowd_culling_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 40.0);
    let (mesh, skin) = skinned_cylinder(0.3, 24, 24);
    let h = scene.add_skinned_mesh(mesh, skin);
    let pink = scene.add_material(Material {
        base_color: [0.8, 0.3, 0.5, 1.0],
        roughness: 0.5,
        ..Default::default()
    });
    let orange = scene.add_material(Material {
        base_color: [0.95, 0.5, 0.1, 1.0],
        roughness: 0.5,
        ..Default::default()
    });
    let wall = scene.add_mesh(og_render::shapes::cuboid(Vec3::new(30.0, 10.0, 0.4)));
    let grey = scene.add_material(Material {
        base_color: [0.5, 0.5, 0.52, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    scene.add_instance(
        wall,
        grey,
        Mat4::from_translation(Vec3::new(0.0, 5.0, -2.6)),
    );
    scene.baked_clips.push(og_render::BakedClip {
        joints: CYLINDER_BONES as u32,
        frames: [0.0, 0.3, 0.6]
            .iter()
            .flat_map(|&b| bend_palette(b, 0.0).matrices)
            .collect(),
    });
    for z in 0..5 {
        for x in 0..7 {
            let k = z * 7 + x;
            let f = (k as f32 * 0.37) % 2.0;
            let at =
                Mat4::from_translation(Vec3::new((x as f32 - 3.0) * 1.4, 0.0, -(z as f32) * 1.6));
            let mut inst = SkinnedInstance::new(h, pink, at, 0);
            inst.baked = Some(og_render::BakedPose {
                clip: 0,
                frame: f,
                previous_clip: 0,
                previous_frame: f,
                previous_transform: at,
            });
            scene.skinned_instances.push(inst);
        }
    }
    scene.joint_palettes.push(JointPalette::from(vec![
        Mat4::from_translation(Vec3::new(
            12.0, 0.0, 0.0
        ));
        CYLINDER_BONES
    ]));
    scene.skinned_instances.push(SkinnedInstance::new(
        h,
        orange,
        Mat4::from_translation(Vec3::new(-12.0, 0.0, 2.2)),
        0,
    ));
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.6, 8.0),
        target: Vec3::new(0.0, 1.0, -2.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

/// Per-instance looks (kestrel-stair #11): three instances of one glTF
/// model, its materials shared, the middle one flashing white and the right
/// one tinted amber; in front, two skinned cylinders of one mesh and
/// material, the right one flashing red. Each look rides in its instance's
/// record (`InstanceFx`): one draw per (mesh, material), whatever each
/// instance shows.
pub fn model_flash_scene() -> Result<RenderScene, String> {
    let model = load("trickster.glb")?;
    let mut scene = RenderScene::new();
    terrain(&mut scene, 40.0);
    let parts = model.add_to(&mut scene);
    let scale = 2.0 / (model.max - model.min).max_element().max(1e-3);
    let looks = [
        InstanceFx::NONE,
        InstanceFx {
            flash: 0.6,
            ..InstanceFx::NONE
        },
        InstanceFx {
            tint: Vec3::new(1.0, 0.6, 0.25),
            ..InstanceFx::NONE
        },
    ];
    for (i, fx) in looks.into_iter().enumerate() {
        let m =
            Mat4::from_translation(Vec3::new(-2.4 + i as f32 * 2.4, -model.min.y * scale, -0.6))
                * Mat4::from_rotation_y(crate::scenes::TRICKSTER_TO_Z + 0.5)
                * Mat4::from_scale(Vec3::splat(scale));
        for &(mesh, material) in &parts.parts {
            scene.instances.push(Instance {
                mesh,
                material,
                transform: m,
                material_layer: None,
                fx,
                cast_shadows: true,
                receive_shadows: true,
                sway: 0.0,
            });
        }
    }
    let (mesh, skin) = skinned_cylinder(0.18, 24, 24);
    let cylinder = scene.add_skinned_mesh(mesh, skin);
    let flesh = scene.add_material(Material {
        base_color: [0.75, 0.3, 0.55, 1.0],
        roughness: 0.45,
        ..Default::default()
    });
    for i in 0..2u32 {
        scene
            .joint_palettes
            .push(bend_palette(-0.2 + i as f32 * 0.4, 0.3));
        let mut inst = SkinnedInstance::new(
            cylinder,
            flesh,
            Mat4::from_translation(Vec3::new(-1.0 + i as f32 * 2.0, 0.0, 1.6))
                * Mat4::from_scale(Vec3::splat(0.6)),
            i,
        );
        if i == 1 {
            inst.fx = InstanceFx {
                flash: 0.5,
                flash_color: Vec3::new(1.0, 0.1, 0.05),
                ..InstanceFx::NONE
            };
        }
        scene.skinned_instances.push(inst);
    }
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.4, 6.4),
        target: Vec3::new(0.0, 1.0, 0.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    Ok(scene)
}
