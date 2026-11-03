//! Grading, ambient occlusion, depth of field, god rays and motion blur scenes.

use super::*;

// ---------------------------------------------------------------------------
// The post stack: grading, vignette, chromatic aberration, grain, flash.
// ---------------------------------------------------------------------------

/// A teal-and-orange grade with lifted blacks: shadows towards teal,
/// highlights towards orange, a little more contrast.
pub fn teal_orange() -> ColorLut {
    ColorLut::from_fn(|c| {
        let l = c.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        let shadow = Vec3::new(-0.02, 0.03, 0.06) * (1.0 - l);
        let high = Vec3::new(0.06, 0.02, -0.05) * l;
        let s = |x: f32| x * x * (3.0 - 2.0 * x);
        let graded = c + shadow + high;
        Vec3::new(s(graded.x), s(graded.y), s(graded.z)) * 0.92 + Vec3::splat(0.03)
    })
}

pub fn grade(scene: &mut RenderScene) {
    let lut = scene.add_lut(teal_orange());
    scene.post.color_grading = Some(ColorGrading {
        lut,
        intensity: 1.0,
    });
    scene.post.vignette = Some(Vignette::default());
    scene.post.chromatic_aberration = Some(ChromaticAberration { intensity: 2.5 });
    scene.post.film_grain = Some(FilmGrain::default());
}

/// The teal-orange LUT at 0.6, then colour adjustments baked with it.
pub fn adjust(scene: &mut RenderScene) {
    let lut = scene.add_lut(teal_orange());
    scene.post.color_grading = Some(ColorGrading {
        lut,
        intensity: 0.6,
    });
    scene.post.color_adjust = Some(ColorAdjust {
        temperature: 0.06,
        contrast: 0.15,
        vibrance: 0.25,
        shadows: Vec3::new(-0.04, 0.0, 0.05),
        ..Default::default()
    });
}

// ---------------------------------------------------------------------------
// Ambient occlusion.
// ---------------------------------------------------------------------------

