//! Structured author contracts through the registry and emitted TypeScript.
use super::*;
use crate::cook::{source_schemas, test_dir::TempDir};
use og_core::reflect::source::typescript;
use serde_json::{Value, json};

fn texture() -> Value {
    json!({
        "mode": "noise", "size": 16, "frames": 3, "columns": 2,
        "grid": 8, "steps": 2, "dt": 0.025, "pressure": 7, "seed": 16777219,
        "frequency": 2.5, "range": 1.5, "turbulence": 0.2,
        "source": [0.1, 0.35, 0.8], "detail": 0.25, "vorticity": 0.4,
        "radius": 0.15, "inject": 0.2, "density": 0.7, "heat": 0.8,
        "buoyancy": -0.3, "expansion": 0.4, "decay": 0.6, "cooling": 0.9,
        "absorption": 1.5, "color": [0.1, 0.5, 0.8], "emission": [0.9, 0.4, 0.2],
        "light": [-1, 2, 3], "ambient": 0.3, "direct": 0.7
    })
}

#[test]
fn procedural_inputs_share_their_registry_value_and_typescript_contracts() {
    let fixtures = [
        (
            "scatter",
            "ScatterInput",
            json!({
                "target": {"kind": "model", "asset": {"$ogRef": "base/model/gate"}},
                "positions": [-2, 3, 7, 5, 11, -13], "quaternions": [0, 0, 0, 1, 0, 1, 0, 0],
                "scales": [1, 2, 3, 4, 5, 6], "project": {"$ogRef": "base/terrain/north"}, "align": true
            }),
        ),
        (
            "look",
            "LookInput",
            json!({
                "name": "Cold ink", "extends": ["anime-cel-3d", {"$ogRef": "base/look/common"}],
                "mood": "cold", "variant": "8bit", "antialias": "taa",
                "RenderSettings": {"exposure": 1.25, "toneMapping": "agx"},
                "PostEffects": {"outlineColor": "#1b1420", "lut": {"$ogRef": "base/lut/dusk"}},
                "uiSeeds": {"primary": "#1122aa", "team": "#bb88cc"}
            }),
        ),
        (
            "effect",
            "EffectInput",
            json!({
                "main": {"texture": {"$ogRef": "base/effect-texture/smoke"}, "channel": "red", "uv": {"tile": [2, 3], "scroll": [-0.2, 0.7]}},
                "erosion": {"kind": "curve", "keys": [[0, 0.25], [1, 0.8]]},
                "intensity": {"kind": "random", "min": 0.5, "max": 3},
                "shader": {"$ogRef": "base/shader/glow"}, "params": {"tint": [1, 0.25, 0.5, 1]}
            }),
        ),
        (
            "particles",
            "ParticlesInput",
            json!({
                "maxParticles": 257, "shape": {"kind": "cone", "angleDegrees": 23, "radius": 0.7},
                "speed": [2, 7], "lifetime": [0.3, 1.1], "color": [2, 0.5, 0.25, 1],
                "bursts": [{"time": 0.2, "count": 17, "cycles": 3, "interval": 0.4}],
                "subEmitters": [{"trigger": "death", "emitter": {"$ogRef": "base/particles/sparks"}, "count": 3}],
                "field": {"kind": "grid", "cells": 1, "size": 2, "strength": 0.5, "vectors": [[1, -2, 3]]},
                "colorOverLife": [[0, [2, 0.5, 0.25, 1]], [1, [0, 0.2, 0.8, 0]]]
            }),
        ),
        (
            "vfx",
            "VfxInput",
            json!({
                "duration": 1.5, "events": [{"time": 0.3, "name": "impact"}], "params": [{"name": "power", "default": 0.75}],
                "elements": [
                    {"at": 0.2, "life": 0.5, "offset": [-1, 2, 3], "node": {"kind": "mesh", "shape": {"kind": "ring", "ring": {"inner": 0.3, "segments": 23}}, "effect": {"$ogRef": "base/effect/shock"}, "scale": {"kind": "curve", "keys": [[0, 0.5], [1, 3]]}}},
                    {"at": 0.3, "node": {"kind": "sound", "asset": {"$ogRef": "base/sfx/impact"}, "pitch": [0.8, 1.2]}},
                    {"life": 0.7, "node": {"kind": "decal", "texture": {"$ogRef": "base/texture/scorch"}, "normal": {"$ogRef": "base/texture/cracks"}, "normalScale": 2.3, "emissive": [4, 0.3, 0.1], "roughness": 0.8, "metallic": 0.2, "wet": false, "angleFade": 65, "turn": [-35, 17]}}
                ]
            }),
        ),
        (
            "sfx",
            "SfxInput",
            json!({
                "seed": 16777219, "volume": 0.7, "layers": [
                    {"wave": "triangle", "freq": 523.25, "freqEnd": 130.5, "attack": 0.003, "sustainLevel": 0.6, "duty": 0.35},
                    {"freq": 440, "sample": {"kind": "pcm", "samples": [-1, 0.25, 0.7], "rate": 22050, "reference": 440, "looping": false}}
                ]
            }),
        ),
        ("effect-texture", "EffectTextureInput", texture()),
    ];
    for (kind, _, value) in &fixtures {
        (source_schemas::get(kind).unwrap().schema)()
            .check(value)
            .unwrap_or_else(|e| panic!("{kind}: {e}"));
        let mut unknown = value.clone();
        unknown["typo"] = json!(true);
        assert!(
            (source_schemas::get(kind).unwrap().schema)()
                .check(&unknown)
                .is_err(),
            "{kind} accepted unknown field"
        );
    }
    let scatter = ScatterInput::schema()
        .decode(fixtures[0].2.clone())
        .unwrap();
    assert_eq!(scatter.positions, [-2., 3., 7., 5., 11., -13.]);
    assert_eq!(scatter.scales.unwrap(), [1., 2., 3., 4., 5., 6.]);
    let tex = EffectTextureInput::schema().decode(texture()).unwrap();
    assert_eq!(tex.seed, 16777219);
    assert_eq!(tex.buoyancy, -0.3);
    assert_eq!(tex.source[2], 0.8);

    for (kind, pointer, wrong) in [
        (
            "scatter",
            "/target",
            json!({"kind": "sound", "asset": {"$ogRef": "base/sfx/a"}}),
        ),
        ("scatter", "/project", json!("base/terrain/north")),
        ("look", "/RenderSettings", json!({"toneMapping": "wrong"})),
        ("look", "/PostEffects", json!({"flash": 0.5})),
        ("look", "/uiSeeds", json!({"primary": "red"})),
        ("effect", "/main", json!({"channel": "rgb"})),
        (
            "effect",
            "/erosion",
            json!({"kind": "curve", "keys": [[0, 0, 1]]}),
        ),
        ("particles", "/maxParticles", json!(1048577)),
        ("particles", "/bursts", json!([{"time": 0.2, "count": 1.5}])),
        (
            "particles",
            "/shape",
            json!({"kind": "spline", "points": [[0, 1, 2]]}),
        ),
        (
            "vfx",
            "/elements",
            json!([{"node": {"kind": "sound", "asset": 12}}]),
        ),
        (
            "vfx",
            "/elements",
            json!([{"node": {"kind": "shake", "effect": {"$ogRef": "base/effect/a"}}}]),
        ),
        (
            "vfx",
            "/elements",
            json!([{"node": {"kind": "decal", "normalScale": 4.01}}]),
        ),
        (
            "vfx",
            "/elements",
            json!([{"node": {"kind": "decal", "emissive": [4, -0.01, 0.1]}}]),
        ),
        (
            "vfx",
            "/elements",
            json!([{"node": {"kind": "decal", "emissive": [1e39, 0.3, 0.1]}}]),
        ),
        (
            "vfx",
            "/elements",
            json!([{"node": {"kind": "decal", "roughness": -1}}]),
        ),
        (
            "vfx",
            "/elements",
            json!([{"node": {"kind": "decal", "angleFade": 90.01}}]),
        ),
        (
            "sfx",
            "/layers",
            json!([{"sample": {"kind": "pcm", "samples": [1.01], "rate": 22050}}]),
        ),
        ("sfx", "/layers", json!([{"duty": 0.009}])),
        ("sfx", "/layers", json!([{"freq": 0}])),
        ("effect-texture", "/source", json!([0.2, -0.01, 0.8])),
        ("effect-texture", "/size", json!(7)),
        ("effect-texture", "/dt", json!(0)),
        ("effect-texture", "/seed", json!(4294967296u64)),
        ("effect-texture", "/mode", json!("cloud")),
    ] {
        let mut invalid = fixtures.iter().find(|f| f.0 == kind).unwrap().2.clone();
        *invalid.pointer_mut(pointer).unwrap() = wrong;
        assert!(
            (source_schemas::get(kind).unwrap().schema)()
                .check(&invalid)
                .is_err(),
            "{kind} accepted {pointer}: {invalid}"
        );
    }
    let mut declarations = String::new();
    for (kind, name, value) in &fixtures {
        declarations.push_str(&typescript(source_schemas::schema(kind).unwrap(), name).unwrap());
        declarations.push_str(&format!("\nconst valid{name}: {name} = {value};\n"));
    }
    declarations.push_str(r#"
// @ts-expect-error A scatter target is model or prefab, not sound.
const badScatter: ScatterInput = { target: { kind: "sound", asset: { $ogRef: "base/sfx/a" } }, positions: [] };
// @ts-expect-error Reflected fields are closed and keep the enum type.
const badLook: LookInput = { RenderSettings: { toneMapping: "wrong" } };
// @ts-expect-error A curve key is a pair, not a triple.
const badEffect: EffectInput = { erosion: { kind: "curve", keys: [[0, 0, 1]] } };
// @ts-expect-error Cone shape fields are not sphere shape fields.
const badParticles: ParticlesInput = { shape: { kind: "sphere", angleDegrees: 23, radius: 1 } };
// @ts-expect-error Sound nodes use a tagged content identity, not a numeric handle.
const badVfx: VfxInput = { duration: 1, elements: [{ node: { kind: "sound", asset: 12 } }] };
// @ts-expect-error Per-decal normal maps are content tokens, not runtime handles.
const badDecal: VfxInput = { duration: 1, elements: [{ node: { kind: "decal", normal: 12 } }] };
// @ts-expect-error Oscillator waveform is an enum.
const badSfx: SfxInput = { layers: [{ wave: "pulse" }] };
// @ts-expect-error The injection centre is exactly three coordinates.
const badTexture: EffectTextureInput = { ...validEffectTextureInput, source: [0, 1] };
// @ts-expect-error Author shapes, including nested tuples, are readonly.
validScatterInput.positions[0] = 1;
"#);
    let dir = TempDir::new("procedural-contract");
    std::fs::write(dir.join("inputs.ts"), declarations).unwrap();
    let check = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "inputs.ts"])
        .current_dir(&dir)
        .output()
        .expect("pinned tsrs is installed");
    assert!(
        check.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}
