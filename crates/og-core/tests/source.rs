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
    child: Option<Box<Input>>,
}

#[test]
fn one_author_schema_checks_values_and_typescript() {
    let schema = InputSchema::<Input>::new().unwrap();
    let valid = json!({
        "display-name": "north",
        "mode": {"kind": "grid", "cell-size": [2, 3, 7]},
        "tuple": [-1.5, 257, true],
        "duration": 0.25,
        "samples": vec![0; 65],
        "values": {"wind": [2.5, -3], "water": null}
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
    assert!(decoded.child.is_none());
    assert_eq!(
        schema.document()["properties"]["duration"]["x-og-unit"],
        "s"
    );

    for (pointer, wrong) in [
        ("/tuple/1", json!(-1)),
        ("/duration", json!(0.249)),
        ("/duration", json!(4.001)),
        ("/samples", json!(vec![0; 64])),
        ("/mode", json!({"kind": "grid", "cell-size": [2, 3]})),
        ("/mode", json!({"kind": "random"})),
        ("/values/wind/1", json!("gust")),
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

    let dir = temp::TempDir::new("source-schema");
    let positive = r#"
const input: AuthorInput = {
  "display-name": "north", mode: { kind: "grid", "cell-size": [2, 3, 7] },
  tuple: [-1.5, 257, true], duration: 0.25,
  samples: SAMPLES, values: { wind: [2.5, -3], water: null },
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
// @ts-expect-error Fixed arrays retain their length even above 64 entries.
const wrongCount: AuthorInput["samples"] = [0];
// @ts-expect-error No unknown fields on a closed input object.
const unknown: AuthorInput = { ...input, typo: 1 };
// @ts-expect-error Generated author inputs are readonly.
input.enabled = true;
"#.replace("SAMPLES", &format!("[{}]", vec!["0"; 65].join(",")));
    std::fs::write(
        dir.join("input.ts"),
        format!(
            "{}\n{positive}",
            typescript(schema.document(), "AuthorInput").unwrap()
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
