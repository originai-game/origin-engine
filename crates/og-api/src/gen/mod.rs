//! Generators from the validated model. Shared helpers live here.

pub mod tables;
pub mod ts;

use crate::index::{Index, TypeDecl};
use crate::json::Json;
use crate::model::*;
use crate::ogt::Value;

pub const GENERATED_BY: &str = "Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.";

/// `defineComponent` → `define_component`, `RuntimeProbe` → `runtime_probe`.
pub fn snake(name: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() {
            let prev_lower =
                i > 0 && (chars[i - 1].is_ascii_lowercase() || chars[i - 1].is_ascii_digit());
            let next_lower = chars.get(i + 1).is_some_and(|n| n.is_ascii_lowercase());
            if i > 0 && (prev_lower || (next_lower && chars[i - 1].is_ascii_uppercase())) {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(*c);
        }
    }
    out
}

/// A callable the runtime binds: a module function, an interface member or a
/// resource method.
pub struct Binding<'a> {
    pub kind: &'static str,
    pub module: &'a Module,
    /// The interface or resource, for members.
    pub owner: Option<&'a str>,
    pub name: &'a str,
    /// The TypeScript path: `og.load.gltf`, `defineComponent`, `World.spawn`.
    pub path: String,
    /// The host-side symbol: `og_load_gltf`, `og_world_spawn`.
    pub symbol: String,
    pub function: Option<&'a Function>,
    pub prop: Option<&'a Prop>,
    pub publish: bool,
}

pub fn bindings<'a>(ix: &Index<'a>) -> Vec<Binding<'a>> {
    let mut out = Vec::new();
    for module in &ix.api.modules {
        for item in &module.items {
            match item {
                Item::Fn(f) => out.push(Binding {
                    kind: "fn",
                    module,
                    owner: None,
                    name: &f.name,
                    path: match module.place {
                        Place::Top => f.name.clone(),
                        Place::Root => format!("og.{}", f.name),
                        Place::Namespace => format!("og.{}.{}", module.name, f.name),
                    },
                    symbol: format!("og_{}_{}", module.name, snake(&f.name)),
                    function: Some(f),
                    prop: None,
                    publish: f.publish.unwrap_or(module.publish),
                }),
                Item::Interface(i) => {
                    for m in &i.members {
                        let publish = m.publish().or(i.publish).unwrap_or(module.publish);
                        out.push(match m {
                            Member::Fn(f) => Binding {
                                kind: "method",
                                module,
                                owner: Some(&i.name),
                                name: &f.name,
                                path: format!("{}.{}", i.name, f.name),
                                symbol: format!("og_{}_{}", snake(&i.name), snake(&f.name)),
                                function: Some(f),
                                prop: None,
                                publish,
                            },
                            Member::Prop(p) => Binding {
                                kind: "prop",
                                module,
                                owner: Some(&i.name),
                                name: &p.name,
                                path: format!("{}.{}", i.name, p.name),
                                symbol: format!("og_{}_get_{}", snake(&i.name), snake(&p.name)),
                                function: None,
                                prop: Some(p),
                                publish,
                            },
                        });
                    }
                }
                Item::Record(r) => {
                    for f in &r.methods {
                        out.push(Binding {
                            kind: "method",
                            module,
                            owner: Some(&r.name),
                            name: &f.name,
                            path: format!("{}.{}", r.name, f.name),
                            symbol: format!("og_{}_{}", snake(&r.name), snake(&f.name)),
                            function: Some(f),
                            prop: None,
                            publish: f.publish.unwrap_or(module.publish),
                        });
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// The default of a value of `ty` as JSON: the explicit literal when given,
/// else the type's zero value. `None` when the type has no zero value.
pub fn default_json(ix: &Index<'_>, ty: &TypeRef, explicit: Option<&[Value]>) -> Option<Json> {
    if let Some(values) = explicit {
        return Some(literal_json(ix, ty, values));
    }
    let name = ty.name();
    Some(match name {
        "bool" => Json::Bool(false),
        "i32" | "u32" | "f32" | "f64" => Json::num(0),
        "String" => Json::str(""),
        "Option" | "Json" => Json::Null,
        "List" => Json::Arr(Vec::new()),
        "Map" | "Partial" => Json::obj(),
        "Array" => {
            let [inner, TypeRef::Int(n)] = ty.args() else {
                return None;
            };
            let one = default_json(ix, inner, None)?;
            Json::Arr(vec![one; *n as usize])
        }
        _ => match ix.decl(name)? {
            TypeDecl::Enum(e) => Json::str(e.variants.first()?.name.clone()),
            TypeDecl::Handle(_) => Json::num(0),
            TypeDecl::Struct(s) if s.desc => Json::obj(),
            TypeDecl::Struct(s) => {
                let mut obj = Json::obj();
                for f in &s.fields {
                    if f.optional {
                        continue;
                    }
                    obj.set(&f.name, default_json(ix, &f.ty, f.default.as_deref())?);
                }
                obj
            }
            _ => return None,
        },
    })
}

fn literal_json(ix: &Index<'_>, ty: &TypeRef, values: &[Value]) -> Json {
    if let [single] = values {
        match single {
            Value::Word(w) if w == "null" => {
                return if matches!(ix.decl(ty.name()), Some(TypeDecl::Handle(_))) {
                    Json::num(0)
                } else {
                    Json::Null
                };
            }
            Value::Word(w) => return Json::str(w.clone()),
            Value::Str(s) => return Json::str(s.clone()),
            Value::Bool(b) => return Json::Bool(*b),
            Value::Num(n)
                if ix.flat(ty).is_some_and(|f| f.len() == 1)
                    || ty.name() == "Option"
                    || ty.name() == "Json" =>
            {
                return Json::Num(n.trim_start_matches('+').to_string());
            }
            _ => {}
        }
    }
    // A flat numeric struct: rebuild it field by field.
    let mut cursor = values.iter();
    build_flat(ix, ty, &mut cursor)
}

fn build_flat<'v>(
    ix: &Index<'_>,
    ty: &TypeRef,
    values: &mut impl Iterator<Item = &'v Value>,
) -> Json {
    let name = ty.name();
    if name == "Option" {
        return build_flat(ix, &ty.args()[0], values);
    }
    if let Some(TypeDecl::Struct(s)) = ix.decl(name) {
        let mut obj = Json::obj();
        for f in &s.fields {
            obj.set(&f.name, build_flat(ix, &f.ty, values));
        }
        return obj;
    }
    if name == "Array" {
        if let [inner, TypeRef::Int(n)] = ty.args() {
            return Json::Arr((0..*n).map(|_| build_flat(ix, inner, values)).collect());
        }
    }
    match values.next() {
        Some(Value::Num(n)) => Json::Num(n.trim_start_matches('+').to_string()),
        Some(Value::Bool(b)) => Json::Bool(*b),
        Some(Value::Str(s)) => Json::str(s.clone()),
        _ => Json::Null,
    }
}

/// Doc lines joined for single-string sinks (JSON descriptions).
pub fn joined(doc: &[String]) -> String {
    doc.join("\n")
}

#[cfg(test)]
mod tests {
    #[test]
    fn snake_case_follows_word_boundaries() {
        assert_eq!(super::snake("defineComponent"), "define_component");
        assert_eq!(super::snake("RuntimeProbe"), "runtime_probe");
        assert_eq!(super::snake("worldToScreen"), "world_to_screen");
        assert_eq!(super::snake("GLTFInfo"), "gltf_info");
        assert_eq!(super::snake("f32"), "f32");
    }
}
