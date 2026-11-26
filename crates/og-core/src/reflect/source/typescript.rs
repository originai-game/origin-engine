//! TS shapes from the same structural schema the decoder validates. Numeric,
//! length and pattern constraints remain authoritative at validation time.
use serde_json::Value;
use std::collections::BTreeSet;

/// Emit a root type and an indexed definitions interface. Definition keys are
/// quoted, so recursive/generic Rust names never become invalid TS identifiers.
/// Only local refs are accepted; generation never fetches another document.
pub fn typescript(document: &Value, name: &str) -> Result<String, String> {
    if name.is_empty()
        || !name.bytes().enumerate().all(|(i, b)| {
            b.is_ascii_alphabetic() || b == b'_' || b == b'$' || i > 0 && b.is_ascii_digit()
        })
    {
        return Err(format!("invalid TypeScript type name: {name}"));
    }
    let mut out = format!(
        "{}export type {name} = {};\n",
        doc(document),
        shape(document, document, name)?
    );
    if let Some(defs) = document.get("$defs").and_then(Value::as_object) {
        out.push_str(&format!("export interface {name}Definitions {{\n"));
        for (key, schema) in defs {
            out.push_str(&format!(
                "{}readonly {}: {};\n",
                doc(schema),
                quote(key),
                shape(schema, document, name)?
            ));
        }
        out.push_str("}\n");
    }
    Ok(out)
}

fn quote(value: &str) -> String {
    serde_json::to_string(value).expect("a string serializes")
}

fn doc(schema: &Value) -> String {
    schema
        .get("description")
        .and_then(Value::as_str)
        .map_or_else(String::new, |text| {
            format!("/** {} */\n", text.replace("*/", "* /"))
        })
}

/// Only a finite closed object can exclude other variants' named keys. Follow
/// local aliases without recursing forever; maps and open objects stay open.
fn closed_keys<'a>(mut schema: &'a Value, root: &'a Value) -> Option<BTreeSet<&'a str>> {
    let mut seen = BTreeSet::new();
    loop {
        if schema.get("type").and_then(Value::as_str) == Some("object")
            && schema.get("additionalProperties") == Some(&Value::Bool(false))
            && schema
                .get("patternProperties")
                .and_then(Value::as_object)
                .is_none_or(|p| p.is_empty())
        {
            return Some(
                schema
                    .get("properties")
                    .and_then(Value::as_object)
                    .into_iter()
                    .flat_map(|p| p.keys().map(String::as_str))
                    .collect(),
            );
        }
        let reference = schema.get("$ref")?.as_str()?;
        if !seen.insert(reference) {
            return None;
        }
        schema = root.pointer(reference.strip_prefix('#')?)?;
    }
}

fn shape(schema: &Value, root: &Value, name: &str) -> Result<String, String> {
    match schema.as_bool() {
        Some(true) => return Ok("unknown".into()),
        Some(false) => return Ok("never".into()),
        None => {}
    }
    if !schema.is_object() {
        return Err("a source schema must be an object or boolean".into());
    }
    for key in [
        "not",
        "if",
        "then",
        "else",
        "dependentSchemas",
        "$dynamicRef",
    ] {
        if schema.get(key).is_some() {
            return Err(format!(
                "source schema keyword {key} has no supported TS shape"
            ));
        }
    }
    let mut parts = Vec::new();
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        if reference == "#" {
            parts.push(name.into());
        } else {
            let key = reference
                .strip_prefix("#/$defs/")
                .ok_or_else(|| format!("source refs must be local definitions: {reference}"))?;
            let key = key.replace("~1", "/").replace("~0", "~");
            if root.get("$defs").and_then(|d| d.get(&key)).is_none() {
                return Err(format!("missing source definition: {reference}"));
            }
            parts.push(format!("{name}Definitions[{}]", quote(&key)));
        }
    }
    if let Some(value) = schema.get("const") {
        parts.push(value.to_string());
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        parts.push(if values.is_empty() {
            "never".into()
        } else {
            values
                .iter()
                .map(Value::to_string)
                .collect::<Vec<_>>()
                .join(" | ")
        });
    }
    for (key, separator) in [("anyOf", " | "), ("oneOf", " | "), ("allOf", " & ")] {
        if let Some(values) = schema.get(key).and_then(Value::as_array) {
            let keys: Vec<_> = values.iter().map(|s| closed_keys(s, root)).collect();
            let all_keys: BTreeSet<_> = keys.iter().flatten().flatten().copied().collect();
            let types = values
                .iter()
                .zip(&keys)
                .map(|(s, keys)| {
                    let mut ty = shape(s, root, name)?;
                    if key != "allOf"
                        && let Some(keys) = keys
                    {
                        // Plain TS unions accept mixed branch keys, even for
                        // fresh literals. Closed schema objects never do.
                        let forbidden = all_keys
                            .difference(keys)
                            .map(|k| format!("readonly {}?: never;", quote(k)))
                            .collect::<Vec<_>>();
                        if !forbidden.is_empty() {
                            ty = format!("({ty}) & {{ {} }}", forbidden.join(" "));
                        }
                    }
                    Ok::<_, String>(ty)
                })
                .collect::<Result<Vec<_>, _>>()?;
            parts.push(
                types
                    .into_iter()
                    .map(|s| format!("({s})"))
                    .collect::<Vec<_>>()
                    .join(separator),
            );
        }
    }
    if let Some(types) = schema.get("type").and_then(Value::as_array) {
        let mut variants = Vec::new();
        for ty in types {
            let mut variant = schema.clone();
            variant["type"] = ty.clone();
            // Other restrictions are already intersected outside this union.
            for key in ["$ref", "const", "enum", "anyOf", "oneOf", "allOf"] {
                variant.as_object_mut().unwrap().remove(key);
            }
            variants.push(shape(&variant, root, name)?);
        }
        parts.push(variants.join(" | "));
    } else if let Some(ty) = schema.get("type").and_then(Value::as_str) {
        parts.push(match ty {
            "null" => "null".into(),
            "boolean" => "boolean".into(),
            "number" | "integer" => "number".into(),
            "string" => "string".into(),
            "array" => array(schema, root, name)?,
            "object" => object(schema, root, name)?,
            _ => return Err(format!("unknown source schema type: {ty}")),
        });
    }
    Ok(if parts.is_empty() {
        "unknown".into()
    } else {
        parts
            .into_iter()
            .map(|s| format!("({s})"))
            .collect::<Vec<_>>()
            .join(" & ")
    })
}

