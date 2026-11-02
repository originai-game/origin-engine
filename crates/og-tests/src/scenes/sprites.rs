//! Sprite fields, billboards, shadows and texture arrays.

use super::*;

/// Sprite textures: a soft disc, a 2x2 atlas of shapes, a ring.
pub(crate) fn sprite_textures(scene: &mut RenderScene) -> [og_render::TextureHandle; 3] {
    let disc = scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let a = ((1.0 - d) * 4.0).clamp(0.0, 1.0);
        [255, 255, 255, (a * 255.0) as u8]
    }));
    let atlas = scene.add_texture(texture(128, ColorSpace::Srgb, |u, v| {
        let (qu, qv) = ((u * 2.0).fract(), (v * 2.0).fract());
        let quadrant = (u >= 0.5) as u8 + 2 * (v >= 0.5) as u8;
        let inside = match quadrant {
            0 => (qu - 0.5).abs() < 0.4 && (qv - 0.5).abs() < 0.4,
            1 => (qu - 0.5).abs() + (qv - 0.5).abs() < 0.45,
            2 => qv > 0.1 && (qu - 0.5).abs() < (qv - 0.1) * 0.5,
            _ => ((qu - 0.5).powi(2) + (qv - 0.5).powi(2)).sqrt() < 0.42,
        };
        let colors = [
            [255, 200, 60],
            [80, 220, 255],
            [140, 255, 120],
            [255, 110, 200],
        ];
        let c = colors[quadrant as usize];
        [c[0], c[1], c[2], if inside { 255 } else { 0 }]
    }));
    let ring = scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let a = (1.0 - ((d - 0.75).abs() * 8.0)).clamp(0.0, 1.0);
        [255, 255, 255, (a * 255.0) as u8]
    }));
    [disc, atlas, ring]
}

/// Where sprite `i` of `n` sits at `frame`: a slowly turning sunflower spiral.
fn sprite_at(i: usize, n: usize, frame: u32) -> (Vec2, f32) {
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    let r = (i as f32 / n as f32).sqrt() * 9.5;
    let a = i as f32 * golden + frame as f32 * 0.01;
    (Vec2::new(r * a.cos() * 1.7, r * a.sin()), a)
}

pub(super) fn sprite_field(n: usize) -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let [disc, atlas, ring] = sprite_textures(&mut scene);
    for i in 0..n {
        let (p, a) = sprite_at(i, n, 0);
        let kind = i % 5;
        let (texture, uv_rect, layer, size) = match kind {
            0 | 1 => (disc, [0.0, 0.0, 1.0, 1.0], 0, 0.22),
            2 => (ring, [0.0, 0.0, 1.0, 1.0], 1, 0.3),
            _ => {
                let q = (i / 5) % 4;
                let (qu, qv) = ((q % 2) as f32 * 0.5, (q / 2) as f32 * 0.5);
                (atlas, [qu, qv, qu + 0.5, qv + 0.5], 1, 0.26)
            }
        };
        let hue = i as f32 / n as f32;
        scene.add_sprite(Sprite {
            texture: Some(texture),
            position: p.extend(0.0),
            size: Vec2::splat(size),
            rotation: a,
            color: [
                0.4 + 0.6 * (hue * std::f32::consts::TAU).cos().abs(),
                0.4 + 0.6 * (hue * 9.0).sin().abs(),
                0.9,
                0.9,
            ],
            uv_rect,
            layer,
            ..Default::default()
        });
    }
    scene.camera = Camera::orthographic_2d(Vec2::ZERO, 20.0);
    scene.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    scene.post.tonemap = Tonemap::None;
    Ok(scene)
}

