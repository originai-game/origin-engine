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
    scene.look.post.color_grading = Some(ColorGrading {
        lut,
        intensity: 1.0,
    });
    scene.look.post.vignette = Some(Vignette::default());
    scene.look.post.chromatic_aberration = Some(ChromaticAberration { intensity: 2.5 });
    scene.look.post.film_grain = Some(FilmGrain::default());
}

/// The teal-orange LUT at 0.6, then colour adjustments baked with it.
pub fn adjust(scene: &mut RenderScene) {
    let lut = scene.add_lut(teal_orange());
    scene.look.post.color_grading = Some(ColorGrading {
        lut,
        intensity: 0.6,
    });
    scene.look.post.color_adjust = Some(ColorAdjust {
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

/// A corner of a rubble-walled yard under an overcast sky: crates stacked
/// in the corner, oak barrels, stone steps up to a door, a ladder and a
/// broken column stump: contact shadows only the ambient occlusion makes.
pub fn ao_scene(method: AoMethod) -> RenderScene {
    let mut scene = RenderScene::new();
    // Pale limestone setts, their joints pointed with lime: the setts' own
    // relief and roughness under an even colour, so the darkening at the
    // props' feet is the occlusion and not the stones' mottle.
    let [_, setts_normal, setts_orm] = crate::kit::surface_textures(&mut scene, "cobbles");
    let flags = scene.add_material(Material {
        base_color: [0.4, 0.37, 0.32, 1.0],
        normal_texture: Some(setts_normal),
        metallic_roughness_texture: Some(setts_orm),
        roughness: 1.0,
        ..Default::default()
    });
    let yard = scene.add_mesh(crate::kit::slab(
        Vec2::splat(-40.0),
        Vec2::splat(40.0),
        0.0,
        2.0,
    ));
    scene.add_instance(yard, flags, Mat4::IDENTITY);
    let grass = crate::kit::surface_tinted(&mut scene, "grass", [0.86, 1.0, 0.72]);
    for (min, max) in [
        (Vec2::new(-260.0, -260.0), Vec2::new(-40.0, 260.0)),
        (Vec2::new(40.0, -260.0), Vec2::new(260.0, 260.0)),
        (Vec2::new(-40.0, -260.0), Vec2::new(40.0, -40.0)),
        (Vec2::new(-40.0, 40.0), Vec2::new(40.0, 260.0)),
    ] {
        let m = scene.add_mesh(crate::kit::slab(min, max, 0.0, 7.0));
        scene.add_instance(m, grass, Mat4::IDENTITY);
    }
    crate::kit::hills(&mut scene, None);
    let stone = crate::kit::surface_tinted(&mut scene, "stone", [0.95, 0.9, 0.84]);
    let limewash = crate::kit::plaster(&mut scene, [0.5, 0.47, 0.41]);
    let block = |s: &mut RenderScene, min: Vec3, max: Vec3, material| {
        let m = s.add_mesh(crate::kit::block(min, max, 2.4));
        s.add_instance(m, material, Mat4::IDENTITY);
    };
    // A corner of a limewashed yard: the back wall and the left wall, a
    // buttress where they meet, a stone coping along their tops.
    block(
        &mut scene,
        Vec3::new(-4.0, 0.0, -3.2),
        Vec3::new(4.5, 4.0, -2.8),
        limewash,
    );
    block(
        &mut scene,
        Vec3::new(-4.0, 0.0, -2.8),
        Vec3::new(-3.6, 4.0, 1.4),
        limewash,
    );
    block(
        &mut scene,
        Vec3::new(-4.1, 4.0, -3.3),
        Vec3::new(4.6, 4.18, -2.7),
        stone,
    );
    block(
        &mut scene,
        Vec3::new(-3.6, 0.0, -2.8),
        Vec3::new(-3.0, 1.6, -2.2),
        limewash,
    );
    // Stone steps up to a door in the back wall, on the right.
    for i in 0..4 {
        let h = 0.25 * (4 - i) as f32;
        let z = -2.8 + 0.4 * i as f32;
        block(
            &mut scene,
            Vec3::new(1.2, 0.0, z),
            Vec3::new(3.2, h, z + 0.4),
            stone,
        );
    }
    let door = crate::kit::surface_tinted(&mut scene, "planks", [0.7, 0.6, 0.5]);
    let leaf = scene.add_mesh(crate::kit::block(
        Vec3::new(1.6, 1.0, -2.84),
        Vec3::new(2.8, 2.9, -2.76),
        1.2,
    ));
    scene.add_instance(leaf, door, Mat4::IDENTITY);
    // The yard's clutter pushed tight into the walls and their creases:
    // crates stacked against the buttress, one along the left wall,
    // barrels in the angle of the steps, a boulder against the left wall,
    // a broken column lying on the setts.
    let crate_ = crate::kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-2.25, 0.0, -2.4), 0.0, 0.8);
    crate_.place(&mut scene, Vec3::new(-2.2, 0.8, -2.42), -0.08, 0.8);
    crate_.place(
        &mut scene,
        Vec3::new(-3.28, 0.0, -0.9),
        std::f32::consts::FRAC_PI_2,
        0.65,
    );
    crate_.place(&mut scene, Vec3::new(-1.2, 0.0, -2.48), 0.05, 0.6);
    let barrel = crate::kit::prop(&mut scene, "winebarrel");
    barrel.place(&mut scene, Vec3::new(0.15, 0.0, -2.42), 0.4, 1.05);
    barrel.place(&mut scene, Vec3::new(0.84, 0.0, -2.42), 1.7, 1.05);
    barrel.place(&mut scene, Vec3::new(0.5, 1.05, -2.45), 0.9, 1.05);
    // Two boards leant on the back wall, a broken column stump by the steps.
    let board = scene.add_mesh(crate::kit::block(
        Vec3::new(-0.16, 0.0, -0.025),
        Vec3::new(0.16, 2.7, 0.025),
        1.2,
    ));
    for (x, turn) in [(-0.6, 0.04), (-0.3, -0.03)] {
        scene.add_instance(
            board,
            door,
            Mat4::from_translation(Vec3::new(x, 0.0, -2.3))
                * Mat4::from_rotation_y(turn)
                * Mat4::from_rotation_x(-0.2),
        );
    }
    crate::kit::prop(&mut scene, "column_broken").place(
        &mut scene,
        Vec3::new(0.6, 0.0, 0.2),
        0.8,
        1.3,
    );
    crate::kit::prop(&mut scene, "boulder").place(
        &mut scene,
        Vec3::new(-3.0, -0.15, 0.9),
        2.0,
        0.8,
    );
    // A hazy afternoon, the sun behind the back wall: the wall's shadow
    // falls over the corner, where the sky's light from everywhere is all
    // there is, so the occlusion in the corner, under the stacks, in the
    // steps' angle and where the column meets the setts carries the shot.
    let sun = Vec3::new(0.5, 0.75, -0.45);
    crate::sky_scenes::physical_light_under(&mut scene, sun);
    scene.lighting.light.intensity *= 0.5;
    scene.atmosphere.clouds = Some(og_render::Clouds {
        coverage: 0.6,
        color: Vec3::splat(0.75),
        base: 900.0,
        layer: 0.0,
        ..og_render::Clouds::default()
    });
    // Two metres' reach (Unreal's default): GTAO's falloff starts at half
    // the radius, so a crate's top still shades the setts in front of it.
    scene.look.post.ao = Some(AmbientOcclusion {
        radius: 2.0,
        intensity: 1.3,
        method,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(1.0, 1.75, 3.4),
        target: Vec3::new(-0.7, 0.75, -1.7),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.4 },
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
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.97, 0.92),
            look: None,
        },
        intensity: 0.25,
        background: Background::Sky,
        ..Environment::default()
    };
    scene.look.post.gi = gi.then(og_render::ScreenSpaceGlobalIllumination::default);
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 2.2, 6.0),
        target: Vec3::new(0.0, 0.9, -1.0),
        exposure: Exposure::linear(0.8),
        ..Default::default()
    };
    scene
}

