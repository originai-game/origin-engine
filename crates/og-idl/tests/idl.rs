//! The IDL contract: the repository's api.ogt is valid and covers what the
//! design names; broken IDLs are refused with the right reason; the committed
//! generated files are what api.ogt generates; and the generated declarations
//! type-check with the design's example.

use og_idl::model::{Item, Member, RecordKind};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn api() -> og_idl::Api {
    og_idl::read(&repo().join(og_idl::IDL_PATH)).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_repository_idl_is_valid_and_covers_the_design() {
    let api = api();
    let modules: Vec<&str> = api.modules.iter().map(|m| m.name.as_str()).collect();
    for want in [
        "math",
        "reflect",
        "world",
        "system",
        "game",
        "transform",
        "time",
        "rng",
        "input",
        "camera",
        "render",
        "load",
        "physics",
        "audio",
        "ui",
        "systems",
        "commands",
        "probe",
        "anim",
        "atmosphere",
        "nav",
        "ai",
        "platform",
        "i18n",
    ] {
        assert!(
            modules.contains(&want),
            "module `{want}` missing from {modules:?}"
        );
    }

    let records: Vec<(RecordKind, &str)> = api
        .modules
        .iter()
        .flat_map(|m| &m.items)
        .filter_map(|i| match i {
            Item::Record(r) => Some((r.kind, r.name.as_str())),
            _ => None,
        })
        .collect();
    for name in [
        "Time",
        "Input",
        "Pointers",
        "Gamepad",
        "Camera",
        "Rng",
        "PostEffects",
        "Stylise",
        "MaterialStyle",
        "Lighting2D",
    ] {
        assert!(
            records.contains(&(RecordKind::Resource, name)),
            "resource {name}"
        );
    }
    for name in [
        "Transform",
        "Decal",
        "ParticleSystem",
        "Trail",
        "Beam",
        "Light2D",
        "Occluder2D",
    ] {
        assert!(
            records.contains(&(RecordKind::Component, name)),
            "component {name}"
        );
    }

    let fns = |module: &str| -> Vec<String> {
        let m = api.modules.iter().find(|m| m.name == module).unwrap();
        m.items
            .iter()
            .filter_map(|i| match i {
                Item::Fn(f) => Some(f.name.clone()),
                _ => None,
            })
            .collect()
    };
    assert_eq!(
        fns("load"),
        [
            "bundle",
            "bundleInfo",
            "defaultBundle",
            "gltf",
            "scene",
            "font",
            "fontInfo",
            "sound",
            "lut",
            "environment",
            "emitter",
            "shader",
            "post",
            "texture",
            "textureArray",
            "style",
            "look",
            "lookInfo",
            "palette",
            "soundInfo",
            "status",
            "error",
            "gltfInfo",
            "animation",
            "rig",
            "flipbook",
            "tilemap",
            "video",
            "videoInfo",
            "timeline",
            "rig2d"
        ]
    );
    // og.anim.clip three times: a flipbook's clip, a model's and a 2D rig's (TypeScript overloads).
    assert_eq!(
        fns("anim"),
        [
            "graph",
            "humanoid",
            "retarget",
            "mirror",
            "clip",
            "clip",
            "clip",
            "rig2dInfo",
            "skeleton2dBoxes"
        ]
    );
    // E4: time scale, camera feel, tweens, tilemaps, floating text.
    assert_eq!(fns("time"), ["setScale", "hitStop"]);
    assert_eq!(fns("camera"), ["shake", "punch", "kick"]);
    assert_eq!(
        fns("tween"),
        [
            "to",
            "sequence",
            "parallel",
            "cancel",
            "cancelEntity",
            "active",
            "squash",
            "flash"
        ]
    );
    assert_eq!(
        fns("tilemap"),
        ["setTile", "getTile", "cellAt", "cellCenter"]
    );
    assert!(fns("render").contains(&"floatingText".to_string()));
    for name in [
        "Sprite",
        "Afterimage",
        "Flipbook",
        "Tilemap",
        "CameraZone",
        "CameraRail",
    ] {
        assert!(
            records.contains(&(RecordKind::Component, name)),
            "component {name}"
        );
    }
    assert_eq!(fns("reflect"), ["defineComponent", "defineResource"]);
    assert_eq!(fns("system"), ["defineSystem"]);
    assert_eq!(fns("game"), ["game"]);
    assert!(fns("commands").contains(&"register".to_string()));
    assert!(fns("commands").contains(&"schedule".to_string()));
}

/// Every protocol-5 member, then the engine's wire members, with the publish
/// trim of design §3.4: hello, tick, state, pick, entity, input and capture
/// (and the version) stay in publish builds.
#[test]
fn the_probe_is_protocol_5_plus_the_wire_members_with_the_publish_trim() {
    let api = api();
    let probe = api.modules.iter().find(|m| m.name == "probe").unwrap();
    let iface = |name: &str| {
        probe
            .items
            .iter()
            .find_map(|i| match i {
                Item::Interface(x) if x.name == name => Some(x),
                _ => None,
            })
            .unwrap_or_else(|| panic!("interface {name}"))
    };
    let members = |name: &str| -> Vec<(String, bool)> {
        let i = iface(name);
        i.members
            .iter()
            .map(|m: &Member| {
                let publish = m.publish().or(i.publish).unwrap_or(probe.publish);
                (m.name().to_string(), publish)
            })
            .collect()
    };
    let runtime = members("RuntimeProbe");
    let names: Vec<&str> = runtime.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [
            "version", "tick", "state", "advance", "pause", "resume", "input", "accepted", "pick",
            "entity", "bounds", "patch", "run"
        ]
    );
    let engine = members("EngineProbe");
    let names: Vec<&str> = engine.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [
            "hello",
            "capture",
            "sweep",
            "reload",
            "snapshot",
            "restore",
            "dump",
            "commands",
            "listen",
            "memory",
            "describe",
            "expect",
            "expectations",
            "saveScene"
        ]
    );
    assert_eq!(
        iface("EngineProbe").extends.as_deref(),
        Some("RuntimeProbe")
    );

    let published: Vec<&str> = runtime
        .iter()
        .chain(&engine)
        .filter(|(_, p)| *p)
        .map(|(n, _)| n.as_str())
        .collect();
    assert_eq!(
        published,
        [
            "version", "tick", "state", "input", "pick", "entity", "hello", "capture"
        ]
    );
}

