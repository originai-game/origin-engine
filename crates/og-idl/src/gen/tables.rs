//! The JSON generators: the runtime binding table
//! (`packages/engine/types/bindings.json`) and the JSON Schema with the API
//! catalog (`packages/engine/schema/api.schema.json`).

use super::{GENERATED_BY, bindings as all_bindings, default_json, joined};
use crate::index::{Index, TypeDecl};
use crate::json::Json;
use crate::model::*;

fn strings<'s>(items: impl IntoIterator<Item = &'s str>) -> Json {
    Json::Arr(items.into_iter().map(Json::str).collect())
}

fn params_json(ix: &Index<'_>, params: &[Param], with_doc: bool) -> Json {
    let omit = crate::model::omittable(params, ix);
    Json::Arr(
        params
            .iter()
            .zip(omit)
            .map(|(p, optional)| {
                let mut o = Json::obj()
                    .with("name", Json::str(p.name.clone()))
                    .with("type", Json::str(p.ty.to_string()))
                    .with("optional", Json::Bool(optional));
                if p.flatten {
                    o.set("flatten", Json::Bool(true));
                }
                if let Some(d) = &p.default
                    && let Some(j) = default_json(ix, &p.ty, Some(d))
                {
                    o.set("default", j);
                }
                if with_doc {
                    o.set("doc", Json::str(joined(&p.doc)));
                }
                o
            })
            .collect(),
    )
}

