//! `skeleton-2d`: a 2D rig (og-anim's `type rig2d`, the game-2d template's
//! lamp keeper) posed by og-anim and drawn through the sprite batch's
//! triangles (`RenderScene::sprite_meshes`), lit by 2D lighting.
//!
//! Three keepers stand on three planks, flat, rising 20° and falling 20°,
//! at idle-clip times 0, 0.5 and 1 (the third mirrored by flipX): each leg
//! is a two-bone IK chain whose target is the plank's top under the foot,
//! and the cape is a weighted mesh over the hip and two cape bones, bent
//! by the clip's bone and deform keys. One warm light and a cool ambient
//! light the layer, as og.Lighting2D lights a sprite there. Native only:
//! the rig is posed by og-anim, which the browser's scene build does not
//! carry.

use crate::scenes::asset;
use glam::{Vec2, Vec3, Vec4};
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

/// `skeleton-2d` (module docs).
pub fn skeleton_scene() -> Result<RenderScene, String> {
    let rig = Rig2dAsset::parse(
        std::str::from_utf8(&asset("rig2d/keeper.ogt")?).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("rig2d/keeper.ogt: {}", e.reason()))?;
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
    let leg = |name: &str| rig.iks.iter().position(|k| k.name == name).unwrap() as u32;
    let (front, back) = (leg("leg-f"), leg("leg-b"));
    for (x, top, slope, time, flip) in STANDS {
        let ground = |gx: f32| top + slope * (gx - x);
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
        let place = Transform {
            position: Vec3::new(x, ground(x) + LIFT, 0.0),
            ..Default::default()
        };
        let s = Skeleton2D {
            rig: 1,
            time,
            flip_x: flip,
            layer: 1,
            ..Default::default()
        };
        let side = if flip { -1.0 } else { 1.0 };
        let targets: Vec<(u32, Vec2, f32)> = [front, back]
            .into_iter()
            .zip(FEET)
            .map(|(c, dx)| {
                let fx = x + dx * side;
                (c, Vec2::new(fx, ground(fx) + ANKLE), 1.0)
            })
            .collect();
        let frame = skeleton_frame(&rig, &place, flip);
        let pose = skeleton_pose(&rig, &s, &frame, &targets);
        let (corners, indices) = rig.triangles(&pose);
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
        scene.sprite_meshes.push(SpriteMesh {
            texture: Some(texture),
            vertices,
            indices,
            layer: 1,
            ..Default::default()
        });
    }
    scene.lighting_2d = Some(Lighting2d {
        ambient: Vec3::new(0.35, 0.38, 0.55),
        min_layer: 0,
        max_layer: 10,
        resolution: 0.5,
    });
    scene.lights_2d = vec![Light2d {
        position: Vec2::new(0.6, 0.4),
        color: Vec3::new(1.0, 0.72, 0.35),
        intensity: 1.8,
        radius: 6.0,
        falloff: 1.5,
        source_radius: 0.2,
        cast_shadows: false,
    }];
    scene.camera = Camera::orthographic_2d(Vec2::new(0.0, 0.0), 6.0);
    scene.environment.background = Background::Color(Vec3::new(0.012, 0.014, 0.04));
    scene.post.tonemap = Tonemap::None;
    Ok(scene)
}
