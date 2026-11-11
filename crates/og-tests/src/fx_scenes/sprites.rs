//! Lit 2D sprite and tilemap scenes.

use super::*;

// ---------------------------------------------------------------------------
// 2D: lights and soft shadows from sprite occluders, glow, 2D particles,
// a sprite hit flash.
// ---------------------------------------------------------------------------

pub(crate) fn tile_texture(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let edge = (u.min(1.0 - u).min(v).min(1.0 - v) * 16.0).min(1.0);
        let n = fbm(u * 5.0, v * 5.0, 2);
        let c = (70.0 + 50.0 * n) * (0.6 + 0.4 * edge);
        [c as u8, (c * 0.95) as u8, (c * 0.9) as u8, 255]
    }))
}

pub(crate) fn crate_texture(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        let border = u.min(1.0 - u).min(v).min(1.0 - v) < 0.1;
        let diag = ((u - v).abs() < 0.08) || ((u + v - 1.0).abs() < 0.08);
        let n = fbm(u * 3.0, v * 9.0, 4);
        let (r, g, b) = if border || diag {
            (120.0, 80.0, 40.0)
        } else {
            (170.0 + 40.0 * n, 120.0 + 30.0 * n, 60.0)
        };
        [r as u8, g as u8, b as u8, 255]
    }))
}

fn hero_texture(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(64, ColorSpace::Srgb, |u, v| {
        // A round-bodied character with eyes, v down.
        let body = ((u - 0.5).powi(2) / 0.16 + (v - 0.6).powi(2) / 0.13) < 1.0;
        let head = ((u - 0.5).powi(2) + (v - 0.28).powi(2)).sqrt() < 0.2;
        let eye = ((u - 0.43).powi(2) + (v - 0.26).powi(2)).sqrt() < 0.035
            || ((u - 0.57).powi(2) + (v - 0.26).powi(2)).sqrt() < 0.035;
        if eye {
            [20, 20, 30, 255]
        } else if head {
            [250, 210, 170, 255]
        } else if body {
            [60, 120, 230, 255]
        } else {
            [0, 0, 0, 0]
        }
    }))
}

fn coin_texture(scene: &mut RenderScene) -> TextureHandle {
    scene.add_texture(texture(32, ColorSpace::Srgb, |u, v| {
        let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt() * 2.0;
        let a = 1.0 - smooth(0.85, 1.0, d);
        let shine = 1.0
            - smooth(
                0.0,
                0.5,
                ((u - 0.35).powi(2) + (v - 0.35).powi(2)).sqrt() * 3.0,
            );
        [
            255,
            (200.0 + 55.0 * shine) as u8,
            (60.0 + 150.0 * shine) as u8,
            (a * 255.0) as u8,
        ]
    }))
}