/// Stacked blocks, steps, a sphere on the ground and a wall corner under a
/// bright overcast sky: contact shadows only the ambient occlusion makes.
pub fn ao_scene(method: AoMethod) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 30.0, 1.0, 0.9);
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    let white = scene.add_material(Material {
        base_color: [0.8, 0.78, 0.74, 1.0],
        roughness: 0.85,
        ..Default::default()
    });
    let place = |s: &mut RenderScene, m, p: Vec3, size: Vec3| {
        s.add_instance(m, white, Mat4::from_translation(p) * Mat4::from_scale(size));
    };
    // A wall corner.
    place(
        &mut scene,
        cube,
        Vec3::new(0.0, 1.5, -3.0),
        Vec3::new(8.0, 3.0, 0.4),
    );
    place(
        &mut scene,
        cube,
        Vec3::new(-3.8, 1.5, -1.0),
        Vec3::new(0.4, 3.0, 4.0),
    );
    // Steps.
    for i in 0..4 {
        let h = 0.25 * (i + 1) as f32;
        place(
            &mut scene,
            cube,
            Vec3::new(2.2, h / 2.0, -2.4 + i as f32 * 0.0)
                + Vec3::new(0.0, 0.0, 0.5 * i as f32 - 0.5),
            Vec3::new(2.0, h, 0.5),
        );
    }
    // Blocks stacked and leaning.
    place(
        &mut scene,
        cube,
        Vec3::new(-1.6, 0.4, -0.6),
        Vec3::splat(0.8),
    );
    place(
        &mut scene,
        cube,
        Vec3::new(-1.5, 1.1, -0.6),
        Vec3::splat(0.6),
    );
    place(
        &mut scene,
        sphere,
        Vec3::new(0.3, 0.5, 0.2),
        Vec3::splat(1.0),
    );
    place(
        &mut scene,
        sphere,
        Vec3::new(-0.6, 0.25, 1.0),
        Vec3::splat(0.5),
    );
    let sun = Vec3::new(0.3, 0.9, 0.5);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 0.6,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.98, 0.95),
            look: None,
        },
        intensity: 1.0,
        background: Background::Sky,
        ..Environment::default()
    };
    scene.post.ao = Some(AmbientOcclusion {
        radius: 0.7,
        intensity: 1.2,
        method,
        ..Default::default()
    });
    scene.camera = Camera {
        position: Vec3::new(1.0, 2.6, 5.0),
        target: Vec3::new(-0.2, 0.6, -1.2),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// Screen-space global illumination.
// ---------------------------------------------------------------------------

/// An open-topped room under a low sun: a red wall on the left, a green one
/// on the right, a white back wall and floor, a white block and sphere in
/// the middle, under a dim sky (the ambient light the bounce stands out
/// against). With `gi` the sunlit floor lights the walls and the block, and
/// the coloured walls tint the floor and the white faces beside them.
pub fn gi_scene(gi: bool) -> RenderScene {
    let mut scene = RenderScene::new();
    ground(&mut scene, 30.0, 1.0, 0.9);
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    let matte = |s: &mut RenderScene, c: [f32; 3]| {
        s.add_material(Material {
            base_color: [c[0], c[1], c[2], 1.0],
            roughness: 0.9,
            ..Default::default()
        })
    };
    let white = matte(&mut scene, [0.8, 0.8, 0.8]);
    let red = matte(&mut scene, [0.8, 0.08, 0.06]);
    let green = matte(&mut scene, [0.08, 0.7, 0.1]);
    let place = |s: &mut RenderScene, m, mat, p: Vec3, size: Vec3| {
        s.add_instance(m, mat, Mat4::from_translation(p) * Mat4::from_scale(size));
    };
    place(
        &mut scene,
        cube,
        white,
        Vec3::new(0.0, 1.5, -2.5),
        Vec3::new(6.0, 3.0, 0.2),
    );
    place(
        &mut scene,
        cube,
        red,
        Vec3::new(-2.9, 1.5, 0.0),
        Vec3::new(0.2, 3.0, 5.0),
    );
    place(
        &mut scene,
        cube,
        green,
        Vec3::new(2.9, 1.5, 0.0),
        Vec3::new(0.2, 3.0, 5.0),
    );
    place(
        &mut scene,
        cube,
        white,
        Vec3::new(-1.0, 0.6, -0.8),
        Vec3::new(1.0, 1.2, 1.0),
    );
    place(
        &mut scene,
        sphere,
        white,
        Vec3::new(1.1, 0.5, 0.2),
        Vec3::splat(1.0),
    );
    // High and from the front right: the floor and the red wall in sun, the
    // green wall's inside in shade.
    let sun = Vec3::new(0.45, 0.8, 0.4);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.97, 0.92),
            look: None,
        },
        intensity: 0.25,
        background: Background::Sky,
        ..Environment::default()
    };
    scene.post.gi = gi.then(og_render::ScreenSpaceGlobalIllumination::default);
    scene.camera = Camera {
        position: Vec3::new(0.0, 2.2, 6.0),
        target: Vec3::new(0.0, 0.9, -1.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// Realtime light volumes.
// ---------------------------------------------------------------------------

/// A closed room lit by one lamp under the middle of its ceiling: the
/// left wall red, the right green, the rest white, a white block and a
/// sphere on the floor, almost no environment light (a black background,
/// intensity 0.05), so whatever lights the shade is the lamp's bounce, and
/// ambient occlusion in its corners and under the block and sphere. With
/// `realtime` a realtime light volume fills the room (5 × 3 × 5 probes)
/// and is relit 8 probes a frame: the shade takes the lamp's light off the
/// walls, pink on the left, mint on the right, and the lamp's `colour`.
/// The probes' box ends 0.3 short of the walls, floor and ceiling, which
/// take its light from just outside it. With `mirror` a chrome sphere of
/// that roughness stands in the middle of the frame under a realtime
/// reflection probe over the room: it reflects the room as the probe
/// captures it, lit by the probes when they are there (red on its left,
/// green on its right), by the lamp alone when not, blurring into the
/// probes' own soft light as the roughness rises past 0.4.
pub fn light_volume_scene(realtime: bool, colour: Vec3, mirror: Option<f32>) -> RenderScene {
    let mut scene = RenderScene::new();
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    let matte = |s: &mut RenderScene, c: [f32; 3]| {
        s.add_material(Material {
            base_color: [c[0], c[1], c[2], 1.0],
            roughness: 0.9,
            ..Default::default()
        })
    };
    let white = matte(&mut scene, [0.8, 0.8, 0.8]);
    let red = matte(&mut scene, [0.8, 0.08, 0.06]);
    let green = matte(&mut scene, [0.08, 0.7, 0.1]);
    let boxes = [
        // Floor, ceiling, back, front (behind the camera), left, right.
        (white, Vec3::new(0.0, -0.1, 0.0), Vec3::new(6.4, 0.2, 6.4)),
        (white, Vec3::new(0.0, 3.1, 0.0), Vec3::new(6.4, 0.2, 6.4)),
        (white, Vec3::new(0.0, 1.5, -3.1), Vec3::new(6.4, 3.4, 0.2)),
        (white, Vec3::new(0.0, 1.5, 3.1), Vec3::new(6.4, 3.4, 0.2)),
        (red, Vec3::new(-3.1, 1.5, 0.0), Vec3::new(0.2, 3.4, 6.4)),
        (green, Vec3::new(3.1, 1.5, 0.0), Vec3::new(0.2, 3.4, 6.4)),
        (white, Vec3::new(-0.6, 0.6, -0.9), Vec3::new(1.0, 1.2, 1.0)),
    ];
    for (mat, p, size) in boxes {
        scene.add_instance(
            cube,
            mat,
            Mat4::from_translation(p) * Mat4::from_scale(size),
        );
    }
    scene.add_instance(
        sphere,
        white,
        Mat4::from_translation(Vec3::new(1.2, 0.5, 0.3)),
    );
    if let Some(roughness) = mirror {
        let chrome = scene.add_material(Material {
            base_color: [0.95, 0.95, 0.95, 1.0],
            metallic: 1.0,
            roughness,
            ..Default::default()
        });
        // On the camera's axis, clear of the block and the other sphere.
        scene.add_instance(
            sphere,
            chrome,
            Mat4::from_translation(Vec3::new(0.0, 1.215, 0.39))
                * Mat4::from_scale(Vec3::splat(0.8)),
        );
        // Over the room, its capture point above the sphere (the box's
        // ceiling 0.8 above the room's: a little parallax upwards).
        let mut probe =
            og_render::ReflectionProbe::new(2, Vec3::new(0.0, 1.9, 0.0), Vec3::new(6.0, 3.8, 6.0));
        probe.mode = og_render::ProbeMode::Realtime;
        // 256² a face: the sphere, about 200 px across, shows the room
        // at about its own pixels' sharpness.
        probe.resolution = 256;
        probe.interval = 8;
        probe.blend_distance = 0.3;
        scene.reflection_probes.push(probe);
    }
    scene.light = DirectionalLight {
        intensity: 0.0,
        ..Default::default()
    };
    scene.lights.push(Light {
        kind: LightKind::Point,
        position: Vec3::new(0.0, 2.7, -0.4),
        color: colour,
        intensity: 14.0,
        range: 9.0,
        cast_shadows: true,
    });
    scene.environment = Environment {
        intensity: 0.05,
        background: Background::Color(Vec3::ZERO),
        ..Default::default()
    };
    // Short-range: corners and contact darken, the open walls (unoccluded)
    // keep their bounce and their lamp light. Most of this room's light is
    // the lamp's, so the occlusion reaches a little of it too (at most a
    // fifth, where fully occluded), or the contact would not read.
    scene.post.ao = Some(AmbientOcclusion {
        radius: 0.5,
        intensity: 1.0,
        method: AoMethod::Gtao,
        direct: 0.3,
        ..Default::default()
    });
    if realtime {
        scene.light_volumes.push(og_render::RealtimeLightVolume {
            id: 1,
            min: Vec3::new(-2.7, 0.15, -2.7),
            max: Vec3::new(2.7, 2.75, 2.7),
            spacing: 1.35,
            updates_per_frame: 8,
            intensity: 1.0,
        });
    }
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.6, 2.7),
        target: Vec3::new(0.0, 0.9, -1.5),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// Depth of field.
// ---------------------------------------------------------------------------

/// A row of tricksters receding into the dark with small bright lamps
/// behind (bokeh) and one in front; focus on the middle figure.
pub fn dof_scene(quality: DofQuality) -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    ground(&mut scene, 60.0, 0.5, 0.7);
    let model = load("trickster.glb")?;
    let parts = model.add_to(&mut scene);
    let scale = 1.8 / (model.max - model.min).max_element().max(1e-3);
    for i in 0..6 {
        let z = 2.0 - i as f32 * 2.6;
        let x = -2.0 + i as f32 * 0.8;
        parts.instance(
            &mut scene,
            Mat4::from_translation(Vec3::new(x, -model.min.y * scale, z))
                * Mat4::from_rotation_y(0.3 - i as f32 * 0.15)
                * Mat4::from_scale(Vec3::splat(scale)),
        );
    }
    let lamp = scene.add_mesh(shapes::sphere(12, 8));
    for i in 0..18 {
        let c = [[8.0, 5.0, 2.0], [2.0, 5.0, 9.0], [7.0, 2.0, 6.0]][i % 3];
        let m = scene.add_material(Material {
            base_color: [0.0, 0.0, 0.0, 1.0],
            emissive: c,
            ..Default::default()
        });
        let x = -6.0 + (i as f32 * 0.71) % 12.0;
        let y = 0.6 + (i as f32 * 1.37) % 2.8;
        scene.add_instance(
            lamp,
            m,
            Mat4::from_translation(Vec3::new(x, y, -16.0 - (i % 4) as f32 * 2.0))
                * Mat4::from_scale(Vec3::splat(0.12)),
        );
    }
    let m = scene.add_material(Material {
        base_color: [0.0, 0.0, 0.0, 1.0],
        emissive: [9.0, 6.0, 2.0],
        ..Default::default()
    });
    scene.add_instance(
        lamp,
        m,
        Mat4::from_translation(Vec3::new(1.2, 1.4, 5.2)) * Mat4::from_scale(Vec3::splat(0.15)),
    );
    let sun = Vec3::new(-0.4, 0.5, 0.6);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.8, 0.6),
        intensity: 1.6,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.7, 0.45),
            look: None,
        },
        intensity: 0.25,
        background: Background::Color(Vec3::new(0.01, 0.012, 0.02)),
        ..Environment::default()
    };
    scene.post.bloom = Some(Bloom::default());
    scene.post.depth_of_field = Some(DepthOfField {
        focus_distance: 10.2,
        focus_range: 1.2,
        max_blur: 0.014,
        falloff: 3.0,
        quality,
    });
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.5, 7.0),
        target: Vec3::new(0.0, 1.1, -4.0),
        exposure: Exposure::linear(1.1),
        ..Default::default()
    };
    Ok(scene)
}

