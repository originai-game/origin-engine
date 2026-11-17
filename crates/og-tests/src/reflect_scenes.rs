//! The GI and reflection scenes as shots of a game, where a test measures
//! its own plain fixture by calling it (`depth_render_scenes::ssr_scene`
//! for `probe_scenes::ssr_floor_scene`): the shots here are the registered
//! scenes.

use crate::kit;
use crate::lighting_scenes::{car_with_headlights, high_street, light_lanterns, night, wares};
use glam::{Vec2, Vec3};
use og_render::{Camera, Exposure, RenderScene, ScreenSpaceReflections};

/// `ssr-wet-floor`: the old town's high street at night after rain, its
/// setts and pavements wet (their roughness map at a quarter), PostEffects.ssr
/// on: the lanterns, the car's headlights and the lit house fronts stand
/// in the street below them. The sky is a dark night's, so what the setts
/// show of the lamps and the houses is the screen's reflection, not the
/// environment's.
pub fn ssr_street_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let lanterns = high_street(&mut scene, 80.0);
    // The street's setts and the pavements' flags, wet: darker and
    // polished by the water.
    let wet_setts = kit::surface_finish(&mut scene, "cobbles", [0.55, 0.53, 0.5], 0.22);
    let wet_flags = kit::surface_finish(&mut scene, "cobbles", [0.62, 0.6, 0.56], 0.3);
    // high_street adds the road and the square, four fields of grass and
    // the hills, then the left pavement: the setts' material is the first
    // instance's, the flags' the eighth's.
    let (setts, flags) = (
        scene.objects.instances[0].material,
        scene.objects.instances[7].material,
    );
    for instance in &mut scene.objects.instances {
        if instance.material == setts {
            instance.material = wet_setts;
        } else if instance.material == flags {
            instance.material = wet_flags;
        }
    }
    night(&mut scene);
    light_lanterns(&mut scene, &lanterns[..6], 24.0, 2, 4.0);
    wares(&mut scene, Vec3::new(-2.4, 0.0, -6.0), 1.5);
    // Driving away down the street, its headlights on the setts ahead.
    car_with_headlights(
        &mut scene,
        Vec3::new(1.6, 0.0, -24.0),
        Vec2::new(0.08, -1.0),
        120.0,
    );
    scene.look.post.ssr = Some(ScreenSpaceReflections {
        max_distance: 40.0,
        steps: 48,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.4, 1.7, 7.5),
        target: Vec3::new(-0.2, 1.2, -14.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.2 },
        ..Default::default()
    };
    scene
}

/// Which global illumination [`atrium_scene`] draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtriumGi {
    /// The screen-space GI (`gi-room`, and `gi-room-taa` under TAA).
    Screen,
    /// The probe GI alone (`gi-room-probes`), at `fx_scenes::room_probes`'
    /// settings.
    Probes,
}

