//! The standard material's reflected schema against the IDL.

use og_core::reflect::Reflect;
use og_render::StandardMaterial;
use std::path::PathBuf;

/// The reflected schema is the IDL's `render.StandardMaterialDesc`: every
/// IDL field exists under the same three.js name with the same default.
/// Fields the IDL does not carry yet are listed (and only those).
#[test]
fn the_material_schema_matches_the_idl() {
    let schema = StandardMaterial::schema();
    schema.validate().expect("a valid schema");
    let idl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../og-api/api/render.ogt");
    let idl = std::fs::read_to_string(&idl_path).expect("the IDL");
    let start = idl
        .find("struct StandardMaterialDesc {")
        .expect("StandardMaterialDesc in the IDL");
    let body = &idl[start..];
    let end = body.find("\n  }\n").expect("the struct's end");
    let mut fields: Vec<(String, Vec<f64>)> = Vec::new();
    for line in body[..end].lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("field ") {
            let name = rest.split_whitespace().next().expect("a name").to_string();
            fields.push((name, Vec::new()));
        } else if let Some(rest) = t.strip_prefix("default ") {
            let values = rest
                .split_whitespace()
                .map(|v| match v {
                    "true" => 1.0,
                    "false" => 0.0,
                    n => n.parse().expect("a number"),
                })
                .collect();
            fields.last_mut().expect("a field").1 = values;
        }
    }
    assert!(fields.len() >= 15, "{} IDL fields", fields.len());
    let zero = |n: usize| vec![0.0; n];
    for (name, default) in &fields {
        let f = schema
            .fields
            .iter()
            .find(|f| &f.name == name)
            .unwrap_or_else(|| panic!("IDL field {name} is missing from og.StandardMaterial"));
        let lanes = f.ty.lanes();
        let ours = f.default.clone().unwrap_or_else(|| zero(lanes));
        let theirs = if default.is_empty() {
            zero(lanes)
        } else {
            default.clone()
        };
        assert_eq!(ours, theirs, "default of {name}");
    }
    let extra: Vec<&str> = schema
        .fields
        .iter()
        .map(|f| f.name.as_str())
        .filter(|n| !fields.iter().any(|(m, _)| m == n))
        .collect();
    println!("og.StandardMaterial fields the IDL does not have yet: {extra:?}");
    for n in &extra {
        assert!(
            [
                "normalScale",
                "aoMapIntensity",
                "unlit",
                // The effects (crates/og-render/README.md §From TypeScript).
                "toon",
                "gradientMap",
                "rimColor",
                "rimIntensity",
                "rimPower",
                "flashColor",
                "flash",
                "dissolve",
                "dissolveMap",
                "dissolveEdgeColor",
                "dissolveEdgeWidth",
                "dissolveScale",
                "outlineColor",
                "outlineWidth",
            ]
            .contains(n),
            "{n} is in neither the IDL nor the list of fields awaiting it"
        );
    }
}
