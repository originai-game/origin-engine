//! Effect assets through the cook and back: a `.cube` colour table and a
//! particle emitter (`.ogt` of `type particles`) cook to blobs the runtime
//! loads into og-render's ColorLut and ParticleEmitter (every emitter line
//! reaching its field), and the refusals name the line or the missing file.
//! An emitter's `array` line names a texture array and picks its layers;
//! a `.cube` can be held by a streaming group.

use og_cook::fx::parse_emitter;
use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use og_render::{
    EmitterShape, LUT_SIZE, ParticleAlign, ParticleBlend, ParticleRender, SheetMode,
    SimulationSpace,
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
    assert_eq!(c.against, og_render::CollisionTarget::Depth);
    // `collision` names what compute particles meet: the depth prepass
    // (unsaid) or the scene's colliders; a third word or a third number is
    // refused.
    let collision = |line: &str| {
        let text = format!("type particles\nversion 1\n---\n{line}\n");
        parse_emitter(&text, &mut |_| Ok(None)).map(|e| e.collision.unwrap())
    };
    let c = collision("collision 0.3 0.5 colliders").unwrap();
    assert_eq!(
        (c.bounce, c.friction, c.against),
        (0.3, 0.5, og_render::CollisionTarget::Colliders)
    );
    let c = collision("collision colliders").unwrap();
    assert_eq!(
        (c.bounce, c.against),
        (0.4, og_render::CollisionTarget::Colliders)
    );
    assert_eq!(
        collision("collision 0.3 depth").unwrap().against,
        og_render::CollisionTarget::Depth
    );
    for bad in [
        "collision 0.3 0.5 walls",
        "collision 0.3 0.5 0.1",
        "collision colliders 0.3",
    ] {
        let err = collision(bad).unwrap_err();
        assert!(err.contains("depth | colliders"), "{bad}: {err}");
    }
    assert_eq!(em.render, ParticleRender::Sprite { layer: 7 });
    assert_eq!(em.glow, 1.5);
}

