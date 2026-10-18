//! The IDL contract: the repository's api.ogt is valid and covers what the
//! design names; broken IDLs are refused with the right reason; the committed
//! generated files are what api.ogt generates; and the generated declarations
//! type-check with the design's example.

use og_api::model::{Item, Member, RecordKind};
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn api() -> og_api::Api {
    og_api::read(&repo().join(og_api::IDL_PATH)).unwrap_or_else(|e| panic!("{e}"))
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
            "emitter",
            "texture",
            "skin",
            "soundInfo",
            "status",
            "error",
            "gltfInfo",
            "animation",
            "flipbook",
            "tilemap"
        ]
    );
    assert_eq!(fns("anim"), ["graph", "clip"]);
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
            "hello", "capture", "sweep", "reload", "snapshot", "restore", "dump", "commands"
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
    if let Err(d) = og_api::parse(&valid) {
        panic!("valid.ogt refused: {d:?}");
    }
    // og-core's reader is the IDL's reader: comments and blank lines may
    // precede the header, and diagnostics keep the lines of the file.
    let commented = format!("# The fixture API.\n\n{valid}");
    if let Err(d) = og_api::parse(&commented) {
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
        match og_api::parse(&text) {
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

#[test]
fn the_committed_generated_files_match_the_idl() {
    let drift = og_api::check(&repo()).unwrap_or_else(|e| panic!("{e}"));
    assert!(
        drift.is_empty(),
        "generated files drifted; run `og api gen`:\n{}",
        drift.iter().map(|d| format!("  {d}\n")).collect::<String>()
    );
    for file in og_api::generate(&api()) {
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
/// contract assertions compile together under strict TypeScript.
#[test]
fn the_generated_declarations_type_check_with_the_examples() {
    let package = repo().join("packages/engine");
    let out = Command::new("bun")
        .args([
            "x",
            "--package",
            "typescript@5.9.3",
            "tsc",
            "--noEmit",
            "-p",
            "tsconfig.json",
        ])
        .current_dir(&package)
        .output()
        .unwrap_or_else(|e| {
            panic!("cannot run bun ({e}); install it: curl -fsSL https://bun.sh/install | bash")
        });
    assert!(
        out.status.success(),
        "tsc failed in {}:\n{}{}",
        package.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