/// One layer of the `sprites-array` flipbook: a 16 x 16 pixel-art wheel
/// whose spoke points at `frame` eighths of a turn, in that frame's hue.
pub fn flipbook_frame(frame: u32) -> TextureData {
    let hues = [
        [255, 80, 70],
        [255, 170, 50],
        [240, 230, 70],
        [110, 220, 90],
        [60, 210, 200],
        [70, 130, 255],
        [160, 90, 255],
        [250, 100, 200],
    ];
    let c = hues[frame as usize % 8];
    let angle = frame as f32 * std::f32::consts::TAU / 8.0;
    let mut t = texture(16, ColorSpace::Srgb, |u, v| {
        let (x, y) = (u - 0.5, 0.5 - v);
        let r = (x * x + y * y).sqrt();
        let along = x * angle.cos() + y * angle.sin();
        let across = -x * angle.sin() + y * angle.cos();
        if (0.36..0.47).contains(&r) {
            [c[0], c[1], c[2], 255]
        } else if along > 0.0 && r < 0.4 && across.abs() < 0.07 {
            [255, 255, 255, 255]
        } else if r < 0.1 {
            [40, 40, 50, 255]
        } else {
            [0, 0, 0, 0]
        }
    });
    t.mips = Mips::None;
    t
}

/// A 16 x 16 pixel-art figure (head, tunic of `tunic`, legs), feet on
/// the bottom row; transparent round it.
fn pixel_figure(tunic: [u8; 3]) -> TextureData {
    let mut t = texture(16, ColorSpace::Srgb, |u, v| {
        let (x, y) = ((u * 16.0) as i32, (v * 16.0) as i32);
        let skin = [236, 188, 150, 255];
        let dark = [40, 32, 48, 255];
        match (x, y) {
            // Hair and head.
            (6..=9, 1) => dark,
            (5..=10, 2..=5) if (x - 7).abs() + (y - 3).abs() <= 4 => {
                if y == 2 {
                    dark
                } else {
                    skin
                }
            }
            // Arms and tunic.
            (4 | 11, 7..=10) => skin,
            (5..=10, 6..=11) => [tunic[0], tunic[1], tunic[2], 255],
            // Legs and boots.
            (6 | 9, 12..=13) => [70, 60, 90, 255],
            (5..=6 | 9..=10, 14..=15) => dark,
            _ => [0, 0, 0, 0],
        }
    });
    t.mips = Mips::None;
    t
}

