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
    scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 20.0);
    scene.lighting.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    scene.look.post.tonemap = Tonemap::None;
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

/// The painted pixel-art sprite `sprites2d/{name}.png` (nearest
/// filtered) and its width over its height.
pub(crate) fn painted(scene: &mut RenderScene, name: &str) -> (og_render::TextureHandle, f32) {
    let rel = format!("sprites2d/{name}.png");
    let t = crate::kit::picture(scene, &rel, true);
    let img = image::load_from_memory(&crate::scenes::asset(&rel).expect("2D sprite"))
        .expect("2D sprite");
    (t, img.width() as f32 / img.height() as f32)
}

/// The `sprites-shadow` scene (see [`build`]): an HD-2D party in the
/// shrine court in the late afternoon. Three painted heroes (the knight,
/// the mage, the ranger) stand on the flags with drop shadows along the
/// sun, the ranger's darker; a second knight by the column has none;
/// a spinning rune token drawn from a texture array casts one too; the
/// column and a crate cast their own.
pub fn sprites_shadow_scene() -> RenderScene {
    use crate::kit;
    use og_render::{SpriteArray, SpriteShadow, TextureArrayData, TextureFilter, TextureSource};
    // The flags' top, which the party stands on and the drop shadows lie on.
    const FLAGS: f32 = 0.01;
    let mut scene = RenderScene::new();
    crate::shadow_scenes::court(&mut scene);
    kit::prop(&mut scene, "column").place(&mut scene, Vec3::new(3.2, 0.0, -2.0), 0.3, 3.4);
    kit::prop(&mut scene, "crate").place(&mut scene, Vec3::new(-2.9, 0.0, 0.2), 0.4, 0.9);
    // Each hero at its painting's aspect, 1.7 m tall.
    let knight = painted(&mut scene, "hero-knight");
    let mage = painted(&mut scene, "hero-mage");
    let ranger = painted(&mut scene, "hero-ranger");
    let wheel = scene
        .add_texture_array(TextureArrayData {
            layers: (0..8)
                .map(|k| TextureSource::Rgba(flipbook_frame(k)))
                .collect(),
        })
        .expect("eight frames of one size");
    scene.set_texture_array_filter(wheel, TextureFilter::Nearest);
    let stand = |x: f32, z: f32, (t, aspect): (og_render::TextureHandle, f32)| Sprite {
        texture: Some(t),
        position: Vec3::new(x, FLAGS, z),
        size: Vec2::new(1.7 * aspect, 1.7),
        anchor: Vec2::new(0.5, 0.0),
        ..Default::default()
    };
    let add = |scene: &mut RenderScene, sprite: Sprite, shadow: f32| {
        let i = scene.sprite.sprites.len() as u32;
        scene.add_sprite(sprite);
        if shadow > 0.0 {
            scene.sprite.sprite_shadows.push(SpriteShadow {
                sprite: i,
                opacity: shadow,
                ground: og_render::ShadowGround {
                    height: FLAGS,
                    ..Default::default()
                },
            });
        }
        i
    };
    add(&mut scene, stand(-1.6, 0.6, knight), 0.5);
    add(&mut scene, stand(0.2, -0.4, mage), 0.5);
    add(&mut scene, stand(1.4, 1.2, ranger), 0.8);
    add(&mut scene, stand(2.6, -1.0, knight), 0.0);
    let w = add(
        &mut scene,
        Sprite {
            position: Vec3::new(-0.6, FLAGS, -2.2),
            size: Vec2::splat(1.2),
            anchor: Vec2::new(0.5, 0.0),
            ..Default::default()
        },
        0.6,
    );
    scene.sprite.sprite_arrays.push(SpriteArray {
        sprite: w,
        array: wheel,
        layer: 3,
    });
    // Late afternoon: the sun 22° up in front of the party to the right,
    // so the shadows run long back and to the left across the flags.
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(0.72, 0.4, 0.57));
    scene.lighting.light.cast_shadows = true;
    scene.frame.camera = Camera {
        position: Vec3::new(-0.2, 2.3, 6.2),
        target: Vec3::new(0.2, 0.9, -0.6),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// The `sprites-billboard` scene (see [`build`]): the HD-2D party's
/// shrine court seen from above and to the side. A knight flat in the xy
/// plane for comparison (sheared by the camera's angle); upright (cameraY)
/// oaks and the mage casting sun-facing drop shadows; a heraldic shield
/// sign facing the camera, turned in its plane; two translucent spirit
/// puffs facing the camera, sorted back to front though submitted near
/// first; one oak behind the camera culled; the ranger 2D-lit by a warm
/// lantern glow; and a picture-in-picture view from the side that every
/// billboard faces too.
pub fn billboard_scene() -> RenderScene {
    use crate::kit;
    use og_render::{
        Light2d, Lighting2d, ShadowGround, SpriteFacing, SpriteShadow, View, ViewTarget,
    };
    // The flags' top, which the cards stand on and the shadows lie on.
    const FLAGS: f32 = 0.01;
    let mut scene = RenderScene::new();
    crate::shadow_scenes::court(&mut scene);
    kit::prop(&mut scene, "crate").place(&mut scene, Vec3::new(-1.6, 0.0, 2.2), 0.5, 0.9);
    let toro = kit::prop(&mut scene, "toro");
    toro.place(&mut scene, Vec3::new(-6.3, 0.0, -0.6), 0.3, 1.6);
    let (oak, oak_aspect) = painted(&mut scene, "tree-oak");
    let (sign, sign_aspect) = painted(&mut scene, "sign-shield");
    // A soft puff (white; the sprite's colour tints it).
    let puff = scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let d = (Vec2::new(u - 0.5, v - 0.5).length() * 2.0).min(1.0);
        [255, 255, 255, ((1.0 - d * d) * 255.0) as u8]
    }));
    let (knight, knight_aspect) = painted(&mut scene, "hero-knight");
    let (mage, mage_aspect) = painted(&mut scene, "hero-mage");
    let (ranger, ranger_aspect) = painted(&mut scene, "hero-ranger");
    let stand = |texture, x: f32, z: f32, size: Vec2| Sprite {
        texture: Some(texture),
        position: Vec3::new(x, FLAGS, z),
        size,
        anchor: Vec2::new(0.5, 0.0),
        ..Default::default()
    };
    let add = |scene: &mut RenderScene, sprite: Sprite, facing: SpriteFacing, shadow: f32| {
        let i = scene.sprite.sprites.len();
        scene.add_sprite(sprite);
        scene.sprite.sprite_facing.resize(i, SpriteFacing::None);
        scene.sprite.sprite_facing.push(facing);
        if shadow > 0.0 {
            scene.sprite.sprite_shadows.push(SpriteShadow {
                sprite: i as u32,
                opacity: shadow,
                ground: ShadowGround {
                    height: FLAGS,
                    ..Default::default()
                },
            });
        }
    };
    let hero = |aspect: f32| Vec2::new(1.7 * aspect, 1.7);
    // In the xy plane (for comparison): sheared by the camera's angle.
    add(
        &mut scene,
        stand(knight, -3.4, -0.4, hero(knight_aspect)),
        SpriteFacing::None,
        0.0,
    );
    // Upright, turned about y: two oaks and the mage, with drop shadows
    // cast by their sun-facing cards.
    let oak_size = Vec2::new(3.0 * oak_aspect, 3.0);
    add(
        &mut scene,
        stand(oak, -1.2, -1.6, oak_size),
        SpriteFacing::CameraY,
        0.6,
    );
    add(
        &mut scene,
        stand(oak, 3.6, -3.0, oak_size),
        SpriteFacing::CameraY,
        0.6,
    );
    add(
        &mut scene,
        stand(mage, 0.6, 0.4, hero(mage_aspect)),
        SpriteFacing::CameraY,
        0.5,
    );
    // In the view plane, turned 0.35 rad within it: the sign floats.
    add(
        &mut scene,
        Sprite {
            texture: Some(sign),
            position: Vec3::new(2.6, 1.9, -0.6),
            size: Vec2::new(1.1 * sign_aspect, 1.1),
            rotation: 0.35,
            ..Default::default()
        },
        SpriteFacing::Camera,
        0.0,
    );
    // Two translucent spirit puffs facing the camera, submitted near
    // first: the far red one must draw first, the near blue one over it.
    add(
        &mut scene,
        Sprite {
            texture: Some(puff),
            position: Vec3::new(3.7, 3.4, 3.1),
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
            position: Vec3::new(2.4, 2.6, 1.2),
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
        stand(oak, 0.0, 30.0, oak_size),
        SpriteFacing::CameraY,
        0.0,
    );
    // 2D-lit (layer 1 is the lit range): the ranger by the lantern's warm
    // glow, darkened to the ambient away from it.
    add(
        &mut scene,
        Sprite {
            layer: 1,
            ..stand(ranger, -5.0, 0.0, hero(ranger_aspect))
        },
        SpriteFacing::CameraY,
        0.0,
    );
    scene.sprite.lighting_2d = Some(Lighting2d {
        ambient: Vec3::splat(0.25),
        min_layer: 1,
        max_layer: 1,
        resolution: 1.0,
        shadow_angles: 1024,
    });
    scene.sprite.lights_2d.push(Light2d {
        position: Vec2::new(-5.7, 0.1),
        color: Vec3::new(1.0, 0.7, 0.3),
        intensity: 2.0,
        radius: 1.1,
        cast_shadows: false,
        ..Default::default()
    });
    // Afternoon: the sun 40° up, over the camera's right shoulder.
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(0.7, 0.75, 0.35));
    scene.lighting.light.cast_shadows = true;
    let exposure = Exposure::Manual { ev: 0.0 };
    scene.frame.camera = Camera {
        position: Vec3::new(6.2, 3.9, 6.0),
        target: Vec3::new(-0.4, 1.0, -0.9),
        far: 2500.0,
        exposure,
        ..Default::default()
    };
    // From the side: every billboard turns to this camera too (the flat
    // knight is seen nearly edge on).
    scene.frame.views.push(View {
        camera: Camera {
            position: Vec3::new(-10.0, 2.2, -0.8),
            target: Vec3::new(0.0, 1.0, -0.8),
            far: 2500.0,
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
    use og_render::{ShadowGround, SpriteShadow, Terrain, TerrainData, TerrainLayer};
    let mut scene = RenderScene::new();
    // 48 m of grassy hills sampled every half metre (the clipmap's finest
    // level), the kit's hills beyond them.
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
        holes: None,
    };
    let (grass, grass_normal) = crate::kit::layer_maps(&mut scene, "grass");
    let handle = scene.add_terrain(data.clone());
    scene.world.terrains.push(Terrain {
        layers: vec![TerrainLayer {
            albedo: Some(grass),
            normal: Some(grass_normal),
            color: Vec3::new(0.72, 1.05, 0.5),
            tile: 3.5,
            anti_tiling: 0.5,
            ..Default::default()
        }],
        ..Terrain::new(handle)
    });
    crate::kit::hills(&mut scene, None);
    // A plank watch platform whose top is 1.9 m up, and a rubble wall
    // casting the sun's shadow.
    let deck_top = 1.9;
    let planks = crate::kit::surface_tinted(&mut scene, "planks", [0.62, 0.5, 0.4]);
    let deck = scene.add_mesh(crate::kit::block(
        Vec3::new(1.5, -0.6, -0.7),
        Vec3::new(3.9, deck_top, 1.1),
        1.2,
    ));
    scene.add_instance(deck, planks, Mat4::IDENTITY);
    let rubble = crate::kit::surface_tinted(&mut scene, "stone", [0.75, 0.7, 0.62]);
    let wall = scene.add_mesh(crate::kit::block(
        Vec3::new(-3.5, -0.4, -2.4),
        Vec3::new(-0.9, 3.6, -2.0),
        1.5,
    ));
    scene.add_instance(wall, rubble, Mat4::IDENTITY);
    let figures = [
        painted(&mut scene, "hero-knight"),
        painted(&mut scene, "hero-mage"),
        painted(&mut scene, "hero-ranger"),
    ];
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
        let (texture, aspect) = figures[figure % figures.len()];
        let sprite = Sprite {
            texture: Some(texture),
            position: Vec3::new(x, height, z),
            size: Vec2::new(1.6 * aspect, 1.6),
            anchor: Vec2::new(0.5, 0.0),
            ..Default::default()
        };
        let ground = if deck {
            ShadowGround::level(deck_top)
        } else {
            ShadowGround::fit(|x, z| data.height(Vec2::new(x, z)), &sprite, sun)
        };
        let i = scene.sprite.sprites.len() as u32;
        scene.add_sprite(sprite);
        scene.sprite.sprite_shadows.push(SpriteShadow {
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
    // Afternoon: the sun 49° up behind the hills to the right.
    crate::sky_scenes::physical_light_under(&mut scene, sun);
    scene.lighting.light.cast_shadows = true;
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 5.6, 8.5),
        target: Vec3::new(0.0, 0.8, -0.2),
        far: 2500.0,
        exposure: Exposure::Manual { ev: 0.0 },
        ..Default::default()
    };
    scene
}

/// Layer `k` of the `sprites-array` smooth array: a wisp's soft glow,
/// orange, green or blue, fading out from a white-hot core.
fn glow_layer(k: usize) -> og_render::TextureSource {
    og_render::TextureSource::Rgba(texture(64, ColorSpace::Srgb, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let a = ((1.0 - d) * 1.5).clamp(0.0, 1.0).powf(1.3);
        let core = ((0.3 - d) / 0.3).clamp(0.0, 1.0) * 0.85;
        let rgb = [[255, 120, 60], [80, 255, 140], [90, 140, 255]][k]
            .map(|c: u8| (c as f32 + (255.0 - c as f32) * core) as u8);
        [rgb[0], rgb[1], rgb[2], (a * 255.0) as u8]
    }))
}

