//! The effects family's game shots: each registered effects scene's subject
//! (`fx_scenes`' campfire and magic, the lit effects, the trails and beams,
//! the effect materials, the decals, the 3D fluids) set in the old town or
//! the ruined processional way. The plain fixtures the tests measure
//! (`fx::particles_scene` and the rest) stay as they are; the registry
//! draws these.

use crate::atmo_scenes::colonnade;
use crate::fx_scenes::{self as fx, DECAL_CRACK, DECAL_PAINT, DECAL_RUNE, DECAL_SCORCH};
use crate::kit;
use crate::lighting_scenes::{market_square, night, wares};
use crate::sky_scenes::physical_light_under;
use glam::{Mat4, Vec2, Vec3};
use og_render::{Bloom, Camera, Decal, Exposure, Light, RenderScene, shapes};

/// `particles` and `particles-cpu`: travellers camped for the night on the
/// ruined processional way. Their fire (flames, smoke, embers bouncing on
/// the setts) burns between their crates, a fountain of blue sparks rises
/// from a broken column, sparkles crawl over a glowing orb set on a
/// boulder, and a burst of sparks flies off the fire every second.
pub fn camp_night() -> RenderScene {
    let mut scene = RenderScene::new();
    colonnade(&mut scene);
    night(&mut scene);
    let fire = fx::campfire(&mut scene);
    fx::logs(&mut scene, Vec3::ZERO);
    fx::light_campfire(&mut scene, &fire, Vec3::ZERO, 10);
    kit::prop(&mut scene, "column_broken").place(&mut scene, Vec3::new(-2.6, 0.0, -1.4), 0.6, 1.4);
    kit::prop(&mut scene, "boulder").place(&mut scene, Vec3::new(2.4, 0.0, -1.0), 1.2, 0.4);
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-1.7, 0.0, 1.1), 0.4, 0.6);
    crate_.place(&mut scene, Vec3::new(-2.3, 0.0, 0.5), -0.3, 0.6);
    kit::prop(&mut scene, "winebarrel").place(&mut scene, Vec3::new(1.5, 0.0, -2.4), 0.8, 1.0);
    fx::camp_magic(&mut scene);
    scene.frame.camera = Camera {
        position: Vec3::new(0.6, 2.2, 6.2),
        target: Vec3::new(0.0, 1.0, -1.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -1.0 },
        ..Default::default()
    };
    scene
}

/// `particles-lit`: a smithy's yard in the market square under a low
/// afternoon sun. A pile of crates smoulders (lit smoke, shaded by the
/// sun from the right), a powder keg's blast throws lit stones out of a
/// scorched crater that bounce on the setts and kick up dust, and the heat
/// over the forge's coals bends the rubble wall behind them.
pub fn forge_yard() -> RenderScene {
    let mut scene = RenderScene::new();
    market_square(&mut scene);
    let stone = kit::surface(&mut scene, "stone");
    let wall = scene.add_mesh(kit::block(
        Vec3::new(-4.5, 0.0, -2.75),
        Vec3::new(4.5, 3.5, -2.45),
        2.0,
    ));
    scene.add_instance(wall, stone, Mat4::IDENTITY);
    // The forge's hearth round the coals (0.7 m square, 0.5 m high, at
    // 2.8, -1.2): only their top glows.
    let (cx, cz) = (2.8, -1.2);
    for (min, max) in [
        (Vec3::new(-0.55, 0.0, -0.55), Vec3::new(0.55, 0.58, -0.35)),
        (Vec3::new(-0.55, 0.0, 0.35), Vec3::new(0.55, 0.58, 0.55)),
        (Vec3::new(-0.55, 0.0, -0.35), Vec3::new(-0.35, 0.58, 0.35)),
        (Vec3::new(0.35, 0.0, -0.35), Vec3::new(0.55, 0.58, 0.35)),
    ] {
        let at = Vec3::new(cx, 0.0, cz);
        let mesh = scene.add_mesh(kit::block(min + at, max + at, 1.0));
        scene.add_instance(mesh, stone, Mat4::IDENTITY);
    }
    let crate_ = kit::prop(&mut scene, "crate");
    for (x, z, yaw, y) in [
        (-3.3, -1.0, 0.3, 0.0),
        (-2.5, -1.2, -0.4, 0.0),
        (-2.9, -1.1, 0.9, 0.6),
    ] {
        crate_.place(&mut scene, Vec3::new(x, y, z), yaw, 0.6);
    }
    kit::prop(&mut scene, "winebarrel").place(&mut scene, Vec3::new(3.9, 0.0, -1.7), 0.4, 1.0);
    scene.effects.decal_atlas = Some(fx::decal_atlas(&mut scene));
    scene.effects.decals = vec![Decal {
        uv_rect: DECAL_SCORCH,
        ..Decal::on_surface(
            Vec3::new(0.0, 0.0, 0.4),
            Vec3::Y,
            Vec2::splat(2.2),
            0.6,
            0.7,
        )
    }];
    fx::lit_effects(&mut scene);
    physical_light_under(&mut scene, Vec3::new(0.85, 0.4, 0.3));
    scene.lighting.light.cast_shadows = true;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 2.2, 6.4),
        target: Vec3::new(0.0, 1.1, -0.8),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// `trails-beams`: a night ritual on the processional way. Three wisps