#[test]
fn broken_idls_are_refused_with_the_right_reason() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let valid = std::fs::read_to_string(dir.join("valid.ogt")).unwrap();
    if let Err(d) = og_idl::parse(&valid) {
        panic!("valid.ogt refused: {d:?}");
    }
    // og-core's reader is the IDL's reader: comments and blank lines may
    // precede the header, and diagnostics keep the lines of the file.
    let commented = format!("# The fixture API.\n\n{valid}");
    if let Err(d) = og_idl::parse(&commented) {
        panic!("valid.ogt behind a comment refused: {d:?}");
    }
    let mut checked = 0;
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.file_name().unwrap() == "valid.ogt" {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let expect = text
            .lines()
            .find_map(|l| l.strip_prefix("# expect: "))
            .unwrap_or_else(|| panic!("{} has no `# expect: <line>: <reason>`", path.display()));
        let (line, reason) = expect.split_once(": ").unwrap();
        let line: usize = line.parse().unwrap();
        match og_idl::parse(&text) {
            Ok(_) => panic!("{} was accepted", path.display()),
            Err(diags) => {
                assert_eq!(
                    diags.len(),
                    1,
                    "{}: expected one diagnostic, got {diags:#?}",
                    path.display()
                );
                assert_eq!(diags[0].line, line, "{}: {}", path.display(), diags[0]);
                assert!(
                    diags[0].message.contains(reason),
                    "{}: expected `{reason}`, got `{}`",
                    path.display(),
                    diags[0].message
                );
            }
        }
        checked += 1;
    }
    assert!(checked >= 8, "only {checked} broken fixtures ran");
}

