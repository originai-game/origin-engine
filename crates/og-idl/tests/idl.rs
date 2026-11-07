//! The IDL contract: the repository's api/ parts are valid and cover what the
//! design names; broken IDLs are refused with the right reason; the committed
//! generated files are what the IDL generates; and the generated declarations
//! type-check with the design's example.

use og_idl::model::{Item, RecordKind};
use std::path::{Path, PathBuf};

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
fn enum_patterns_are_typed_documented_and_regex_escaped() {
    let source = r#"type api
version 1
---
package "@origingame/pattern-test"
doc "Pattern fixture."
module test {
  doc "Patterns."
  struct FieldOptions {
    doc "Field metadata."
    desc true
  }
  enum Choice {
    doc "A choice."
    variant plain "Fixed."
    pattern "a.b(${string})[${string}]" "Two arbitrary text slots."
  }
}
"#;
    let api = og_idl::parse(source).unwrap();
    let files = og_idl::generate(&api);
    let dts = &files
        .iter()
        .find(|f| f.path.ends_with("types/index.d.ts"))
        .unwrap()
        .contents;
    assert!(dts.contains("export type Choice = \"plain\" | `a.b(${string})[${string}]`;"));
    let schema: serde_json::Value = serde_json::from_str(
        &files
            .iter()
            .find(|f| f.path.ends_with("schema/api.schema.json"))
            .unwrap()
            .contents,
    )
    .unwrap();
    assert_eq!(
        schema["$defs"]["Choice"]["anyOf"][1]["pattern"],
        r"^a\.b\([\s\S]*\)\[[\s\S]*\]$"
    );
    assert!(
        og_idl::generate_wiki(&api)
            .iter()
            .any(|f| f.contents.contains("Two arbitrary text slots."))
    );
    for invalid in [
        "literal",
        "a(${number})",
        "a(`${string})",
        "a(${string})${bad}",
    ] {
        assert!(
            og_idl::parse(&source.replace("a.b(${string})[${string}]", invalid)).is_err(),
            "{invalid}"
        );
    }
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
            "prefab",
            "font",
            "fontInfo",
            "sound",
            "lut",
            "environment",
            "emitter",
            "shader",
            "post",
            "vector",
            "uiMaterial",
            "texture",
            "textureArray",
            "style",
            "document",
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
            "vfx",
            "rig2d",
            "imageData",
            "gltfData",
            "environmentData",
            "environmentInfo",
            "release",
            "imageInfo",
            "completions",
            "replayCompletions"
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
    assert_eq!(
        fns("time"),
        ["setScale", "hitStop", "setGroupScale", "groupDt"]
    );
    assert_eq!(fns("camera"), ["shake", "punch", "kick"]);
    assert_eq!(
        fns("tween"),
        [
            "to",
            "from",
            "fromTo",
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
        [
            "setTile",
            "fill",
            "create",
            "resize",
            "neighbours",
            "getTile",
            "cellAt",
            "cellCenter"
        ]
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
    assert_eq!(
        fns("reflect"),
        ["defineComponent", "defineResource", "defineEvent"]
    );
    assert_eq!(fns("system"), ["defineHook", "defineSystem"]);
    assert_eq!(fns("game"), ["game"]);
    assert_eq!(
        fns("i18n"),
        [
            "load",
            "setLocale",
            "locale",
            "t",
            "has",
            "formatNumber",
            "formatCompact",
            "formatCurrency",
            "formatDate",
            "formatTime",
            "formatDuration"
        ]
    );
    // G2-8: dialogue and quests.
    assert_eq!(
        fns("story"),
        [
            "load",
            "parse",
            "start",
            "view",
            "history",
            "conversations",
            "advance",
            "choose",
            "stop",
            "bark",
            "ambient",
            "barks",
            "get",
            "set",
            "reset",
            "startQuest",
            "progress",
            "setStage",
            "completeQuest",
            "failQuest",
            "quest",
            "quests",
            "track",
            "markSeen",
            "mark"
        ]
    );
    for name in [
        "Story",
        "DialogueShown",
        "DialogueEnded",
        "DialogueCheck",
        "QuestUpdated",
        "StoryEvent",
        "BarkSaid",
    ] {
        assert!(
            records.iter().any(|(_, n)| *n == name),
            "story record {name}"
        );
    }
    assert!(fns("commands").contains(&"register".to_string()));
    assert!(fns("commands").contains(&"schedule".to_string()));
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

    let host = out("og-records/src/generated/records.rs");
    assert!(host.contains("pub list: u32,"), "{host}");
    let field = |name: &str| {
        let at = host
            .find(&format!("name: \"{name}\""))
            .unwrap_or_else(|| panic!("no {name} in records.rs"));
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

    let wiki = out("game/record-Chain.md");
    assert!(
        wiki.contains("a list of at most 4 `Post` (`List.Post.4`)"),
        "{wiki}"
    );
}