/// circle a glowing rune with trails behind them, a ribbon of light
/// spirals up beside the road, and between two columns a scrolling beam
/// burns over a crackling lightning arc.
pub fn ritual_night() -> RenderScene {
    let mut scene = RenderScene::new();
    // The wisps' orbs first: [`move_ritual_wisps`] moves instances 0 to 2.
    fx::add_wisps(&mut scene);
    colonnade(&mut scene);
    night(&mut scene);
    fx::spiral_and_beams(&mut scene, false);
    let column = kit::prop(&mut scene, "column");
    for x in [2.2f32, 4.6] {
        column.place(&mut scene, Vec3::new(x, 0.0, -2.5), 0.3, 2.5);
    }
    kit::prop(&mut scene, "column_broken").place(&mut scene, Vec3::new(-3.6, 0.0, -2.0), 1.0, 0.25);
    scene.effects.decal_atlas = Some(fx::decal_atlas(&mut scene));
    scene.effects.decals = vec![Decal {
        uv_rect: DECAL_RUNE,
        emissive: Vec3::new(0.8, 1.6, 2.6),
        ..Decal::on_surface(Vec3::ZERO, Vec3::Y, Vec2::splat(2.4), 0.4, 0.0)
    }];
    scene.look.post.bloom = Some(Bloom::default());
    scene.frame.camera = Camera {
        position: Vec3::new(0.5, 2.8, 7.0),
        target: Vec3::new(0.3, 1.1, -0.8),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -1.0 },
        ..Default::default()
    };
    scene
}

/// [`ritual_night`]'s update: the wisps and their orbs (instances 0 to 2).
pub fn move_ritual_wisps(scene: &mut RenderScene, frame: u32) {
    fx::move_wisps_from(scene, frame, 0);
}

/// `effect-materials`: a duel of spells on the processional way at night.
/// Fire puffs boil off a brazier stone, a portal turns between two broken
/// columns, a shield dome covers a supply cache, the trickster's blade
/// leaves a slash, a fire trail loops beside it, a green beacon rises
/// from a column's stump, and banded spirit puffs drift at the end.
pub fn spell_duel() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    colonnade(&mut scene);
    night(&mut scene);
    let broken = kit::prop(&mut scene, "column_broken");
    broken.place(&mut scene, Vec3::new(-4.5, 0.0, -0.3), 0.4, 2.4);
    broken.place(&mut scene, Vec3::new(-1.9, 0.0, -0.3), 2.0, 2.0);
    broken.place(&mut scene, Vec3::new(4.6, 0.0, 0.0), 1.1, 0.3);
    kit::prop(&mut scene, "boulder").place(&mut scene, Vec3::new(-5.4, 0.0, 0.0), 0.3, 0.45);
    kit::prop(&mut scene, "crate").place(&mut scene, Vec3::new(-1.1, 0.0, 0.1), 0.5, 0.6);
    kit::prop(&mut scene, "winebarrel").place(&mut scene, Vec3::new(-0.4, 0.0, -0.3), 0.2, 0.9);
    let model = crate::scenes::load("trickster.glb")?;
    let parts = model.add_to(&mut scene);
    let scale = 1.8 / (model.max - model.min).max_element().max(1e-3);
    parts.instance(
        &mut scene,
        Mat4::from_translation(Vec3::new(1.0, -model.min.y * scale, -0.6))
            * Mat4::from_rotation_y(crate::scenes::TRICKSTER_TO_Z + 0.9)
            * Mat4::from_scale(Vec3::splat(scale)),
    );
    fx::effect_row(&mut scene);
    scene.look.post.bloom = Some(Bloom::default());
    scene.frame.camera = Camera {
        position: Vec3::new(0.6, 2.2, 9.0),
        target: Vec3::new(0.6, 1.3, 0.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -1.0 },
        ..Default::default()
    };
    Ok(scene)
}