/// One layer of the vault's coin flipbook: a 32 x 32 pixel-art gold coin
/// `frame` eighths of a turn about its upright axis (0 face on, 2 and 6
/// edge on, 4 its back). The face carries a crescent opening to the right
/// and a shine on its upper left, so a coin drawn flipped reads mirrored.
pub fn coin_frame(frame: u32) -> TextureData {
    let turn = frame as f32 * std::f32::consts::TAU / 8.0;
    let (squash, front) = (turn.cos().abs().max(0.14), turn.cos() > 0.01);
    let mut t = texture(32, ColorSpace::Srgb, |u, v| {
        // The coin's face coordinates: unit circle, y up.
        let (x, y) = ((u - 0.5) / (0.44 * squash), (0.5 - v) / 0.44);
        let r = (x * x + y * y).sqrt();
        if r > 1.0 {
            return [0, 0, 0, 0];
        }
        if squash < 0.2 {
            // Edge on: the milled rim, lit from the left.
            return if x < -0.2 {
                [255, 236, 150, 255]
            } else if ((y * 9.0).floor() as i32).rem_euclid(2) == 0 {
                [196, 132, 30, 255]
            } else {
                [168, 104, 22, 255]
            };
        }
        let rim = r > 0.8;
        let shine = x < -0.25 && y > 0.25 && (0.55..0.8).contains(&r);
        let emblem = if front {
            let inner = ((x + 0.12).powi(2) + y.powi(2)).sqrt() < 0.5;
            let bite = ((x - 0.1).powi(2) + (y - 0.08).powi(2)).sqrt() < 0.42;
            inner && !bite
        } else {
            x.abs() + y.abs() < 0.38
        };
        let face = if front {
            [250, 200, 64]
        } else {
            [222, 162, 44]
        };
        let c = if rim {
            if x < 0.0 && y > 0.0 {
                [255, 226, 120]
            } else {
                [176, 112, 22]
            }
        } else if shine {
            [255, 246, 196]
        } else if emblem {
            [170, 104, 20]
        } else {
            face
        };
        [c[0], c[1], c[2], 255]
    });
    t.mips = Mips::None;
    t
}