/// A record's text field (`field text str.48`) reaches every output with
/// its capacity: the TypeScript value is a string and its token a `str`
/// field, the binding table and the schema carry the capacity, the hosts'
/// record table hands it to the runtime, and the wiki states it.
#[test]
fn a_record_text_field_carries_its_capacity_into_every_output() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let api = og_idl::parse(&std::fs::read_to_string(dir.join("valid.ogt")).unwrap())
        .unwrap_or_else(|d| panic!("valid.ogt refused: {d:?}"));
    let out = |path: &str| -> String {
        og_idl::generate(&api)
            .into_iter()
            .chain(og_idl::generate_wiki(&api))
            .find(|g| g.path.ends_with(path))
            .unwrap_or_else(|| panic!("no generated {path}"))
            .contents
    };
    let dts = out("types/index.d.ts");
    assert!(dts.contains("  text: string;"), "{dts}");
    assert!(dts.contains("  name: string;"), "{dts}");
    assert!(
        dts.contains("@capacity 48 bytes of UTF-8 (str(48))"),
        "{dts}"
    );
    assert!(
        dts.contains("readonly text: FieldSpec<\"str\">; readonly size: FieldSpec<\"f32\">"),
        "{dts}"
    );

    let bindings: serde_json::Value = serde_json::from_str(&out("types/bindings.json")).unwrap();
    let label = &bindings["components"][0]["fields"][0];
    assert_eq!(label["column"], "str");
    assert_eq!(label["capacity"], 48);
    assert_eq!(label["default"], "Ada");
    let profile = &bindings["resources"][0]["fields"][0];
    assert_eq!(
        (
            &profile["column"],
            &profile["capacity"],
            &profile["default"]
        ),
        (&"str".into(), &16.into(), &"".into())
    );
    assert!(
        bindings["components"][0]["fields"][1]
            .get("capacity")
            .is_none()
    );

    let schema: serde_json::Value = serde_json::from_str(&out("schema/api.schema.json")).unwrap();
    let text = &schema["$defs"]["Label"]["properties"]["text"];
    assert_eq!(
        (&text["type"], &text["x-og-capacity"]),
        (&"string".into(), &48.into())
    );

    let host = out("og-api/src/generated/host.rs");
    assert!(host.contains("pub capacity: u32,"), "{host}");
    let field = |name: &str| {
        let at = host
            .find(&format!("name: \"{name}\""))
            .unwrap_or_else(|| panic!("no {name} in host.rs"));
        host[at..at + host[at..].find("}").unwrap()].to_string()
    };
    assert!(
        field("text").contains("column: \"str\""),
        "{}",
        field("text")
    );
    assert!(field("text").contains("capacity: 48,"), "{}", field("text"));
    assert!(field("name").contains("capacity: 16,"), "{}", field("name"));
    assert!(field("size").contains("capacity: 0,"), "{}", field("size"));

    let wiki = out("game.md");
    assert!(
        wiki.contains("`str(48)`: at most 48 bytes of UTF-8"),
        "{wiki}"
    );
}