// ---------------------------------------------------------------------------
// God rays.
// ---------------------------------------------------------------------------

/// A colonnade against a low sun in view: shafts of light between the columns.
pub fn god_rays_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 120.0);
    let column = scene.add_mesh(shapes::cuboid(Vec3::new(0.7, 7.0, 0.7)));
    let beam = scene.add_mesh(shapes::cuboid(Vec3::new(24.0, 0.8, 1.0)));
    let stone = scene.add_material(Material {
        base_color: [0.7, 0.66, 0.6, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    for i in 0..13 {
        let x = -10.8 + i as f32 * 1.8;
        scene.add_instance(
            column,
            stone,
            Mat4::from_translation(Vec3::new(x, 3.5, -12.0)),
        );
    }
    scene.add_instance(
        beam,
        stone,
        Mat4::from_translation(Vec3::new(0.0, 7.4, -12.0)),
    );
    let sun = Vec3::new(0.1, 0.16, -1.0);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.8, 0.55),
        intensity: 2.5,
        ..Default::default()
    };
    scene.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.75, 0.45),
            look: None,
        },
        intensity: 0.4,
        background: Background::Sky,
        ..Environment::default()
    };
    scene.post.god_rays = Some(GodRays {
        intensity: 0.7,
        ..Default::default()
    });
    scene.post.bloom = Some(Bloom::default());
    scene.camera = Camera {
        position: Vec3::new(0.0, 1.8, 6.0),
        target: Vec3::new(0.3, 3.2, -12.0),
        exposure: Exposure::linear(0.7),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// Motion blur.
// ---------------------------------------------------------------------------

fn spinner(frame: u32) -> Mat4 {
    Mat4::from_translation(Vec3::new(-1.6, 1.2, 0.0))
        * Mat4::from_rotation_z(frame as f32 * 0.35)
        * Mat4::from_scale(Vec3::new(2.4, 0.25, 0.25))
}

fn racer(frame: u32) -> Mat4 {
    Mat4::from_translation(Vec3::new(-3.0 + frame as f32 * 0.9, 0.5, 1.4))
}

/// A spinning bar, a ball racing across and the camera panning: per-object
/// and camera motion blur.
pub fn motion_blur_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    terrain(&mut scene, 60.0);
    let cube = scene.add_mesh(shapes::cuboid(Vec3::ONE));
    let ball = scene.add_mesh(shapes::sphere(32, 24));
    let red = scene.add_material(Material {
        base_color: [0.85, 0.2, 0.15, 1.0],
        roughness: 0.4,
        ..Default::default()
    });
    let yellow = scene.add_material(Material {
        base_color: [0.95, 0.8, 0.2, 1.0],
        roughness: 0.3,
        ..Default::default()
    });
    scene.add_instance(cube, red, spinner(0));
    scene.add_instance(ball, yellow, racer(0));
    for i in 0..6 {
        scene.add_instance(
            cube,
            red,
            Mat4::from_translation(Vec3::new(-5.0 + i as f32 * 2.0, 0.5, -3.0)),
        );
    }
    scene.post.motion_blur = Some(MotionBlur::default());
    scene
}

fn motion_camera(frame: u32) -> Camera {
    Camera {
        position: Vec3::new(frame as f32 * 0.25, 2.4, 6.0),
        target: Vec3::new(frame as f32 * 0.25, 0.8, -1.0),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    }
}

/// Moves the spinner, the racer and the camera; records the previous
/// transforms for per-object motion vectors.
pub fn move_blur(scene: &mut RenderScene, frame: u32) {
    let prev = frame.saturating_sub(1);
    scene.previous_transforms = scene.instances.iter().map(|i| i.transform).collect();
    // Instance 0 is the ground.
    scene.previous_transforms[1] = spinner(prev);
    scene.previous_transforms[2] = racer(prev);
    scene.instances[1].transform = spinner(frame);
    scene.instances[2].transform = racer(frame);
    scene.camera = motion_camera(frame);
    tick_all(scene, frame as u64);
}
