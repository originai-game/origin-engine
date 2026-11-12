//! The importer against Effekseer's own samples (CC0, credited in
//! `third_party/effekseer/`): facts read from their XML by hand.

use super::*;

const SPRITE: &[u8] =
    include_bytes!("../../../tests/fixtures/effekseer/Simple_Sprite_BillBoard.efkproj");
const RING: &[u8] = include_bytes!("../../../tests/fixtures/effekseer/Simple_Ring_Shape1.efkproj");
const RIBBON: &[u8] =
    include_bytes!("../../../tests/fixtures/effekseer/Simple_Ribbon_Sword.efkproj");
const EFKEFC: &[u8] = include_bytes!("../../../tests/fixtures/effekseer/iwai-ume.efkefc");

/// A PNG's first bytes, for a texture `w` by `h`.
fn png(w: u32, h: u32) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    b.extend_from_slice(&w.to_be_bytes());
    b.extend_from_slice(&h.to_be_bytes());
    b
}

fn run(bytes: &[u8]) -> Conversion {
    let own = |leaf: &str| format!("fx/a.efkproj#{leaf}");
    let asset = |p: &str| format!("fx/{p}");
    let read = |_: &str| Some(png(128, 128));
    convert(
        bytes,
        &Links {
            own: &own,
            asset: &asset,
            read: &read,
        },
    )
    .unwrap()
}

fn file<'a>(c: &'a Conversion, leaf: &str) -> &'a str {
    let id = format!("fx/a.efkproj#{leaf}");
    let f = c.files.iter().find(|f| f.id == id).unwrap_or_else(|| {
        panic!(
            "no {id} in {:?}",
            c.files.iter().map(|f| &f.id).collect::<Vec<_>>()
        )
    });
    std::str::from_utf8(&f.bytes).unwrap()
}

fn line<'a>(text: &'a str, key: &str) -> &'a str {
    text.lines()
        .map(str::trim)
        .find(|l| l.split_whitespace().next() == Some(key))
        .unwrap_or_else(|| panic!("no `{key}` in\n{text}"))
}

#[test]
fn a_billboard_sprite_node_is_an_emitter_of_its_counts_life_size_and_blend() {
    let c = run(SPRITE);
    // The vfx parses and names one emitter, the node's.
    let v = og_bundle::vfx::parse_vfx(&c.vfx).unwrap();
    assert_eq!(v.references().emitters, ["fx/a.efkproj#n0.emitter.ogt"]);
    let e = file(&c, "n0.emitter.ogt");
    // MaxGeneration 100 every 0.1 frames: 600 a second for 100 of them.
    assert_eq!(line(e, "rate"), "rate 600");
    assert_eq!(line(e, "duration"), "duration 0.1667");
    // Life 30 frames at 60 fps.
    assert_eq!(line(e, "lifetime"), "lifetime 0.5 0.5");
    // AlphaBlend 2 on the sprite renderer (an older project's place for it).
    assert_eq!(line(e, "blend"), "blend additive");
    assert_eq!(line(e, "texture"), "texture \"fx/Texture/Particle01.png\"");
    // Scale PVA: 1 growing 0.1 a frame, 4 at frame 30.
    assert_eq!(line(e, "size"), "size 4 4");
    assert_eq!(line(e, "sizeOverLife"), "sizeOverLife 0 0.25 1 1");
    // Easing to an end of ±5 per axis: every direction.
    assert_eq!(line(e, "spread"), "spread 180");
    // Colour easing from opaque white to transparent.
    let keys: Vec<&str> = e.lines().filter(|l| l.trim().starts_with("key ")).collect();
    assert_eq!(keys.first().unwrap().trim(), "key 0 1 1 1 1");
    assert!(keys.last().unwrap().trim().ends_with(" 0"), "{keys:?}");
    // The random end colour is said, not dropped.
    assert!(
        c.findings
            .iter()
            .any(|f| f.what.starts_with("a random colour per particle")),
        "{:?}",
        c.findings
    );
    // The last spawns 9.9 frames in and lives 0.5 s.
    assert!(
        (v.duration - (9.9 / 60.0 + 0.5)).abs() < 1e-3,
        "{}",
        v.duration
    );
}