/// The `sprites-shadow` scene (see [`build`]).
pub fn sprites_shadow_scene() -> RenderScene {
    use og_render::{SpriteArray, SpriteShadow, TextureArrayData, TextureFilter, TextureSource};
    let mut scene = RenderScene::new();
    ground(&mut scene, 40.0, 1.0, 0.9);
    // The set: a pillar and a crate casting their own shadows.
    let stone = scene.add_material(Material {
        base_color: [0.72, 0.7, 0.66, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    let pillar = scene.add_mesh(shapes::cuboid(Vec3::new(0.8, 2.6, 0.8)));
    let crate_mesh = scene.add_mesh(shapes::cuboid(Vec3::splat(0.9)));
    scene.add_instance(
        pillar,
        stone,
        Mat4::from_translation(Vec3::new(3.2, 1.3, -2.0)),
    );
    scene.add_instance(
        crate_mesh,
        stone,
        Mat4::from_translation(Vec3::new(-3.4, 0.45, 0.4)) * Mat4::from_rotation_y(0.4),
    );
    let figures: Vec<_> = [[200, 60, 60], [60, 120, 210], [80, 170, 90]]
        .into_iter()
        .map(|tunic| {
            let t = scene.add_texture(pixel_figure(tunic));
            scene.set_texture_filter(t, TextureFilter::Nearest);
            t
        })
        .collect();
    let wheel = scene
        .add_texture_array(TextureArrayData {
            layers: (0..8)
                .map(|k| TextureSource::Rgba(flipbook_frame(k)))
                .collect(),
        })
        .expect("eight frames of one size");
    scene.set_texture_array_filter(wheel, TextureFilter::Nearest);
    let stand = |x: f32, z: f32, size: f32| Sprite {
        position: Vec3::new(x, 0.0, z),
        size: Vec2::splat(size),
        anchor: Vec2::new(0.5, 0.0),
        ..Default::default()
    };
    // Three figures with drop shadows, the third darker; a fourth without
    // one beside the pillar; a wheel drawn from the texture array with one.
    let add = |scene: &mut RenderScene, sprite: Sprite, shadow: f32| {
        let i = scene.sprites.len() as u32;
        scene.add_sprite(sprite);
        if shadow > 0.0 {
            scene.sprite_shadows.push(SpriteShadow {
                sprite: i,
                opacity: shadow,
                ..Default::default()
            });
        }
        i
    };
    add(
        &mut scene,
        Sprite {
            texture: Some(figures[0]),
            ..stand(-1.6, 0.6, 1.4)
        },
        0.5,
    );
    add(
        &mut scene,
        Sprite {
            texture: Some(figures[1]),
            ..stand(0.2, -0.4, 1.4)
        },
        0.5,
    );
    add(
        &mut scene,
        Sprite {
            texture: Some(figures[2]),
            ..stand(1.4, 1.2, 1.4)
        },
        0.8,
    );
    add(
        &mut scene,
        Sprite {
            texture: Some(figures[1]),
            ..stand(2.2, -1.6, 1.4)
        },
        0.0,
    );
    let w = add(&mut scene, stand(-0.6, -2.2, 1.2), 0.6);
    scene.sprite_arrays.push(SpriteArray {
        sprite: w,
        array: wheel,
        layer: 3,
    });
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(0.6, 0.8, 0.5),
        intensity: 3.0,
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 3.6, 7.5),
        target: Vec3::new(0.0, 0.6, -0.4),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

/// The `sprites-billboard` scene (see [`build`]).
pub fn billboard_scene() -> RenderScene {
    use og_render::{
        Light2d, Lighting2d, SpriteFacing, SpriteShadow, TextureFilter, View, ViewTarget,
    };
    let mut scene = RenderScene::new();
    ground(&mut scene, 40.0, 1.0, 0.9);
    let stone = scene.add_material(Material {
        base_color: [0.72, 0.7, 0.66, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    let crate_mesh = scene.add_mesh(shapes::cuboid(Vec3::splat(0.9)));
    scene.add_instance(
        crate_mesh,
        stone,
        Mat4::from_translation(Vec3::new(-1.6, 0.45, 2.2)) * Mat4::from_rotation_y(0.5),
    );
    // A pixel tree: a round crown over a trunk, clear round it.
    let tree = scene.add_texture(texture(32, ColorSpace::Srgb, |u, v| {
        let crown = Vec2::new(u - 0.5, v - 0.36).length() < 0.34;
        let trunk = (u - 0.5).abs() < 0.08 && v > 0.6;
        match (crown, trunk) {
            (true, _) => {
                let shade = if u + v < 0.75 { 40 } else { 0 };
                [50 + shade, 140 + shade, 60, 255]
            }
            (false, true) => [110, 70, 40, 255],
            _ => [0, 0, 0, 0],
        }
    }));
    scene.set_texture_filter(tree, TextureFilter::Nearest);
    // A sign: a red ring round a white disc with a bar across it, so its
    // in-plane turn shows.
    let sign = scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let d = Vec2::new(u - 0.5, v - 0.5).length();
        if d > 0.48 {
            [0, 0, 0, 0]
        } else if d > 0.36 {
            [210, 40, 40, 255]
        } else if (v - 0.5).abs() < 0.08 {
            [30, 30, 40, 255]
        } else {
            [240, 240, 235, 255]
        }
    }));
    // A soft puff (white; the sprite's colour tints it).
    let puff = scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let d = (Vec2::new(u - 0.5, v - 0.5).length() * 2.0).min(1.0);
        [255, 255, 255, ((1.0 - d * d) * 255.0) as u8]
    }));
    let figure = scene.add_texture(pixel_figure([200, 60, 60]));
    let figure_blue = scene.add_texture(pixel_figure([60, 120, 210]));
    let figure_green = scene.add_texture(pixel_figure([80, 170, 90]));
    for t in [figure, figure_blue, figure_green] {
        scene.set_texture_filter(t, TextureFilter::Nearest);
    }
    let stand = |texture, x: f32, z: f32, size: Vec2| Sprite {
        texture: Some(texture),
        position: Vec3::new(x, 0.0, z),
        size,
        anchor: Vec2::new(0.5, 0.0),
        ..Default::default()
    };
    let add = |scene: &mut RenderScene, sprite: Sprite, facing: SpriteFacing, shadow: f32| {
        let i = scene.sprites.len();
        scene.add_sprite(sprite);
        scene.sprite_facing.resize(i, SpriteFacing::None);
        scene.sprite_facing.push(facing);
        if shadow > 0.0 {
            scene.sprite_shadows.push(SpriteShadow {
                sprite: i as u32,
                opacity: shadow,
                ..Default::default()
            });
        }
    };
    // In the xy plane (for comparison): sheared by the camera's angle.
    add(
        &mut scene,
        stand(figure, -3.4, -0.4, Vec2::splat(1.6)),
        SpriteFacing::None,
        0.0,
    );
    // Upright, turned about y: two trees and a figure, with drop shadows
    // cast by their sun-facing cards.
    let tree_size = Vec2::new(2.0, 2.6);
    add(
        &mut scene,
        stand(tree, -1.2, -1.6, tree_size),
        SpriteFacing::CameraY,
        0.6,
    );
    add(
        &mut scene,
        stand(tree, 3.6, -3.0, tree_size),
        SpriteFacing::CameraY,
        0.6,
    );
    add(
        &mut scene,
        stand(figure_blue, 0.6, 0.4, Vec2::splat(1.4)),
        SpriteFacing::CameraY,
        0.5,
    );
    // In the view plane, turned 0.35 rad within it: the sign floats.
    add(
        &mut scene,
        Sprite {
            texture: Some(sign),
            position: Vec3::new(2.6, 1.9, -0.6),
            size: Vec2::splat(1.1),
            rotation: 0.35,
            ..Default::default()
        },
        SpriteFacing::Camera,
        0.0,
    );
    // Two translucent puffs facing the camera, submitted near first: the
    // far red one must draw first, the near blue one over it.
    add(
        &mut scene,
        Sprite {
            texture: Some(puff),
            position: Vec3::new(3.7, 2.3, 3.1),
            size: Vec2::splat(1.2),
            color: [0.3, 0.55, 1.0, 0.6],
            ..Default::default()
        },
        SpriteFacing::Camera,
        0.0,
    );
    add(
        &mut scene,
        Sprite {
            texture: Some(puff),
            position: Vec3::new(2.4, 1.3, 1.2),
            size: Vec2::splat(1.7),
            color: [1.0, 0.35, 0.25, 0.6],
            ..Default::default()
        },
        SpriteFacing::Camera,
        0.0,
    );
    // Behind the camera: culled by its bounds (FrameStats::sprites_culled).
    add(
        &mut scene,
        stand(tree, 0.0, 30.0, tree_size),
        SpriteFacing::CameraY,
        0.0,
    );
    // 2D-lit (layer 1 is the lit range): an upright figure by a warm
    // light, darkened to the ambient away from it.
    add(
        &mut scene,
        Sprite {
            layer: 1,
            ..stand(figure_green, -5.0, 0.0, Vec2::splat(1.5))
        },
        SpriteFacing::CameraY,
        0.0,
    );
    scene.lighting_2d = Some(Lighting2d {
        ambient: Vec3::splat(0.25),
        min_layer: 1,
        max_layer: 1,
        resolution: 1.0,
        shadow_angles: 1024,
    });
    scene.lights_2d.push(Light2d {
        position: Vec2::new(-5.7, 0.1),
        color: Vec3::new(1.0, 0.7, 0.3),
        intensity: 2.0,
        radius: 1.1,
        cast_shadows: false,
        ..Default::default()
    });
    scene.light = DirectionalLight {
        direction_to_light: Vec3::new(0.7, 0.75, 0.35),
        intensity: 3.0,
        ..Default::default()
    };
    let exposure = Exposure::linear(0.9);
    scene.camera = Camera {
        position: Vec3::new(7.5, 5.5, 7.0),
        target: Vec3::new(0.0, 0.8, -0.8),
        exposure,
        ..Default::default()
    };
    // From the side: every billboard turns to this camera too (the flat
    // figure is seen nearly edge on).
    scene.views.push(View {
        camera: Camera {
            position: Vec3::new(-10.0, 2.2, -0.8),
            target: Vec3::new(0.0, 1.0, -0.8),
            exposure,
            ..Default::default()
        },
        target: ViewTarget::Viewport([0.68, 0.03, 0.29, 0.29]),
        ..Default::default()
    });
    scene
}