/// A garden wall of `material` across the square, its face at z -2.0,
/// 8 m long and 3 m high (the decals' wall).
fn yard_wall(scene: &mut RenderScene, material: og_render::MaterialHandle) {
    let wall = scene.add_mesh(kit::block(
        Vec3::new(-4.0, 0.0, -2.4),
        Vec3::new(4.0, 3.0, -2.0),
        2.0,
    ));
    scene.add_instance(wall, material, Mat4::IDENTITY);
}

/// `decals`: the market square the morning after a fight with a mage.
/// Scorch marks blacken the setts, a rune circle still glows before a
/// plastered garden wall, a crack runs up the wall, and a splash of red
/// paint from a broken pot crosses the corner where wall meets ground.
pub fn square_after_fight() -> RenderScene {
    let mut scene = RenderScene::new();
    market_square(&mut scene);
    let plaster = kit::plaster(&mut scene, [0.66, 0.58, 0.48]);
    yard_wall(&mut scene, plaster);
    wares(&mut scene, Vec3::new(5.4, 0.0, -0.6), -0.7);
    kit::prop(&mut scene, "winebarrel").place(&mut scene, Vec3::new(2.6, 0.0, 0.2), 0.3, 1.0);
    kit::prop(&mut scene, "streetlamp").place(&mut scene, Vec3::new(-4.6, 0.0, -1.4), 0.0, 4.2);
    scene.effects.decal_atlas = Some(fx::decal_atlas(&mut scene));
    let up = Vec3::Y;
    scene.effects.decals = vec![
        Decal {
            uv_rect: DECAL_SCORCH,
            ..Decal::on_surface(Vec3::new(-1.6, 0.0, 0.6), up, Vec2::splat(2.4), 0.6, 0.3)
        },
        Decal {
            uv_rect: DECAL_SCORCH,
            ..Decal::on_surface(Vec3::new(2.0, 0.0, 1.4), up, Vec2::splat(1.8), 0.6, 1.4)
        },
        Decal {
            uv_rect: DECAL_RUNE,
            color: [1.0, 1.0, 1.0, 1.0],
            emissive: Vec3::new(1.5, 3.5, 5.0),
            roughness: 0.2,
            ..Decal::on_surface(Vec3::new(0.2, 0.0, 1.8), up, Vec2::splat(2.0), 0.4, 0.0)
        },
        Decal {
            uv_rect: DECAL_CRACK,
            ..Decal::on_surface(
                Vec3::new(-2.0, 1.6, -2.0),
                Vec3::Z,
                Vec2::new(2.2, 2.6),
                0.6,
                0.2,
            )
        },
        Decal {
            uv_rect: DECAL_PAINT,
            angle_fade_degrees: 80.0,
            ..Decal::on_surface(
                Vec3::new(1.0, 0.05, -1.8),
                Vec3::new(0.0, 1.0, 1.0),
                Vec2::splat(1.6),
                1.4,
                0.5,
            )
        },
    ];
    physical_light_under(&mut scene, Vec3::new(0.5, 0.6, 0.6));
    scene.look.post.bloom = Some(Bloom::default());
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 3.4, 6.8),
        target: Vec3::new(0.0, 0.7, -0.8),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// `decals-many`: the market square's garden wall after a firefight,
/// 640 normal-mapped bullet holes across its plaster and the setts
/// before it, and two shell craters on the setts whose rims the low sun
/// lights on one side and shades on the other.
pub fn firefight_wall() -> RenderScene {
    let mut scene = RenderScene::new();
    market_square(&mut scene);
    let plaster = kit::plaster(&mut scene, [0.66, 0.58, 0.48]);
    yard_wall(&mut scene, plaster);
    let crate_ = kit::prop(&mut scene, "crate");
    crate_.place(&mut scene, Vec3::new(-4.6, 0.0, -1.2), 0.2, 0.6);
    crate_.place(&mut scene, Vec3::new(4.5, 0.0, -1.0), -0.5, 0.6);
    crate_.place(&mut scene, Vec3::new(4.5, 0.6, -1.0), -0.3, 0.6);
    scene.effects.decal_atlas = Some(fx::bullet_hole_atlas(&mut scene));
    scene.effects.decals = fx::bullet_holes();
    physical_light_under(&mut scene, Vec3::new(-0.8, 0.45, 0.4));
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 2.8, 5.8),
        target: Vec3::new(0.0, 0.8, -0.8),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// `fluid-3d`: an alchemist's fountain in the market square. Its stone