// ---------------------------------------------------------------------------
// Probe global illumination.
// ---------------------------------------------------------------------------

/// The probe GI's settings the room scenes use: probes 0.75 m apart, two
/// cascades (12 × 6 × 12 m, then twice that), so the room and the floor
/// around it are inside the first.
pub fn room_probes() -> og_render::ProbeGlobalIllumination {
    og_render::ProbeGlobalIllumination {
        spacing: 0.75,
        cascades: 2,
        ..Default::default()
    }
}

/// [`gi_scene`]'s room lit by the probe GI instead of the screen-space one
/// (`probes`; without, no GI at all): the sunlit red wall and floor bounce
/// into the shade wherever the camera looks, so the floor along the red
/// wall and the block's left face turn pink, the floor by the green wall
/// and the sphere's right side mint.
pub fn gi_probe_scene(probes: bool) -> RenderScene {
    let mut scene = gi_scene(false);
    scene.look.post.gi_probes = probes.then(room_probes);
    scene
}

/// A farm's lean-to under an overcast sky: a plank roof on a rubble back
/// wall and an ochre-plastered left wall, open to the yard in front and on
/// the right, its stores inside (crates, barrels, a ladder's worth of
/// boards, a millstone of a boulder), the yard's setts and the meadow
/// beyond. No sun: the sky is all the light, so what the roof and walls
/// hide of it is what the probe GI's sky occlusion shows (`probes`) and
/// the environment's light alone does not: the floor under the roof dark,
/// darkest in the back left corner, the open yard in front as bright as
/// without. With `ao`, GTAO (whose bent normals the probes are read along)
/// darkens the wall-floor joins and the stores' feet.
pub fn gi_shelter_scene(probes: bool, ao: bool) -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let setts = kit::surface_tinted(&mut scene, "cobbles", [0.95, 0.92, 0.86]);
    let yard = scene.add_mesh(kit::slab(Vec2::splat(-12.0), Vec2::splat(12.0), 0.0, 2.0));
    scene.add_instance(yard, setts, Mat4::IDENTITY);
    let grass = kit::surface_tinted(&mut scene, "grass", [0.86, 1.0, 0.72]);
    for (min, max) in [
        (Vec2::new(-260.0, -260.0), Vec2::new(-12.0, 260.0)),
        (Vec2::new(12.0, -260.0), Vec2::new(260.0, 260.0)),
        (Vec2::new(-12.0, -260.0), Vec2::new(12.0, -12.0)),
        (Vec2::new(-12.0, 12.0), Vec2::new(12.0, 260.0)),
    ] {
        let m = scene.add_mesh(kit::slab(min, max, -0.01, 7.0));
        scene.add_instance(m, grass, Mat4::IDENTITY);
    }
    kit::hills(&mut scene, None);
    let stone = kit::surface_tinted(&mut scene, "stone", [0.95, 0.9, 0.84]);
    let ochre = kit::surface_tinted(&mut scene, "stone", [1.25, 0.85, 0.45]);
    let planks = kit::surface_tinted(&mut scene, "planks", [0.85, 0.75, 0.65]);
    let block = |s: &mut RenderScene, m, min: Vec3, max: Vec3| {
        let mesh = s.add_mesh(kit::block(min, max, 1.6));
        s.add_instance(mesh, m, Mat4::IDENTITY);
    };
    // Roof, back wall, left wall: the porch's slabs, the roof on rafters.
    block(
        &mut scene,
        planks,
        Vec3::new(-3.2, 3.0, -3.2),
        Vec3::new(3.2, 3.2, 2.2),
    );
    for x in [-2.4f32, -0.8, 0.8, 2.4] {
        block(
            &mut scene,
            planks,
            Vec3::new(x - 0.08, 2.84, -3.0),
            Vec3::new(x + 0.08, 3.0, 2.2),
        );
    }
    block(
        &mut scene,
        planks,
        Vec3::new(-3.2, 2.84, 1.95),
        Vec3::new(3.2, 3.0, 2.15),
    );
    block(
        &mut scene,
        stone,
        Vec3::new(-3.2, 0.0, -3.2),
        Vec3::new(3.2, 3.0, -3.0),
    );
    block(
        &mut scene,
        ochre,
        Vec3::new(-3.2, 0.0, -3.0),
        Vec3::new(-3.0, 3.0, 2.2),
    );
    // A post holding the roof's open corner.
    block(
        &mut scene,
        planks,
        Vec3::new(2.95, 0.0, 1.95),
        Vec3::new(3.15, 2.84, 2.15),
    );
    // The stores: crates and barrels in the back left corner, boards
    // leant on the back wall, a boulder where the sphere lay.
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-2.4, 0.0, -2.4), 0.1, 0.7);
    crate_.place(&mut scene, Vec3::new(-2.35, 0.7, -2.45), -0.15, 0.65);
    crate_.place(&mut scene, Vec3::new(-2.5, 0.0, -1.3), 0.5, 0.6);
    let barrel = kit::prop(&mut scene, "winebarrel");
    barrel.place(&mut scene, Vec3::new(-1.3, 0.0, -2.5), 0.4, 1.0);
    barrel.place(&mut scene, Vec3::new(-0.4, 0.0, -2.55), 1.6, 1.0);
    let board = scene.add_mesh(kit::block(
        Vec3::new(-0.15, 0.0, -0.025),
        Vec3::new(0.15, 2.6, 0.025),
        1.2,
    ));
    for (x, turn) in [(0.7f32, 0.03f32), (1.05, -0.04), (1.4, 0.02)] {
        scene.add_instance(
            board,
            planks,
            Mat4::from_translation(Vec3::new(x, 0.0, -2.75))
                * Mat4::from_rotation_y(turn)
                * Mat4::from_rotation_x(-0.18),
        );
    }
    kit::prop(&mut scene, "boulder").place(&mut scene, Vec3::new(1.2, -0.1, -1.2), 0.9, 0.9);
    // The farm's pines past the yard, its watchtower on the hill.
    let pine = kit::prop(&mut scene, "pine");
    for (x, z, h) in [
        (-9.0f32, -10.0f32, 10.0f32),
        (-4.5, -13.0, 12.0),
        (6.0, -11.0, 11.0),
        (10.5, -7.0, 9.5),
        (14.0, -14.0, 12.5),
    ] {
        pine.place(&mut scene, Vec3::new(x, 0.0, z), x * 0.4, h);
    }
    kit::prop(&mut scene, "watchtower").place(&mut scene, Vec3::new(30.0, 0.0, -60.0), 1.2, 18.0);
    // Overcast: no sun, a grey sky.
    scene.lighting.light = DirectionalLight {
        direction_to_light: Vec3::new(0.3, 0.9, 0.2),
        intensity: 0.0,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: Vec3::new(0.3, 0.9, 0.2),
            sun_color: Vec3::ONE,
            look: None,
        },
        intensity: 1.0,
        background: Background::Sky,
        ..Environment::default()
    };
    scene.atmosphere.clouds = Some(og_render::Clouds {
        coverage: 0.9,
        color: Vec3::splat(0.8),
        base: 900.0,
        layer: 0.0,
        ..og_render::Clouds::default()
    });
    scene.look.post.gi_probes = probes.then(|| og_render::ProbeGlobalIllumination {
        spacing: 1.0,
        cascades: 2,
        ..Default::default()
    });
    scene.look.post.ao = ao.then(|| AmbientOcclusion {
        radius: 0.5,
        intensity: 1.0,
        method: og_render::AoMethod::Gtao,
        ..Default::default()
    });
    // From the yard, under the roof's edge, into the lean-to.
    scene.frame.camera = Camera {
        position: Vec3::new(1.6, 1.6, 6.2),
        target: Vec3::new(-0.4, 0.8, -1.2),
        far: 2500.0,
        exposure: Exposure::linear(1.0),
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
    let sphere = scene.add_mesh(shapes::sphere(32, 24));
    // A painted room of an inn: limewashed plaster, an ochre-red wall on
    // the left and a green one on the right (their bounce is what the
    // light volume carries), a plank floor and ceiling.
    let plaster =
        |s: &mut RenderScene, tint: [f32; 3]| crate::kit::surface_tinted(s, "stone", tint);
    let white = plaster(&mut scene, [1.15, 1.1, 1.0]);
    let red = plaster(&mut scene, [1.3, 0.22, 0.16]);
    let green = plaster(&mut scene, [0.25, 1.0, 0.3]);
    let planks = crate::kit::surface_tinted(&mut scene, "planks", [0.9, 0.82, 0.72]);
    for (mat, min, max) in [
        (
            planks,
            Vec3::new(-3.2, -0.2, -3.2),
            Vec3::new(3.2, 0.0, 3.2),
        ),
        (planks, Vec3::new(-3.2, 3.0, -3.2), Vec3::new(3.2, 3.2, 3.2)),
        (
            white,
            Vec3::new(-3.2, -0.2, -3.2),
            Vec3::new(3.2, 3.2, -3.0),
        ),
        (white, Vec3::new(-3.2, -0.2, 3.0), Vec3::new(3.2, 3.2, 3.2)),
        (red, Vec3::new(-3.2, -0.2, -3.2), Vec3::new(-3.0, 3.2, 3.2)),
        (green, Vec3::new(3.0, -0.2, -3.2), Vec3::new(3.2, 3.2, 3.2)),
    ] {
        let m = scene.add_mesh(crate::kit::block(min, max, 1.6));
        scene.add_instance(m, mat, Mat4::IDENTITY);
    }
    // A dresser-high stack of crates where the block stood, an oak barrel
    // where the white ball sat, beams under the ceiling.
    let crate_ = crate::kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-0.6, 0.0, -0.9), 0.1, 0.62);
    crate_.place(&mut scene, Vec3::new(-0.55, 0.62, -0.95), -0.12, 0.6);
    crate::kit::prop(&mut scene, "winebarrel").place(
        &mut scene,
        Vec3::new(1.2, 0.0, 0.3),
        0.7,
        1.0,
    );
    for x in [-1.6f32, 0.0, 1.6] {
        let beam = scene.add_mesh(crate::kit::block(
            Vec3::new(x - 0.12, 2.78, -3.0),
            Vec3::new(x + 0.12, 3.0, 3.0),
            1.2,
        ));
        scene.add_instance(beam, planks, Mat4::IDENTITY);
    }
    if let Some(roughness) = mirror {
        let chrome = scene.add_material(Material {
            base_color: [0.95, 0.95, 0.95, 1.0],
            metallic: 1.0,
            roughness,
            ..Default::default()
        });
        // On the camera's axis, clear of the crates and the barrel: a
        // gazing ball on a broken column's stump.
        scene.add_instance(
            sphere,
            chrome,
            Mat4::from_translation(Vec3::new(0.0, 1.215, 0.39))
                * Mat4::from_scale(Vec3::splat(0.8)),
        );
        crate::kit::prop(&mut scene, "column_broken").place(
            &mut scene,
            Vec3::new(0.0, 0.0, 0.39),
            0.5,
            0.95,
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
        probe.fade = 0.3;
        scene.lighting.reflection_probes.push(probe);
    }
    scene.lighting.light = DirectionalLight {
        intensity: 0.0,
        ..Default::default()
    };
    scene.lighting.lights.push(Light {
        kind: LightKind::Point,
        position: Vec3::new(0.0, 2.7, -0.4),
        color: colour,
        intensity: 14.0,
        range: 9.0,
        cast_shadows: true,
    });
    scene.lighting.environment = Environment {
        intensity: 0.05,
        background: Background::Color(Vec3::ZERO),
        ..Default::default()
    };
    // Short-range: corners and contact darken, the open walls (unoccluded)
    // keep their bounce and their lamp light. Most of this room's light is
    // the lamp's, so the occlusion reaches a little of it too (at most a
    // fifth, where fully occluded), or the contact would not read.
    scene.look.post.ao = Some(AmbientOcclusion {
        radius: 0.5,
        intensity: 1.0,
        method: AoMethod::Gtao,
        direct: 0.3,
        ..Default::default()
    });
    if realtime {
        scene
            .lighting
            .light_volumes
            .push(og_render::RealtimeLightVolume {
                id: 1,
                min: Vec3::new(-2.7, 0.15, -2.7),
                max: Vec3::new(2.7, 2.75, 2.7),
                fade: 1.35,
                spacing: 1.35,
                updates_per_frame: 8,
                intensity: 1.0,
            });
    }
    scene.frame.camera = Camera {
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
                * Mat4::from_rotation_y(crate::scenes::TRICKSTER_TO_Z + 0.3 - i as f32 * 0.15)
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
    scene.lighting.light = DirectionalLight {
        direction_to_light: sun,
        color: Vec3::new(1.0, 0.8, 0.6),
        intensity: 1.6,
        ..Default::default()
    };
    scene.lighting.environment = Environment {
        sky: ProceduralSky {
            sun_direction: sun,
            sun_color: Vec3::new(1.0, 0.7, 0.45),
            look: None,
        },
        intensity: 0.25,
        background: Background::Color(Vec3::new(0.01, 0.012, 0.02)),
        ..Environment::default()
    };
    scene.look.post.bloom = Some(Bloom::default());
    scene.look.post.depth_of_field = Some(DepthOfField {
        focus_distance: 10.2,
        focus_range: 1.2,
        max_blur: 0.014,
        falloff: 3.0,
        tilt_shift: 0.0,
        quality,
    });
    scene.frame.camera = Camera {
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

// ---------------------------------------------------------------------------
// Lens flare.
// ---------------------------------------------------------------------------