/// The `sprites-shadow-terrain` hills: heights in metres at world x, z,
/// with a hollow 0.4 m deep and about 1.4 m across under the middle of
/// the figure at (-0.9, 0.9)'s shadow (round four: the shadow bends into
/// it).
fn shadow_hills(x: f32, z: f32) -> f32 {
    let pit = Vec2::new(x + 1.25, z - 1.45).length_squared();
    0.55 * (0.45 * x + 0.3).sin() * (0.38 * z).cos() + 0.25 * (0.9 * z + 1.1).sin() + 0.6
        - 0.4 * (-pit / (0.6 * 0.6)).exp()
}

/// The `sprites-shadow-terrain` scene (see [`build`]).
pub fn sprites_shadow_terrain_scene() -> RenderScene {
    use og_render::{
        ShadowGround, SpriteShadow, Terrain, TerrainData, TerrainLayer, TextureFilter,
    };
    let mut scene = RenderScene::new();
    // 48 m of hills sampled every half metre (the clipmap's finest level),
    // checkered so the slopes read.
    let (columns, rows, size) = (97u32, 97u32, Vec2::splat(48.0));
    let cell = size / Vec2::new(columns as f32 - 1.0, rows as f32 - 1.0);
    let heights: Vec<f32> = (0..rows)
        .flat_map(|r| {
            (0..columns).map(move |c| {
                let p = -size * 0.5 + Vec2::new(c as f32, r as f32) * cell;
                shadow_hills(p.x, p.y)
            })
        })
        .collect();
    let data = TerrainData {
        size,
        columns,
        rows,
        heights: heights.into(),
        control: None,
    };
    let checker = scene.add_texture(shapes::checker(512, 16));
    let handle = scene.add_terrain(data.clone());
    scene.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            albedo: Some(checker),
            color: Vec3::new(0.85, 0.95, 0.75),
            tile: 4.0,
            ..Default::default()
        }],
        ..Terrain::new(handle)
    });
    let stone = scene.add_material(Material {
        base_color: [0.72, 0.7, 0.66, 1.0],
        roughness: 0.8,
        ..Default::default()
    });
    // A deck whose top is 1.9 m up, and a wall casting the sun's shadow.
    let deck_top = 1.9;
    let deck = scene.add_mesh(shapes::cuboid(Vec3::new(2.4, 2.0, 1.8)));
    scene.add_instance(
        deck,
        stone,
        Mat4::from_translation(Vec3::new(2.7, deck_top - 1.0, 0.2)),
    );
    let wall = scene.add_mesh(shapes::cuboid(Vec3::new(2.6, 4.0, 0.4)));
    scene.add_instance(
        wall,
        stone,
        Mat4::from_translation(Vec3::new(-2.2, 1.6, -2.2)),
    );
    let figures: Vec<_> = [[200, 60, 60], [60, 120, 210], [80, 170, 90]]
        .into_iter()
        .map(|tunic| {
            let t = scene.add_texture(pixel_figure(tunic));
            scene.set_texture_filter(t, TextureFilter::Nearest);
            t
        })
        .collect();
    // Each figure stands on the ground under it; its shadow falls on the
    // plane fitted to the terrain under it (ShadowGround::fit, as
    // og-runtime fits it), bent where the plane misses the ground (the
    // narrow hollow), or on the deck.
    let sun = Vec3::new(0.35, 0.75, -0.6);
    let stand = |scene: &mut RenderScene, x: f32, z: f32, figure: usize, deck: bool| {
        let height = if deck {
            deck_top
        } else {
            data.height(Vec2::new(x, z))
        };
        let sprite = Sprite {
            texture: Some(figures[figure % figures.len()]),
            position: Vec3::new(x, height, z),
            size: Vec2::splat(1.4),
            anchor: Vec2::new(0.5, 0.0),
            ..Default::default()
        };
        let ground = if deck {
            ShadowGround::level(deck_top)
        } else {
            ShadowGround::fit(|x, z| data.height(Vec2::new(x, z)), &sprite, sun)
        };
        let i = scene.sprites.len() as u32;
        scene.add_sprite(sprite);
        scene.sprite_shadows.push(SpriteShadow {
            sprite: i,
            opacity: 0.6,
            ground,
        });
    };
    // On the hills: down a slope facing away from the sun, up one facing
    // it, in a hollow, and before the narrow hollow its shadow dips into.
    stand(&mut scene, 0.6, 2.2, 0, false);
    stand(&mut scene, 0.4, -1.2, 1, false);
    stand(&mut scene, -3.6, 2.4, 2, false);
    stand(&mut scene, -0.9, 0.9, 1, false);
    // On the deck.
    stand(&mut scene, 3.5, -0.4, 2, true);
    // Beside the wall: its drop shadow runs into the wall's shadow.
    stand(&mut scene, -2.8, 0.1, 0, false);
    scene.light = DirectionalLight {
        direction_to_light: sun,
        intensity: 3.0,
        ..Default::default()
    };
    scene.camera = Camera {
        position: Vec3::new(0.0, 5.6, 8.5),
        target: Vec3::new(0.0, 0.8, -0.2),
        exposure: Exposure::linear(0.9),
        ..Default::default()
    };
    scene
}