/// basin holds a clear blue pool over a mosaic (refracted and tinted
/// through it) and an orange draught falling into it (denser, shading to
/// red), lit by the sun, the sky and a brazier's orange glow.
pub fn alchemist_fountain() -> RenderScene {
    let mut scene = RenderScene::new();
    market_square(&mut scene);
    wares(&mut scene, Vec3::new(-3.6, 0.0, -2.6), 0.4);
    let stone = kit::surface(&mut scene, "stone");
    for (min, max) in [
        (Vec3::new(-1.8, 0.0, -1.2), Vec3::new(1.8, 0.9, -1.0)),
        (Vec3::new(-1.8, 0.0, -1.0), Vec3::new(-1.6, 0.9, 1.0)),
        (Vec3::new(1.6, 0.0, -1.0), Vec3::new(1.8, 0.9, 1.0)),
        (Vec3::new(-1.8, 0.0, 1.0), Vec3::new(1.8, 0.3, 1.2)),
    ] {
        let mesh = scene.add_mesh(kit::block(min, max, 1.0));
        scene.add_instance(mesh, stone, Mat4::IDENTITY);
    }
    // The mosaic: glazed tiles in six hues.
    let tile = scene.add_mesh(shapes::cuboid(Vec3::new(0.42, 0.02, 0.42)));
    let hues = [
        [0.85, 0.3, 0.25],
        [0.9, 0.7, 0.3],
        [0.35, 0.7, 0.4],
        [0.25, 0.55, 0.85],
        [0.55, 0.4, 0.85],
        [0.9, 0.5, 0.65],
    ];
    let glazes: Vec<_> = hues
        .iter()
        .map(|c| {
            scene.add_standard_material(og_render::StandardMaterial {
                color: Vec3::from(*c),
                roughness: 0.35,
                ..Default::default()
            })
        })
        .collect();
    for gx in 0..6 {
        for gz in 0..4 {
            scene.add_instance(
                tile,
                glazes[(gx + gz) % 6],
                Mat4::from_translation(Vec3::new(
                    -1.25 + gx as f32 * 0.5,
                    0.01,
                    -0.75 + gz as f32 * 0.5,
                )),
            );
        }
    }
    let spacing = 0.1;
    scene.world.fluids_3d.push(og_render::Fluid3d {
        particles: crate::p2_scenes::fluid_3d_pool(spacing).into(),
        color: Vec3::new(0.2, 0.55, 0.9),
        opacity: 0.85,
        radius: 0.12,
        volume: spacing * spacing * spacing,
        ..Default::default()
    });
    let drop = crate::p2_scenes::fluid_3d_drop(Vec3::new(0.35, 1.25, 0.1), 0.3, spacing);
    let colors: Vec<Vec3> = drop
        .iter()
        .map(|p| {
            Vec3::new(0.95, 0.5, 0.08).lerp(Vec3::new(0.85, 0.1, 0.05), (1.25 - p.y) / 0.6 + 0.5)
        })
        .collect();
    scene.world.fluids_3d.push(og_render::Fluid3d {
        particles: drop.into(),
        colors: Some(colors.into()),
        color: Vec3::new(0.95, 0.45, 0.1),
        opacity: 0.95,
        radius: 0.12,
        volume: spacing * spacing * spacing,
        ..Default::default()
    });
    physical_light_under(&mut scene, Vec3::new(-0.45, 0.7, 0.4));
    scene.lighting.lights.push(Light {
        position: Vec3::new(-1.0, 1.0, 0.9),
        color: Vec3::new(1.0, 0.6, 0.3),
        intensity: 6.0,
        range: 4.0,
        ..Default::default()
    });
    scene.frame.camera = Camera {
        position: Vec3::new(0.3, 2.3, 3.9),
        target: Vec3::new(0.0, 0.45, 0.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}
