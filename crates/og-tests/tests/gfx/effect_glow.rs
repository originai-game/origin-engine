//! A vfx flash and an effect column read as light, not geometry: the glow
//! material (`EffectMaterial::glow`, the og.Vfx flash) is gone well inside
//! its sphere's limb, and the effect cylinder (`effect_cylinder`, og.Vfx's
//! `mesh cylinder`) with a soft silhouette fades at its sides and its open
//! ends. Measured on one effect mesh over black, through an orthographic
//! camera 4 units tall (180 pixels to the unit), no bloom and no tonemap,
//! in linear light (the frame decoded from sRGB).

use super::*;
use og_render::{
    EffectMaterial, EffectMesh, EffectValue, Environment, Exposure, Fresnel, MeshData,
    ParticleBlend, PostEffects, Projection, effect_cylinder, shapes,
};

const UNIT: f32 = 180.0;

/// One effect mesh (`material`, white times `color`) drawn at the origin
/// scaled by `scale`, seen down -z; the frame's red channel, linear (0 to 1).
fn draw(mesh: MeshData, material: EffectMaterial, scale: Vec3, color: f32) -> (Vec<f32>, u32) {
    let mut scene = RenderScene::new();
    scene.lighting.environment = Environment {
        background: Background::Color(Vec3::ZERO),
        intensity: 0.0,
        ..Environment::default()
    };
    scene.lighting.light.intensity = 0.0;
    scene.look.post = PostEffects {
        tonemap: Tonemap::None,
        ..Default::default()
    };
    scene.frame.camera = Camera {
        position: Vec3::new(0.0, 0.0, 10.0),
        target: Vec3::ZERO,
        projection: Projection::Orthographic { height: 4.0 },
        exposure: Exposure::linear(1.0),
        ..Default::default()
    };
    let mesh = scene.add_mesh(mesh);
    let material = scene.add_effect_material(material);
    scene.effects.effect_meshes.push(EffectMesh {
        id: 1,
        mesh,
        material,
        transform: Mat4::from_scale(scale),
        life: 0.0,
        seed: 1,
        color: [color, color, color, 1.0],
        blend: ParticleBlend::Additive,
        layers: 1,
    });
    let desc = scenes::base_desc();
    let (img, _) = frame(&desc, &scene, 1);
    let red = (0..img.width * img.height)
        .map(|i| {
            let c = img.pixels[(i * 4) as usize] as f32 / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
        .collect();
    (red, img.width)
}

/// The mean of `red` over the pixels whose distance from the frame's
/// centre, in units, is in `r`.
fn ring(red: &[f32], width: u32, r: std::ops::Range<f32>) -> f32 {
    let height = red.len() as u32 / width;
    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);
    let (mut sum, mut n) = (0.0, 0.0);
    for y in 0..height {
        for x in 0..width {
            let d = ((x as f32 + 0.5 - cx).hypot(y as f32 + 0.5 - cy)) / UNIT;
            if r.contains(&d) {
                sum += red[(y * width + x) as usize];
                n += 1.0;
            }
        }
    }
    sum / n
}

/// The flash on a unit sphere: its centre is bright, the ring just inside
/// its limb (0.88 to 0.98 of the radius) under a tenth of it, and nothing
/// shows outside it, so its outline is not a silhouette. The bounced flash
/// (`1 − (1 − n·v)^0.35`, and `^1.6` before it) kept a fifth to a half of
/// its centre right up to the limb: a ball.
#[test]
#[ignore = "requires GPU"]
fn a_flash_glows_out_to_nothing_well_inside_its_limb() {
    let (red, width) = draw(
        shapes::sphere(48, 32),
        EffectMaterial::glow(),
        Vec3::splat(2.0),
        0.8,
    );
    let centre = ring(&red, width, 0.0..0.15);
    let mid = ring(&red, width, 0.45..0.55);
    let limb = ring(&red, width, 0.88..0.98);
    let outside = ring(&red, width, 1.03..1.2);
    println!(
        "flash: centre {centre:.3}, half radius {mid:.3}, limb {limb:.3}, outside {outside:.3}"
    );
    assert!(centre > 0.4, "the flash's centre is {centre:.3}");
    assert!(
        mid < 0.85 * centre && mid > 0.3 * centre,
        "no falloff from the centre: {mid:.3} at half its radius"
    );
    assert!(
        limb < 0.1 * centre,
        "its limb is {limb:.3} against a centre of {centre:.3}"
    );
    assert!(outside < 0.002, "it shows outside its sphere: {outside:.3}");
}

/// A column of the effect cylinder, 1 unit wide and 3 tall, with a soft
/// silhouette (`fresnel 3 1 invert`): its middle is bright, its outer
/// sixths and its top and bottom twelfths under half of it. Bounced, its
/// sides were as bright as its middle and its open ends cut square: a box.
#[test]
#[ignore = "requires GPU"]
fn an_effect_column_fades_at_its_sides_and_its_open_ends() {
    let material = EffectMaterial {
        fresnel: Some(Fresnel {
            power: 3.0,
            strength: 1.0,
            invert: true,
        }),
        intensity: EffectValue::Constant(1.0),
        blend: Some(ParticleBlend::Additive),
        double_sided: true,
        ..Default::default()
    };
    let (red, width) = draw(effect_cylinder(), material, Vec3::new(2.0, 3.0, 2.0), 0.4);
    let height = red.len() as u32 / width;
    let px = |u: f32| (u * UNIT) as i32;
    let (cx, cy) = (width as i32 / 2, height as i32 / 2);
    let mean = |x: std::ops::Range<i32>, y: std::ops::Range<i32>| {
        let n = (x.len() * y.len()) as f32;
        y.flat_map(|y| x.clone().map(move |x| (x, y)))
            .map(|(x, y)| red[((cy + y) * width as i32 + cx + x) as usize])
            .sum::<f32>()
            / n
    };
    // Half its width is 1, half its height 1.5.
    let rows = -px(0.5)..px(0.5);
    let middle = mean(-px(0.2)..px(0.2), rows.clone());
    let left = mean(-px(1.0)..-px(0.67), rows.clone());
    let right = mean(px(0.67)..px(1.0), rows);
    let core = -px(0.2)..px(0.2);
    let top = mean(core.clone(), -px(1.5)..-px(1.25));
    let bottom = mean(core, px(1.25)..px(1.5));
    println!("column: middle {middle:.3}, sides {left:.3} {right:.3}, ends {top:.3} {bottom:.3}");
    assert!(middle > 0.3, "the column's middle is {middle:.3}");
    for (name, v) in [
        ("left side", left),
        ("right side", right),
        ("top end", top),
        ("bottom end", bottom),
    ] {
        assert!(
            v < 0.5 * middle,
            "its {name} is {v:.3} against a middle of {middle:.3}"
        );
    }
}
