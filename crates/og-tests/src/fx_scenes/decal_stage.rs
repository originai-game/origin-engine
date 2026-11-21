//! `decals`: the corner of a shrine court at dusk, where something struck.
//! A raking low sun crosses a coped rubble wall and the flags before it;
//! every mark on them is a projected decal: an impact crater in the wall
//! with cracks running out of it (colour and a normal map, so the raking
//! light catches its rim), a scorch blast on the flags beneath, a rune
//! circle glowing blue (emissive, with a small light of its own over it),
//! a rain puddle (a wet decal: the flags darkened and flattened under it,
//! glossy enough that it mirrors the lit stone lantern through SSR) and a
//! moss stain projected diagonally into the corner, covering the wall's
//! foot and the ground at once (the angle fade). The measured fixture is
//! [`super::decals_scene`].

use super::*;

/// The stage's atlas: 4 × 4 cells of 256 px; the first row the colours
/// (scorch, rune, crack, puddle), the second moss, the third the first
/// row's normal maps.
const CELLS: f32 = 4.0;

fn cell(col: u32, row: u32) -> [f32; 4] {
    let (c, r) = (col as f32, row as f32);
    [c / CELLS, r / CELLS, (c + 1.0) / CELLS, (r + 1.0) / CELLS]
}

/// The crack's height at a cell point (x, y in -0.5..0.5): a pit chipped
/// into the plaster, a raised broken rim and branching cracks cut out of it.
fn crack_height(x: f32, y: f32) -> f32 {
    let r = (x * x + y * y).sqrt();
    let a = y.atan2(x);
    let pit = -(1.0 - smooth(0.05, 0.11, r));
    let rim = (-((r - 0.12) / 0.03).powi(2)).exp() * 0.5 * (0.7 + 0.6 * fbm(x * 9.0, y * 9.0, 41));
    let mut cut = 0.0f32;
    for k in 0..13 {
        let base = k as f32 * std::f32::consts::TAU / 13.0 + (hash2(k, 0, 0x5c) - 0.5) * 0.7;
        let wob = (fbm(r * 9.0, k as f32 * 3.1, 43) - 0.5) * 1.1 * r.sqrt();
        let d = ((a - base - wob + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI)
            .abs()
            * r;
        let reach = 0.16 + 0.32 * hash2(k, 1, 0x5c);
        let width = 0.012 * (1.0 - smooth(0.1, reach, r)) + 0.002;
        let line = (1.0 - smooth(0.0, width, d)) * (1.0 - smooth(reach - 0.06, reach, r));
        // A side branch off every other crack.
        let side = if k % 2 == 0 {
            let b = base + 0.45 * if k % 4 == 0 { 1.0 } else { -1.0 };
            let db = ((a - b - wob + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                - std::f32::consts::PI)
                .abs()
                * r;
            (1.0 - smooth(0.0, 0.006, db)) * smooth(0.16, 0.2, r) * (1.0 - smooth(0.26, 0.3, r))
        } else {
            0.0
        };
        cut = cut.max(line).max(side);
    }
    pit + rim - cut * 0.8 * smooth(0.08, 0.12, r)
}

fn scorch_height(x: f32, y: f32) -> f32 {
    let r = (x * x + y * y).sqrt();
    (fbm(x * 14.0, y * 14.0, 47) - 0.5) * 0.6 * (1.0 - smooth(0.2, 0.45, r))
}

/// A tangent-space normal from a height field by central differences,
/// encoded as glTF stores it (+y up the image).
fn normal_of(h: impl Fn(f32, f32) -> f32, x: f32, y: f32, strength: f32) -> [u8; 4] {
    let e = 1.0 / 512.0;
    let dx = (h(x + e, y) - h(x - e, y)) / (2.0 * e) * strength;
    let dy = (h(x, y - e) - h(x, y + e)) / (2.0 * e) * strength;
    let n = Vec3::new(-dx, -dy, 1.0).normalize();
    let enc = |c: f32| ((c * 0.5 + 0.5) * 255.0).round() as u8;
    [enc(n.x), enc(n.y), enc(n.z), 255]
}

fn atlas(scene: &mut RenderScene) -> og_render::TextureHandle {
    scene.add_texture(texture(1024, og_render::ColorSpace::Srgb, |u, v| {
        let (col, row) = ((u * CELLS) as u32, (v * CELLS) as u32);
        let (x, y) = ((u * CELLS).fract() - 0.5, (v * CELLS).fract() - 0.5);
        let r = (x * x + y * y).sqrt();
        let a = y.atan2(x);
        let n = fbm(x * 8.0 + col as f32 * 5.0, y * 8.0 + row as f32 * 3.0, 49);
        match (row, col) {
            // Scorch: solid soot in the middle, ragged and streaked
            // outwards at its edge as a blast leaves it.
            (0, 0) => {
                let streak = 0.5 + 0.5 * (a * 17.0 + n * 7.0).sin();
                let reach = 0.34 + 0.1 * streak + (n - 0.5) * 0.1;
                let core = 1.0 - smooth(0.1, 0.3, r + (n - 0.5) * 0.08);
                let edge = (1.0 - smooth(0.2, reach, r)) * (0.55 + 0.45 * streak);
                let al = core.max(edge);
                let ember = 1.0 - smooth(0.0, 0.06, r + (n - 0.5) * 0.05);
                [
                    (12.0 + 60.0 * ember) as u8,
                    (10.0 + 20.0 * ember) as u8,
                    8,
                    (al * 250.0) as u8,
                ]
            }
            // Rune: two rings, glyph ticks between them and a six-pointed
            // star inside.
            (0, 1) => {
                let ring = |c: f32, w: f32| 1.0 - smooth(0.0, w, (r - c).abs());
                let ticks = (1.0 - smooth(0.0, 0.05, (a * 12.0).sin().abs()))
                    * smooth(0.385, 0.395, r)
                    * (1.0 - smooth(0.43, 0.44, r));
                let glyphs = (1.0
                    - smooth(0.0, 0.004, ((a * 24.0 + 0.5).sin() * 0.02 + 0.41 - r).abs()))
                    * (1.0 - ticks);
                let tri = |turn: f32| {
                    (0..3)
                        .map(|k| {
                            let t = turn + k as f32 * std::f32::consts::TAU / 3.0;
                            // Distance to the side whose normal is at angle t.
                            (x * t.cos() + y * t.sin() - 0.17).abs()
                        })
                        .fold(f32::MAX, f32::min)
                };
                let star = (1.0 - smooth(0.0, 0.007, tri(0.0).min(tri(std::f32::consts::PI))))
                    * (1.0 - smooth(0.33, 0.35, r));
                let al = ring(0.45, 0.008)
                    .max(ring(0.37, 0.006))
                    .max(ticks * 0.9)
                    .max(glyphs * 0.8)
                    .max(star)
                    .max(ring(0.08, 0.01));
                [200, 240, 255, (al * 255.0) as u8]
            }
            // Crack: the chipped plaster's pale rim, dark in the pit and
            // the cracks.
            (0, 2) => {
                let h = crack_height(x, y);
                let dark = (-h).clamp(0.0, 1.0);
                let rim = h.clamp(0.0, 1.0);
                let dust = (1.0 - smooth(0.1, 0.32, r)) * 0.25 * n;
                let al = (dark * 1.2 + rim * 0.8 + dust).min(1.0) * (1.0 - smooth(0.44, 0.5, r));
                let c = 40.0 + 170.0 * rim - 30.0 * dark;
                [
                    c.max(0.0) as u8,
                    (c * 0.95).max(0.0) as u8,
                    (c * 0.88).max(0.0) as u8,
                    (al * 255.0) as u8,
                ]
            }
            // Puddle: a wet tint (multiplied: the flags at about half their
            // brightness) with a soft, wandering shore.
            (0, 3) => {
                let shore = 0.36 + (fbm(x * 4.0, y * 4.0, 53) - 0.5) * 0.22;
                // Standing water over the core, a damp fringe at the shore.
                let al = (1.0 - smooth(shore - 0.1, shore - 0.04, r)) * 0.55
                    + (1.0 - smooth(shore - 0.04, shore, r)) * 0.45;
                [180, 182, 188, (al * 255.0) as u8]
            }
            // Moss: green blotches thinning out from the middle.
            (1, 0) => {
                // Patches with crisp edges and gaps, denser to the middle.
                let m = 0.6 * fbm(x * 9.0, y * 9.0, 59) + 0.4 * fbm(x * 26.0, y * 26.0, 67);
                let al = smooth(0.47, 0.53, m + 0.42 * (1.0 - smooth(0.1, 0.48, r)) - 0.12)
                    * (1.0 - smooth(0.4, 0.5, r));
                let g = 0.6 + 0.4 * fbm(x * 30.0, y * 30.0, 61);
                [
                    (62.0 * g) as u8,
                    (118.0 * g) as u8,
                    (30.0 * g) as u8,
                    (al * 255.0) as u8,
                ]
            }
            // Normals: scorch, then crack.
            (2, 0) => normal_of(scorch_height, x, y, 0.12),
            (2, 2) => normal_of(crack_height, x, y, 0.06),
            (2, _) => [128, 128, 255, 255],
            _ => [0, 0, 0, 0],
        }
    }))
}

pub fn decals_stage_scene() -> RenderScene {
    use crate::kit;
    let mut scene = RenderScene::new();
    let rubble = crate::shadow_scenes::court(&mut scene);
    // The wall the marks stand on: its plastered face to the court.
    let plaster = kit::plaster(&mut scene, [0.62, 0.58, 0.52]);
    let face = scene.add_mesh(kit::block(
        Vec3::new(-5.0, 0.0, -3.4),
        Vec3::new(3.0, 2.8, -3.0),
        2.0,
    ));
    scene.add_instance(face, plaster, Mat4::IDENTITY);
    let planks = kit::surface_tinted(&mut scene, "planks", [0.42, 0.36, 0.3]);
    let coping = scene.add_mesh(kit::block(
        Vec3::new(-5.1, 2.8, -3.55),
        Vec3::new(3.1, 2.96, -2.85),
        1.2,
    ));
    scene.add_instance(coping, planks, Mat4::IDENTITY);
    let end = scene.add_mesh(kit::block(
        Vec3::new(3.0, 0.0, -3.4),
        Vec3::new(3.6, 2.96, -0.4),
        1.6,
    ));
    scene.add_instance(end, rubble, Mat4::IDENTITY);
    let toro = kit::prop(&mut scene, "toro");
    let fire = crate::shadow_scenes::toro(&mut scene, &toro, Vec3::new(-3.9, 0.0, -1.6), 0.4, 1.7);
    // The lantern lit for the evening: the puddle mirrors it.
    scene.lighting.lights.push(LightDesc {
        position: fire,
        color: Vec3::new(1.0, 0.6, 0.3),
        intensity: 4.0,
        range: 5.0,
        radius: 0.12,
        ..Default::default()
    });
    let atlas = atlas(&mut scene);
    scene.effects.decal_atlas = Some(atlas);
    let up = Vec3::Y;
    let rune_at = Vec3::new(1.0, 0.0, 0.7);
    scene.effects.decals = vec![
        // The crater in the wall, its cracks running out over the plaster.
        Decal {
            uv_rect: cell(2, 0),
            normal_rect: cell(2, 2),
            normal_scale: 1.5,
            roughness: 0.85,
            ..Decal::on_surface(
                Vec3::new(0.2, 1.5, -3.0),
                Vec3::Z,
                Vec2::splat(2.6),
                0.5,
                0.3,
            )
        },
        // The blast's scorch on the flags below it.
        Decal {
            uv_rect: cell(0, 0),
            normal_rect: cell(0, 2),
            roughness: 0.95,
            angle_fade_degrees: 80.0,
            ..Decal::on_surface(
                Vec3::new(0.3, 0.1, -2.9),
                Vec3::new(0.0, 1.0, 1.0),
                Vec2::splat(3.0),
                1.6,
                0.8,
            )
        },
        // The rune, glowing.
        Decal {
            uv_rect: cell(1, 0),
            color: [1.0, 1.0, 1.0, 1.0],
            emissive: Vec3::new(1.2, 3.2, 6.0),
            roughness: 0.3,
            ..Decal::on_surface(rune_at, up, Vec2::splat(2.2), 0.4, 0.2)
        },
        // A rain puddle where the lantern's reflection meets the lens.
        Decal {
            uv_rect: cell(3, 0),
            roughness: 0.04,
            wet: true,
            ..Decal::on_surface(Vec3::new(-1.1, 0.0, 1.5), up, Vec2::new(2.0, 1.4), 0.4, 0.3)
        },
        // Moss projected diagonally into the corner: on the wall's foot
        // and the flags at once.
        Decal {
            uv_rect: cell(0, 1),
            angle_fade_degrees: 80.0,
            roughness: 0.9,
            ..Decal::on_surface(
                Vec3::new(2.85, 0.05, -2.85),
                Vec3::new(-1.0, 1.4, 1.0),
                Vec2::splat(2.0),
                1.6,
                0.4,
            )
        },
    ];
    // The rune's own light, a hand's breadth over it.
    scene.lighting.lights.push(LightDesc {
        position: rune_at + Vec3::Y * 0.35,
        color: Vec3::new(0.35, 0.7, 1.0),
        intensity: 4.0,
        range: 5.0,
        ..Default::default()
    });
    // Dusk: the sun 5° up from the right, raking along the wall.
    crate::sky_scenes::physical_light_under(&mut scene, Vec3::new(0.9, 0.09, 0.42));
    scene.lighting.light.cast_shadows = true;
    scene.look.post.bloom = Some(Bloom::default());
    // The puddle's gloss reflects the lantern and the wall.
    scene.look.post.ssr = Some(og_render::ScreenSpaceReflections::default());
    scene.frame.camera = Camera {
        position: Vec3::new(0.5, 0.75, 3.7),
        target: Vec3::new(-0.6, 0.8, -3.0),
        far: 2500.0,
        exposure: Exposure::Manual { ev: -0.2 },
        ..Default::default()
    };
    scene
}
