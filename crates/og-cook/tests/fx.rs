//! Effect assets through the cook and back: a `.cube` colour table and a
//! particle emitter (`.ogt` of `type particles`) cook to blobs the runtime
//! loads into og-render's ColorLut and ParticleEmitter (every emitter line
//! reaching its field), and the refusals name the line or the missing file.

use og_cook::fx::parse_emitter;
use og_cook::runtime::Bundle;
use og_cook::{cook_project, CookOptions};
use og_render::{
    EmitterShape, ParticleAlign, ParticleBlend, ParticleRender, SheetMode, SimulationSpace,
    LUT_SIZE,
};
use std::path::PathBuf;

const EMITTER: &str = r#"type particles
version 1
---
maxParticles 512
shape cone 25 0.2
spread 10
rate 80
burst 0.5 30 3 0.25
duration 2
looping false
lifetime 0.5 1.25
speed 2
size 0.1 0.3
sizeOverLife 0 1 1 0.25
colorOverLife {
  key 0 4 2 0.5 1
  key 1 1 0.2 0.05 0
}
color 1 0.8 0.6
rotation 0 360
angularVelocity -90 90
gravity 0 -3 0
drag 0.5
noise 2 0.75 0.4
texture "textures/spark.png"
sheet 4 2 8 life 2
blend alpha
space local
align velocity 0.1
softDistance 0.3
collision 0.6 0.1
render sprite 7
glow 1.5
"#;

/// A 2³ table that swaps red and blue.
const CUBE: &str =
    "TITLE \"swap\"\nLUT_3D_SIZE 2\n0 0 0\n0 0 1\n0 1 0\n0 1 1\n1 0 0\n1 0 1\n1 1 0\n1 1 1\n";

fn project(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("og-cook-fx")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in files {
        let p = root.join("assets").join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }
    root
}

fn png() -> Vec<u8> {
    let mut out = Vec::new();
    let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([255, 255, 255, 255]));
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

#[test]
fn a_cube_and_an_emitter_cook_and_load() {
    let png = png();
    let root = project(
        "ok",
        &[
            ("luts/swap.cube", CUBE.as_bytes()),
            ("fx/sparks.ogt", EMITTER.as_bytes()),
            ("textures/spark.png", &png),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();

    let id = "luts/swap.cube";
    let bytes = std::fs::read(root.join("cooked").join(&bundle.output(id).unwrap().file)).unwrap();
    let lut = bundle.lut(id, &bytes).unwrap();
    assert_eq!(lut.size, LUT_SIZE);
    // Pure red maps to pure blue, white stays white (trilinear over the 2³ table).
    let texel = |r: usize, g: usize, b: usize| {
        let n = LUT_SIZE as usize;
        let i = (r + g * n + b * n * n) * 4;
        [lut.rgba[i], lut.rgba[i + 1], lut.rgba[i + 2]]
    };
    assert_eq!(texel(31, 0, 0), [0, 0, 255]);
    assert_eq!(texel(31, 31, 31), [255, 255, 255]);

    let id = "fx/sparks.ogt";
    let bytes = std::fs::read(root.join("cooked").join(&bundle.output(id).unwrap().file)).unwrap();
    let text = bundle.emitter_text(id, &bytes).unwrap();
    let mut asked = Vec::new();
    let em = parse_emitter(&text, &mut |t| {
        asked.push(t.to_string());
        Ok(None)
    })
    .unwrap();
    assert_eq!(asked, ["textures/spark.png"]);
    assert_eq!(em.max_particles, 512);
    assert_eq!(
        em.shape,
        EmitterShape::Cone {
            angle_degrees: 25.0,
            radius: 0.2
        }
    );
    assert_eq!((em.spread_degrees, em.rate, em.duration), (10.0, 80.0, 2.0));
    assert_eq!(em.bursts.len(), 1);
    assert_eq!(
        (
            em.bursts[0].time,
            em.bursts[0].count,
            em.bursts[0].cycles,
            em.bursts[0].interval
        ),
        (0.5, 30, 3, 0.25)
    );
    assert!(!em.looping);
    assert_eq!(
        (em.lifetime, em.speed, em.size),
        ([0.5, 1.25], [2.0, 2.0], [0.1, 0.3])
    );
    assert_eq!(em.size_over_life.keys, [(0.0, 1.0), (1.0, 0.25)]);
    assert_eq!(
        em.color_over_life.keys,
        [(0.0, [4.0, 2.0, 0.5, 1.0]), (1.0, [1.0, 0.2, 0.05, 0.0])]
    );
    assert_eq!(em.color, [1.0, 0.8, 0.6, 1.0]);
    assert_eq!(
        (em.rotation_degrees, em.angular_velocity_degrees),
        ([0.0, 360.0], [-90.0, 90.0])
    );
    assert_eq!(em.gravity, glam::Vec3::new(0.0, -3.0, 0.0));
    assert_eq!(em.drag, 0.5);
    assert_eq!(
        (em.noise.strength, em.noise.frequency, em.noise.scroll),
        (2.0, 0.75, 0.4)
    );
    assert_eq!(
        (em.sheet.columns, em.sheet.rows, em.sheet.frames),
        (4, 2, 8)
    );
    assert_eq!(em.sheet.mode, SheetMode::OverLife { cycles: 2.0 });
    assert_eq!(em.blend, ParticleBlend::Alpha);
    assert_eq!(em.space, SimulationSpace::Local);
    assert_eq!(em.align, ParticleAlign::Velocity { stretch: 0.1 });
    assert_eq!(em.soft_distance, 0.3);
    let c = em.collision.unwrap();
    assert_eq!((c.bounce, c.friction), (0.6, 0.1));
    assert_eq!(em.render, ParticleRender::Sprite { layer: 7 });
    assert_eq!(em.glow, 1.5);
}

#[test]
fn bad_effect_assets_are_refused_with_the_reason() {
    let unknown = EMITTER.replace("glow 1.5", "glowing 1.5");
    let root = project(
        "bad",
        &[
            (
                "fx/missing.ogt",
                b"type particles\nversion 1\n---\ntexture \"textures/none.png\"\n",
            ),
            ("fx/unknown.ogt", unknown.as_bytes()),
            ("luts/short.cube", b"LUT_3D_SIZE 2\n0 0 0\n1 1 1\n"),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(errors.contains("textures/none.png"), "{errors}");
    assert!(
        errors.contains("line 33: unknown emitter setting `glowing`"),
        "{errors}"
    );
    assert!(errors.contains("LUT_3D_SIZE 2 needs 8 entries"), "{errors}");
}
