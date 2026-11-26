//! Public kind schemas must agree with serde and the actual TS checker.
use crate::cook::source_schemas;
use crate::cook::test_dir::TempDir;
use og_core::reflect::source::typescript;
use serde_json::{Value, json};

#[test]
fn media_kind_inputs_have_checked_typescript_contracts() {
    // Each negative changes a structural choice that TypeScript can express,
    // not a numeric range which remains the schema validator's responsibility.
    let cases: &[(&str, Value, Value)] = &[
        (
            "animgraph",
            json!({"motions":{"idle":{"kind":"clip","clip":"idle"}},"layers":[{"name":"base","entry":"idle","states":[{"name":"idle","motion":"idle"}]}]}),
            json!({"motions":{"idle":{"kind":"clip","clip":4}},"layers":[]}),
        ),
        (
            "rig",
            json!({"scheme":"mixamo","footHeight":[-0.2,0.3],"bones":{"hips":"Root","leftToes":null}}),
            json!({"bones":{"hipz":"Root"}}),
        ),
        (
            "motion-action",
            json!({"source":{"$ogRef":"base/model/wave"},"clip":"wave","holdStart":0.2,"holdEnd":0.8,"hand":"right","handTarget":[0.2,0.1,-0.3]}),
            json!({"source":"base/model/wave","clip":"wave","holdEnd":0.8}),
        ),
        (
            "timeline",
            json!({"duration":3,"tracks":[{"kind":"camera","keys":[{"time":0,"value":{"position":[2,3,4],"target":[0,1,0],"fov":63}}]},{"kind":"scene","keys":[{"time":1,"value":{"scene":{"$ogRef":"base/scene/room"}}}]}]}),
            json!({"tracks":[{"kind":"camera","keys":[{"time":0,"value":{"position":[2,3]}}]}]}),
        ),
        (
            "flipbook",
            json!({"texture":{"$ogRef":"base/texture/hero"},"frames":[{"name":"idle","rect":[0,0,16,24],"pivot":[0.25,0.8]}],"clips":[{"name":"idle","frames":["idle"],"durations":[0.12],"mode":"pingPong"}]}),
            json!({"texture":7,"frames":[]}),
        ),
        (
            "rig2d",
            json!({"texture":{"$ogRef":"base/texture/puppet"},"unit":100,"rig":{"bones":[{"name":"root"},{"name":"arm","parent":"root","at":[3,4]}],"slots":[{"name":"hand","bone":"arm"}],"attachments":{"hand":{"kind":"region","slot":"hand","rect":[0,0,12,18]}}}}),
            json!({"texture":{"$ogRef":"base/texture/puppet"},"rig":{"bones":[{"name":"root","at":[1,2,3]}]}}),
        ),
        (
            "tileset",
            json!({"texture":{"$ogRef":"base/texture/ground"},"tileWidth":16,"tileHeight":24,"columns":4,"rows":2,"tiles":{"1":{"collision":{"kind":"rect","x":1,"y":2,"w":12,"h":20},"animation":{"frames":[1,2],"durations":[0.1,0.2]}}}}),
            json!({"tileWidth":16,"tileHeight":24,"columns":4,"rows":2,"filter":"bilinear"}),
        ),
        (
            "tilemap",
            json!({"orientation":"isometric","width":2,"height":1,"tileWidth":16,"tileHeight":24,"tilesets":[{"$ogRef":"base/tileset/ground"}],"layers":[{"name":"floor","tileset":0,"cells":[1,2],"opacity":0.8}]}),
            json!({"width":2,"height":1,"tileWidth":16,"tileHeight":24,"tilesets":["base/tileset/ground"],"layers":[]}),
        ),
        (
            "video",
            json!({"source":"intro.webm","size":[640,360],"fps":30,"posterSize":128,"renditions":["hevc","h264"],"audioLocale":{"fr":"intro.fr.wav"}}),
            json!({"source":"intro.webm","size":[640,360,1]}),
        ),
        (
            "vfx2d",
            json!({"recipe":{"w":64,"h":32,"seed":23,"layers":[{"type":"particles","count":12,"mode":"spray","shape":"shard","size":[0.01,0.04],"life":[0.2,0.7]},{"type":"ring","r0":0.1,"r1":0.8,"th":0.05}]},"events":[{"time":0.1,"name":"impact"}]}),
            json!({"recipe":{"layers":[{"type":"ring","shape":"shard"}]}}),
        ),
        (
            "behaviour",
            json!({"name":"guard","keys":{"speed":2.3},"root":{"kind":"tree","node":{"kind":"sequence","children":[{"kind":"check","check":{"operand":"alert","op":"truthy"}},{"kind":"action","action":{"kind":"wait","seconds":0.2}}]}}}),
            json!({"name":"guard","root":{"kind":"tree","node":{"kind":"action","action":{"kind":"wait","seconds":"0.2"}}}}),
        ),
        (
            "dialogue",
            json!({"name":"greeting","start":"hello","strings":{"greeting.hello":"Welcome {name}"},"nodes":[{"id":"hello","steps":[{"kind":"line","id":"greeting.hello","speaker":"guard","text":{"key":"greeting.hello","args":{"name":{"kind":"variable","name":"player"}}}},{"kind":"if","condition":{"kind":"binary","op":"ge","left":{"kind":"variable","name":"gold"},"right":{"kind":"literal","value":10}},"node":"trade"},{"kind":"command","name":"wave","args":[{"kind":"literal","value":"right"}]}]}]}),
            json!({"name":"greeting","start":"hello","nodes":[{"id":"hello","steps":[{"kind":"if","condition":"gold >= 10","node":"trade"}]}]}),
        ),
        (
            "quest",
            json!({"id":"supplies","title":"Gather supplies","stages":[{"id":"gather","objectives":[{"id":"wood","text":{"key":"quest.supplies.wood"},"target":3,"marker":[2,0,-4],"when":{"kind":"literal","value":true}}]}]}),
            json!({"id":"supplies","title":"Gather","stages":[{"id":"gather","objectives":[{"id":"wood","text":"Gather wood"}]}]}),
        ),
        (
            "shader",
            json!({"code":"fn surface_roughness(h: Hook, p: Params, r: f32) -> f32 { return p.amount; }","mode":{"kind":"hooks"},"params":[{"kind":"float","name":"amount","default":0.4},{"kind":"color","name":"tint","default":[0.1,0.3,0.8]}]}),
            json!({"code":"","params":[{"kind":"vec3","name":"tint","default":[1,2]}]}),
        ),
        (
            "post",
            json!({"code":"fn post(f: PostInput, p: Params) -> vec4<f32> { return f.color; }","params":[{"kind":"vec2","name":"offset","default":[0.2,-0.1]}]}),
            json!({"code":"","mode":{"kind":"hooks"}}),
        ),
        (
            "ui-material",
            json!({"code":"fn material(m: MaterialInput, p: Params) -> vec4<f32> { return m.color; }","params":[{"kind":"vec4","name":"tint","default":[0.2,0.4,0.8,1]}]}),
            json!({"code":"","params":[{"kind":"matrix","name":"tint"}]}),
        ),
    ];
    assert_eq!(cases.len(), super::KINDS.len());
    let dir = TempDir::new("media-source-contract");
    let mut ts = String::new();
    for (i, (kind, valid, invalid)) in cases.iter().enumerate() {
        let schema = (source_schemas::get(kind).unwrap().schema)();
        schema
            .check(valid)
            .unwrap_or_else(|e| panic!("{kind}: {e}"));
        assert!(schema.check(invalid).is_err(), "{kind} accepted {invalid}");
        let name = format!("Input{i}");
        ts.push_str(
            &typescript(schema.document(), &name).unwrap_or_else(|e| panic!("{kind}: {e}")),
        );
        ts.push_str(&format!("\nconst valid{i}: {name} = {valid};\n// @ts-expect-error {kind} rejects this author shape.\nconst invalid{i}: {name} = {invalid};\n"));
    }
    let timeline = (source_schemas::get("timeline").unwrap().schema)();
    let reflected = json!({"tracks":[
        {"kind":"light","entity":"sun","keys":[{"time":0,"value":{"intensity":3}}]},
        {"kind":"post","keys":[{"time":0,"value":{"aoIntensity":1.2}}]},
        {"kind":"blendshape","entity":"face","keys":[{"time":0,"value":{"w0":0.3,"w2":0.7}}]},
        {"kind":"material","entity":"crate","keys":[{"time":0,"value":{"standard":{"roughness":0.35,"transparent":1},"mapRepeat":[2,3],"mapRotation":0.4,"params":{"glow":0.8,"offset":[0.1,-0.2]}}}]}
    ]});
    timeline.check(&reflected).unwrap();
    ts.push_str(&format!("\nconst reflected: Input3 = {reflected};\n"));
    for (i, track) in [
        json!({"kind":"light","entity":"sun","keys":[{"time":0,"value":{"intenstiy":3}}]}),
        json!({"kind":"post","keys":[{"time":0,"value":{"aoIntensity":"strong"}}]}),
        json!({"kind":"blendshape","entity":"face","keys":[{"time":0,"value":{"mesh":1}}]}),
        json!({"kind":"material","entity":"crate","keys":[{"time":0,"value":{"standard":{"rougness":0.3}}}]}),
        json!({"kind":"material","entity":"crate","keys":[{"time":0,"value":{"standard":{"side":1}}}]}),
        json!({"kind":"material","entity":"crate","keys":[{"time":0,"value":{"standard":{"mapRepeat":[2,3]},"mapRepeat":[4,5]}}]}),
        json!({"kind":"material","entity":"crate","keys":[{"time":0,"value":{"params":{"roughness":0.3}}}]}),
        json!({"kind":"material","entity":"crate","keys":[{"time":0,"value":{"params":{"mapRepeat":[2,3]}}}]}),
        json!({"kind":"material","entity":"crate","keys":[{"time":0,"value":{"params":{"glow":"red"}}}]}),
        json!({"kind":"material","entity":"crate","keys":[{"time":0,"value":{"mapOffset":[1,2,3]}}]}),
    ].into_iter().enumerate() {
        let invalid = json!({"tracks":[track]});
        assert!(timeline.check(&invalid).is_err(), "accepted {invalid}");
        ts.push_str(&format!("// @ts-expect-error reflected fields, UV controls and uniforms stay distinct.\nconst badTrack{i}: Input3 = {invalid};\n"));
    }
    std::fs::write(dir.join("media.ts"), ts).unwrap();
    let check = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "media.ts"])
        .current_dir(&dir)
        .output()
        .expect("the pinned tsrs checker must be installed (docs/ci.md)");
    assert!(
        check.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    // Bounds and item constraints cannot be bypassed with a TS assertion.
    let video = (source_schemas::get("video").unwrap().schema)();
    for value in [
        json!({"source":"x.webm","fps":120,"maxSeconds":0.01,"size":[2,4096],"posterSize":0}),
        json!({"source":"x.webm","posterSize":64}),
        json!({"source":"x.webm","posterSize":4096}),
    ] {
        video.check(&value).unwrap();
    }
    for value in [
        json!({"source":"x.webm","fps":121}),
        json!({"source":"x.webm","maxSeconds":0}),
        json!({"source":"x.webm","posterSize":63}),
        json!({"source":"x.webm","size":[640,1]}),
    ] {
        assert!(video.check(&value).is_err(), "accepted {value}");
    }
    let wrong = video
        .check(&json!({"source":"x.webm","size":[640,1]}))
        .unwrap_err();
    assert_eq!(wrong.path, "/size/1");
    // Custom per-frame constraints must retain f32 bounds, not allow serde
    // to turn a finite JSON number into an infinite animation duration.
    for (index, path) in [
        (4, "/clips/0/durations/0"),
        (6, "/tiles/1/animation/durations/0"),
    ] {
        let (kind, valid, _) = &cases[index];
        let schema = (source_schemas::get(kind).unwrap().schema)();
        for duration in [0.0, -0.1, 1e100] {
            let mut invalid = valid.clone();
            *invalid.pointer_mut(path).unwrap() = json!(duration);
            assert!(schema.check(&invalid).is_err(), "{kind} accepted {invalid}");
        }
    }
    let dialogue = (source_schemas::get("dialogue").unwrap().schema)();
    dialogue
        .check(&json!({"yarn":"title: Start\n---\nHello\n==="}))
        .unwrap();
    assert!(dialogue.check(&json!({"source":"legacy.yarn"})).is_err());
}