/// Layer `k` of the `sprites-array` smooth array: a soft disc, orange,
/// green or blue.
fn glow_layer(k: usize) -> og_render::TextureSource {
    og_render::TextureSource::Rgba(texture(64, ColorSpace::Srgb, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let a = ((1.0 - d) * 2.0).clamp(0.0, 1.0);
        let rgb = [[255, 120, 60], [80, 255, 140], [90, 140, 255]][k];
        [rgb[0], rgb[1], rgb[2], (a * 255.0) as u8]
    }))
}

/// The `sprites-array-fx` scene (see [`build`]): the `sprites-array`
/// flipbook and smooth arrays through the paths that scene leaves out.
/// Top: four flipbook sprites glowing (PostEffects glow) over a dim plain
/// strip. Bottom left: a fountain of round, glowing, added particles whose
/// layers of the smooth array are picked at random; bottom right: a
/// sprite-mode emitter stepping each particle through the flipbook's eight
/// layers over its life. Top left of the screen: four overlay sprites of
/// the flipbook's layers 0, 2, 4 and 6 (one draw) beside a plain one.
pub fn sprite_array_fx_scene() -> RenderScene {
    use og_render::{
        Burst, EmitterShape, Glow, Overlay, ParticleBlend, ParticleEmitter, ParticleRender,
        ParticleSystem, SheetMode, SpriteArray, SpriteFx, SpriteSheet, TextureArrayData,
        TextureFilter, TextureSource,
    };
    let mut scene = RenderScene::new();
    let flipbook = scene
        .add_texture_array(TextureArrayData {
            layers: (0..8)
                .map(|k| TextureSource::Rgba(flipbook_frame(k)))
                .collect(),
        })
        .expect("eight frames of one size");
    scene.set_texture_array_filter(flipbook, TextureFilter::Nearest);
    let glows = scene
        .add_texture_array(TextureArrayData {
            layers: (0..3).map(glow_layer).collect(),
        })
        .expect("three layers of one size");
    let plain = scene.add_texture(texture(32, ColorSpace::Srgb, |u, v| {
        let on = ((u * 4.0) as u32 + (v * 4.0) as u32).is_multiple_of(2);
        if on {
            [120, 120, 130, 255]
        } else {
            [40, 40, 50, 255]
        }
    }));
    // The strip under the glowing row.
    scene.add_sprite(Sprite {
        texture: Some(plain),
        position: Vec3::new(0.0, 2.5, 0.0),
        size: Vec2::new(14.0, 1.2),
        layer: 0,
        ..Default::default()
    });
    // Four glowing flipbook sprites: one draw, and one glow draw.
    for (n, k) in [1u32, 3, 5, 7].into_iter().enumerate() {
        let i = scene.sprites.len() as u32;
        scene.add_sprite(Sprite {
            position: Vec3::new(-4.5 + n as f32 * 3.0, 2.5, 0.0),
            size: Vec2::splat(2.0),
            layer: 1,
            ..Default::default()
        });
        scene.sprite_arrays.push(SpriteArray {
            sprite: i,
            array: flipbook,
            layer: k,
        });
        scene.sprite_fx.push(SpriteFx {
            sprite: i,
            glow: 0.25 + n as f32 * 0.25,
            ..Default::default()
        });
    }
    // Round, glowing, added particles from the smooth array, a random layer each.
    let motes = scene.add_emitter(ParticleEmitter {
        max_particles: 96,
        shape: EmitterShape::Cone {
            angle_degrees: 40.0,
            radius: 0.3,
        },
        rate: 30.0,
        bursts: vec![Burst::once(0.0, 12)],
        lifetime: [1.4, 2.0],
        speed: [3.0, 4.5],
        size: [0.5, 0.9],
        gravity: Vec3::new(0.0, -3.0, 0.0),
        array: Some(glows),
        round: true,
        sheet: SpriteSheet {
            mode: SheetMode::Random,
            ..Default::default()
        },
        blend: ParticleBlend::Additive,
        render: ParticleRender::Sprite { layer: 2 },
        glow: 0.6,
        ..Default::default()
    });
    scene.particles.push(ParticleSystem::new(
        1,
        motes,
        Mat4::from_translation(Vec3::new(-4.0, -4.5, 0.0)),
        5,
        0,
    ));
    // The flipbook's eight layers over each particle's life, alpha blended.
    let wheels = scene.add_emitter(ParticleEmitter {
        max_particles: 64,
        shape: EmitterShape::Cone {
            angle_degrees: 45.0,
            radius: 0.2,
        },
        rate: 6.0,
        bursts: vec![Burst::once(0.0, 6)],
        lifetime: [2.0, 2.0],
        speed: [3.0, 4.0],
        size: [1.0, 1.0],
        gravity: Vec3::new(0.0, -2.5, 0.0),
        array: Some(flipbook),
        sheet: SpriteSheet {
            mode: SheetMode::OverLife { cycles: 1.0 },
            ..Default::default()
        },
        blend: ParticleBlend::Alpha,
        render: ParticleRender::Sprite { layer: 2 },
        ..Default::default()
    });
    scene.particles.push(ParticleSystem::new(
        2,
        wheels,
        Mat4::from_translation(Vec3::new(4.0, -4.5, 0.0)),
        6,
        0,
    ));
    // The overlay: four flipbook layers (one draw) and a plain sprite.
    let mut overlay = Overlay::new(1280.0, 720.0, 0.0);
    for (n, k) in [0u32, 2, 4, 6].into_iter().enumerate() {
        let i = overlay.sprites.len() as u32;
        overlay.sprites.push(Sprite {
            position: Vec3::new(24.0 + n as f32 * 72.0, 24.0, 0.0),
            size: Vec2::splat(64.0),
            anchor: Vec2::ZERO,
            ..Default::default()
        });
        overlay.sprite_arrays.push(SpriteArray {
            sprite: i,
            array: flipbook,
            layer: k,
        });
    }
    overlay.sprites.push(Sprite {
        texture: Some(plain),
        position: Vec3::new(24.0 + 4.0 * 72.0, 24.0, 0.0),
        size: Vec2::splat(64.0),
        anchor: Vec2::ZERO,
        ..Default::default()
    });
    scene.overlay = Some(overlay);
    scene.post.glow = Some(Glow { intensity: 0.8 });
    scene.camera = Camera::orthographic_2d(Vec2::new(0.0, -1.0), 12.0);
    scene.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    scene.post.tonemap = Tonemap::None;
    scene
}

