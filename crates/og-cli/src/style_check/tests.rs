use super::*;
use og_cook::runtime::{CookedMesh, CookedNode, CookedPrimitive, CookedScene, Lod};
use og_render::Vertex;

const CLEAR: [u8; 4] = [0, 0, 0, 0];

#[test]
fn contract_follows_game_desc_look_without_starting_the_game() {
    let root = std::env::temp_dir().join(format!("og-style-game-look-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("assets/looks")).unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"Test\"\nentry \"main.ts\"\n",
    )
    .unwrap();
    std::fs::write(root.join("main.ts"), r#"
            import { og } from "@origingame/engine";
            const family = "custom";
            export default og.game({ look: `looks/${family}.ogt`, start() { throw new Error("must not start"); } });
        "#).unwrap();
    std::fs::write(
        root.join("assets/looks/custom.ogt"),
        "type look\nversion 1\n---\nArtLine {\n  width 2\n}\n",
    )
    .unwrap();
    assert_eq!(
        declared_look(&root).unwrap().as_deref(),
        Some("looks/custom.ogt")
    );
    crate::cook::cook(&root, 0, false).unwrap();
    let c = super::contract(&root).unwrap().unwrap();
    assert_eq!(c.look, "looks/custom.ogt");
    assert_eq!(c.line, 2.0);
    std::fs::write(
        root.join("main.ts"),
        "import { og } from '@origingame/engine'; export default og.game({});",
    )
    .unwrap();
    assert!(declared_look(&root).unwrap().is_none());
    std::fs::remove_dir_all(root).unwrap();
}

/// An image from rows of letters, each a colour of `key` ('.' clear).
fn art(rows: &[&str], key: &[(char, [u8; 3])]) -> Image {
    let h = rows.len() as u32;
    let w = rows[0].len() as u32;
    let mut px = Vec::new();
    for r in rows {
        for ch in r.chars() {
            px.push(match key.iter().find(|(k, _)| *k == ch) {
                Some((_, c)) => [c[0], c[1], c[2], 255],
                None => CLEAR,
            });
        }
    }
    Image { w, h, px }
}

fn contract() -> Contract {
    Contract {
        look: "looks/main.ogt".into(),
        palette: Some((
            "palettes/two.hex".into(),
            vec![[128, 128, 128], [200, 40, 40]],
        )),
        strict: true,
        pixels: true,
        line: 1.0,
        line_rgb: [0, 0, 0],
        normals: "flat".into(),
        ..Contract::default()
    }
}

// The palette's grey 128 and grey 136 differ by ΔE00 2.95 (by hand: L*
// 53.585 and 56.708, S_L 1.0583 at L̄ 55.15), grey 129 by about 0.95.
#[test]
fn an_on_palette_sprite_hits_and_one_three_delta_e_off_misses() {
    let key = [
        ('g', [128, 128, 128]),
        ('r', [200, 40, 40]),
        ('o', [136, 136, 136]),
        ('n', [129, 129, 129]),
    ];
    // 5 grey, 3 red, and a clear texel whose colour is ignored.
    let on = art(&["ggggg", "rrr.."], &key);
    let off = art(&["ooooo", "rrr.."], &key);
    let near = art(&["nnnnn", "rrr.."], &key);
    let c = contract();
    let pal = &c.palette.as_ref().unwrap().1;
    assert_eq!(palette_hit(&colours(&on), pal), Some(1.0));
    assert_eq!(palette_hit(&colours(&off), pal), Some(3.0 / 8.0));
    assert_eq!(palette_hit(&colours(&near), pal), Some(1.0));
    let judged = |img: &Image| {
        judge_sprite(&c, "s.png", img, false, false, &[])
            .checks
            .into_iter()
            .find(|k| k.name == "palette")
            .unwrap()
            .ok
    };
    assert!(judged(&on));
    assert!(!judged(&off));
    // Not strict: the rate is measured, not judged.
    let loose = Contract {
        strict: false,
        ..contract()
    };
    let a = judge_sprite(&loose, "s.png", &off, false, false, &[]);
    assert!(a.checks.iter().all(|k| k.name != "palette"));
    assert_eq!(a.measures["paletteHit"], json!(0.375));
}

/// `rows` drawn at `k` texels per letter.
fn scaled(rows: &[&str], k: usize) -> Vec<String> {
    rows.iter()
        .flat_map(|r| {
            let wide: String = r.chars().flat_map(|c| std::iter::repeat_n(c, k)).collect();
            std::iter::repeat_n(wide, k)
        })
        .collect()
}

#[test]
fn a_two_texel_sprite_with_a_half_pixel_column_is_found() {
    let key = [
        ('a', [10, 10, 10]),
        ('b', [90, 200, 90]),
        ('c', [220, 220, 40]),
    ];
    let logical = ["abcab.cb", "bcabca.a", ".abcabcb", "cab.bcab"];
    let clean: Vec<String> = scaled(&logical, 2);
    let rows: Vec<&str> = clean.iter().map(String::as_str).collect();
    let g = grid(&art(&rows, &key));
    assert_eq!((g.kx, g.ky), (2, 2));
    assert!(g.half_columns.is_empty() && g.half_rows.is_empty(), "{g:?}");
    // Column 13 (odd) takes column 14's colour in every row: the boundary
    // between logical pixels 6 and 7 moves half a pixel left.
    let mut half = clean.clone();
    for r in &mut half {
        let mut chars: Vec<char> = r.chars().collect();
        chars[13] = chars[14];
        *r = chars.into_iter().collect();
    }
    let rows: Vec<&str> = half.iter().map(String::as_str).collect();
    let g = grid(&art(&rows, &key));
    assert_eq!((g.kx, g.ky), (2, 2));
    assert_eq!(g.half_columns, [13]);
    assert!(g.half_rows.is_empty());
    // One texel per pixel art is not taken for a coarser grid.
    let g = grid(&art(&logical, &key));
    assert_eq!((g.kx, g.ky), (1, 1));

    // Judged against the project's grid of 2: the clean one holds, the
    // half pixel does not; against a project of 1, neither.
    let c = contract();
    let mut assets = vec![
        judge_sprite(
            &c,
            "clean.png",
            &art(&clean.iter().map(String::as_str).collect::<Vec<_>>(), &key),
            false,
            false,
            &[],
        ),
        judge_sprite(
            &c,
            "half.png",
            &art(&half.iter().map(String::as_str).collect::<Vec<_>>(), &key),
            false,
            false,
            &[],
        ),
    ];
    let mut ones = assets.clone();
    judge_grids(&c, &mut assets, Some(2));
    let ok = |a: &Asset| a.checks.iter().find(|k| k.name == "grid").unwrap().ok;
    assert!(ok(&assets[0]));
    assert!(!ok(&assets[1]));
    judge_grids(&c, &mut ones, Some(1));
    assert!(!ok(&ones[0]) && !ok(&ones[1]));
}

#[test]
fn a_tile_of_flat_ground_under_a_fringe_is_not_taken_for_a_coarser_grid() {
    // monochrome-pixel's cooked ground (32x32, one texel per pixel):
    // grass in its top five rows, solid ink below. Its row boundaries
    // are rows 1 to 5; the 26 flat rows carry none, and counted as off
    // a period of 2 they let rows 2 and 4 pass as its half pixels.
    let key = [('#', [20, 38, 27]), ('.', [201, 218, 143])];
    let mut rows = vec![
        "................................",
        ".##....#....###......#....####..",
        "###.#.####.####.###.###.##.####.",
        "######.############.############",
        "##############..##.#############",
    ];
    rows.resize(32, "################################");
    let g = grid(&art(&rows, &key));
    assert_eq!((g.ky, g.half_rows.len()), (1, 0), "{g:?}");
    // The same tile drawn at 2 texels per pixel still reads as 2 down.
    let wide = scaled(&rows, 2);
    let g = grid(&art(
        &wide.iter().map(String::as_str).collect::<Vec<_>>(),
        &key,
    ));
    assert_eq!((g.ky, g.half_rows.len()), (2, 0), "{g:?}");
}

#[test]
fn the_project_grid_is_the_sprites_most_common() {
    assert_eq!(project_grid(&[(2, 2), (1, 1), (2, 2), (3, 1)]), Some(2));
    assert_eq!(project_grid(&[(1, 1), (2, 2)]), Some(1));
    assert_eq!(project_grid(&[]), None);
}

#[test]
fn the_outline_width_is_the_rings_of_one_dark_colour() {
    let key = [
        ('k', [0, 0, 0]),
        ('f', [230, 120, 60]),
        ('s', [250, 200, 160]),
    ];
    let one = art(
        &["......", ".kkkk.", ".kffk.", ".kfsk.", ".kkkk.", "......"],
        &key,
    );
    assert_eq!(outline_width(&one, Some([0, 0, 0]), &[]), Some(1));
    let two = art(
        &[
            "........", ".kkkkkk.", ".kkkkkk.", ".kkffkk.", ".kkfskk.", ".kkkkkk.", ".kkkkkk.",
            "........",
        ],
        &key,
    );
    assert_eq!(outline_width(&two, Some([0, 0, 0]), &[]), Some(2));
    let none = art(
        &["......", ".ffff.", ".fssf.", ".fssf.", ".ffff.", "......"],
        &key,
    );
    assert_eq!(outline_width(&none, Some([0, 0, 0]), &[]), Some(0));
    // A tile cut by a clear gutter: inside its cell, its edge is not
    // clear, so only the line round its own hole is the silhouette.
    let tiles = art(&["ffff.ffff", "fkkf.ffff", "fk.f.ffff", "fkkf.ffff"], &key);
    assert_eq!(
        outline_width(&tiles, Some([0, 0, 0]), &[[0, 0, 4, 4], [5, 0, 4, 4]]),
        Some(1)
    );
    assert_eq!(outline_width(&tiles, Some([0, 0, 0]), &[]), Some(0));
}

/// A model of `tris` triangles in one primitive, drawn by `nodes` nodes;
/// the first `tilted` triangles' corners carry a normal 10° off their face.
fn model(tris: usize, nodes: usize, tilted: usize) -> CookedModel {
    let mut vertices = Vec::new();
    let n10 = [10f32.to_radians().sin(), 0.0, 10f32.to_radians().cos()];
    for t in 0..tris {
        let x = t as f32;
        let normal = if t < tilted { n10 } else { [0.0, 0.0, 1.0] };
        for p in [[x, 0.0, 0.0], [x + 1.0, 0.0, 0.0], [x, 1.0, 0.0]] {
            vertices.push(Vertex {
                position: p,
                normal,
                ..Vertex::default()
            });
        }
    }
    let count = tris * 3;
    CookedModel {
        id: "m.glb".into(),
        lights: Vec::new(),
        instances: Default::default(),
        variants: Vec::new(),
        variant_materials: Vec::new(),
        impostor: None,
        skin_binds: Default::default(),
        primitives: vec![CookedPrimitive {
            material: 0,
            vertices,
            tangents: Vec::new(),
            indices: (0..count as u32).collect(),
            lods: vec![Lod {
                first: 0,
                count,
                error: 0.0,
                ..Default::default()
            }],
            clusters: Vec::new(),
            cluster_indices: Vec::new(),
            min: glam::Vec3::ZERO,
            max: glam::Vec3::ONE,
            bvh_parts: None,
            collider: None,
            foliage: None,
        }],
        meshes: vec![CookedMesh {
            name: "m".into(),
            primitives: 0..1,
        }],
        materials: Vec::new(),
        textures: Vec::new(),
        nodes: (0..nodes)
            .map(|i| CookedNode {
                name: format!("n{i}"),
                parent: None,
                mesh: Some(0),
                translation: glam::Vec3::new(0.0, i as f32 * 2.0, 0.0),
                rotation: glam::Quat::IDENTITY,
                scale: glam::Vec3::ONE,
                visible: true,
            })
            .collect(),
        scenes: vec![CookedScene {
            name: "s".into(),
            roots: (0..nodes).collect(),
        }],
        default_scene: Some(0),
        min: glam::Vec3::ZERO,
        max: glam::Vec3::ONE,
        skipped_animations: 0,
        skipped_skins: 0,
    }
}

#[test]
fn a_300_face_prop_is_over_200_and_within_500() {
    // 100 triangles drawn three times: 300 faces, not 100.
    let m = measure_model(&model(100, 3, 30), &|_| None);
    assert_eq!(m.faces, 300);
    // 30 of 100 triangles' corners are 10° off their face, in every node.
    assert_eq!(m.flat_share, Some(0.7));
    let ok = |budget: u32| {
        let c = Contract {
            prop_faces: budget,
            ..contract()
        };
        judge_model(&c, "m.glb", &m, ModelRole::Prop)
            .checks
            .iter()
            .find(|k| k.name == "faces")
            .unwrap()
            .ok
    };
    assert!(!ok(200));
    assert!(ok(500));
    let normals = judge_model(&contract(), "m.glb", &m, ModelRole::Prop);
    assert!(
        !normals
            .checks
            .iter()
            .find(|k| k.name == "normals")
            .unwrap()
            .ok
    );
    // A character is held to characterFaces, not propFaces.
    let c = Contract {
        prop_faces: 200,
        character_faces: 500,
        ..contract()
    };
    let a = judge_model(&c, "m.glb", &m, ModelRole::Character);
    assert!(a.checks.iter().find(|k| k.name == "faces").unwrap().ok);
    // An effect's mesh is held to no face budget.
    let a = judge_model(&c, "m.glb", &m, ModelRole::Effect);
    assert!(a.checks.iter().all(|k| k.name != "faces"));
    assert_eq!(
        measure_model(&model(10, 1, 0), &|_| None).flat_share,
        Some(1.0)
    );
}

/// The rotation looking `pitch` degrees below the horizon turned `yaw`
/// about +Y (Ry(yaw) Rx(-pitch)), as {x, y, z, w}, worked out by hand.
fn camera_quat(pitch: f64, yaw: f64) -> Value {
    let (sx, cx) = (-pitch.to_radians() / 2.0).sin_cos();
    let (sy, cy) = (yaw.to_radians() / 2.0).sin_cos();
    json!({ "x": cy * sx, "y": sy * cx, "z": -sy * sx, "w": cy * cx })
}

/// ArtCamera measured on the captured camera: projection, the fixed
/// angle and the field of view, each judged only where the contract
/// sets it. Old code measured no camera at all.
#[test]
fn the_check_measures_the_frames_camera_against_art_camera() {
    let iso = Contract {
        projection: "orthographic".into(),
        fixed_angle: true,
        pitch: 30.0,
        yaw: 45.0,
        ..contract()
    };
    let cam = |ortho: bool, pitch: f64, yaw: f64, fov: f64| json!({ "orthographic": ortho, "quaternion": camera_quat(pitch, yaw), "fov": fov });
    let by = |checks: &[Check], n: &str| checks.iter().find(|c| c.name == n).map(|c| c.ok);
    let (m, checks) = judge_camera(&iso, &cam(true, 30.0, 45.0, 50.0));
    assert!((m["pitch"].as_f64().unwrap() - 30.0).abs() < 1e-6, "{m}");
    assert!((m["yaw"].as_f64().unwrap() - 45.0).abs() < 1e-6, "{m}");
    assert!(checks.iter().all(|c| c.ok), "{checks:?}");
    let (_, checks) = judge_camera(&iso, &cam(false, 30.0, 45.0, 50.0));
    assert_eq!(
        by(&checks, "projection"),
        Some(false),
        "a perspective camera"
    );
    let (_, checks) = judge_camera(&iso, &cam(true, 20.0, -45.0, 50.0));
    assert_eq!(by(&checks, "pitch"), Some(false));
    assert_eq!(by(&checks, "yaw"), Some(false));
    // Straight down has no heading: any yaw holds.
    let top = Contract {
        pitch: 90.0,
        yaw: 0.0,
        ..iso.clone()
    };
    let (_, checks) = judge_camera(&top, &cam(true, 90.0, 0.0, 50.0));
    assert_eq!(by(&checks, "yaw"), Some(true), "{checks:?}");
    // The field of view, for a perspective camera only.
    let lens = Contract {
        fov_min: 30.0,
        fov_max: 45.0,
        ..contract()
    };
    let (_, checks) = judge_camera(&lens, &cam(false, 0.0, 0.0, 60.0));
    assert_eq!(by(&checks, "fov"), Some(false));
    let (_, checks) = judge_camera(&lens, &cam(false, 0.0, 0.0, 40.0));
    assert_eq!(by(&checks, "fov"), Some(true));
    let (_, checks) = judge_camera(&contract(), &cam(false, 10.0, 0.0, 90.0));
    assert!(checks.is_empty(), "no ArtCamera, no checks: {checks:?}");
}

/// ArtMotion measured on the cooked clips: a model clip's rate and held
/// poses against stepFps and its frames against maxFrames, a
/// flipbook's frame durations as whole steps.
#[test]
fn the_check_measures_clips_against_art_motion() {
    let c = Contract {
        step_fps: 12.0,
        max_frames: 8,
        ..contract()
    };
    let clip = |rate: f32, frames: u32, step: bool| og_anim::ClipAsset {
        name: "walk".into(),
        duration: (frames - 1) as f32 / rate,
        rate,
        source_rate: 30.0,
        frames,
        channels: vec![og_anim::Channel {
            node: 0,
            node_name: "hip".into(),
            path: og_anim::ChannelPath::Translation,
            step,
            lanes: 3,
            samples: og_anim::Samples::F32(vec![0.0; frames as usize * 3]),
        }],
        events: Vec::new(),
    };
    let ok = |a: &Asset, n: &str| a.checks.iter().find(|k| k.name == n).unwrap().ok;
    let held = judge_clip(&c, "m.glb#anim", &clip(12.0, 7, true));
    assert!(
        ok(&held, "stepFps") && ok(&held, "maxFrames"),
        "{:?}",
        held.checks
    );
    assert_eq!(held.role, Role::Clip);
    let smooth = judge_clip(&c, "m.glb#anim", &clip(30.0, 31, false));
    assert!(!ok(&smooth, "stepFps") && !ok(&smooth, "maxFrames"));
    let interpolated = judge_clip(&c, "m.glb#anim", &clip(12.0, 7, false));
    assert!(!ok(&interpolated, "stepFps"), "a pose not held");
    let sheet = og_anim::FlipbookSheet::parse(
            "type flipbook\nversion 1\n---\ntexture \"t.png\"\nsize 64 8\ngrid 8 8\nclip \"run\" {\n  frames 0 1 2 3\n  fps 30\n}\n",
        )
        .unwrap();
    let fb = judge_flipbook(&c, "run.ogt", &sheet);
    assert_eq!(fb.len(), 1);
    assert!(
        !ok(&fb[0], "stepFps"),
        "1/30 s frames are not whole 1/12 s steps"
    );
    assert!(ok(&fb[0], "maxFrames"));
    let mut stepped = sheet.clone();
    stepped.clips[0].durations.fill(2.0 / 12.0);
    assert!(ok(&judge_flipbook(&c, "run.ogt", &stepped)[0], "stepFps"));
}

fn project(text: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "og-style-screens-{}-{}",
        std::process::id(),
        N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("project.ogt"),
        format!("type project\nversion 1\n---\nname \"x\"\n{text}"),
    )
    .unwrap();
    dir
}

// A game's own screens are captured too: title, play and results first
// (whatever order the block writes them), then the block's others in
// order, then the ui block's screens it does not name; a flag replaces
// its screen's script. A scene with no screen of its name is uncovered,
// but the start scene counts as covered by play.
#[test]
fn every_declared_screen_is_captured_and_uncovered_scenes_are_named() {
    let dir = project(
        "scenes {\n  main \"scenes/main.ogt\"\n  shop \"scenes/shop.ogt\"\n  map \"scenes/map.ogt\"\n}\nscene main\nscreens {\n  shop \"1:screenshot\"\n  results \"2:screenshot\"\n  play \"0.5:key:Enter\"\n}\nui {\n  screen \"hud\" \"0:key:Enter\"\n  screen \"shop\" \"9:key:KeyS\"\n}\n",
    );
    let flags = BTreeMap::from([("results".to_string(), "3:key:Escape".to_string())]);
    let got = screens(&dir, &flags).unwrap();
    assert_eq!(
        got,
        vec![
            (
                "play".to_string(),
                "0.5:key:Enter;0.5:screenshot".to_string()
            ),
            (
                "results".to_string(),
                "3:key:Escape;3:screenshot".to_string()
            ),
            ("shop".to_string(), "1:screenshot".to_string()),
            ("hud".to_string(), "0:key:Enter;0:screenshot".to_string()),
        ]
    );
    let names: Vec<String> = got.into_iter().map(|(n, _)| n).collect();
    assert_eq!(
        uncovered_scenes(&dir, &names).unwrap(),
        vec!["map".to_string()]
    );
    // Without play, the start scene is uncovered too.
    assert_eq!(
        uncovered_scenes(&dir, &["shop".to_string()]).unwrap(),
        vec!["main".to_string(), "map".to_string()]
    );
    // A screen without commands, or named twice, is refused with its line.
    let bad = project("screens {\n  shop 3\n}\n");
    assert!(
        screens(&bad, &BTreeMap::new())
            .unwrap_err()
            .contains("project.ogt:6")
    );
    let twice = project("screens {\n  shop \"1:screenshot\"\n  shop \"2:screenshot\"\n}\n");
    assert!(
        screens(&twice, &BTreeMap::new())
            .unwrap_err()
            .contains("twice")
    );
    for d in [dir, bad, twice] {
        let _ = std::fs::remove_dir_all(d);
    }
}

// `phone` follows the game's orientation; WxH passes as itself; an
// unknown name or a size given twice is refused.
#[test]
fn sizes_name_viewports_and_phone_follows_the_orientation() {
    assert_eq!(
        viewports("desktop,phone", false).unwrap(),
        vec![
            ("desktop".to_string(), "1280x720".to_string()),
            ("phoneLandscape".to_string(), "844x390".to_string()),
        ]
    );
    assert_eq!(
        viewports("phone,1600x900", true).unwrap(),
        vec![
            ("phonePortrait".to_string(), "390x844".to_string()),
            ("1600x900".to_string(), "1600x900".to_string()),
        ]
    );
    assert!(viewports("watch", false).is_err());
    assert!(viewports("desktop,1280x720", false).is_err());
    let p = project("publish {\n  orientation portrait\n}\n");
    assert!(portrait(&p));
    let _ = std::fs::remove_dir_all(p);
}

#[test]
fn the_tier_comes_from_the_runs_last_json_line() {
    let out = "{\"event\":\"check\",\"ok\":true}\n{\"tick\":120,\"tier\":\"high\"}\n";
    assert_eq!(run_tier(out).as_deref(), Some("high"));
    assert_eq!(run_tier("tick 120 hash 1\n"), None);
}

#[test]
fn a_script_gets_a_screenshot_at_its_last_step() {
    assert_eq!(
        with_screenshot("1:key:Escape;2.5:wait"),
        "1:key:Escape;2.5:wait;2.5:screenshot"
    );
    assert_eq!(
        with_screenshot("3:screenshot;4:quit"),
        "3:screenshot;4:quit"
    );
}