fn array(schema: &Value, root: &Value, name: &str) -> Result<String, String> {
    if let Some(prefix) = schema.get("prefixItems").and_then(Value::as_array) {
        let min = schema.get("minItems").and_then(Value::as_u64).unwrap_or(0) as usize;
        let max = schema.get("maxItems").and_then(Value::as_u64);
        let mut fields = prefix
            .iter()
            .enumerate()
            .map(|(i, s)| {
                shape(s, root, name).map(|t| if i >= min { format!("({t})?") } else { t })
            })
            .collect::<Result<Vec<_>, _>>()?;
        if schema.get("items") != Some(&Value::Bool(false))
            && max.is_none_or(|n| n > prefix.len() as u64)
        {
            let rest = shape(
                schema.get("items").unwrap_or(&Value::Bool(true)),
                root,
                name,
            )?;
            fields.push(format!("...({rest})[]"));
        }
        return Ok(format!("readonly [{}]", fields.join(", ")));
    }
    let ty = shape(
        schema.get("items").unwrap_or(&Value::Bool(true)),
        root,
        name,
    )?;
    let min = schema.get("minItems").and_then(Value::as_u64);
    let max = schema.get("maxItems").and_then(Value::as_u64);
    if let Some(length) = min.filter(|n| Some(*n) == max) {
        return Ok(format!(
            "ReadonlyArray<{ty}> & {{ readonly length: {length} }}"
        ));
    }
    // Ask TS whether the emitted item type accepts any number. This also
    // handles aliases and nullable unions without widening literal enums or
    // duplicating schema resolution here. Fixed arrays/tuples stay above.
    Ok(format!(
        "ReadonlyArray<{ty}> | (number extends ({ty}) ? Readonly<Int8Array | Uint8Array | Uint8ClampedArray | Int16Array | Uint16Array | Int32Array | Uint32Array | Float32Array | Float64Array> : never)"
    ))
}

fn object(schema: &Value, root: &Value, name: &str) -> Result<String, String> {
    let properties = schema.get("properties").and_then(Value::as_object);
    let additional = schema
        .get("additionalProperties")
        .unwrap_or(&Value::Bool(true));
    let patterns = schema.get("patternProperties").and_then(Value::as_object);
    if patterns.is_some_and(|p| !p.is_empty()) && properties.is_some_and(|p| !p.is_empty()) {
        return Err("patternProperties beside named properties has no supported TS shape".into());
    }
    if properties.is_none_or(|p| p.is_empty()) {
        let Some(patterns) = patterns.filter(|p| !p.is_empty()) else {
            return Ok(format!(
                "Readonly<Record<string, {}>>",
                shape(additional, root, name)?
            ));
        };
        // TS cannot express regex keys or overlapping-pattern intersections.
        // Preserve each possible value type; schema + serde check the keys and
        // their ranges. Unmatched keys use additionalProperties, not a pattern.
        let mut values = patterns
            .values()
            .map(|s| shape(s, root, name))
            .collect::<Result<Vec<_>, _>>()?;
        if additional != &Value::Bool(false) {
            values.push(shape(additional, root, name)?);
        }
        let value = values
            .into_iter()
            .map(|s| format!("({s})"))
            .collect::<Vec<_>>()
            .join(" | ");
        return Ok(format!("Readonly<Record<string, {value}>>"));
    }
    let mut fields = String::from("{\n");
    for (key, value) in properties.unwrap() {
        let required = schema
            .get("required")
            .and_then(Value::as_array)
            .is_some_and(|a| a.iter().any(|v| v.as_str() == Some(key)));
        fields.push_str(&format!(
            "{}readonly {}{}: {};\n",
            doc(value),
            quote(key),
            if required { "" } else { "?" },
            shape(value, root, name)?
        ));
    }
    if additional != &Value::Bool(false) {
        if additional != &Value::Bool(true) {
            // Forbidden names cannot conflict with the index's value type.
            // Other named schemas may, so do not relax the general rejection.
            if properties
                .unwrap()
                .values()
                .all(|s| s == &Value::Bool(false))
            {
                fields.push('}');
                return Ok(format!(
                    "({fields}) & Readonly<Record<string, {}>>",
                    shape(additional, root, name)?
                ));
            }
            return Err("typed additionalProperties beside named properties cannot be represented faithfully in TS".into());
        }
        fields.push_str("readonly [key: string]: unknown;\n");
    }
    fields.push('}');
    Ok(fields)
}