/// A night dungeon room in 2D: a torch (2D fire particles that glow) and a
/// cold magic light cast soft shadows from crates, a hit-flashed hero, glowing
/// coins (order-independent glow, one of them under a crate), and an unlit UI
/// label on a layer above the lit range.
pub fn fx_2d_scene() -> Result<RenderScene, String> {
    let mut scene = RenderScene::new();
    let tile = tile_texture(&mut scene);
    let crate_tex = crate_texture(&mut scene);
    let hero = hero_texture(&mut scene);
    let coin = coin_texture(&mut scene);
    let flame = flame_sheet(&mut scene);
    for y in -7..7 {
        for x in -12..12 {
            scene.add_sprite(Sprite {
                texture: Some(tile),
                position: Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0),
                size: Vec2::ONE,
                layer: 0,
                ..Default::default()
            });
        }
    }
    let crates = [
        (Vec2::new(-3.0, 0.5), 1.2, 0.0),
        (Vec2::new(-1.6, -1.8), 1.0, 0.3),
        (Vec2::new(2.2, 1.6), 1.4, -0.2),
        (Vec2::new(4.0, -1.2), 0.9, 0.5),
        (Vec2::new(0.6, 2.8), 0.8, 0.1),
    ];
    for (p, s, r) in crates {
        let sprite = Sprite {
            texture: Some(crate_tex),
            position: p.extend(0.0),
            size: Vec2::splat(s),
            rotation: r,
            layer: 2,
            ..Default::default()
        };
        scene
            .sprite
            .occluders_2d
            .push(Occluder2d::from_sprite(&sprite, 0.02));
        scene.add_sprite(sprite);
    }
    // The hero, hit: a red flash.
    let hero_index = scene.sprite.sprites.len() as u32;
    scene.add_sprite(Sprite {
        texture: Some(hero),
        position: Vec3::new(0.8, -0.6, 0.0),
        size: Vec2::splat(1.3),
        layer: 3,
        ..Default::default()
    });
    scene.sprite.sprite_fx.push(SpriteFx {
        sprite: hero_index,
        flash: [1.0, 1.0, 1.0, 0.6],
        ..Default::default()
    });
    // Glowing coins (the last one under a crate: the glow still shows).
    for (i, p) in [
        Vec2::new(-4.6, -2.6),
        Vec2::new(-4.0, -2.9),
        Vec2::new(5.5, 2.4),
        Vec2::new(3.4, -3.2),
        Vec2::new(2.2, 1.6),
    ]
    .iter()
    .enumerate()
    {
        let index = scene.sprite.sprites.len() as u32;
        scene.add_sprite(Sprite {
            texture: Some(coin),
            position: p.extend(0.0),
            size: Vec2::splat(0.45),
            color: [1.4, 1.2, 0.6, 1.0],
            layer: 1,
            ..Default::default()
        });
        scene.sprite.sprite_fx.push(SpriteFx {
            sprite: index,
            glow: 1.5 + i as f32 * 0.2,
            ..Default::default()
        });
    }
    // The torch: 2D flames that glow, in the sprite batch.
    let torch = scene.add_emitter(ParticleEmitter {
        max_particles: 120,
        shape: EmitterShape::Cone {
            angle_degrees: 10.0,
            radius: 0.12,
        },
        rate: 60.0,
        lifetime: [0.5, 0.8],
        speed: [0.6, 1.0],
        size: [0.5, 0.7],
        size_over_life: Curve::new(&[(0.0, 0.7), (0.3, 1.0), (1.0, 0.3)]),
        color_over_life: ColorCurve::new(&[
            (0.0, [1.0, 1.0, 1.0, 0.0]),
            (0.15, [1.0, 1.0, 1.0, 1.0]),
            (1.0, [1.0, 0.5, 0.2, 0.0]),
        ]),
        color: [1.6, 1.3, 1.0, 1.0],
        gravity: Vec3::new(0.0, 1.0, 0.0),
        noise: ParticleNoise {
            strength: 1.0,
            frequency: 2.0,
            scroll: 1.0,
        },
        texture: Some(flame),
        sheet: SpriteSheet {
            columns: 4,
            rows: 4,
            frames: 16,
            mode: SheetMode::OverLife { cycles: 1.0 },
        },
        render: ParticleRender::Sprite { layer: 4 },
        glow: 0.8,
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        1,
        torch,
        Mat4::from_translation(Vec3::new(-5.5, 2.2, 0.0)),
        3,
        0,
    ));
    // Magic motes (untextured: soft dots), additive.
    let motes = scene.add_emitter(ParticleEmitter {
        max_particles: 80,
        shape: EmitterShape::Sphere {
            radius: 0.6,
            surface: false,
        },
        rate: 30.0,
        lifetime: [1.5, 2.5],
        speed: [0.1, 0.3],
        size: [0.08, 0.16],
        size_over_life: Curve::new(&[(0.0, 0.0), (0.3, 1.0), (1.0, 0.0)]),
        color: [0.6, 0.9, 1.6, 1.0],
        gravity: Vec3::new(0.0, 0.2, 0.0),
        render: ParticleRender::Sprite { layer: 4 },
        glow: 1.0,
        ..Default::default()
    });
    scene.effects.particles.push(ParticleSystem::new(
        2,
        motes,
        Mat4::from_translation(Vec3::new(3.6, -3.4, 0.0)),
        4,
        0,
    ));
    scene.sprite.lighting_2d = Some(Lighting2d {
        ambient: Vec3::new(0.06, 0.06, 0.1),
        min_layer: 0,
        max_layer: 50,
        resolution: 0.5,
        shadow_angles: 1024,
    });
    scene.sprite.lights_2d = vec![
        Light2d {
            position: Vec2::new(-5.5, 2.4),
            color: Vec3::new(1.0, 0.65, 0.35),
            intensity: 2.2,
            radius: 9.0,
            falloff: 1.6,
            source_radius: 0.3,
            cast_shadows: true,
            ..Default::default()
        },
        Light2d {
            position: Vec2::new(3.6, -3.2),
            color: Vec3::new(0.35, 0.6, 1.0),
            intensity: 1.6,
            radius: 6.5,
            falloff: 1.4,
            source_radius: 0.5,
            cast_shadows: true,
            ..Default::default()
        },
        Light2d {
            position: Vec2::new(0.8, -0.2),
            color: Vec3::new(0.9, 0.9, 1.0),
            intensity: 0.5,
            radius: 2.2,
            falloff: 2.0,
            source_radius: 0.1,
            cast_shadows: false,
            ..Default::default()
        },
    ];
    let font = crate::scenes::load_font(&mut scene)?;
    scene.sprite.texts.push(Text {
        position: Vec3::new(-9.6, 5.2, 0.0),
        size: 0.6,
        color: [1.0, 0.95, 0.8, 1.0],
        layer: 100,
        ..Text::new(font, "Dungeon 1   coins x3")
    });
    scene.look.post.glow = Some(Glow { intensity: 1.0 });
    scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 12.0);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    Ok(scene)
}