/// The runtime binding table: every callable with a stable id and host
/// symbol, every column kind, and every registered record's layout.
pub fn bindings(ix: &Index<'_>) -> String {
    let api = ix.api;
    let mut root = Json::obj()
        .with("$comment", Json::str(GENERATED_BY))
        .with("package", Json::str(api.package.clone()))
        .with("api", Json::num(api.version));

    let mut columns = Vec::new();
    let mut records: [Vec<Json>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    let mut callbacks = Vec::new();
    for m in &api.modules {
        for item in &m.items {
            match item {
                Item::Column(c) => {
                    let mut o = Json::obj()
                        .with("name", Json::str(c.name.clone()))
                        .with("array", Json::str(c.array.clone()))
                        .with("stride", Json::num(c.stride))
                        .with("value", Json::str(c.value.to_string()));
                    if c.capacity {
                        // The stride is each field's capacity (`str(32)`).
                        o.set("capacity", Json::Bool(true));
                    }
                    columns.push(o);
                }
                Item::Record(r) => {
                    let slot = match r.kind {
                        RecordKind::Component => 0,
                        RecordKind::Resource => 1,
                        RecordKind::Event => 2,
                    };
                    let id = records[slot].len();
                    let fields = r
                        .fields
                        .iter()
                        .map(|f| {
                            let mut o = Json::obj().with("name", Json::str(f.name.clone()));
                            let key = if r.kind == RecordKind::Event {
                                "type"
                            } else {
                                "column"
                            };
                            o.set(key, Json::str(f.ty.to_string()));
                            if let Some(d) = default_json(ix, &f.ty, f.default.as_deref()) {
                                o.set("default", d);
                            }
                            if f.readonly {
                                o.set("readonly", Json::Bool(true));
                            }
                            if f.tunable {
                                o.set("tunable", Json::Bool(true));
                            }
                            o
                        })
                        .collect();
                    records[slot].push(
                        Json::obj()
                            .with("id", Json::num(id))
                            .with("name", Json::str(r.name.clone()))
                            .with("schema", Json::str(super::schema_name(&r.name)))
                            .with("module", Json::str(m.name.clone()))
                            .with("fields", Json::Arr(fields)),
                    );
                }
                Item::Callback(c) => callbacks.push(
                    Json::obj()
                        .with("name", Json::str(c.name.clone()))
                        .with("module", Json::str(m.name.clone()))
                        .with("params", params_json(ix, &c.params, false))
                        .with("returns", Json::str(c.returns.to_string())),
                ),
                _ => {}
            }
        }
    }
    let [components, resources, events] = records;
    root.set("columns", Json::Arr(columns));
    root.set("components", Json::Arr(components));
    root.set("resources", Json::Arr(resources));
    root.set("events", Json::Arr(events));

    let calls = all_bindings(ix)
        .into_iter()
        .enumerate()
        .map(|(id, b)| {
            let mut o = Json::obj()
                .with("id", Json::num(id))
                .with("kind", Json::str(b.kind))
                .with("path", Json::str(b.path.clone()))
                .with("symbol", Json::str(b.symbol.clone()))
                .with("module", Json::str(b.module.name.clone()));
            if let Some(owner) = b.owner {
                o.set("owner", Json::str(owner));
            }
            o.set("name", Json::str(b.name));
            if let Some(f) = b.function {
                if !f.generics.is_empty() {
                    o.set(
                        "generics",
                        Json::Arr(
                            f.generics
                                .iter()
                                .map(|g| {
                                    Json::obj()
                                        .with("name", Json::str(g.name.clone()))
                                        .with("bound", Json::str(g.bound.to_string()))
                                })
                                .collect(),
                        ),
                    );
                }
                o.set("params", params_json(ix, &f.params, false));
                o.set("returns", Json::str(f.returns.to_string()));
            }
            if let Some(p) = b.prop {
                o.set("type", Json::str(p.ty.to_string()));
            }
            o.set("publish", Json::Bool(b.publish));
            o
        })
        .collect();
    root.set("calls", Json::Arr(calls));
    root.set("callbacks", Json::Arr(callbacks));
    root.render()
}

/// JSON Schema (2020-12) for every data type, plus `x-og-*` annotations:
/// the module catalog (functions, interfaces, callbacks with their docs) and
/// the unit table. The platform's knowledge package reads this file.
pub fn schema(ix: &Index<'_>) -> String {
    let api = ix.api;
    let mut root = Json::obj()
        .with("$comment", Json::str(GENERATED_BY))
        .with(
            "$schema",
            Json::str("https://json-schema.org/draft/2020-12/schema"),
        )
        .with(
            "$id",
            Json::str(format!("urn:origingame:engine:api:{}", api.version)),
        )
        .with(
            "title",
            Json::str(format!("{} API v{}", api.package, api.version)),
        )
        .with("description", Json::str(joined(&api.doc)));

    let mut units = Json::obj();
    for u in &api.units {
        units.set(&u.name, Json::str(joined(&u.doc)));
    }
    root.set("x-og-units", units);

    let mut defs = Json::obj();
    let mut modules = Vec::new();
    for m in &api.modules {
        let mut catalog = Json::obj()
            .with("name", Json::str(m.name.clone()))
            .with("doc", Json::str(joined(&m.doc)))
            .with("place", Json::str(m.place.as_str()))
            .with("publish", Json::Bool(m.publish));
        let mut fns = Vec::new();
        let mut types = Vec::new();
        let mut interfaces = Vec::new();
        let mut callbacks = Vec::new();
        let mut consts = Vec::new();
        let mut values = Vec::new();
        for item in &m.items {
            match item {
                Item::Fn(f) => fns.push(function_json(ix, f, f.publish.unwrap_or(m.publish))),
                Item::Interface(i) => {
                    let members = i
                        .members
                        .iter()
                        .map(|mem| {
                            let publish = mem.publish().or(i.publish).unwrap_or(m.publish);
                            match mem {
                                Member::Fn(f) => function_json(ix, f, publish),
                                Member::Prop(p) => Json::obj()
                                    .with("name", Json::str(p.name.clone()))
                                    .with("prop", Json::str(p.ty.to_string()))
                                    .with("doc", Json::str(joined(&p.doc)))
                                    .with("publish", Json::Bool(publish)),
                            }
                        })
                        .collect();
                    let mut o = Json::obj()
                        .with("name", Json::str(i.name.clone()))
                        .with("doc", Json::str(joined(&i.doc)));
                    if let Some(b) = &i.extends {
                        o.set("extends", Json::str(b.clone()));
                    }
                    o.set("members", Json::Arr(members));
                    interfaces.push(o);
                }
                Item::Callback(c) => callbacks.push(
                    Json::obj()
                        .with("name", Json::str(c.name.clone()))
                        .with("doc", Json::str(joined(&c.doc)))
                        .with("params", params_json(ix, &c.params, true))
                        .with("returns", Json::str(c.returns.to_string())),
                ),
                Item::Const(c) => consts.push(
                    Json::obj()
                        .with("name", Json::str(c.name.clone()))
                        .with("type", Json::str(c.ty.to_string()))
                        .with(
                            "value",
                            default_json(ix, &c.ty, Some(std::slice::from_ref(&c.value)))
                                .unwrap_or(Json::Null),
                        )
                        .with("doc", Json::str(joined(&c.doc))),
                ),
                Item::Column(c) => {
                    types.push(c.name.clone());
                }
                Item::Value(v) => values.push(
                    Json::obj()
                        .with("name", Json::str(v.name.clone()))
                        .with("type", Json::str(v.ty.to_string()))
                        .with("doc", Json::str(joined(&v.doc))),
                ),
                other => {
                    types.push(other.name().to_string());
                    if let Some(def) = def_json(ix, m, other) {
                        defs.set(other.name(), def);
                    }
                }
            }
        }
        catalog.set("types", strings(types.iter().map(String::as_str)));
        catalog.set("consts", Json::Arr(consts));
        if !values.is_empty() {
            catalog.set("values", Json::Arr(values));
        }
        catalog.set("functions", Json::Arr(fns));
        catalog.set("interfaces", Json::Arr(interfaces));
        catalog.set("callbacks", Json::Arr(callbacks));
        modules.push(catalog);
    }
    root.set("x-og-modules", Json::Arr(modules));
    root.set("$defs", defs);
    root.render()
}

fn function_json(ix: &Index<'_>, f: &Function, publish: bool) -> Json {
    let mut o = Json::obj()
        .with("name", Json::str(f.name.clone()))
        .with("doc", Json::str(joined(&f.doc)));
    if !f.generics.is_empty() {
        o.set(
            "generics",
            Json::Arr(
                f.generics
                    .iter()
                    .map(|g| {
                        Json::obj()
                            .with("name", Json::str(g.name.clone()))
                            .with("bound", Json::str(g.bound.to_string()))
                    })
                    .collect(),
            ),
        );
    }
    o.set("params", params_json(ix, &f.params, true));
    o.set("returns", Json::str(f.returns.to_string()));
    if !f.returns_doc.is_empty() {
        o.set("returnsDoc", Json::str(joined(&f.returns_doc)));
    }
    o.set("publish", Json::Bool(publish));
    if !f.host {
        o.set("host", Json::Bool(false));
    }
    o
}

fn def_json(ix: &Index<'_>, m: &Module, item: &Item) -> Option<Json> {
    let base = |kind: &str, doc: &[String]| {
        Json::obj()
            .with("x-og-kind", Json::str(kind))
            .with("x-og-module", Json::str(m.name.clone()))
            .with("description", Json::str(joined(doc)))
    };
    Some(match item {
        Item::Handle(h) => base("handle", &h.doc)
            .with("type", Json::str("integer"))
            .with("minimum", Json::num(0)),
        Item::Struct(s) => {
            let mut o = base(if s.desc { "desc" } else { "struct" }, &s.doc);
            object_body(ix, &mut o, &s.fields, !s.desc, None);
            o
        }
        Item::Record(r) => {
            let mut o = base(r.kind.as_str(), &r.doc);
            object_body(ix, &mut o, &r.fields, r.kind == RecordKind::Event, None);
            o
        }
        Item::Enum(e) => {
            let mut variants = Json::obj();
            for v in &e.variants {
                variants.set(&v.name, Json::str(joined(&v.doc)));
            }
            base("enum", &e.doc)
                .with("type", Json::str("string"))
                .with("enum", strings(e.variants.iter().map(|v| v.name.as_str())))
                .with("x-og-variants", variants)
        }
        Item::Union(u) => {
            let cases = u
                .cases
                .iter()
                .map(|c| {
                    let fields = c
                        .payload
                        .as_deref()
                        .and_then(|p| ix.structure(p))
                        .map(|s| s.fields.as_slice())
                        .unwrap_or(&[]);
                    let mut o = Json::obj().with("description", Json::str(joined(&c.doc)));
                    object_body(ix, &mut o, fields, true, Some((&u.tag, &c.name)));
                    if let Some(p) = &c.payload {
                        o.set("x-og-payload", Json::str(p.clone()));
                    }
                    o
                })
                .collect();
            base("union", &u.doc)
                .with("x-og-tag", Json::str(u.tag.clone()))
                .with("oneOf", Json::Arr(cases))
        }
        _ => return None,
    })
}

fn object_body(
    ix: &Index<'_>,
    o: &mut Json,
    fields: &[Field],
    required: bool,
    tag: Option<(&str, &str)>,
) {
    o.set("type", Json::str("object"));
    let mut props = Json::obj();
    let mut req = Vec::new();
    if let Some((t, v)) = tag {
        props.set(t, Json::obj().with("const", Json::str(v)));
        req.push(t.to_string());
    }
    for f in fields {
        let mut p = type_schema(ix, &f.ty);
        p.set("description", Json::str(joined(&f.doc)));
        if let Some(d) = default_json(ix, &f.ty, f.default.as_deref())
            && (f.default.is_some() || !required)
        {
            p.set("default", d);
        }
        if let Some(n) = f.min {
            p.set("minimum", Json::num(n));
        }
        if let Some(n) = f.max {
            p.set("maximum", Json::num(n));
        }
        if let Some(n) = f.step {
            p.set("x-og-step", Json::num(n));
        }
        if let Some(u) = &f.unit {
            p.set("x-og-unit", Json::str(u.clone()));
        }
        if let Some(l) = &f.label {
            p.set("x-og-label", Json::str(l.clone()));
        }
        if f.tunable {
            p.set("x-og-tunable", Json::Bool(true));
        }
        if f.readonly {
            p.set("readOnly", Json::Bool(true));
        }
        props.set(&f.name, p);
        if required && !f.optional {
            req.push(f.name.clone());
        }
    }
    o.set("properties", props);
    if !req.is_empty() {
        o.set("required", strings(req.iter().map(String::as_str)));
    }
    o.set("additionalProperties", Json::Bool(false));
}

fn type_schema(ix: &Index<'_>, ty: &TypeRef) -> Json {
    let name = ty.name();
    let args = ty.args();
    let t = |s: &str| Json::obj().with("type", Json::str(s));
    match name {
        "Void" => t("null"),
        "bool" => t("boolean"),
        "u32" => t("integer")
            .with("minimum", Json::num(0))
            .with("maximum", Json::num(u32::MAX)),
        "i32" => t("integer")
            .with("minimum", Json::num(i32::MIN))
            .with("maximum", Json::num(i32::MAX)),
        "f32" | "f64" => t("number"),
        "String" => t("string"),
        "Json" => Json::obj(),
        "Bytes" => t("string").with("contentEncoding", Json::str("base64")),
        "Float32Array" | "Float64Array" | "Uint32Array" | "Int32Array" | "Uint8Array" => {
            t("array").with("items", t("number"))
        }
        "List" => t("array").with("items", type_schema(ix, &args[0])),
        "Map" => t("object").with("additionalProperties", type_schema(ix, &args[0])),
        "Option" => Json::obj().with(
            "anyOf",
            Json::Arr(vec![type_schema(ix, &args[0]), t("null")]),
        ),
        "OneOf" => Json::obj().with(
            "anyOf",
            Json::Arr(vec![type_schema(ix, &args[0]), type_schema(ix, &args[1])]),
        ),
        "Array" => {
            let n = match args.get(1) {
                Some(TypeRef::Int(n)) => *n,
                _ => 0,
            };
            t("array")
                .with("items", type_schema(ix, &args[0]))
                .with("minItems", Json::num(n))
                .with("maxItems", Json::num(n))
        }
        _ => match ix.decl(name) {
            Some(TypeDecl::Interface(_) | TypeDecl::Callback(_)) | None => {
                Json::obj().with("x-og-type", Json::str(ty.to_string()))
            }
            Some(_) => Json::obj().with("$ref", Json::str(format!("#/$defs/{name}"))),
        },
    }
}
