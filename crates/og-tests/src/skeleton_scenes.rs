//! `skeleton-2d`: a native 2D rig (the game-2d template's
//! lamp keeper) posed by og-anim and drawn through the sprite batch's
//! triangles (`RenderScene::sprite_meshes`), lit by 2D lighting.
//!
//! Three keepers stand on three planks on posts over a moonlit town's
//! roofs (a painted backdrop below the lit layers), flat, rising 20° and falling 20°,
//! at idle-clip times 0, 0.5 and 1 (the third mirrored by flipX): each leg
//! is a two-bone IK chain whose target is the plank's top under the foot,
//! and the cape is a weighted mesh over the hip and two cape bones, bent
//! by the clip's bone and deform keys. One warm light and a cool ambient
//! light the layer, as og.Lighting2D lights a sprite there. Native only:
//! the rig is posed by og-anim, which the browser's scene build does not
//! carry.

use crate::scenes::asset;
use glam::{Mat4, Vec2, Vec3, Vec4};
use og_anim::{Affine2, Rig2dAsset, Skeleton2D, Transform, skeleton_frame, skeleton_pose};
use og_render::{
    Background, Camera, ColorSpace, Light2d, Lighting2d, Mips, RenderScene, Sprite, SpriteMesh,
    SpriteMeshVertex, TextureData, TextureFilter, Tonemap,
};

/// The planks: (centre x, the top's height at the centre, slope as rise per unit, clip time, flipX).
const STANDS: [(f32, f32, f32, f32, bool); 3] = [
    (-4.0, -1.6, 0.0, 0.0, false),
    (0.0, -1.6, 0.36397, 0.5, false),
    (4.0, -1.6, -0.36397, 1.0, true),
];
/// The ankle above the plank's top, the keeper's root above the ground
/// (its legs reach 1.1 from a hip 1.1 up: a slight bend), and each foot's
/// offset from the keeper (front, back).
const ANKLE: f32 = 0.12;
const LIFT: f32 = 0.08;
const FEET: [f32; 2] = [0.15, -0.15];

mod keeper;

/// The keeper's native reference rig.
fn keeper_rig() -> Result<Rig2dAsset, String> {
    let rig = keeper::rig();
    rig.validate().map_err(|e| e.to_string())?;
    Ok(rig)
}

/// `skeleton-2d` (module docs).
pub fn skeleton_scene(skinned: bool) -> Result<RenderScene, String> {
    let rig = keeper_rig()?;
    let img = image::load_from_memory(&asset("rig2d/keeper.png")?)
        .map_err(|e| format!("rig2d/keeper.png: {e}"))?
        .to_rgba8();
    let mut scene = RenderScene::new();
    let texture = scene.add_texture(TextureData {
        width: img.width(),
        height: img.height(),
        rgba: img.into_raw(),
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    });
    scene.set_texture_filter(texture, TextureFilter::Nearest);
    // The moonlit roofs behind, below the lit layers: unlit.
    crate::atmo_scenes::backdrop(
        &mut scene,
        "sprites2d/rooftops.jpg",
        Vec2::ZERO,
        6.0,
        [0.8, 0.82, 0.9, 1.0],
    );
    for (x, top, slope, time, flip) in STANDS {
        let ground = |gx: f32| top + slope * (gx - x);
        // Two posts under each plank's ends, down past the frame: the
        // plank's region of the keeper's texture, stood on end.
        for end in [-1.25f32, 1.25] {
            let px = x + end * slope.atan().cos();
            let (from, to) = (ground(px) - 0.25, -3.2);
            scene.add_sprite(Sprite {
                texture: Some(texture),
                position: Vec3::new(px, (from + to) * 0.5, 0.0),
                size: Vec2::new(from - to, 0.18),
                rotation: std::f32::consts::FRAC_PI_2,
                uv_rect: [0.375, 0.5, 1.0, 0.625],
                color: [0.7, 0.62, 0.55, 1.0],
                layer: 0,
                ..Default::default()
            });
        }
        // The plank: a region of the keeper's texture (pixels 24..64 x 32..40).
        let angle = slope.atan();
        scene.add_sprite(Sprite {
            texture: Some(texture),
            position: Vec3::new(x, top - 0.15, 0.0),
            size: Vec2::new(3.2, 0.3),
            rotation: angle,
            uv_rect: [0.375, 0.5, 1.0, 0.625],
            layer: 1,
            ..Default::default()
        });
        let feet = FEET.map(|dx| {
            let fx = x + dx * if flip { -1.0 } else { 1.0 };
            Vec2::new(fx, ground(fx) + ANKLE)
        });
        let s = Skeleton2D {
            rig: 1,
            time,
            flip_x: flip,
            layer: 1,
            ..Default::default()
        };
        let mesh = keeper(
            &rig,
            texture,
            Vec2::new(x, ground(x) + LIFT),
            &s,
            feet,
            skinned,
        );
        scene.sprite.sprite_meshes.push(mesh);
    }
    scene.sprite.lighting_2d = Some(Lighting2d {
        ambient: Vec3::new(0.35, 0.38, 0.55),
        min_layer: 0,
        max_layer: 10,
        resolution: 0.5,
        shadow_angles: 1024,
    });
    scene.sprite.lights_2d = vec![Light2d {
        position: Vec2::new(0.6, 0.4),
        color: Vec3::new(1.0, 0.72, 0.35),
        intensity: 1.8,
        radius: 6.0,
        falloff: 1.5,
        source_radius: 0.2,
        cast_shadows: false,
        ..Default::default()
    }];
    scene.frame.camera = Camera::orthographic_2d(Vec2::new(0.0, 0.0), 6.0);
    scene.lighting.environment.background = Background::Color(Vec3::new(0.012, 0.014, 0.04));
    scene.look.post.tonemap = Tonemap::None;
    Ok(scene)
}