// ---------------------------------------------------------------------------
// A lit tilemap.
// ---------------------------------------------------------------------------

/// The four 16 px tiles of the lit cave: rock, rock with a mossy top, a
/// brick back wall and a starry night sky (64 x 16, nearest).
fn cave_tiles(scene: &mut RenderScene) -> TextureHandle {
    let mut rgba = Vec::with_capacity(64 * 16 * 4);
    for y in 0..16i32 {
        for x in 0..64i32 {
            let (tile, u) = (x / 16, x % 16);
            let n = hash2(u, y, tile as u32 * 7 + 1);
            let c: [f32; 3] = match tile {
                // Rock: grey-brown speckle, a darker crack on the diagonal.
                0 | 1 => {
                    let crack = (u + y) % 11 == 0 && n > 0.3;
                    let k = if crack { 0.55 } else { 0.85 + 0.3 * n };
                    if tile == 1 && y < 3 + (u % 3 == 0) as i32 {
                        [0.30 + 0.15 * n, 0.55 + 0.2 * n, 0.22]
                    } else {
                        [0.52 * k, 0.46 * k, 0.40 * k]
                    }
                }
                // Bricks: offset courses of 8 x 4 with dark mortar.
                2 => {
                    let course = y / 4;
                    let mortar = y % 4 == 3 || (u + course % 2 * 4) % 8 == 7;
                    if mortar {
                        [0.16, 0.12, 0.11]
                    } else {
                        let k = 0.85 + 0.25 * n;
                        [0.50 * k, 0.30 * k, 0.24 * k]
                    }
                }
                // The night sky through a window: blue, a few stars.
                _ => {
                    if n > 0.985 {
                        [0.95, 0.95, 0.85]
                    } else {
                        let t = y as f32 / 15.0;
                        [0.10 + 0.08 * t, 0.16 + 0.12 * t, 0.42 + 0.15 * t]
                    }
                }
            };
            rgba.extend(c.iter().map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8));
            rgba.push(255);
        }
    }
    let t = scene.add_texture(TextureData {
        width: 64,
        height: 16,
        rgba,
        color_space: ColorSpace::Srgb,
        mips: Mips::None,
    });
    scene.set_texture_filter(t, og_render::TextureFilter::Nearest);
    t
}

/// One tile layer of 1-unit cells: `cell(x, y)` (x right, y up from the
/// bottom-left of a 32 x 18 map centred on the origin) is a tile of
/// [`cave_tiles`] or none.
fn cave_layer(
    texture: TextureHandle,
    layer: i32,
    lit: bool,
    cell: impl Fn(i32, i32) -> Option<u32>,
) -> og_render::TileLayer {
    let mut quads = Vec::new();
    for y in 0..18 {
        for x in 0..32 {
            if let Some(t) = cell(x, y) {
                let u = t as f32 * 0.25;
                quads.push(og_render::TileQuad {
                    min: Vec2::new(x as f32, y as f32),
                    size: Vec2::ONE,
                    uv: [u, 0.0, u + 0.25, 1.0],
                });
            }
        }
    }
    og_render::TileLayer {
        texture,
        chunks: std::sync::Arc::new(vec![og_render::TileChunk {
            min: Vec2::ZERO,
            max: Vec2::new(32.0, 18.0),
            quads,
            animated: Vec::new(),
        }]),
        origin: Vec3::new(-16.0, -9.0, 0.0),
        layer,
        color: [1.0; 4],
        repeat: [None, None],
        time: 0.0,
        lit,
        light_mask: 1,
    }
}

