use super::*;

/// A graph asset resolves clip names at cook, keeps bindings in its
/// typed blob, and revalidates them when the source changes.
#[test]
fn animation_graph_cooks_with_tracked_clip_bindings() {
    use og_anim::cooked::BundleAnim;
    let root = TempDir::new("og-generated-graph");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"Graph\"\n",
    )
    .unwrap();
    let model =
        std::fs::read(repo().join("tests/fixtures/anim-markers/assets/models/walker.glb")).unwrap();
    std::fs::write(assets.join("walker.glb"), &model).unwrap();
    std::fs::write(
        assets.join("walker.glb.ogt"),
        "type import\nversion 1\n---\nconform off\n",
    )
    .unwrap();
    let graph = "type animgraph\nversion 1\n---\nbind \"stride\" \"walker.glb\" \"walk\"\nclip moving \"stride\"\nlayer base {\nstate active moving\nentry active\n}\n";
    std::fs::write(assets.join("graph.ogt"), graph).unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let source = bundle
        .animgraph("graph.ogt", &bundle.read("graph.ogt").unwrap())
        .unwrap();
    assert!(!source.sprite);
    assert_eq!(
        (
            source.bindings[0].name.as_str(),
            source.bindings[0].clip.as_str()
        ),
        ("stride", "walk")
    );
    let mut renamed = model;
    for i in 0..renamed.len() - 3 {
        if &renamed[i..i + 4] == b"walk" {
            renamed[i..i + 4].copy_from_slice(b"hike");
        }
    }
    std::fs::write(assets.join("walker.glb"), renamed).unwrap();
    let out = cook();
    assert!(!out.status.success());
    assert!(text(&out).contains("has no clip 'walk'"), "{}", text(&out));
    std::fs::write(
        assets.join("graph.ogt"),
        graph.replace("\"walk\"", "\"hike\""),
    )
    .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::write(
        assets.join("graph.ogt"),
        graph.replace("bind \"stride\" \"walker.glb\" \"walk\"", "sprite true"),
    )
    .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    std::fs::write(
        assets.join("graph.ogt"),
        graph.replace("state active moving", "state active absent"),
    )
    .unwrap();
    let out = cook();
    assert!(!out.status.success(), "invalid graph accepted");
}

/// Public lattice cook/load and emitter consumption retain channel order,
/// float precision and tracked generator dependencies.
#[test]
fn generated_fields_load_losslessly_and_feed_particles() {
    let root = TempDir::new("og-generated-fields");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"Field\"\n",
    )
    .unwrap();
    std::fs::write(assets.join("source.ogt"), "type field\nversion 1\n---\ngenerate {\nscript \"field.ts\"\ninput \"amplitude\" \"input.bin\"\n}\n").unwrap();
    std::fs::write(assets.join("field.ts"), "export default ({inputs}) => ({dimensions: [2, 2, 2], channels: 3, origin: [-3, 2, 7], spacing: [0.5, 2, 3], values: Float32Array.from({length: 24}, (_, i) => inputs.amplitude[0] + i / 32)});").unwrap();
    std::fs::write(assets.join("input.bin"), [7]).unwrap();
    std::fs::write(
        assets.join("particles.ogt"),
        "type particles\nversion 1\n---\nfield \"source.ogt\" 0.25 9\n",
    )
    .unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    for amplitude in [7, 11] {
        std::fs::write(assets.join("input.bin"), [amplitude]).unwrap();
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let field = bundle.load_field("source.ogt").unwrap();
        assert_eq!(field.dimensions, [2, 2, 2]);
        assert_eq!(field.channels, 3);
        assert_eq!(field.origin, [-3.0, 2.0, 7.0]);
        assert_eq!(field.spacing, [0.5, 2.0, 3.0]);
        let expected: Vec<f32> = (0..24)
            .map(|i| f32::from(amplitude) + i as f32 / 32.0)
            .collect();
        assert_eq!(field.values, expected);
        let bytes = bundle.read("particles.ogt").unwrap();
        let text = bundle.emitter_text("particles.ogt", &bytes).unwrap();
        let emitter = og_cook::fx::parse_emitter(&text, &mut |_| Ok(None)).unwrap();
        let og_render_data::VectorField::Grid {
            strength,
            size,
            cells,
            vectors,
        } = emitter.field.unwrap()
        else {
            panic!("not a grid")
        };
        assert_eq!((strength, size, cells), (0.25, 9.0, 2));
        assert_eq!(
            vectors
                .into_iter()
                .flat_map(|v| v.to_array())
                .collect::<Vec<_>>(),
            expected
        );
    }
    // A rectangular scalar lattice is valid by itself, not a particle grid.
    std::fs::remove_file(assets.join("particles.ogt")).unwrap();
    std::fs::write(assets.join("field.ts"), "export default () => ({dimensions: [3, 2, 1], channels: 1, values: [117, -2, 0.125, 4, 9, 0]});").unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle.load_field("source.ogt").unwrap().values,
        [117.0, -2.0, 0.125, 4.0, 9.0, 0.0]
    );
    for invalid in [
        "({dimensions: [3, 2, 1], channels: 1, values: [1, 2]})",
        "({dimensions: [0, 2, 1], channels: 1, values: []})",
        "({dimensions: [1, 1, 1], channels: 1, spacing: [1, 0, 1], values: [1]})",
        "({dimensions: [1, 1, 1], channels: 5, values: [1, 2, 3, 4, 5]})",
    ] {
        std::fs::write(
            assets.join("field.ts"),
            format!("export default () => {invalid};"),
        )
        .unwrap();
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}: {}", text(&out));
    }
}