#[test]
fn ring_nodes_are_ring_meshes_at_their_start_times() {
    let c = run(RING);
    let v = og_bundle::vfx::parse_vfx(&c.vfx).unwrap();
    let rings: Vec<_> = v
        .elements
        .iter()
        .filter_map(|e| match &e.kind {
            og_bundle::vfx::VfxKind::Mesh {
                shape: og_bundle::vfx::VfxShape::Ring(Some(r)),
                ..
            } => Some((e, r)),
            _ => None,
        })
        .collect();
    assert_eq!(rings.len(), 8, "{}", c.vfx);
    // A ring is no file: its shape is lines of its element.
    assert!(
        !c.files.iter().any(|f| f.kind == OutKind::Model),
        "{:?}",
        c.files.iter().map(|f| &f.id).collect::<Vec<_>>()
    );
    // `Ring`: life 50 frames, the default outer radius 2 at a fixed scale
    // of 2; `Ring(Inner)` 60 frames later.
    let (e0, r0) = rings[0];
    assert!((e0.life - 50.0 / 60.0).abs() < 1e-3);
    assert_eq!(e0.at, 0.0);
    assert!((rings[1].0.at - 1.0).abs() < 1e-4);
    let og_bundle::vfx::VfxKind::Mesh { scale, blend, .. } = &e0.kind else {
        unreachable!()
    };
    assert_eq!(*scale, og_render_data::EffectValue::Constant(4.0));
    assert_eq!(*blend, og_render_data::ParticleBlend::Alpha);
    // Inner radius 1 of the outer's 2, flat, its colours across: the
    // edges clear, the band opaque.
    assert_eq!(r0.inner, 0.5);
    assert_eq!(r0.height, [0.0; 2]);
    assert_eq!(r0.length, 360.0);
    assert_eq!(
        [r0.colors[0][3], r0.colors[1][3], r0.colors[2][3]],
        [0.0, 1.0, 0.0]
    );
}

#[test]
fn a_ring_whose_edges_differ_in_height_is_a_tube() {
    // Outer and inner at radius 1, the outer 0.8 up the ring's axis
    // (Effekseer's x a radius, y a height): a tube, not a ring of no width.
    let c = run(&project(
        "<Node><DrawingValues><Type>4</Type><Ring>\
         <Outer_Fixed><Location><X>1</X><Y>0.8</Y></Location></Outer_Fixed>\
         <Inner_Fixed><Location><X>1</X><Y>0</Y></Location></Inner_Fixed>\
         </Ring></DrawingValues><Name>T</Name><Children /></Node>",
    ));
    let mesh = &c.vfx[c.vfx.find("mesh ring {").expect("a ring")..];
    assert_eq!(line(mesh, "inner"), "inner 1", "{}", c.vfx);
    assert_eq!(line(mesh, "height"), "height 0.8 0", "{}", c.vfx);
    let v = og_bundle::vfx::parse_vfx(&c.vfx).unwrap();
    let og_bundle::vfx::VfxKind::Mesh {
        shape: og_bundle::vfx::VfxShape::Ring(Some(r)),
        ..
    } = &v.elements[0].kind
    else {
        panic!("{}", c.vfx)
    };
    // Its triangles have area.
    let m = og_render_data::effect_ring(r);
    let area: f32 = m
        .indices
        .chunks(3)
        .map(|t| {
            let p = |i: u32| glam::Vec3::from(m.vertices[i as usize].position);
            (p(t[1]) - p(t[0])).cross(p(t[2]) - p(t[0])).length() * 0.5
        })
        .sum();
    assert!(area > 1.0, "{area}");
}

#[test]
fn a_ribbon_is_a_trail_with_its_texture_and_lifetime() {
    let c = run(RIBBON);
    let v = og_bundle::vfx::parse_vfx(&c.vfx).unwrap();
    let t = v
        .elements
        .iter()
        .find_map(|e| match &e.kind {
            og_bundle::vfx::VfxKind::Trail {
                lifetime, effect, ..
            } => Some((*lifetime, effect.clone())),
            _ => None,
        })
        .expect("a trail");
    // Life 15 frames: each point of the ribbon lives 0.25 s.
    assert!((t.0 - 0.25).abs() < 1e-4, "{}", t.0);
    let fx = file(&c, "n0.effect.ogt");
    assert!(fx.contains("main \"fx/Texture/SwordLine01.png\""), "{fx}");
    assert_eq!(t.1.as_deref(), Some("fx/a.efkproj#n0.effect.ogt"));
}