/// The vault's coin flipbook as a nearest-filtered texture array.
fn coin_flipbook(scene: &mut RenderScene) -> og_render::TextureArrayHandle {
    use og_render::{TextureArrayData, TextureFilter, TextureSource};
    let coins = scene
        .add_texture_array(TextureArrayData {
            layers: (0..8).map(|k| TextureSource::Rgba(coin_frame(k))).collect(),
        })
        .expect("eight frames of one size");
    scene.set_texture_array_filter(coins, TextureFilter::Nearest);
    coins
}

/// A painted sprite `name` standing on `foot` (its base's middle),
/// `height` tall, on `layer`.
fn stand_painted(scene: &mut RenderScene, name: &str, foot: Vec2, height: f32, layer: i32) {
    let (texture, aspect) = painted(scene, name);
    scene.add_sprite(Sprite {
        texture: Some(texture),
        position: foot.extend(0.0),
        size: Vec2::new(height * aspect, height),
        anchor: Vec2::new(0.5, 0.0),
        layer,
        ..Default::default()
    });
}

/// The `sprites-array-fx` scene (see [`build`]): the `sprites-array`
/// vault at night, its coin flipbook and glow array through the paths
/// that scene leaves out. Up on a stone shelf four flipbook coins glow
/// (PostEffects glow), each brighter than the last. Bottom left a
/// cauldron's brew throws round, glowing, added sparks whose layers of the
/// smooth array are picked at random; bottom right an open chest bursts
/// coins, a sprite-mode emitter stepping each through the flipbook's eight
/// layers over its life. The HUD, top left: four overlay coins of the
/// flipbook's layers 0, 2, 4 and 6 (one draw) beside a plain chest icon.
pub fn sprite_array_fx_scene() -> RenderScene {
    use og_render::{
        Burst, EmitterShape, Glow, Overlay, ParticleBlend, ParticleEmitter, ParticleRender,
        ParticleSystem, SheetMode, SpriteArray, SpriteFx, SpriteSheet, TextureArrayData,
    };
    let mut scene = RenderScene::new();
    let center = Vec2::new(0.0, -1.0);
    crate::atmo_scenes::backdrop(
        &mut scene,
        "sprites2d/vault.jpg",
        center,
        12.0,
        [0.42, 0.44, 0.56, 1.0],
    );
    let flipbook = coin_flipbook(&mut scene);
    let glows = scene
        .add_texture_array(TextureArrayData {
            layers: (0..3).map(glow_layer).collect(),
        })
        .expect("three layers of one size");
    // The shelf under the glowing coins: a plain run on layer 0.
    let stone = crate::kit::picture(&mut scene, "sprites2d/vault-block.png", true);
    crate::atmo_scenes::blocks(
        &mut scene,
        stone,
        Vec2::new(-6.6, 0.7),
        Vec2::new(6.6, 1.7),
        1.0,
        [0.55, 0.56, 0.66, 1.0],
        0,
    );
    // Four glowing flipbook coins: one draw, and one glow draw.
    for (n, k) in [1u32, 3, 5, 7].into_iter().enumerate() {
        let i = scene.sprite.sprites.len() as u32;
        scene.add_sprite(Sprite {
            position: Vec3::new(-4.5 + n as f32 * 3.0, 2.5, 0.0),
            size: Vec2::splat(1.6),
            layer: 1,
            ..Default::default()
        });
        scene.sprite.sprite_arrays.push(SpriteArray {
            sprite: i,
            array: flipbook,
            layer: k,
        });
        scene.sprite.sprite_fx.push(SpriteFx {
            sprite: i,
            glow: 0.25 + n as f32 * 0.25,
            ..Default::default()
        });
    }
    // The cauldron and the chest the emitters stand in, drawn over their
    // particles so the sparks and coins rise out of them.
    stand_painted(&mut scene, "cauldron", Vec2::new(-4.0, -6.9), 2.5, 3);
    stand_painted(&mut scene, "chest", Vec2::new(4.0, -6.9), 2.3, 3);
    // Round, glowing, added sparks from the smooth array, a random layer each.
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
        blend: Some(ParticleBlend::Additive),
        render: ParticleRender::Sprite { layer: 2 },
        glow: 0.6,
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        1,
        motes,
        Mat4::from_translation(Vec3::new(-4.0, -4.7, 0.0)),
        5,
        0,
    ));
    // The flipbook's eight layers over each coin's life, alpha blended.
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
        size: [0.8, 0.8],
        gravity: Vec3::new(0.0, -2.5, 0.0),
        array: Some(flipbook),
        sheet: SpriteSheet {
            mode: SheetMode::OverLife { cycles: 1.0 },
            ..Default::default()
        },
        blend: Some(ParticleBlend::Alpha),
        render: ParticleRender::Sprite { layer: 2 },
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        2,
        wheels,
        Mat4::from_translation(Vec3::new(4.0, -5.4, 0.0)),
        6,
        0,
    ));
    // The HUD: four flipbook coins (one draw) and a plain chest icon.
    let (chest, aspect) = painted(&mut scene, "chest");
    let mut overlay = Overlay::new(1280.0, 720.0, 0.0);
    for (n, k) in [0u32, 2, 4, 6].into_iter().enumerate() {
        let i = overlay.sprites.len() as u32;
        overlay.sprites.push(Sprite {
            position: Vec3::new(24.0 + n as f32 * 56.0, 24.0, 0.0),
            size: Vec2::splat(48.0),
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
        texture: Some(chest),
        position: Vec3::new(24.0 + 4.0 * 56.0 + 8.0, 20.0, 0.0),
        size: Vec2::new(56.0 * aspect, 56.0),
        anchor: Vec2::ZERO,
        ..Default::default()
    });
    scene.ui.overlay = Some(overlay);
    scene.look.post.glow = Some(Glow { intensity: 0.8 });
    scene.frame.camera = Camera::orthographic_2d(center, 12.0);
    scene.lighting.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    scene.look.post.tonemap = Tonemap::None;
    scene
}

