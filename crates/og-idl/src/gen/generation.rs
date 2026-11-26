//! The pure generator host: the same IDL signatures as the live hosts,
//! without a world, renderer, filesystem or second parameter reader.

use super::host::{Wire, signature};
use super::{GENERATED_BY, bindings};
use crate::index::{Index, TypeDecl};
use crate::json::Json;
use crate::model::TypeRef;
use std::fmt::Write;

/// `og-generation`'s offline JSON dispatcher and closed module exports.
pub fn rust(ix: &Index<'_>) -> String {
    let mut out = format!(
        "// {GENERATED_BY}\nuse og_api::wire::{{self, ToJson}};\nuse serde_json::{{Value, json}};\n\npub const FUNCTIONS: &[&str] = &[\n"
    );
    let all: Vec<_> = bindings(ix)
        .into_iter()
        .filter(|b| b.module.name == "procgen" && b.owner.is_none())
        .collect();
    for b in &all {
        let _ = writeln!(out, "    {:?},", b.name);
    }
    let shapes = Json::Obj(
        all.iter()
            .filter_map(|b| {
                let shape = output_shape(ix, &b.function.expect("function").returns);
                (shape != Json::Null).then(|| (b.name.to_string(), shape))
            })
            .collect(),
    );
    let names = Json::Arr(all.iter().map(|b| Json::str(b.name)).collect());
    let source = format!(
        r#"
        const invoke = globalThis.__procgen;
        const shapes = {};
        function typed(value, shape) {{
            if (value == null || shape == null) return value;
            if (typeof shape === 'string') return new globalThis[shape](value);
            if (Array.isArray(shape)) return value.map(v => typed(v, shape[0]));
            for (const key of Object.keys(shape)) value[key] = typed(value[key], shape[key]);
            return value;
        }}
        export const procgen = Object.freeze(Object.fromEntries({}.map(name => [name, (...args) =>
            typed(JSON.parse(invoke(name, JSON.stringify(args, (_, v) => {{
                if (typeof v === 'number' && !Number.isFinite(v)) throw new Error('non-finite generator argument');
                return ArrayBuffer.isView(v) ? Array.from(v) : v;
            }}))), shapes[name])
        ])));
    "#,
        shapes.compact(),
        names.compact()
    );
    let _ = writeln!(out, "];\n\npub const MODULE_SOURCE: &str = {source:?};");
    out.push_str("\npub fn invoke(name: &str, args: &[Value]) -> Result<Value, String> {\n    Ok(match name {\n");
    for b in &all {
        let (params, _) = signature(ix, b);
        let f = b.function.expect("pure module function");
        let _ = writeln!(out, "        {:?} => {{", b.name);
        let mut names = Vec::new();
        for (i, p) in params.iter().enumerate() {
            let name = format!("a{i}");
            let get = if i == 0 {
                "args.first()".to_string()
            } else {
                format!("args.get({i})")
            };
            let required = format!(
                "{get}.ok_or({:?})?",
                format!("{}.{}: required argument", b.path, p.name)
            );
            let expression = match (p.wire, p.optional) {
                (Wire::Json, true) => format!("{get}.map(Value::to_string)"),
                (Wire::Json, false) => format!("{get}.unwrap_or(&Value::Null).to_string()"),
                (Wire::Binary, true) => {
                    format!("{get}.map(|v| v.to_string().into_bytes())")
                }
                (Wire::Binary, false) => format!("{required}.to_string().into_bytes()"),
                (w @ (Wire::Bool | Wire::I32 | Wire::U32 | Wire::F64 | Wire::Text), false) => {
                    let ty = match w {
                        Wire::Bool => "bool",
                        Wire::I32 => "i32",
                        Wire::U32 => "u32",
                        Wire::F64 => "f64",
                        _ => "String",
                    };
                    format!("wire::value::<{ty}>({required}, {:?})?", p.name)
                }
                _ => panic!("{} is not a pure JSON-callable signature", b.path),
            };
            let _ = writeln!(out, "            let {name} = {expression};");
            names.push(name);
        }
        let call = format!("crate::{}({})", b.symbol, names.join(", "));
        if f.returns.name() == "Result" {
            let _ = writeln!(
                out,
                "            match {call} {{\n                Ok(value) => json!({{\"ok\": true, \"value\": value.to_json()}}),\n                Err(reason) => json!({{\"ok\": false, \"reason\": reason}}),\n            }}"
            );
        } else {
            let _ = writeln!(out, "            {call}.to_json()");
        }
        out.push_str("        }\n");
    }
    out.push_str(
        "        _ => return Err(format!(\"unknown pure generator: {name}\")),\n    })\n}\n",
    );
    out
}

/// The typed buffers in an output; every other field is already JSON.
fn output_shape(ix: &Index<'_>, ty: &TypeRef) -> Json {
    match ty.name() {
        "Float32Array" | "Float64Array" | "Uint32Array" | "Int32Array" | "Uint8Array" => {
            Json::str(ty.name())
        }
        "Option" => output_shape(ix, &ty.args()[0]),
        "Result" => {
            let shape = output_shape(ix, &ty.args()[0]);
            if shape == Json::Null {
                shape
            } else {
                Json::Obj(vec![("value".into(), shape)])
            }
        }
        "List" => {
            let shape = output_shape(ix, &ty.args()[0]);
            if shape == Json::Null {
                shape
            } else {
                Json::Arr(vec![shape])
            }
        }
        name => match ix.decl(name) {
            Some(TypeDecl::Struct(s)) => {
                let fields: Vec<_> = s
                    .fields
                    .iter()
                    .filter_map(|f| {
                        let shape = output_shape(ix, &f.ty);
                        (shape != Json::Null).then(|| (f.name.clone(), shape))
                    })
                    .collect();
                if fields.is_empty() {
                    Json::Null
                } else {
                    Json::Obj(fields)
                }
            }
            _ => Json::Null,
        },
    }
}