/// R7's lines: `render mesh`, `lit`, `distortion`, `events` and
/// `subEmitter` reach their fields through the linked parser (each
/// reference resolved by its own link); the cook refuses a sub-emitter or a
/// mesh model not under assets/, a fifth sub-emitter and distortion on a
/// mesh.
#[test]
fn mesh_lit_distorting_emitters_with_events_and_sub_emitters() {
    use og_cook::fx::{EmitterLinks, ParticleMesh, parse_emitter_linked};
    use og_render::{MeshHandle, ParticleEventKind, RenderScene};
    let mut scene = RenderScene::default();
    let text = "type particles\nversion 1\n---\nrender mesh tetrahedron\nlit true\n\
                events death collision death\nsubEmitter death \"fx/puff.ogt\" 4 0.5\n\
                subEmitter collision \"fx/chip.ogt\" 2\n";
    let mut meshes = Vec::new();
    let mut subs = Vec::new();
    let em = parse_emitter_linked(
        text,
        &mut EmitterLinks {
            texture: &mut |_| Ok(None),
            array: &mut |_| Ok(None),
            mesh: &mut |m| {
                meshes.push(format!("{m:?}"));
                Ok(Some(MeshHandle::from_bits(7)))
            },
            emitter: &mut |id| {
                subs.push(id.to_string());
                Ok(Some(scene.add_emitter(Default::default())))
            },
            effect: &mut |_| Ok(None),
        },
    )
    .unwrap();
    assert_eq!(
        meshes,
        [format!("{:?}", ParticleMesh::Builtin("tetrahedron"))]
    );
    assert_eq!(subs, ["fx/puff.ogt", "fx/chip.ogt"]);
    assert_eq!(
        em.render,
        ParticleRender::Mesh {
            mesh: MeshHandle::from_bits(7)
        }
    );
    assert!(em.lit);
    // Unsaid, a lit mesh blends alpha (a solid chip, added, glows white);
    // light adds: an unlit mesh, a lit billboard, and a blend line said
    // before `lit` still wins.
    assert_eq!(em.blend, ParticleBlend::Alpha);
    let blend = |body: &str| {
        let text = format!("type particles\nversion 1\n---\n{body}");
        parse_emitter(&text, &mut |_| Ok(None)).unwrap().blend
    };
    assert_eq!(blend("render mesh box\n"), ParticleBlend::Additive);
    assert_eq!(blend("lit true\n"), ParticleBlend::Additive);
    assert_eq!(
        blend("blend additive\nrender mesh box\nlit true\n"),
        ParticleBlend::Additive
    );
    assert_eq!(
        blend("lit true\nrender mesh sphere\n"),
        ParticleBlend::Alpha
    );
    assert_eq!(
        em.events,
        [ParticleEventKind::Death, ParticleEventKind::Collision]
    );
    assert_eq!(em.sub_emitters.len(), 2);
    let s = &em.sub_emitters[0];
    assert_eq!(
        (s.trigger, s.count, s.inherit_velocity),
        (ParticleEventKind::Death, 4, 0.5)
    );
    assert_eq!(em.sub_emitters[1].trigger, ParticleEventKind::Collision);
    assert_eq!(em.sub_emitters[1].inherit_velocity, 0.0);
    let haze = parse_emitter(
        "type particles\nversion 1\n---\ndistortion 0.05\n",
        &mut |_| Ok(None),
    )
    .unwrap();
    assert_eq!(haze.distortion, 0.05);

    let five = format!(
        "type particles\nversion 1\n---\n{}",
        "subEmitter birth \"fx/puff.ogt\" 1\n".repeat(5)
    );
    let root = project(
        "linked",
        &[
            ("fx/puff.ogt", b"type particles\nversion 1\n---\nrate 0\n"),
            (
                "fx/orphan.ogt",
                b"type particles\nversion 1\n---\nsubEmitter death \"fx/none.ogt\" 2\nrender mesh \"models/none.glb\"\n",
            ),
            ("fx/five.ogt", five.as_bytes()),
            (
                "fx/warp.ogt",
                b"type particles\nversion 1\n---\nrender mesh box\ndistortion 0.1\n",
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(errors.contains("fx/none.ogt"), "{errors}");
    assert!(errors.contains("models/none.glb"), "{errors}");
    assert!(errors.contains("at most 4 subEmitter lines"), "{errors}");
    assert!(errors.contains("fx/warp.ogt"), "{errors}");
    assert!(!errors.contains("fx/puff.ogt:"), "{errors}");
}

/// An effect material (`type effect`) cooks to a blob the runtime parses
/// back with its texture, and an emitter names it with `effect`; a missing
/// texture is refused by name.
#[test]
fn an_effect_material_cooks_and_an_emitter_names_it() {
    let png = png();
    let effect = "type effect\nversion 1\n---\nmain \"textures/noise.png\" red {\n  scroll 0 -0.5\n  polar true\n}\nerosion curve 0 0 1 1\nerosionEdge 0.05 6 2 0.5\nfresnel 2\ncustom 1 random 2 6\n";
    let emitter = "type particles\nversion 1\n---\neffect \"fx/fire.ogt\"\n";
    let root = project(
        "effect",
        &[
            ("fx/fire.ogt", effect.as_bytes()),
            ("fx/flame.ogt", emitter.as_bytes()),
            ("textures/noise.png", &png),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let id = "fx/fire.ogt";
    let bytes = std::fs::read(root.join("cooked").join(&bundle.output(id).unwrap().file)).unwrap();
    let text = bundle.effect_text(id, &bytes).unwrap();
    let mut asked = Vec::new();
    let m = og_cook::effect::parse_effect(
        &text,
        &mut og_cook::effect::EffectLinks {
            texture: &mut |t| {
                asked.push(t.to_string());
                Ok(Some(og_render::TextureHandle::from_bits(2)))
            },
            shader: &mut |_| Ok(None),
        },
    )
    .unwrap();
    assert_eq!(asked, ["textures/noise.png"]);
    assert!(m.main.uv.polar);
    assert_eq!(m.main.texture, Some(og_render::TextureHandle::from_bits(2)));
    assert_eq!(m.custom[0][0], og_render::EffectValue::Random([2.0, 6.0]));

    let missing = project("effect-missing", &[("fx/fire.ogt", effect.as_bytes())]);
    let report = cook_project(&missing, &CookOptions::default()).unwrap();
    assert!(!report.ok());
    assert!(
        format!("{:?}", report.errors()).contains("textures/noise.png"),
        "{:?}",
        report.errors()
    );
}

/// A composed effect (`type vfx`) cooks to a blob the runtime parses back,
/// and one naming an emitter or sound that is not in the project is
/// refused by name.
#[test]
fn a_vfx_cooks_and_what_it_names_must_exist() {
    let vfx = "type vfx\nversion 1\n---\nduration 1\nimpact 0.2\nemitter \"fx/sparks.ogt\" {\n  at 0\n}\n";
    let broken = "type vfx\nversion 1\n---\nduration 1\nemitter \"fx/none.ogt\" {\n  at 0\n}\nsound \"sfx/none.wav\" {\n  at 0\n}\n";
    let root = project(
        "vfx",
        &[
            ("fx/hit.ogt", vfx.as_bytes()),
            ("fx/broken.ogt", broken.as_bytes()),
            (
                "fx/sparks.ogt",
                b"type particles\nversion 1\n---\nrate 10\n",
            ),
        ],
    );
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    let errors = report.errors().join("\n");
    assert!(errors.contains("fx/broken.ogt"), "{errors}");
    assert!(errors.contains("emitter fx/none.ogt"), "{errors}");
    assert!(errors.contains("sound sfx/none.wav"), "{errors}");
    assert!(!errors.contains("fx/hit.ogt"), "{errors}");
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let id = "fx/hit.ogt";
    let bytes = std::fs::read(root.join("cooked").join(&bundle.output(id).unwrap().file)).unwrap();
    let v = og_bundle::vfx::parse_vfx(&bundle.vfx_text(id, &bytes).unwrap()).unwrap();
    assert_eq!(v.events[0].name, "impact");
}