/// The `sprites-array` scene (see [`build`]): a dwarven treasure vault in
/// a 2D platformer. Two rows of spinning coins hang before the back wall,
/// every frame of the coin flipbook in order; the lower row flipped left
/// to right and tipped a little, its crescents opening the other way.
/// Will-o'-wisps from the smooth array, added, hover over a stone dais
/// (a plain run on layer 0), its three layers in turn.
pub fn sprite_array_scene() -> RenderScene {
    use og_render::{SpriteArray, SpriteBlend, SpriteFx, TextureArrayData};
    let mut scene = RenderScene::new();
    let center = Vec2::new(0.0, -1.0);
    crate::atmo_scenes::backdrop(
        &mut scene,
        "sprites2d/vault.jpg",
        center,
        12.0,
        [0.78, 0.8, 0.86, 1.0],
    );
    let flipbook = coin_flipbook(&mut scene);
    let glows = scene
        .add_texture_array(TextureArrayData {
            layers: (0..3).map(glow_layer).collect(),
        })
        .expect("three layers of one size");
    let push = |scene: &mut RenderScene,
                sprite: Sprite,
                array: Option<(og_render::TextureArrayHandle, u32)>| {
        let i = scene.sprite.sprites.len() as u32;
        scene.add_sprite(sprite);
        if let Some((array, layer)) = array {
            scene.sprite.sprite_arrays.push(SpriteArray {
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
                    size: Vec2::splat(1.6),
                    rotation: row as f32 * 0.3,
                    uv_rect,
                    layer: 1,
                    ..Default::default()
                },
                Some((flipbook, k)),
            );
        }
    }
    // The dais under the wisps: a plain run on layer 0.
    let stone = crate::kit::picture(&mut scene, "sprites2d/vault-block.png", true);
    crate::atmo_scenes::blocks(
        &mut scene,
        stone,
        Vec2::new(-8.4, -6.4),
        Vec2::new(8.4, -4.6),
        1.8,
        [0.86, 0.86, 0.9, 1.0],
        0,
    );
    // The smooth array added over the dais, its three layers in turn.
    for k in 0..6u32 {
        let i = push(
            &mut scene,
            Sprite {
                position: Vec3::new(-6.25 + k as f32 * 2.5, -4.6, 0.0),
                size: Vec2::splat(2.4),
                color: [1.0, 1.0, 1.0, 0.9],
                layer: 2,
                ..Default::default()
            },
            Some((glows, k % 3)),
        );
        scene.sprite.sprite_fx.push(SpriteFx {
            sprite: i,
            blend: SpriteBlend::Additive,
            ..Default::default()
        });
    }
    scene.frame.camera = Camera::orthographic_2d(center, 12.0);
    scene.lighting.environment.background = Background::Color(Vec3::new(0.02, 0.02, 0.03));
    scene.look.post.tonemap = Tonemap::None;
    scene
}

