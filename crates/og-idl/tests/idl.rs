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
            "field",
            "bundle",
            "bundleInfo",
            "progress",
            "defaultBundle",
            "gltf",
            "scene",
            "prefab",
            "font",
            "fontInfo",
            "sound",
            "lut",
            "environment",
            "environmentSun",
            "environmentSunRotation",
            "emitter",
            "effect",
            "shader",
            "post",
            "vector",
            "uiMaterial",
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
            "animgraph",
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
            "data",
            "dataJson",
            "dataBytes",
            "pack",
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
            "getNodeMatrix",
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
            "timeline",
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
            "setSaliency",
            "load",
            "start",
            "view",
            "graph",
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