/// A record's list field (`field passes List.Post.4`) reaches every output
/// with its element and capacity: a read-only array in TypeScript and a
/// `ListSpec` of the element in its token, the element's kind and the
/// capacity in the binding table and the hosts' record table, `maxItems` in
/// the schema, and the wiki's note.
#[test]
fn a_record_list_field_carries_its_element_and_capacity_into_every_output() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let api = og_idl::parse(&std::fs::read_to_string(dir.join("valid.ogt")).unwrap())
        .unwrap_or_else(|d| panic!("valid.ogt refused: {d:?}"));
    let out = |path: &str| -> String {
        og_idl::generate(&api)
            .into_iter()
            .chain(og_idl::generate_wiki(&api))
            .find(|g| g.path.ends_with(path))
            .unwrap_or_else(|| panic!("no generated {path}"))
            .contents
    };
    let dts = out("types/index.d.ts");
    assert!(dts.contains("  passes: readonly Post[];"), "{dts}");
    assert!(dts.contains("  weights: readonly number[];"), "{dts}");
    assert!(
        dts.contains(
            "@capacity 4 elements (List.Post.4): reading answers a frozen copy (push throws)"
        ),
        "{dts}"
    );
    assert!(
        dts.contains("readonly passes: ListSpec<\"Post\">; readonly weights: ListSpec<\"f32\">"),
        "{dts}"
    );
    assert!(
        dts.contains("export declare function list<K extends ListKind>(of: FieldSpec<K>, capacity: number, options?: FieldOptions): ListSpec<K>;"),
        "{dts}"
    );
    assert!(
        !dts.contains("  list: {"),
        "a list is not a column kind of its own: {dts}"
    );

    let bindings: serde_json::Value = serde_json::from_str(&out("types/bindings.json")).unwrap();
    let chain = &bindings["resources"][1]["fields"];
    assert_eq!(
        (&chain[0]["column"], &chain[0]["list"], &chain[0]["default"]),
        (&"Post".into(), &4.into(), &serde_json::json!([]))
    );
    assert_eq!(
        (&chain[1]["column"], &chain[1]["list"]),
        (&"f32".into(), &2.into())
    );
    let factory = bindings["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "list")
        .unwrap();
    assert_eq!(
        factory,
        &serde_json::json!({ "name": "list", "list": true })
    );

    let schema: serde_json::Value = serde_json::from_str(&out("schema/api.schema.json")).unwrap();
    let passes = &schema["$defs"]["Chain"]["properties"]["passes"];
    assert_eq!(
        (&passes["type"], &passes["maxItems"]),
        (&"array".into(), &4.into())
    );

    let host = out("og-api/src/generated/host.rs");
    assert!(host.contains("pub list: u32,"), "{host}");
    let field = |name: &str| {
        let at = host
            .find(&format!("name: \"{name}\""))
            .unwrap_or_else(|| panic!("no {name} in host.rs"));
        host[at..at + host[at..].find("}").unwrap()].to_string()
    };
    assert!(
        field("passes").contains("column: \"Post\""),
        "{}",
        field("passes")
    );
    assert!(field("passes").contains("list: 4,"), "{}", field("passes"));
    assert!(
        field("weights").contains("list: 2,"),
        "{}",
        field("weights")
    );
    assert!(field("size").contains("list: 0,"), "{}", field("size"));
    assert!(!host.contains("ColumnDecl { name: \"list\""), "{host}");

    let wiki = out("game.md");
    assert!(
        wiki.contains("a list of at most 4 `Post` (`List.Post.4`)"),
        "{wiki}"
    );
}

#[test]
fn the_committed_generated_files_match_the_idl() {
    let drift = og_idl::check(&repo()).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        drift.is_empty(),
        "generated files drifted; run `og api gen`:\n{}",
        drift.iter().map(|d| format!("  {d}\n")).collect::<String>()
    );
    for file in og_idl::generate(&api()) {
        // The first line of a .d.ts, the first key of a JSON file.
        let head: String = file.contents.lines().take(2).collect();
        assert!(
            head.contains("Generated from crates/og-api/api.ogt"),
            "{}: header is `{head}`",
            file.path
        );
    }
}

/// The generated declarations, the design's hello world and the type-level
/// contract assertions compile together under strict TypeScript: tsrs, the
/// checker `og check` runs (scripts/install-tools.sh pins it), with og
/// check's arguments. og itself cannot be the runner here: og-cli depends
/// on this crate, so its binary is not this test's to name.
#[test]
fn the_generated_declarations_type_check_with_the_examples() {
    let package = repo().join("packages/engine");
    let mut checker = if cfg!(windows) {
        let mut c = Command::new("bun");
        c.args(["x", "--package", "typescript@7.0.2", "tsc"]);
        c
    } else {
        Command::new("tsrs")
    };
    let out = checker
        .args([
            "-p",
            "tsconfig.json",
            "--noEmit",
            "--incremental",
            "false",
            "--pretty",
            "false",
        ])
        .current_dir(&package)
        .output()
        .unwrap_or_else(|e| panic!("cannot run tsrs ({e}); run scripts/install-tools.sh"));
    assert!(
        out.status.success(),
        "tsrs failed in {}:\n{}{}",
        package.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