#[test]
fn an_efkefc_reads_as_its_project() {
    // The binary file's EDIT chunk holds the editor's project.
    let root = xml::read_project(EFKEFC).unwrap();
    assert!(root.at("Root/Children/Node").is_some());
    let c = run(EFKEFC);
    og_bundle::vfx::parse_vfx(&c.vfx).unwrap();
    assert!(!c.files.is_empty());
    // A runtime-only export says how to get the project back.
    let mut stripped = EFKEFC[..8].to_vec();
    stripped.extend_from_slice(b"BIN_\x00\x00\x00\x00");
    let e = xml::read_project(&stripped).unwrap_err();
    assert!(e.contains("no EDIT chunk"), "{e}");
}

#[test]
fn findings_ride_in_the_text_and_parse_as_comments() {
    let c = run(SPRITE);
    let text = vfx_text(&c);
    assert!(
        text.lines()
            .any(|l| l.starts_with("# effekseer: node 0 \"Node\": a random colour")),
        "{text}"
    );
    og_bundle::vfx::parse_vfx(&text).unwrap();
}

#[test]
fn xml_reads_text_entities_and_nesting() {
    let d = xml::parse_xml("<?xml version=\"1.0\"?>\r\n<A><B>1 &amp; 2</B><C/><D><E>x</E></D></A>")
        .unwrap();
    let a = &d.children[0];
    assert_eq!(a.text_at("B"), Some("1 & 2"));
    assert!(a.child("C").is_some());
    assert_eq!(a.text_at("D/E"), Some("x"));
    assert!(xml::parse_xml("<A><B></A>").is_err());
}

#[test]
fn easing_and_curves_follow_effekseer() {
    use value::{Ease, FCurve};
    // LeftRightSpeed 0 0 is linear; Effekseer's cubic through the ends.
    let e = Ease::Cubic([0.0, 0.0, 1.0]);
    assert!((e.eval(0.3) - 0.3).abs() < 1e-6);
    // A fast start (StartSpeed 30) is ahead of linear at the middle.
    let mut x = xml::El::default();
    x.children.push(xml::El {
        name: "StartSpeed".into(),
        text: "30".into(),
        children: vec![],
    });
    let fast = Ease::read(Some(&x));
    assert!(fast.eval(0.5) > 0.55, "{}", fast.eval(0.5));
    assert!((fast.eval(1.0) - 1.0).abs() < 1e-4);
    // An f-curve is linear between its keys, flat beyond.
    let c = FCurve {
        keys: vec![(0.0, 1.0), (10.0, 3.0)],
        ..FCurve::default()
    };
    assert_eq!(c.eval(5.0, 20.0), 2.0);
    assert_eq!(c.eval(15.0, 20.0), 3.0);
}

/// One node (or a parent and its child) as an `.efkproj`.
fn project(nodes: &str) -> Vec<u8> {
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?><EffekseerProject><Root><Name>Root</Name>\
         <Children>{nodes}</Children></Root><EndFrame>60</EndFrame></EffekseerProject>"
    )
    .into_bytes()
}

#[test]
fn rings_spawned_together_keep_the_light_of_those_not_drawn() {
    // 100 additive rings in one frame at random turns: 4 drawn, each
    // carrying 25 of them (Effekseer stacks faint rings by the hundred).
    let c = run(&project(
        "<Node><CommonValues><MaxGeneration><Value>100</Value></MaxGeneration>\
         <Life><Center>20</Center><Max>20</Max><Min>20</Min></Life>\
         <GenerationTime><Center>0</Center><Max>0</Max><Min>0</Min></GenerationTime></CommonValues>\
         <RotationValues><Type>1</Type><PVA><Rotation><X><Max>180</Max><Min>-180</Min></X></Rotation></PVA></RotationValues>\
         <RendererCommonValues><AlphaBlend>2</AlphaBlend></RendererCommonValues>\
         <DrawingValues><Type>4</Type><Ring /></DrawingValues><Name>R</Name><Children /></Node>",
    ));
    let colors: Vec<_> = c
        .vfx
        .lines()
        .filter(|l| l.trim().starts_with("color "))
        .collect();
    assert_eq!(colors.len(), 4, "{}", c.vfx);
    assert!(
        colors.iter().all(|l| l.trim() == "color 25 25 25 1"),
        "{}",
        c.vfx
    );
}