/// `gi-room`, `gi-room-taa` and `gi-room-probes`: the atrium of a Roman
/// villa on a summer afternoon: a Pompeian-red wall on the left, a green
/// one on the right, white plaster at the back with a door, a flagged
/// floor, a marble column's stump and a marble ball on a plinth, a bench,
/// wine barrels and crates, under a roof open in its middle over a stone
/// kerb. The sun falls through the opening on the red wall's foot and the
/// floor there; the rest of the room lies in the roof's shade, lit by
/// what the GI carries: red off the sunlit wall onto the floor, the stump
/// and the barrel, the floor's light up under the roof, green off the
/// right wall onto the ball and the bench. `fx_scenes::gi_scene` is the
/// plain room `tests/gi.rs` measures.
pub fn atrium_scene(gi: AtriumGi) -> RenderScene {
    use og_render::{Material, shapes};
    let mut scene = RenderScene::new();
    let flags = kit::surface_tinted(&mut scene, "cobbles", [1.05, 1.0, 0.92]);
    let floor = scene.add_mesh(kit::slab(Vec2::splat(-3.0), Vec2::splat(3.0), 0.0, 1.6));
    scene.add_instance(floor, flags, glam::Mat4::IDENTITY);
    // The garden outside and the hills past the villa's walls.
    let grass = kit::surface_tinted(&mut scene, "grass", [0.86, 1.0, 0.72]);
    for (min, max) in [
        (Vec2::new(-260.0, -260.0), Vec2::new(-3.0, 260.0)),
        (Vec2::new(3.0, -260.0), Vec2::new(260.0, 260.0)),
        (Vec2::new(-3.0, -260.0), Vec2::new(3.0, -3.0)),
        (Vec2::new(-3.0, 3.0), Vec2::new(3.0, 260.0)),
    ] {
        let m = scene.add_mesh(kit::slab(min, max, -0.01, 7.0));
        scene.add_instance(m, grass, glam::Mat4::IDENTITY);
    }
    kit::hills(&mut scene, None);
    // Painted lime plaster: white, Pompeian red and green.
    let white = kit::plaster(&mut scene, [0.78, 0.75, 0.68]);
    let red = kit::plaster(&mut scene, [0.62, 0.07, 0.04]);
    let green = kit::plaster(&mut scene, [0.07, 0.36, 0.12]);
    let planks = kit::surface_tinted(&mut scene, "planks", [0.8, 0.68, 0.56]);
    let block = |s: &mut RenderScene, m, min: Vec3, max: Vec3| {
        let mesh = s.add_mesh(kit::block(min, max, 1.6));
        s.add_instance(mesh, m, glam::Mat4::IDENTITY);
    };
    // The walls, where gi_scene's stand: back 6 m, the sides 5 m deep.
    block(
        &mut scene,
        white,
        Vec3::new(-3.0, 0.0, -2.6),
        Vec3::new(3.0, 3.0, -2.4),
    );
    block(
        &mut scene,
        red,
        Vec3::new(-3.0, 0.0, -2.5),
        Vec3::new(-2.8, 3.0, 2.5),
    );
    block(
        &mut scene,
        green,
        Vec3::new(2.8, 0.0, -2.5),
        Vec3::new(3.0, 3.0, 2.5),
    );
    // A dado of darker plaster along the foot of the side walls.
    let dado = [
        kit::plaster(&mut scene, [0.22, 0.03, 0.02]),
        kit::plaster(&mut scene, [0.03, 0.13, 0.05]),
    ];
    block(
        &mut scene,
        dado[0],
        Vec3::new(-2.82, 0.0, -2.4),
        Vec3::new(-2.78, 0.5, 2.5),
    );
    block(
        &mut scene,
        dado[1],
        Vec3::new(2.78, 0.0, -2.4),
        Vec3::new(2.82, 0.5, 2.5),
    );
    // The door in the back wall. The roof over the atrium, open in its
    // middle (the compluvium) over a stone kerb (the impluvium): the sun
    // falls through on the red wall's foot and the floor beside it, the
    // rest of the room lies in the roof's shade.
    block(
        &mut scene,
        planks,
        Vec3::new(-0.6, 0.0, -2.42),
        Vec3::new(0.6, 2.2, -2.38),
    );
    // The roof's underside limewashed, its rafters showing.
    for (min, max) in [
        (Vec3::new(-3.0, 3.0, -2.6), Vec3::new(-1.0, 3.15, 2.6)),
        (Vec3::new(1.8, 3.0, -2.6), Vec3::new(3.0, 3.15, 2.6)),
        (Vec3::new(-1.0, 3.0, -2.6), Vec3::new(1.8, 3.15, -1.5)),
        (Vec3::new(-1.0, 3.0, 1.4), Vec3::new(1.8, 3.15, 2.6)),
    ] {
        block(&mut scene, white, min, max);
    }
    for z in [-2.0f32, -0.6, 0.8, 2.2] {
        for (x0, x1) in [(-3.0f32, -1.0f32), (1.8, 3.0)] {
            block(
                &mut scene,
                planks,
                Vec3::new(x0, 2.88, z - 0.07),
                Vec3::new(x1, 3.0, z + 0.07),
            );
        }
    }
    for (min, max) in [
        (Vec3::new(-0.5, 0.0, -1.3), Vec3::new(1.7, 0.12, -1.1)),
        (Vec3::new(-0.5, 0.0, 1.0), Vec3::new(1.7, 0.12, 1.2)),
        (Vec3::new(-0.5, 0.0, -1.1), Vec3::new(-0.3, 0.12, 1.0)),
        (Vec3::new(1.5, 0.0, -1.1), Vec3::new(1.7, 0.12, 1.0)),
    ] {
        block(&mut scene, white, min, max);
    }
    // Where gi_scene's white block stands, the marble column's stump; where
    // its sphere lies, a plinth with a marble ball.
    let marble = scene.add_material(Material {
        base_color: [0.86, 0.85, 0.82, 1.0],
        roughness: 0.35,
        ..Default::default()
    });
    kit::prop(&mut scene, "column_broken").place(&mut scene, Vec3::new(-1.0, 0.0, -0.8), 0.4, 1.3);
    block(
        &mut scene,
        white,
        Vec3::new(0.75, 0.0, -0.15),
        Vec3::new(1.45, 0.35, 0.55),
    );
    let sphere = scene.add_mesh(shapes::sphere(48, 32));
    scene.add_instance(
        sphere,
        marble,
        glam::Mat4::from_translation(Vec3::new(1.1, 0.75, 0.2))
            * glam::Mat4::from_scale(Vec3::splat(0.8)),
    );
    // A bench on the green side, wine and stores on the red side.
    block(
        &mut scene,
        white,
        Vec3::new(2.1, 0.0, -1.8),
        Vec3::new(2.7, 0.45, 0.4),
    );
    let barrel = kit::prop(&mut scene, "winebarrel");
    barrel.place(&mut scene, Vec3::new(-2.3, 0.0, -1.0), 0.4, 0.95);
    barrel.place(&mut scene, Vec3::new(-1.6, 0.0, -1.95), 1.3, 0.95);
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-2.3, 0.0, -2.0), 0.2, 0.6);
    crate_.place(&mut scene, Vec3::new(2.3, 0.0, -2.1), -0.3, 0.55);
    // An afternoon sun from the right, through the opening onto the red
    // wall's lower half and the floor at its foot.
    let sun = Vec3::new(0.75, 0.6, 0.25);
    crate::sky_scenes::physical_light_under(&mut scene, sun);
    match gi {
        AtriumGi::Screen => {
            scene.look.post.gi = Some(og_render::ScreenSpaceGlobalIllumination::default());
        }
        AtriumGi::Probes => scene.look.post.gi_probes = Some(crate::fx_scenes::room_probes()),
    }
    scene.frame.camera = Camera {
        position: Vec3::new(0.2, 1.75, 4.6),
        target: Vec3::new(0.0, 0.85, -1.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}