/// The walk clip's pace: a 0.4 stride each foot every 0.8 s,
/// half a unit a second, a foot on the plank for half the cycle.
const WALK_SPEED: f32 = 0.5;
const WALK_CYCLE: f32 = 0.8;
const STRIDE: f32 = 0.4;
/// How high a foot lifts at the middle of its swing.
const STEP_LIFT: f32 = 0.1;

/// `skeleton-2d`'s frames after its own (og-gfx --sequence): each keeper
/// walks its plank the way it faces, the walk clip a 60th of a second a
/// frame, its feet planted in turn on the plank under them (IK targets as
/// the game-2d template's Plant moves them on the clip's "step" events)
/// and swung forward a stride between. The scene's own frame is as built.
pub fn walk_keepers(scene: &mut RenderScene, frame: u32) {
    let Some(t) = (frame > 0).then(|| frame as f32 / scene.frame.tick_rate.max(1.0)) else {
        return;
    };
    let Ok(rig) = keeper_rig() else {
        return;
    };
    let walk = rig.clips.iter().position(|c| c.name == "walk").unwrap() as u32;
    for (mesh, (x, top, slope, _, flip)) in scene.sprite.sprite_meshes.iter_mut().zip(STANDS) {
        let side = if flip { -1.0 } else { 1.0 };
        let ground = |gx: f32| top + slope * (gx - x);
        let body = |t: f32| x + side * WALK_SPEED * t;
        // A foot comes down a quarter stride ahead of the body every cycle
        // (the back one half a cycle after the front), stays while the body
        // passes over it, and swings a stride on.
        let foot = |offset: f32| {
            let local = (t - offset).rem_euclid(WALK_CYCLE);
            let down = body(t - local) + side * STRIDE * 0.25;
            let swing = ((local - WALK_CYCLE * 0.5) / (WALK_CYCLE * 0.5)).max(0.0);
            let fx = down + side * STRIDE * swing * swing * (3.0 - 2.0 * swing);
            let lift = STEP_LIFT * (std::f32::consts::PI * swing).sin();
            Vec2::new(fx, ground(fx) + ANKLE + lift)
        };
        let s = Skeleton2D {
            rig: 1,
            clip: walk,
            time: t.rem_euclid(WALK_CYCLE),
            flip_x: flip,
            layer: 1,
            ..Default::default()
        };
        let feet = [foot(0.0), foot(WALK_CYCLE * 0.5)];
        let texture = mesh.texture.expect("the keeper's texture");
        let at = Vec2::new(body(t), ground(body(t)) + LIFT);
        *mesh = keeper(&rig, texture, at, &s, feet, mesh.skin.is_some());
    }
}

/// One keeper's triangles: `s` (its clip, time and flip) posed with its
/// root at `at` and its feet's IK targets at `feet` (front, back), in the
/// sprite batch's layer 1.
fn keeper(
    rig: &Rig2dAsset,
    texture: og_render::TextureHandle,
    at: Vec2,
    s: &Skeleton2D,
    feet: [Vec2; 2],
    skinned: bool,
) -> SpriteMesh {
    let leg = |name: &str| rig.iks.iter().position(|k| k.name == name).unwrap() as u32;
    let targets = [(leg("leg-f"), feet[0], 1.0), (leg("leg-b"), feet[1], 1.0)];
    let place = Transform {
        position: at.extend(0.0),
        ..Default::default()
    };
    let frame = skeleton_frame(rig, &place, s.flip_x);
    let pose = skeleton_pose(rig, s, &frame, &targets);
    let (corners, indices, skin) = if skinned {
        let (corners, indices, skin) = rig.skinned_triangles(&pose);
        (
            corners,
            indices,
            Some(og_render::SpriteSkin {
                bones: skin
                    .bones
                    .iter()
                    .map(|bone| {
                        let m = frame.mul(bone);
                        Mat4::from_cols(
                            Vec4::new(m.a, m.b, 0.0, 0.0),
                            Vec4::new(m.c, m.d, 0.0, 0.0),
                            Vec4::Z,
                            Vec4::new(m.tx, m.ty, 0.0, 1.0),
                        )
                    })
                    .collect(),
                influences: skin
                    .influences
                    .iter()
                    .map(|i| og_render::SpriteInfluence {
                        position: Vec2::new(i.x, i.y),
                        weight: i.weight,
                        bone: i.bone as u32,
                    })
                    .collect(),
                ranges: skin.ranges,
            }),
        )
    } else {
        let (corners, indices) = rig.triangles(&pose);
        (corners, indices, None)
    };
    // triangles() answers rig units: the frame without its 1 / unit.
    let to_world = frame.mul(&Affine2::trs(0.0, 0.0, 0.0, rig.unit, rig.unit));
    let vertices = corners
        .iter()
        .map(|c| {
            let p = to_world.apply_to(c.position);
            let w = Vec4::new(p[0], p[1], 0.0, 1.0);
            SpriteMeshVertex {
                position: w.truncate(),
                uv: Vec2::from(c.uv),
                color: c.color,
                offset: c.offset,
            }
        })
        .collect();
    SpriteMesh {
        texture: Some(texture),
        vertices,
        indices,
        skin,
        layer: 1,
        ..Default::default()
    }
}