/// Ticks each frame of the coin flipbook holds when it spins (12 a second).
const COIN_TICKS: u32 = 5;

/// `sprites-array`'s frames after its own (og-gfx --sequence): every coin
/// spins through the flipbook from the frame it shows, a turn in eight
/// frames of 5 ticks (two seconds hold three turns, so a clip of them
/// loops). The scene's own frame is as built.
pub fn spin_coins(scene: &mut RenderScene, frame: u32) {
    let turn = frame / COIN_TICKS;
    // The two rows' sixteen coins come first.
    for (j, a) in scene.sprite.sprite_arrays.iter_mut().take(16).enumerate() {
        a.layer = (j as u32 + turn) % 8;
    }
}

/// Moves every sprite along the spiral (`frame` 0 is the reference frame).
pub fn move_sprites(scene: &mut RenderScene, frame: u32) {
    let n = scene.sprite.sprites.len();
    for (i, s) in scene.sprite.sprites.iter_mut().enumerate() {
        let (p, a) = sprite_at(i, n, frame);
        s.position = p.extend(0.0);
        s.rotation = a;
    }
}

/// R11's 2D lighting cost: 64 shadow-casting lights (points and spots)
/// over a 64 x 36 room of 128 occluders (boxes, turned boxes and concave
/// polygons), each shadow row traced at `angles`; with `normals`, the 128
/// crate sprites on the occluders carry an auto-style dome normal map, so
/// the frame also draws the light directions. The lights drift each frame
/// ([`drift_lights_2d`]). Perf only (no reference).
pub fn lights_2d_scene(angles: u32, normals: bool) -> RenderScene {
    use og_render::{Light2d, Light2dKind, Lighting2d, Occluder2d};
    let mut scene = RenderScene::new();
    scene.add_sprite(Sprite {
        size: Vec2::new(64.0, 36.0),
        color: [0.7, 0.7, 0.7, 1.0],
        ..Default::default()
    });
    let crate_tex = scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let edge = u.min(v).min(1.0 - u).min(1.0 - v) < 0.08;
        if edge {
            [90, 60, 30, 255]
        } else {
            [170, 120, 70, 255]
        }
    }));
    if normals {
        let map = scene.add_texture(texture(64, ColorSpace::Linear, |u, v| {
            let (x, y) = (u * 2.0 - 1.0, 1.0 - v * 2.0);
            let n = Vec3::new(x * 0.6, y * 0.6, 1.0).normalize();
            let enc = |c: f32| ((c * 0.5 + 0.5) * 255.0).round() as u8;
            [enc(n.x), enc(n.y), enc(n.z), 255]
        }));
        scene.set_texture_normal(crate_tex, Some(map));
    }
    for i in 0..128 {
        let (col, row) = ((i % 16) as f32, (i / 16) as f32);
        let center = Vec2::new(-30.0 + col * 4.0, -15.5 + row * 4.4);
        let occluder = match i % 3 {
            0 => Occluder2d {
                center,
                half_size: Vec2::new(0.6, 0.6),
                ..Default::default()
            },
            1 => Occluder2d {
                center,
                half_size: Vec2::new(0.9, 0.3),
                rotation: 0.4 + i as f32 * 0.1,
                ..Default::default()
            },
            _ => Occluder2d::polygon(
                center,
                i as f32 * 0.3,
                &[
                    (-0.7, -0.5),
                    (0.7, -0.5),
                    (0.7, 0.6),
                    (0.2, 0.6),
                    (0.2, 0.0),
                    (-0.2, 0.0),
                    (-0.2, 0.6),
                    (-0.7, 0.6),
                ]
                .map(|(x, y)| Vec2::new(x, y)),
            ),
        };
        scene.sprite.occluders_2d.push(occluder);
        scene.add_sprite(Sprite {
            texture: Some(crate_tex),
            position: center.extend(0.0),
            size: Vec2::splat(1.2),
            layer: 1,
            ..Default::default()
        });
    }
    scene.sprite.lighting_2d = Some(Lighting2d {
        ambient: Vec3::splat(0.08),
        shadow_angles: angles,
        ..Default::default()
    });
    for i in 0..64 {
        let t = i as f32 / 64.0;
        scene.sprite.lights_2d.push(Light2d {
            position: Vec2::new(-30.0 + 60.0 * t, 14.0 * (t * 37.0).sin()),
            color: Vec3::new(1.0, 0.6 + 0.4 * (t * 9.0).sin().abs(), 0.5 + 0.5 * t),
            radius: 6.0 + 4.0 * (t * 13.0).cos().abs(),
            intensity: 0.8,
            kind: if i % 4 == 3 {
                Light2dKind::Spot
            } else {
                Light2dKind::Point
            },
            direction: Vec2::from_angle(t * 20.0),
            ..Default::default()
        });
    }
    scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 18.0);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    scene
}

/// Moves [`lights_2d_scene`]'s lights a little each frame, so every frame
/// traces its shadow rows anew.
pub fn drift_lights_2d(scene: &mut RenderScene, frame: u32) {
    for (i, l) in scene.sprite.lights_2d.iter_mut().enumerate() {
        let a = frame as f32 * 0.02 + i as f32;
        l.position += Vec2::new(a.cos(), a.sin()) * 0.05;
    }
}