/// The public offline executor: dependencies and parameters invalidate a
/// content key, moving a project does not, and ambient authority is absent.
#[test]
fn script_tracks_its_closed_inputs_and_rejects_ambient_authority() {
    let root = TempDir::new("og-script");
    let run = |name: &str, params: &str| {
        og().args([
            "script",
            name,
            "--params",
            params,
            "--input",
            "bytes=input.bin",
            "--json",
        ])
        .current_dir(&root)
        .output()
        .unwrap()
    };
    let value = |out: &Output| -> serde_json::Value {
        assert!(out.status.success(), "{}", text(out));
        serde_json::from_slice(&out.stdout).unwrap()
    };
    std::fs::write(root.join("input.bin"), [7, 11]).unwrap();
    std::fs::write(root.join("shape.ts"), "export const offset: number = 3;\n").unwrap();
    std::fs::write(root.join("main.ts"), r#"
        import {offset} from './shape.ts';
        export default ({params, inputs}: {params: {scale: number}; inputs: Record<string, number[]>}) =>
            new Float32Array([offset * params.scale, inputs.bytes[1], Math.sin(0.7), 2 ** 0.3]);
    "#).unwrap();
    let first = value(&run("main.ts", "{\"scale\":2}"));
    assert_eq!(first["data"][0], 6.0);
    assert_eq!(first["data"][1], 11.0);
    assert_eq!(first, value(&run("main.ts", "{\"scale\":2}")));
    assert_eq!(first, value(&run("main.ts", "{\"scale\":2.0}")));
    assert_ne!(first["key"], value(&run("main.ts", "{\"scale\":4}"))["key"]);
    std::fs::write(root.join("shape.ts"), "export const offset: number = 5;\n").unwrap();
    let changed = value(&run("main.ts", "{\"scale\":2}"));
    assert_ne!(first["key"], changed["key"]);
    assert_eq!(changed["data"][0], 10.0);
    std::fs::write(root.join("input.bin"), [7, 13]).unwrap();
    let input = value(&run("main.ts", "{\"scale\":2}"));
    assert_ne!(changed["key"], input["key"]);
    assert_eq!(input["data"][1], 13.0);
    let moved = TempDir::new("og-script-moved");
    for file in ["main.ts", "shape.ts", "input.bin"] {
        std::fs::copy(root.join(file), moved.join(file)).unwrap();
    }
    let output = og()
        .args([
            "script",
            "main.ts",
            "--params",
            "{\"scale\":2}",
            "--input",
            "bytes=input.bin",
            "--json",
        ])
        .current_dir(&moved)
        .output()
        .unwrap();
    assert_eq!(input, value(&output));

    std::fs::write(root.join("mechanisms.ts"), r#"
        import {procgen} from '@origingame/engine/generate';
        export default () => {
            const h = procgen.heightmap({columns: 3, rows: 2, size: {x: 11, y: 7}, heights: new Float32Array([117, 119, 123, 127, 131, 137])});
            const w = procgen.wfc({width: 3, height: 1, tiles: [
                {weight: 1, up: 0, right: 1, down: 0, left: 2, rotations: false},
                {weight: 1, up: 0, right: 2, down: 0, left: 1, rotations: false},
            ], fixed: [{x: 0, y: 0, tile: 0, quarterTurns: 0}]});
            if (!h.ok || !w.ok) throw new Error('mechanism refused');
            if (!(h.value.heights instanceof Float32Array) || !(w.value.tiles instanceof Uint32Array)) throw new Error('lost typed buffers');
            return {heights: h.value.heights.subarray(1, 5), tiles: w.value.tiles};
        };
    "#).unwrap();
    let mechanisms = value(&run("mechanisms.ts", "{}"));
    assert_eq!(
        mechanisms["data"]["heights"],
        serde_json::json!([119, 123, 127, 131])
    );
    assert_eq!(mechanisms["data"]["tiles"], serde_json::json!([0, 1, 0]));

    let exported = og()
        .args([
            "script",
            "mechanisms.ts",
            "--out",
            "out/nested/result.json",
            "--json",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert_eq!(value(&exported)["data"], serde_json::Value::Null);
    let published = std::fs::read(root.join("out/nested/result.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&published).unwrap(),
        mechanisms["data"]
    );

    for code in [
        "export default () => Date.now()",
        "export default () => Math.random()",
        "export default () => fetch('https://example.invalid')",
        "export default () => process.env.HOME",
        "import fs from 'node:fs'; export default () => fs.readFileSync('/etc/passwd')",
        "export default () => (()=>{}).constructor('return 1')()",
        "export default () => eval('2 ** 0.3')",
        "export default async () => [1]",
        "export default () => [Infinity]",
        "export default () => { while (true) {} }",
    ] {
        std::fs::write(root.join("denied.ts"), code).unwrap();
        let out = run("denied.ts", "{}");
        assert!(
            !out.status.success(),
            "accepted forbidden generator {code}: {}",
            text(&out)
        );
        assert!(out.stdout.is_empty(), "partial result for {code}");
    }
    let refused = og()
        .args([
            "script",
            "denied.ts",
            "--out",
            "out/nested/result.json",
            "--json",
        ])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(!refused.status.success());
    assert_eq!(
        std::fs::read(root.join("out/nested/result.json")).unwrap(),
        published
    );
    let out = run("../outside.ts", "{}");
    assert!(!out.status.success());
    assert!(text(&out).contains("leaves its input root"));
}

/// Generated cells pass through the real map compiler, preserve authored
/// metadata, and change when tracked script input changes.
#[test]
fn generated_tilemap_cooks_loads_and_rejects_invalid_grids() {
    let root = TempDir::new("og-generated-map");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"Map\"\n",
    )
    .unwrap();
    std::fs::write(
        assets.join("tiles.ogt"),
        "type tileset\nversion 1\n---\ntexture \"\"\ngrid 8 8 3 1\n",
    )
    .unwrap();
    let map = "type tilemap\nversion 1\n---\ntile 8 16\nunit 3\ntileset \"tiles.ogt\"\ngenerate {\nscript \"grid.ts\"\ninput \"source\" \"cell.bin\"\n}\nlayer \"cells\" {\nz 4\nparallax 0.6 0.8\n}\nlayer \"manual\" {\nrow 2 -1 0\nrow 1 2 -1\n}\n";
    std::fs::write(assets.join("map.ogt"), map).unwrap();
    std::fs::write(assets.join("cell.bin"), [0]).unwrap();
    std::fs::write(assets.join("grid.ts"), "export default ({inputs}) => ({width: 3, height: 2, layers: [{name: 'cells', tiles: new Int32Array([inputs.source[0], -1, 2, 1, 0, 2])}]});").unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let loaded = || {
        og_cook::runtime::Bundle::open(&root.join("cooked"))
            .unwrap()
            .load_tilemap("map.ogt")
            .unwrap()
    };
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let first = loaded();
    assert_eq!((first.width, first.height), (3, 2));
    assert_eq!(first.tile_size().to_array(), [3.0, 6.0]);
    // Stored cells are tile id + 1, zero for empty. This distinguishes
    // a real tile zero from an empty cell and catches transposed rows.
    assert_eq!(first.layers[0].cells, [1, 0, 3, 2, 1, 3]);
    assert_eq!(first.layers[1].cells, [3, 0, 1, 2, 3, 0]);
    assert_eq!(first.layers[0].z, 4);
    assert_eq!(first.layers[0].parallax.to_array(), [0.6, 0.8]);
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(loaded(), first);
    std::fs::write(assets.join("cell.bin"), [2]).unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    assert_eq!(loaded().layers[0].cells, [3, 0, 3, 2, 1, 3]);

    for source in [
        "({width: 3, height: 2, layers: [{name: 'cells', tiles: [0, 1, 2]}]})",
        "({width: 3, height: 2, layers: [{name: 'absent', tiles: [0, 1, 2, 0, 1, 2]}]})",
        "({width: 3, height: 2, layers: [{name: 'cells', tiles: [0, 1, 2, 0, -2, 2]}]})",
        "({width: 0, height: 2, layers: []})",
    ] {
        std::fs::write(
            assets.join("grid.ts"),
            format!("export default () => {source};"),
        )
        .unwrap();
        let out = cook();
        assert!(
            !out.status.success(),
            "accepted invalid map {source}: {}",
            text(&out)
        );
    }
}

/// Float source samples follow the same clamping, channel order and sound
/// encoding as an authored WAV, rather than a parallel generator codec.
#[test]
fn generated_pcm_cooks_as_an_authored_sound() {
    let root = TempDir::new("og-generated-sound");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"PCM\"\n",
    )
    .unwrap();
    std::fs::write(
        assets.join("made.ogt"),
        "type sound\nversion 1\n---\ngenerate {\nscript \"samples.ts\"\n}\n",
    )
    .unwrap();
    std::fs::write(assets.join("samples.ts"), "export default () => ({rate: 22050, channels: 2, samples: Float32Array.from({length: 600}, (_, i) => [-2, 0.25, 0.5, -0.75, 2, -0.5][i % 6])});").unwrap();
    let mut writer = hound::WavWriter::create(
        assets.join("authored.wav"),
        hound::WavSpec {
            channels: 2,
            sample_rate: 22050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    for _ in 0..100 {
        for sample in [-32767i16, 8192, 16384, -24575, 32767, -16384] {
            writer.write_sample(sample).unwrap();
        }
    }
    writer.finalize().unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let made = bundle.load_sound("made.ogt").unwrap().decode().unwrap();
    let authored = bundle.load_sound("authored.wav").unwrap().decode().unwrap();
    assert_eq!(
        (made.sample_rate, made.channels, made.samples.len()),
        (22050, 2, 600)
    );
    assert_eq!(made.samples, authored.samples);
    for source in [
        "({rate: 22050, channels: 2, samples: [0, 0.5, 1]})",
        "({rate: 7999, channels: 1, samples: [0]})",
        "({rate: 22050, channels: 3, samples: [0, 0, 0]})",
        "({rate: 22050, channels: 1, samples: []})",
    ] {
        std::fs::write(
            assets.join("samples.ts"),
            format!("export default () => {source};"),
        )
        .unwrap();
        let out = cook();
        assert!(
            !out.status.success(),
            "accepted invalid PCM {source}: {}",
            text(&out)
        );
    }
}

/// Typed source attributes cross the public cook/load boundary, including
/// a mirrored merge's tangent handedness and vertex colours without a map.
#[test]
fn generated_mesh_cooks_with_attributes_and_placed_frames() {
    let root = TempDir::new("og-generated-mesh");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"Mesh\"\n",
    )
    .unwrap();
    std::fs::write(
        assets.join("mesh.ogt"),
        r#"type model
version 1
---
conform off
part "source" {
  bufferGeometry {
    generate {
      script "mesh.ts"
    }
  }
}
part "placed" {
  mergeGeometries {
    part {
      bufferGeometry {
        generate {
          script "mesh.ts"
        }
      }
      scale 2 1 3
      mirror x
    }
  }
}
part "mixed" {
  mergeGeometries {
    part {
      bufferGeometry {
        generate {
          script "mesh.ts"
        }
      }
    }
    part {
      bufferGeometry {
        positions 10 0 0 12 0 0 10 1 0
        uvs 0 1 1 1 0 0
      }
    }
  }
}
"#,
    )
    .unwrap();
    let source = "({positions: [-2, 0, 0, 3, 0, 0, 1, 4, 0], colors: [1,0,0, 0,1,0, 0,0,1], uvs: [0,1, 1,1, 0.6,0], tangents: [1,0,0,-1, 1,0,0,-1, 1,0,0,-1], indices: [0,1,2]})";
    std::fs::write(
        assets.join("mesh.ts"),
        format!("export default () => {source};"),
    )
    .unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let model = bundle.load_model("mesh.ogt").unwrap();
    assert_eq!(model.primitives.len(), 3);
    for (i, p) in model.primitives.iter().take(2).enumerate() {
        assert_eq!(p.lods[0].count, 3);
        assert_eq!(p.vertices.len(), 3);
        assert_eq!(p.tangents.len(), 3);
        let (x, w) = if i == 0 { (1.0, -1.0) } else { (-1.0, 1.0) };
        for (v, t) in p.vertices.iter().zip(&p.tangents) {
            assert!((v.normal[2] - 1.0).abs() < 0.001);
            assert!((t[0] - x).abs() < 0.001 && t[1].abs() < 0.001 && t[2].abs() < 0.001);
            assert_eq!(t[3], w);
            let c = if v.position[1] > 3.0 {
                assert!(v.uv[1].abs() < 0.001);
                [0.0, 0.0, 1.0, 1.0]
            } else if v.position[0] * x < 0.0 {
                [1.0, 0.0, 0.0, 1.0]
            } else {
                [0.0, 1.0, 0.0, 1.0]
            };
            assert_eq!(v.color, c);
        }
        assert_eq!(p.min.x, if i == 0 { -2.0 } else { -6.0 });
        assert_eq!(p.max.x, if i == 0 { 3.0 } else { 4.0 });
    }
    let mixed = &model.primitives[2];
    assert_eq!(mixed.lods[0].count, 6);
    assert_eq!(mixed.tangents.len(), 6);
    for (v, t) in mixed.vertices.iter().zip(&mixed.tangents) {
        assert!((t[0] - 1.0).abs() < 0.001 && t[1].abs() < 0.001 && t[2].abs() < 0.001);
        // The supplied frame stays -1; the other triangle's decreasing V
        // gives an image-up bitangent along +Y, hence +1 for N=Z, T=X.
        assert_eq!(t[3], if v.position[0] > 9.0 { 1.0 } else { -1.0 });
    }
    // The consumer sorts before its source. Import order is a dependency,
    // not filename order, and a selected node keeps its rest placement.
    let operand = "type model\nversion 1\n---\nconform off\npart \"copy\" {\nmergeGeometries {\npart {\nmodelGeometry {\nsource \"mesh.ogt\"\nnode \"placed\"\n}\nposition 0 2 0\n}\n}\n}\n";
    std::fs::write(assets.join("a-copy.ogt"), operand).unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    let copy = bundle.load_model("a-copy.ogt").unwrap();
    let p = &copy.primitives[0];
    assert_eq!((p.min.x, p.max.x, p.min.y, p.max.y), (-6.0, 4.0, 2.0, 6.0));
    for (v, t) in p.vertices.iter().zip(&p.tangents) {
        assert!((t[0] + 1.0).abs() < 0.001);
        assert_eq!(t[3], 1.0);
        assert_eq!(v.color.iter().sum::<f32>(), 2.0);
    }
    std::fs::write(
        assets.join("mesh.ts"),
        format!(
            "export default () => {};",
            source.replace("-2, 0, 0", "-4, 0, 0")
        ),
    )
    .unwrap();
    let out = cook();
    assert!(out.status.success(), "{}", text(&out));
    let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
    assert_eq!(
        bundle.load_model("a-copy.ogt").unwrap().primitives[0].max.x,
        8.0
    );
    std::fs::write(
        assets.join("mesh.ogt"),
        operand
            .replace("mesh.ogt", "a-copy.ogt")
            .replace("node \"placed\"", ""),
    )
    .unwrap();
    let out = cook();
    assert!(!out.status.success());
    assert!(text(&out).contains("dependency cycle"), "{}", text(&out));
    std::fs::remove_file(assets.join("a-copy.ogt")).unwrap();
    std::fs::write(assets.join("mesh.ogt"), "type model\nversion 1\n---\nconform off\npart \"source\" {\nbufferGeometry {\ngenerate {\nscript \"mesh.ts\"\n}\n}\n}\n").unwrap();
    for invalid in [
        "({positions: [0,0,0, 1,0,0, 0,1,0], indices: [0,1,3]})",
        "({positions: [0,0,0, 1,0,0, 0,1,0], colors: [1,0,0]})",
        "({positions: [0,0,0, 1,0,0, 0,1,0], tangents: [1,0,0,-1]})",
        "({positions: [0,0,0, 1,0,0, 0,1,0], indices: [0,1,1.5]})",
    ] {
        std::fs::write(
            assets.join("mesh.ts"),
            format!("export default () => {invalid};"),
        )
        .unwrap();
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}: {}", text(&out));
    }
}

/// Generated declarations use the same scene/prefab/reference compiler as
/// authored ones, including input invalidation and sub-scene cycle refusal.
#[test]
fn generated_scene_cooks_as_authored_declarations() {
    let root = TempDir::new("og-generated-scene");
    let assets = root.join("assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join("project.ogt"),
        "type project\nversion 1\n---\nname \"Scene\"\n",
    )
    .unwrap();
    std::fs::write(
        assets.join("sub.ogt"),
        "type scene\nversion 1\n---\nentity \"nested\" {\n}\n",
    )
    .unwrap();
    std::fs::write(
        assets.join("model.ogt"),
        "type model\nversion 1\n---\nconform off\npart \"part\" {\nboxGeometry {\n}\n}\n",
    )
    .unwrap();
    std::fs::write(assets.join("base.ogt"), "type prefab\nversion 1\n---\nentity \"base\" {\nStats {\nweight 2\n}\nentity \"prefab-child\" {\nStats {\nweight 4\n}\n}\n}\n").unwrap();
    let prefix = "type scene\nversion 1\n---\nname \"Generated\"\nentity \"manual\" {\nStats {\nweight 1\n}\n}\n";
    std::fs::write(
        assets.join("made.ogt"),
        format!("{prefix}generate {{\nscript \"scene.ts\"\ninput \"source\" \"weight.bin\"\n}}\n"),
    )
    .unwrap();
    std::fs::write(assets.join("scene.ts"), r#"export default ({inputs}) => ({
        entities: [{name: "generated", prefab: "base.ogt", components: {Stats: {weight: inputs.source[0]}}, children: [
            {name: "instance", model: "model.ogt", static: true},
            {name: "cell", scene: "sub.ogt"}
        ]}], resources: {Run: {count: 3, enabled: true, weights: [2, 7, 3]}}
    });"#).unwrap();
    let cook = || {
        og().args(["cook", ".", "--json"])
            .current_dir(&root)
            .output()
            .unwrap()
    };
    for weight in [7, 9] {
        std::fs::write(assets.join("weight.bin"), [weight]).unwrap();
        std::fs::write(assets.join("authored.ogt"), format!("{prefix}entity \"generated\" {{\nprefab \"base.ogt\"\nStats {{\nweight {weight}\n}}\nentity \"instance\" {{\nmodel \"model.ogt\"\nstatic true\n}}\nentity \"cell\" {{\nscene \"sub.ogt\"\n}}\n}}\nresource Run {{\ncount 3\nenabled true\nweights 2 7 3\n}}\n")).unwrap();
        let out = cook();
        assert!(out.status.success(), "{}", text(&out));
        let bundle = og_cook::runtime::Bundle::open(&root.join("cooked")).unwrap();
        let made = bundle.load_scene("made.ogt").unwrap();
        let authored = bundle.load_scene("authored.ogt").unwrap();
        assert_eq!(made.entities.len(), 5);
        for (mut a, mut b) in made.entities.into_iter().zip(authored.entities) {
            a.line = None;
            b.line = None;
            assert_eq!(a, b);
        }
        assert_eq!(made.components, authored.components);
        assert_eq!(made.resources, authored.resources);
        assert_eq!(made.prefabs, authored.prefabs);
    }
    for invalid in [
        "({entities: [{name: 'bad', model: 'missing.glb'}]})",
        "({entities: [{name: 'bad', scene: 'made.ogt'}]})",
        "({entities: [{name: 'bad', components: {Stats: {weight: {nested: 1}}}}]})",
        "({entities: [{name: 'bad', components: {entity: {}}}]})",
        "({entities: [{name: 'bad', unexpected: true}]})",
    ] {
        std::fs::write(
            assets.join("scene.ts"),
            format!("export default () => {invalid};"),
        )
        .unwrap();
        let out = cook();
        assert!(!out.status.success(), "accepted {invalid}: {}", text(&out));
    }
}