#[test]
fn a_ribbon_under_a_turning_parent_orbits_about_its_axis() {
    // The parent turns 6° a frame about z (one turn a second); the ribbon
    // sits 2 out along x: an orbit of radius 2 about z, centred on the
    // parent (the element's y turned onto z).
    let c = run(&project(
        "<Node><RotationValues><Type>1</Type><PVA><Velocity><Z><Center>6</Center><Max>6</Max><Min>6</Min></Z></Velocity></PVA></RotationValues>\
         <DrawingValues><Type>0</Type></DrawingValues><Name>P</Name><Children>\
         <Node><LocationValues><Fixed><Location><X>2</X></Location></Fixed></LocationValues>\
         <DrawingValues><Type>3</Type></DrawingValues><Name>T</Name><Children /></Node>\
         </Children></Node>",
    ));
    let trail = &c.vfx[c.vfx.find("trail {").expect("a trail")..];
    assert_eq!(line(trail, "orbit"), "orbit 2 1", "{}", c.vfx);
    assert_eq!(line(trail, "rotate"), "rotate 90 0 0", "{}", c.vfx);
    assert!(
        !trail.lines().any(|l| l.trim().starts_with("offset")),
        "{}",
        c.vfx
    );
}

#[test]
fn effects_imported_into_one_folder_share_their_textures() {
    // a/ and b/ each name `Texture/p.png`, two different files; c/ names
    // a's under another folder. Imported into one folder: a's copy once
    // (c shares it), b's beside it under its content's hash; importing
    // again changes nothing.
    let root = super::super::test_dir::TempDir::new("og-efk-shared");
    let sprite = |tex: &str| {
        project(&format!(
            "<Node><RendererCommonValues><ColorTexture>{tex}</ColorTexture></RendererCommonValues>\
             <DrawingValues><Type>2</Type></DrawingValues><Name>S</Name><Children /></Node>"
        ))
    };
    for (dir, tex, bytes) in [
        ("a", "Texture/p.png", png(8, 8)),
        ("b", "Texture/p.png", png(16, 16)),
        ("c", "Other/p.png", png(8, 8)),
    ] {
        let d = root.join("src").join(dir);
        std::fs::create_dir_all(d.join(tex).parent().unwrap()).unwrap();
        std::fs::write(d.join(tex), bytes).unwrap();
        std::fs::write(d.join(format!("{dir}.efkproj")), sprite(tex)).unwrap();
    }
    let out = root.join("assets/fx");
    let import_all = || {
        for dir in ["a", "b", "c"] {
            let f = root.join("src").join(dir).join(format!("{dir}.efkproj"));
            import(&f, &out, Some(&root.join("assets"))).unwrap();
        }
    };
    import_all();
    let mut h = og_core::hash::StableHasher::new();
    h.write(&png(16, 16));
    let b_name = format!("p-{:08x}.png", h.finish() as u32);
    let mut pngs: Vec<String> = std::fs::read_dir(out.join("textures"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    pngs.sort();
    let mut want = vec!["p.png".to_string(), b_name.clone()];
    want.sort();
    assert_eq!(pngs, want);
    assert_eq!(
        std::fs::read(out.join("textures/p.png")).unwrap(),
        png(8, 8)
    );
    for (dir, tex) in [("a", "p.png"), ("b", b_name.as_str()), ("c", "p.png")] {
        let e = std::fs::read_to_string(out.join(dir).join("n0.emitter.ogt")).unwrap();
        assert_eq!(
            line(&e, "texture"),
            format!("texture \"fx/textures/{tex}\""),
            "{dir}"
        );
        assert!(!out.join(dir).join("Texture").exists());
    }
    let snapshot = |dir: &Path| -> BTreeMap<PathBuf, Vec<u8>> {
        let mut m = BTreeMap::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    m.insert(p.clone(), std::fs::read(&p).unwrap());
                }
            }
        }
        m
    };
    let before = snapshot(&out);
    import_all();
    assert_eq!(snapshot(&out), before);
}