/// A dark cave drawn as a tilemap (hollowbell FINDINGS #9): rock, a brick
/// back wall and a ledge in three tile layers inside 2D lighting's range,
/// an ambient of 0.04, a warm torch on the left and a cold crystal over the
/// ledge, both casting shadows: the pillar's occluder throws the torch's
/// shadow across the back wall to its right, the ledge's the crystal's
/// below it. A window shows a night sky through the wall: its layer is in
/// the lit range but `lit` false, so it keeps its authored colour. A figure
/// on the floor is a lit sprite of the same range.
pub fn tilemap_lit_scene() -> RenderScene {
    let mut scene = RenderScene::new();
    let tiles = cave_tiles(&mut scene);
    let window = |x: i32, y: i32| (5..9).contains(&x) && (10..13).contains(&y);
    let rock = |x: i32, y: i32| {
        !(3..15).contains(&y)
            || !(2..30).contains(&x)
            || ((15..17).contains(&x) && y < 11)
            || ((22..27).contains(&x) && y == 7)
    };
    // The sky, behind the wall's window: unlit.
    scene
        .sprite
        .tile_layers
        .push(cave_layer(tiles, -21, false, |x, y| {
            window(x, y).then_some(3)
        }));
    // The back wall, with the window cut out.
    scene
        .sprite
        .tile_layers
        .push(cave_layer(tiles, -20, true, |x, y| {
            (!window(x, y) && !rock(x, y)).then_some(2)
        }));
    // The rock: mossy where open air is above it.
    scene
        .sprite
        .tile_layers
        .push(cave_layer(tiles, -19, true, |x, y| {
            rock(x, y).then(|| if rock(x, y + 1) || y >= 15 { 0 } else { 1 })
        }));
    // A figure on the floor, lit as the tiles are.
    scene.add_sprite(Sprite {
        position: Vec3::new(-5.0, -6.0, 0.0),
        size: Vec2::new(0.8, 1.6),
        anchor: Vec2::new(0.5, 0.0),
        color: [0.85, 0.82, 0.75, 1.0],
        layer: -18,
        ..Default::default()
    });
    scene.sprite.lighting_2d = Some(Lighting2d {
        ambient: Vec3::splat(0.04),
        min_layer: -30,
        max_layer: 50,
        resolution: 0.5,
        shadow_angles: 1024,
    });
    scene.sprite.lights_2d = vec![
        Light2d {
            position: Vec2::new(-10.0, -4.0),
            color: Vec3::new(1.0, 0.62, 0.3),
            intensity: 1.5,
            radius: 11.0,
            falloff: 1.8,
            source_radius: 0.3,
            cast_shadows: true,
            ..Default::default()
        },
        Light2d {
            position: Vec2::new(8.5, 0.6),
            color: Vec3::new(0.4, 0.7, 1.0),
            intensity: 1.3,
            radius: 7.0,
            falloff: 1.6,
            source_radius: 0.15,
            cast_shadows: true,
            ..Default::default()
        },
    ];
    // The pillar (x 15..17, y 3..11) and the ledge (x 22..27, y 7), in world
    // units round the origin.
    scene.sprite.occluders_2d = vec![
        Occluder2d {
            center: Vec2::new(0.0, -2.0),
            half_size: Vec2::new(1.0, 4.0),
            rotation: 0.0,
            ..Default::default()
        },
        Occluder2d {
            center: Vec2::new(8.5, -1.5),
            half_size: Vec2::new(2.5, 0.5),
            rotation: 0.0,
            ..Default::default()
        },
    ];
    scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 18.0);
    scene.lighting.environment.background = Background::Color(Vec3::ZERO);
    scene.look.post.tonemap = Tonemap::None;
    scene
}
