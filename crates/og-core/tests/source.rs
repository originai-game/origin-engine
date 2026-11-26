//! The author schema is shared by validation, serde decoding and emitted TS.
use og_core::reflect::source::{AuthorSchema, InputSchema, schemars::JsonSchema, typescript};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;

#[path = "../../../tests/temp.rs"]
mod temp;

#[derive(Debug, Deserialize, JsonSchema, PartialEq)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Mode {
    Constant {
        value: f32,
    },
    Grid {
        #[serde(rename = "cell-size")]
        cell_size: [f32; 3],
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
struct Input {
    /// Human-readable label, not an asset ID.
    #[serde(rename = "display-name")]
    label: String,
    #[serde(default)]
    enabled: bool,
    mode: Mode,
    tuple: (f32, u16, bool),
    #[schemars(range(min = 0.25, max = 4.0), extend("x-og-unit" = "s"))]
    duration: f64,
    #[schemars(length(equal = 65))]
    samples: Vec<f32>,
    values: BTreeMap<String, Option<Vec<f32>>>,
    tiles: BTreeMap<u32, Mode>,
    masks: BTreeMap<u8, u32>,
    child: Option<Box<Input>>,
}

#[derive(Debug, Deserialize, JsonSchema, PartialEq)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
struct FileSource {
    source: String,
}

#[derive(Debug, Deserialize, JsonSchema, PartialEq)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
struct InlineSource {
    svg: String,
}

#[derive(Debug, Deserialize, JsonSchema, PartialEq)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(untagged)]
enum UntaggedSource {
    File(FileSource),
    Inline(InlineSource),
}

#[derive(Debug, Deserialize, JsonSchema, PartialEq)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
enum TaggedSource {
    Equirect(String),
    Ktx2(String),
}

#[test]
fn one_author_schema_checks_values_and_typescript() {
    let float = InputSchema::<f32>::new().unwrap();
    let overflow = float.decode(json!(1e100));
    assert!(
        overflow.is_err(),
        "finite JSON overflow decoded as {overflow:?}"
    );
    let max = f64::from(f32::MAX);
    assert_eq!(float.decode(json!(max)).unwrap(), f32::MAX);
    assert_eq!(float.decode(json!(-max)).unwrap(), f32::MIN);
    for outside in [max.next_up(), (-max).next_down(), -1e100] {
        assert!(float.decode(json!(outside)).is_err(), "accepted {outside}");
    }
    assert_eq!(
        InputSchema::<f64>::new()
            .unwrap()
            .decode(json!(1e100))
            .unwrap(),
        1e100
    );
    let narrow = InputSchema::<f32>::from_document(json!({
        "type": "number", "format": "float", "minimum": -0.25, "maximum": 0.75
    }))
    .unwrap();
    assert_eq!(narrow.decode(json!(-0.25)).unwrap(), -0.25);
    assert_eq!(narrow.decode(json!(0.75)).unwrap(), 0.75);
    assert!(narrow.decode(json!(-0.251)).is_err());
    assert!(narrow.decode(json!(0.751)).is_err());
    let annotations = json!({"type": "number", "format": "float", "maximum": 1e100});
    let annotated = InputSchema::<serde_json::Value>::from_document(json!({
        "type": "object", "default": annotations, "examples": [annotations],
        "properties": {"gain": {"type": "number", "format": "float"}}
    }))
    .unwrap();
    assert_eq!(annotated.document()["default"], annotations);
    assert_eq!(annotated.document()["examples"][0], annotations);
    assert_eq!(
        annotated
            .validate(&json!({"gain": 1e100}))
            .unwrap_err()
            .path,
        "/gain"
    );
    assert!(
        InputSchema::<f32>::from_document(json!({
            "type": "number", "format": "float", "minimum": "invalid"
        }))
        .is_err()
    );

    let schema = InputSchema::<Input>::new().unwrap();
    let valid = json!({
        "display-name": "north",
        "mode": {"kind": "grid", "cell-size": [2, 3, 7]},
        "tuple": [-1.5, 257, true],
        "duration": 0.25,
        "samples": vec![0; 65],
        "values": {"wind": [2.5, -3], "water": null},
        "tiles": {"257": {"kind": "constant", "value": -3.5}},
        "masks": {"255": 513}
    });
    let decoded = schema.decode(valid.clone()).unwrap();
    assert_eq!(decoded.label, "north");
    assert!(!decoded.enabled);
    assert_eq!(
        decoded.mode,
        Mode::Grid {
            cell_size: [2., 3., 7.]
        }
    );
    assert_eq!(decoded.tuple, (-1.5, 257, true));
    assert_eq!(decoded.duration, 0.25);
    assert_eq!(decoded.samples.len(), 65);
    assert_eq!(decoded.values["wind"], Some(vec![2.5, -3.]));
    assert_eq!(decoded.tiles[&257], Mode::Constant { value: -3.5 });
    assert_eq!(decoded.masks[&255], 513);
    assert!(decoded.child.is_none());
    assert_eq!(
        schema.document()["properties"]["duration"]["x-og-unit"],
        "s"
    );

    for (pointer, wrong) in [
        ("/tuple/1", json!(-1)),
        ("/tuple/0", json!(1e100)),
        ("/duration", json!(0.249)),
        ("/duration", json!(4.001)),
        ("/samples", json!(vec![0; 64])),
        ("/samples/0", json!(-1e100)),
        ("/mode", json!({"kind": "grid", "cell-size": [2, 3]})),
        ("/mode", json!({"kind": "random"})),
        ("/mode", json!({"kind": "constant", "value": 1e100})),
        ("/values/wind/1", json!("gust")),
        ("/values/wind/1", json!(1e100)),
        ("/tiles", json!({"-1": {"kind": "constant", "value": 1}})),
        ("/tiles/257", json!({"kind": "random"})),
        ("/masks", json!({"256": 513})),
        ("/masks/255", json!(-1)),
    ] {
        let mut value = valid.clone();
        *value.pointer_mut(pointer).unwrap() = wrong;
        assert!(schema.decode(value).is_err(), "accepted {pointer}");
    }
    let mut unknown = valid.clone();
    unknown["typo"] = json!(1);
    assert!(schema.decode(unknown).is_err());
    let mut nested = valid.clone();
    nested["child"] = valid.clone();
    nested["child"]["tuple"][1] = json!(-1);
    // A nullable union fails at its branch root, not an invented chosen branch.
    assert_eq!(schema.decode(nested).unwrap_err().path, "/child");
    let mut tuple = valid.clone();
    tuple["tuple"][1] = json!(-1);
    assert_eq!(schema.decode(tuple).unwrap_err().path, "/tuple/1");
    let mut upper = valid.clone();
    upper["duration"] = json!(4);
    schema.decode(upper).unwrap();

    let erased: &dyn AuthorSchema = &schema;
    assert_eq!(erased.document(), schema.document());
    erased.check(&valid).unwrap();
    let mut wrong = valid.clone();
    wrong["tuple"][1] = json!(-1);
    assert_eq!(erased.check(&wrong).unwrap_err().path, "/tuple/1");

    // Reflected additions may describe a broader shape than serde accepts.
    // Erasing the input type must not silently discard decoder constraints.
    let byte = InputSchema::<u8>::from_document(json!({"type": "integer"})).unwrap();
    byte.validate(&json!(256)).unwrap();
    let erased: &dyn AuthorSchema = &byte;
    erased.check(&json!(255)).unwrap();
    assert_eq!(erased.check(&json!(256)).unwrap_err().path, "");
    assert!(InputSchema::<u8>::from_document(json!({"type": "not-a-type"})).is_err());

    let patterned = json!({
        "type": "object",
        "patternProperties": {"^i": {"type": "integer"}, "^s": {"type": "string"}},
        "additionalProperties": {"type": "boolean"}
    });
    let pattern_check =
        InputSchema::<BTreeMap<String, serde_json::Value>>::from_document(patterned.clone())
            .unwrap();
    pattern_check
        .decode(json!({"i_count": 7, "s_name": "west", "other": true}))
        .unwrap();
    assert!(pattern_check.decode(json!({"s_name": 7})).is_err());
    let mut closed_patterned = patterned.clone();
    closed_patterned["additionalProperties"] = json!(false);
    let mut named_patterned = patterned.clone();
    named_patterned["properties"] = json!({"label": {"type": "string"}});
    assert!(typescript(&named_patterned, "NamedPatterned").is_err());

    let params = json!({
        "type": "object",
        "properties": {"roughness": false, "opacity": false},
        "additionalProperties": {"anyOf": [
            {"type": "number"},
            {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}
        ]}
    });
    let params_check =
        InputSchema::<BTreeMap<String, serde_json::Value>>::from_document(params.clone()).unwrap();
    params_check
        .decode(json!({"wind": [2, -3], "phase": 0.7}))
        .unwrap();
    assert!(params_check.decode(json!({"roughness": 0.5})).is_err());
    let mut named_params = params.clone();
    named_params["properties"]["label"] = json!({"type": "string"});
    assert!(typescript(&named_params, "NamedParams").is_err());

    let untagged = InputSchema::<UntaggedSource>::new().unwrap();
    let tagged = InputSchema::<TaggedSource>::new().unwrap();
    assert_eq!(
        untagged.decode(json!({"source": "west.svg"})).unwrap(),
        UntaggedSource::File(FileSource {
            source: "west.svg".into()
        })
    );
    assert_eq!(
        untagged.decode(json!({"svg": "<svg/>"})).unwrap(),
        UntaggedSource::Inline(InlineSource {
            svg: "<svg/>".into()
        })
    );
    assert!(
        untagged
            .decode(json!({"source": "west.svg", "svg": "<svg/>"}))
            .is_err()
    );
    assert_eq!(
        tagged.decode(json!({"equirect": "west.hdr"})).unwrap(),
        TaggedSource::Equirect("west.hdr".into())
    );
    assert_eq!(
        tagged.decode(json!({"ktx2": "east.ktx2"})).unwrap(),
        TaggedSource::Ktx2("east.ktx2".into())
    );
    assert!(
        tagged
            .decode(json!({"equirect": "west.hdr", "ktx2": "east.ktx2"}))
            .is_err()
    );
    // An open dictionary arm still permits mixed keys; closing the named arm
    // must not accidentally make the entire union exclusive.
    let open_union = json!({"anyOf": [
        {"type": "object", "properties": {"source": {"type": "string"}},
            "required": ["source"], "additionalProperties": false},
        {"type": "object", "additionalProperties": {"type": "string"}}
    ]});

    let dir = temp::TempDir::new("source-schema");
    let positive = r#"
const input: AuthorInput = {
  "display-name": "north", mode: { kind: "grid", "cell-size": [2, 3, 7] },
  tuple: [-1.5, 257, true], duration: 0.25,
  samples: SAMPLES, values: { wind: [2.5, -3], water: null },
  tiles: { 257: { kind: "constant", value: -3.5 } }, masks: { 255: 513 },
};
const recursive: AuthorInput = { ...input, enabled: true, child: input };
const variant: AuthorInput["mode"] = { kind: "constant", value: -2.5 };
// @ts-expect-error Missing required name.
const missing: AuthorInput = { mode: input.mode, tuple: input.tuple, duration: 1, samples: input.samples, values: {} };
// @ts-expect-error A discriminator is not an arbitrary string.
const wrongKind: AuthorInput["mode"] = { kind: "random", value: 1 };
// @ts-expect-error A grid is a fixed triple, not an arbitrary numeric array.
const shortGrid: AuthorInput["mode"] = { kind: "grid", "cell-size": [2, 3] };
// @ts-expect-error A heterogeneous tuple cannot swap the flag and integer.
const wrongTuple: AuthorInput["tuple"] = [-1.5, true, 257];
// @ts-expect-error A heterogeneous tuple has no trailing values.
const longTuple: AuthorInput["tuple"] = [-1.5, 257, true, 8];
// @ts-expect-error A map value is numbers or null, not text.
const wrongMap: AuthorInput["values"] = { wind: [2, "gust"] };
// @ts-expect-error Integer-key maps retain their discriminated value type.
const wrongTile: AuthorInput["tiles"] = { 257: { kind: "random", value: 1 } };
// @ts-expect-error Byte-key maps do not widen numeric values to unknown.
const wrongMask: AuthorInput["masks"] = { 3: "north" };
// @ts-expect-error Pattern-key map declarations remain readonly.
input.masks[3] = 9;
// Key patterns and numeric key bounds are checked at decoding, not in TS.
const deferredKey: AuthorInput["masks"] = { 256: 513 };
const patternValues: Patterned = { i_count: 7, s_name: "west", other: true };
const closedValues: ClosedPatterned = { i_count: 7, s_name: "west" };
// @ts-expect-error Pattern value unions do not widen objects to unknown.
const wrongPatternValue: Patterned = { i_count: {} };
// @ts-expect-error additionalProperties=false does not introduce a boolean value arm.
const wrongClosedValue: ClosedPatterned = { other: true };
const shaderParams: UniformParams = { wind: [2, -3], phase: 0.7 };
// @ts-expect-error Reserved material names are not shader parameters.
const reservedParam: UniformParams = { roughness: 0.5 };
// @ts-expect-error Excluding names must not erase the parameter value type.
const wrongParam: UniformParams = { wind: "east" };
// @ts-expect-error Parameter vectors retain their arity.
const wrongParamVector: UniformParams = { wind: [2] };
// @ts-expect-error Shader parameter maps remain readonly.
shaderParams.phase = 0.2;
// @ts-expect-error Fixed arrays retain their length even above 64 entries.
const wrongCount: AuthorInput["samples"] = [0];
// @ts-expect-error No unknown fields on a closed input object.
const unknown: AuthorInput = { ...input, typo: 1 };
// @ts-expect-error Generated author inputs are readonly.
input.enabled = true;
const pathSource: UntaggedSource = { source: "west.svg" };
const inlineSource: UntaggedSource = { svg: "<svg/>" };
const hdr: TaggedSource = { equirect: "west.hdr" };
const compressed: TaggedSource = { ktx2: "east.ktx2" };
const openMix: OpenUnion = { source: "west.svg", svg: "<svg/>" };
// @ts-expect-error Closed untagged branches must not mix their source fields.
const bothSources: UntaggedSource = { source: "west.svg", svg: "<svg/>" };
const mixedSource = { source: "west.svg", svg: "<svg/>" };
// @ts-expect-error Exclusivity also holds after losing object-literal freshness.
const aliasSource: UntaggedSource = mixedSource;
// @ts-expect-error An externally tagged enum selects exactly one source.
const bothTags: TaggedSource = { equirect: "west.hdr", ktx2: "east.ktx2" };
"#.replace("SAMPLES", &format!("[{}]", vec!["0"; 65].join(",")));
    std::fs::write(
        dir.join("input.ts"),
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{positive}",
            typescript(schema.document(), "AuthorInput").unwrap(),
            typescript(&patterned, "Patterned").unwrap(),
            typescript(&closed_patterned, "ClosedPatterned").unwrap(),
            typescript(&params, "UniformParams").unwrap(),
            typescript(untagged.document(), "UntaggedSource").unwrap(),
            typescript(tagged.document(), "TaggedSource").unwrap(),
            typescript(&open_union, "OpenUnion").unwrap()
        ),
    )
    .unwrap();
    let check = std::process::Command::new("tsrs")
        .args(["--ignoreConfig", "--strict", "--noEmit", "input.ts"])
        .current_dir(&dir)
        .output()
        .expect("the pinned tsrs checker must be installed (docs/ci.md)");
    assert!(
        check.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}