#[test]
fn texture_names_that_differ_in_case_are_one_file() {
    // a/ names `star.png`, b/ the same bytes as `Star.png`, c/ other bytes
    // as `STAR.png`: on a case-insensitive file system these are one path,
    // so b shares a's copy under a's name and c takes its hash, and every
    // id names a file as it is spelt on disk.
    let root = super::super::test_dir::TempDir::new("og-efk-case");
    for (dir, tex, bytes) in [
        ("a", "star.png", png(8, 8)),
        ("b", "Star.png", png(8, 8)),
        ("c", "STAR.png", png(16, 16)),
    ] {
        let d = root.join("src").join(dir);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join(tex), bytes).unwrap();
        std::fs::write(
            d.join(format!("{dir}.efkproj")),
            project(&format!(
                "<Node><RendererCommonValues><ColorTexture>{tex}</ColorTexture></RendererCommonValues>\
                 <DrawingValues><Type>2</Type></DrawingValues><Name>S</Name><Children /></Node>"
            )),
        )
        .unwrap();
    }
    let out = root.join("assets/fx");
    for dir in ["a", "b", "c"] {
        let f = root.join("src").join(dir).join(format!("{dir}.efkproj"));
        import(&f, &out, Some(&root.join("assets"))).unwrap();
    }
    let mut h = og_core::hash::StableHasher::new();
    h.write(&png(16, 16));
    let c_name = format!("STAR-{:08x}.png", h.finish() as u32);
    let mut pngs: Vec<String> = std::fs::read_dir(out.join("textures"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    pngs.sort();
    let mut want = vec!["star.png".to_string(), c_name.clone()];
    want.sort();
    assert_eq!(pngs, want);
    for (dir, tex) in [("a", "star.png"), ("b", "star.png"), ("c", c_name.as_str())] {
        let e = std::fs::read_to_string(out.join(dir).join("n0.emitter.ogt")).unwrap();
        assert_eq!(
            line(&e, "texture"),
            format!("texture \"fx/textures/{tex}\""),
            "{dir}"
        );
    }
}

#[test]
fn a_node_under_an_instanced_parent_is_a_life_sub_emitter() {
    // P spawns 6 instances moving up at 0.1 a frame; each spawns sprites
    // S, 3 a frame apart, 10 of them. One parent emitter (P's 6, moving,
    // drawing nothing) whose particles each run S's emission.
    let c = run(&project(
        "<Node><CommonValues><MaxGeneration><Value>6</Value></MaxGeneration>\
         <Life><Center>40</Center><Max>40</Max><Min>40</Min></Life></CommonValues>\
         <LocationValues><Type>1</Type><PVA><Velocity><Y><Center>0.1</Center><Max>0.1</Max><Min>0.1</Min></Y></Velocity></PVA></LocationValues>\
         <DrawingValues><Type>0</Type></DrawingValues><Name>P</Name><Children>\
         <Node><CommonValues><MaxGeneration><Value>10</Value></MaxGeneration>\
         <GenerationTime><Center>3</Center><Max>3</Max><Min>3</Min></GenerationTime></CommonValues>\
         <DrawingValues><Type>2</Type></DrawingValues><Name>S</Name><Children /></Node>\
         </Children></Node>",
    ));
    assert!(
        !c.findings
            .iter()
            .any(|f| f.what.starts_with("nested generation")
                || f.what.starts_with("a moving ancestor")),
        "{:?}",
        c.findings
    );
    let v = og_bundle::vfx::parse_vfx(&c.vfx).unwrap();
    assert_eq!(v.references().emitters, ["fx/a.efkproj#n0-0.parent.ogt"]);
    let parent = file(&c, "n0-0.parent.ogt");
    assert_eq!(line(parent, "size"), "size 0");
    assert_eq!(
        line(parent, "subEmitter"),
        "subEmitter life \"fx/a.efkproj#n0-0.emitter.ogt\""
    );
    // P's own motion: 0.1 a frame up, 6 metres a second.
    assert_eq!(line(parent, "speed"), "speed 6 6", "{parent}");
    let child = file(&c, "n0-0.emitter.ogt");
    assert_eq!(line(child, "burst"), "burst 0 1 10 0.05", "{child}");
    // Room for every parent's children at once: 6 × 10 × 1.25.
    assert_eq!(line(child, "maxParticles"), "maxParticles 75", "{child}");
}

#[test]
fn a_moving_ancestor_moves_the_element_it_carries() {
    // P (one instance) moves up 0.1 a frame and falls 0.01 a frame²,
    // 30 frames after the start; its sprites, spawned 10 frames into it,
    // follow it: their emitter starts where P is then and moves on as P
    // does (no speed left after 10 frames of its fall, 36 m/s² down).
    let c = run(&project(
        "<Node><CommonValues><GenerationTimeOffset><Center>30</Center><Max>30</Max><Min>30</Min></GenerationTimeOffset></CommonValues>\
         <LocationValues><Type>1</Type><PVA>\
         <Velocity><Y><Center>0.1</Center><Max>0.1</Max><Min>0.1</Min></Y></Velocity>\
         <Acceleration><Y><Center>-0.01</Center><Max>-0.01</Max><Min>-0.01</Min></Y></Acceleration></PVA></LocationValues>\
         <DrawingValues><Type>0</Type></DrawingValues><Name>P</Name><Children>\
         <Node><CommonValues><MaxGeneration><Value>20</Value></MaxGeneration>\
         <GenerationTimeOffset><Center>10</Center><Max>10</Max><Min>10</Min></GenerationTimeOffset></CommonValues>\
         <DrawingValues><Type>2</Type></DrawingValues><Name>S</Name><Children /></Node>\
         </Children></Node>",
    ));
    assert!(
        !c.findings
            .iter()
            .any(|f| f.what.starts_with("a moving ancestor")),
        "{:?}",
        c.findings
    );
    let el = &c.vfx[c.vfx.find("emitter \"").expect("an emitter")..];
    assert_eq!(line(el, "at"), "at 0.6667", "{}", c.vfx);
    // 10 frames in: 1 − 0.5 up; then 0.1 − 0.1 = 0 a frame, −0.01 a frame².
    assert_eq!(line(el, "offset"), "offset 0 0.5 0", "{}", c.vfx);
    assert!(
        !el.lines().any(|l| l.trim().starts_with("move")),
        "{}",
        c.vfx
    );
    assert_eq!(line(el, "accelerate"), "accelerate 0 -36 0", "{}", c.vfx);
    let v = og_bundle::vfx::parse_vfx(&c.vfx).unwrap();
    // A second on: 0.5 − 18 m.
    let y = v.elements[0].local_at(1.0).w_axis.y;
    assert!((y - (0.5 - 18.0)).abs() < 1e-3, "{y}");
}

#[test]
fn a_node_out_on_an_arm_of_an_instanced_parent_stays_folded() {
    // S sits 2 out along x from each of P's turned instances (a blade's
    // tip): a sub-emitter's particles are born at their parent's point,
    // so it stays one emitter, said as a finding.
    let c = run(&project(
        "<Node><CommonValues><MaxGeneration><Value>4</Value></MaxGeneration></CommonValues>\
         <DrawingValues><Type>0</Type></DrawingValues><Name>P</Name><Children>\
         <Node><CommonValues><MaxGeneration><Value>10</Value></MaxGeneration></CommonValues>\
         <LocationValues><Fixed><Location><X>2</X></Location></Fixed></LocationValues>\
         <DrawingValues><Type>2</Type></DrawingValues><Name>S</Name><Children /></Node>\
         </Children></Node>",
    ));
    assert!(
        !c.files.iter().any(|f| f.id.ends_with(".parent.ogt")),
        "{}",
        c.vfx
    );
    assert!(
        c.findings
            .iter()
            .any(|f| f.what.starts_with("nested generation: 4 instances")),
        "{:?}",
        c.findings
    );
}

/// A sprite is drawn at its scale's mean under its generator's scale.
///
/// Ways it could fail: a constant uneven scale drawn at its longest axis
/// (its flat size curve taken as 1), or the generator's own scale left
/// off its children (only ancestors above the generator counted). Either
/// draws AndrewFM01's 2000 thin lines or Pierre02's fire as quads many
/// times their area.
#[test]
fn a_sprite_is_its_mean_scale_under_its_generators_scale() {
    let c = run(&project(
        "<Node><CommonValues><MaxGeneration><Value>10</Value></MaxGeneration></CommonValues>\
         <ScalingValues><Fixed><Scale><X>0.5</X><Y>0.5</Y><Z>0.5</Z></Scale></Fixed></ScalingValues>\
         <DrawingValues><Type>0</Type></DrawingValues><Name>G</Name><Children>\
         <Node><ScalingValues><Fixed><Scale><X>0.25</X><Y>4</Y><Z>1</Z></Scale></Fixed></ScalingValues>\
         <DrawingValues><Type>2</Type></DrawingValues><Name>S</Name><Children /></Node>\
         </Children></Node>",
    ));
    let e = file(&c, "n0-0.emitter.ogt");
    // sqrt(0.25 × 4) = 1, times the generator's 0.5.
    assert_eq!(line(e, "size"), "size 0.5 0.5", "{e}");
    assert!(!e.contains("sizeOverLife"), "{e}");
}