/// The `sprites-array` scene (see [`build`]).
pub fn sprite_array_scene() -> RenderScene {
    use og_render::{
        SpriteArray, SpriteBlend, SpriteFx, TextureArrayData, TextureFilter, TextureSource,
    };
    let mut scene = RenderScene::new();
    let flipbook = scene
        .add_texture_array(TextureArrayData {
            layers: (0..8)
                .map(|k| TextureSource::Rgba(flipbook_frame(k)))
                .collect(),
        })
        .expect("eight frames of one size");
    scene.set_texture_array_filter(flipbook, TextureFilter::Nearest);
    let glows = scene
        .add_texture_array(TextureArrayData {
            layers: (0..3).map(glow_layer).collect(),
        })
        .expect("three layers of one size");
    let plain = scene.add_texture(texture(32, ColorSpace::Srgb, |u, v| {
        let on = ((u * 4.0) as u32 + (v * 4.0) as u32).is_multiple_of(2);
        if on {
            [230, 230, 230, 255]
        } else {
            [60, 60, 70, 255]
        }
    }));
    let push = |scene: &mut RenderScene,
                sprite: Sprite,
                array: Option<(og_render::TextureArrayHandle, u32)>| {
        let i = scene.sprites.len() as u32;
        scene.add_sprite(sprite);
        if let Some((array, layer)) = array {
            scene.sprite_arrays.push(SpriteArray {
                sprite: i,
                array,
                layer,
            });
        }
        i
    };
    // Two rows of the flipbook, every frame in order; the second row
    // flipped left to right and turned a little.
    for row in 0..2 {
        for k in 0..8u32 {
            let x = -7.0 + k as f32 * 2.0;
            let y = 2.5 - row as f32 * 2.5;
            let uv_rect = if row == 0 {
                [0.0, 0.0, 1.0, 1.0]
            } else {
                [1.0, 0.0, 0.0, 1.0]
            };
            push(
                &mut scene,
                Sprite {
                    position: Vec3::new(x, y, 0.0),
                    size: Vec2::splat(1.8),
                    rotation: row as f32 * 0.3,
                    uv_rect,
                    layer: 1,
                    ..Default::default()
                },
                Some((flipbook, k)),
            );
        }
    }
    // The plain texture under them, in its own run on layer 0.
    push(
        &mut scene,
        Sprite {
            texture: Some(plain),
            position: Vec3::new(0.0, -5.0, 0.0),
            size: Vec2::new(16.0, 2.0),
            layer: 0,
            ..Default::default()
        },
        None,
    );
    // The smooth array added over the plain strip, its three layers in turn.
    for k in 0..6u32 {
        let i = push(
            &mut scene,
            Sprite {
                position: Vec3::new(-6.25 + k as f32 * 2.5, -5.0, 0.0),
                size: Vec2::splat(2.6),
                color: [1.0, 1.0, 1.0, 0.9],
                layer: 2,
                ..Default::default()
            },
            Some((glows, k % 3)),
        );
        scene.sprite_fx.push(SpriteFx {
            sprite: i,
            blend: SpriteBlend::Additive,
            ..Default::default()
        });
    }
    scene.camera = Camera::orthographic_2d(Vec2::new(0.0, -1.0), 12.0);
    scene.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    scene.post.tonemap = Tonemap::None;
    scene
}

/// Moves every sprite along the spiral (`frame` 0 is the reference frame).
pub fn move_sprites(scene: &mut RenderScene, frame: u32) {
    let n = scene.sprites.len();
    for (i, s) in scene.sprites.iter_mut().enumerate() {
        let (p, a) = sprite_at(i, n, frame);
        s.position = p.extend(0.0);
        s.rotation = a;
    }
}
