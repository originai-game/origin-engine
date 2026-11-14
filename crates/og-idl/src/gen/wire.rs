//! The runtime's typed wire readers (`crates/og-runtime/src/host/generated/wire.rs`):
//! one Rust type per struct, enum and union a binding takes as JSON (or, for
//! enums, as a string), with its IDL defaults and a reader over
//! `serde_json::Value`, so a field name, a type or a default the runtime
//! reads is the IDL's and a mismatch is a build error. `record` holds the
//! rows of the engine's components, resources and events the same way, as
//! their layouts decode them (og-runtime's `codec::Layout::read`), readers
//! only: the runtime writes a row through its layout.
//!
//! The readers implement og-runtime's `wire::FromJson`. Their rules, the
//! same for every type: an absent field takes its default, and so does a
//! `null` one, except a float field (`f32`, `f64`, the float members of
//! vectors, quaternions and colours), where `null` reads as NaN, the form
//! JSON.stringify gives a JavaScript NaN; a
//! present field of the wrong JSON type, an integer out of range or not
//! whole, or an unknown enum variant or union tag is refused, naming the
//! field's path; fields the IDL does not declare are ignored.
//!
//! The math values a column stores map to glam (`Vector3` is `Vec3`,
//! `Quaternion` `Quat`, `Matrix4` `Mat4`); handles are `u64`, interface
//! objects and callbacks `u32`; values the IDL leaves open (`Json`,
//! `Value<T>`, `Partial<T>`, document props, tokens, results) stay
//! `serde_json::Value`.

use super::{GENERATED_BY, bindings, snake};
use crate::index::{Index, TypeDecl};
use crate::json::Json;
use crate::model::*;
use std::collections::BTreeSet;
use std::fmt::Write;

/// IDL math structs with a glam type of the same layout.
const GLAM: &[(&str, &str)] = &[
    ("Vector2", "Vec2"),
    ("Vector3", "Vec3"),
    ("Vector4", "Vec4"),
    ("Quaternion", "Quat"),
    ("Matrix4", "Mat4"),
];

/// Rust's strict and reserved keywords a field name may collide with.
const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
    "where", "while", "abstract", "become", "box", "do", "final", "gen", "macro", "override",
    "priv", "try", "typeof", "unsized", "virtual", "yield",
];

/// A field's Rust name: snake case, raw when it is a keyword.
pub fn field_name(name: &str) -> String {
    let s = snake(name);
    if KEYWORDS.contains(&s.as_str()) {
        format!("r#{s}")
    } else {
        s
    }
}

/// A variant's or case's Rust name: `orientedBox` is `OrientedBox`.
pub fn variant_name(name: &str) -> String {
    let mut out = String::new();
    let mut up = true;
    for c in name.chars() {
        if !c.is_ascii_alphanumeric() {
            up = true;
            continue;
        }
        if up {
            out.push(c.to_ascii_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'V');
    }
    out
}

fn glam(name: &str) -> Option<&'static str> {
    GLAM.iter().find(|(n, _)| *n == name).map(|(_, g)| *g)
}

/// An enum with only fixed variants becomes a Rust enum; one with template
/// patterns stays a string the runtime matches.
fn is_closed(e: &Enum) -> bool {
    e.patterns.is_empty()
}

struct Gen<'a, 'i> {
    ix: &'i Index<'a>,
    /// Types to emit, by IDL name.
    wanted: BTreeSet<String>,
}

impl<'a> Gen<'a, '_> {
    /// Collect `ty` and every struct, enum and union it reaches.
    fn want(&mut self, f: Option<&Function>, ty: &TypeRef) {
        let name = ty.name();
        if f.is_some_and(|f| f.generics.iter().any(|g| g.name == name)) {
            return;
        }
        for a in ty.args() {
            if matches!(
                name,
                "Value" | "Partial" | "ScreenProps" | "ScreenValues" | "ComponentProps"
            ) {
                break;
            }
            self.want(f, a);
        }
        if glam(name).is_some() || self.wanted.contains(name) {
            return;
        }
        match self.ix.decl(name) {
            Some(TypeDecl::Struct(s)) => {
                self.wanted.insert(name.to_string());
                for field in &s.fields {
                    self.want(None, &field.ty);
                }
            }
            Some(TypeDecl::Enum(_)) => {
                self.wanted.insert(name.to_string());
            }
            Some(TypeDecl::Union(u)) => {
                self.wanted.insert(name.to_string());
                for c in &u.cases {
                    if let Some(p) = &c.payload {
                        self.want(None, &TypeRef::named(p));
                    }
                }
            }
            _ => {}
        }
    }

    /// The Rust type of `ty` (in the signature of `f`, whose generics are
    /// open values).
    fn rust(&self, f: Option<&Function>, ty: &TypeRef) -> String {
        let name = ty.name();
        if f.is_some_and(|f| f.generics.iter().any(|g| g.name == name)) {
            return "Value".into();
        }
        let args = ty.args();
        match name {
            "bool" | "i32" | "u32" | "f32" | "f64" | "String" => name.into(),
            "Float32Array" => "Vec<f32>".into(),
            "Float64Array" => "Vec<f64>".into(),
            "Uint32Array" => "Vec<u32>".into(),
            "Int32Array" => "Vec<i32>".into(),
            "Uint8Array" => "Vec<u8>".into(),
            "Void" | "This" => "()".into(),
            "List" => format!("Vec<{}>", self.rust(f, &args[0])),
            "Map" => format!("BTreeMap<String, {}>", self.rust(f, &args[0])),
            "Option" => format!("Option<{}>", self.rust(f, &args[0])),
            "OneOf" => format!(
                "OneOf<{}, {}>",
                self.rust(f, &args[0]),
                self.rust(f, &args[1])
            ),
            "Array" => match &args[1] {
                TypeRef::Int(n) => format!("[{}; {n}]", self.rust(f, &args[0])),
                _ => "Value".into(),
            },
            _ if crate::index::is_builtin(name) => "Value".into(),
            _ => {
                if let Some(g) = glam(name) {
                    return g.into();
                }
                match self.ix.decl(name) {
                    Some(TypeDecl::Handle(_)) => "u64".into(),
                    Some(TypeDecl::Interface(_) | TypeDecl::Callback(_)) => "u32".into(),
                    Some(TypeDecl::Enum(e)) if !is_closed(e) => "String".into(),
                    Some(TypeDecl::Struct(_) | TypeDecl::Enum(_) | TypeDecl::Union(_)) => {
                        name.into()
                    }
                    _ => "Value".into(),
                }
            }
        }
    }

    /// A Rust expression of type `rust(ty)` for the IDL default `value`.
    fn literal(&self, ty: &TypeRef, value: &Json) -> String {
        let name = ty.name();
        let args = ty.args();
        let num = |v: &Json| match v {
            Json::Num(n) => n.clone(),
            _ => "0".into(),
        };
        let float = |v: &Json, t: &str| {
            let n = num(v);
            match n.as_str() {
                "Infinity" | "inf" => format!("{t}::INFINITY"),
                "-Infinity" | "-inf" => format!("{t}::NEG_INFINITY"),
                _ if n.contains(['.', 'e', 'E']) => format!("{n}_{t}"),
                _ => format!("{n}.0_{t}"),
            }
        };
        let obj_num = |v: &Json, key: &str, t: &str| match v {
            Json::Obj(pairs) => pairs
                .iter()
                .find(|(k, _)| k == key)
                .map_or(format!("0.0_{t}"), |(_, v)| float(v, t)),
            _ => format!("0.0_{t}"),
        };
        match name {
            "bool" => matches!(value, Json::Bool(true)).to_string(),
            "i32" | "u32" => num(value),
            "f32" | "f64" => float(value, name),
            "String" => match value {
                Json::Str(s) => format!("String::from({s:?})"),
                _ => "String::new()".into(),
            },
            "Option" => match value {
                Json::Null => "None".into(),
                v => format!("Some({})", self.literal(&args[0], v)),
            },
            "List" => match value {
                Json::Arr(items) if !items.is_empty() => format!(
                    "vec![{}]",
                    items
                        .iter()
                        .map(|i| self.literal(&args[0], i))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                _ => "Vec::new()".into(),
            },
            "Array" => match value {
                Json::Arr(items) => format!(
                    "[{}]",
                    items
                        .iter()
                        .map(|i| self.literal(&args[0], i))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                _ => "Default::default()".into(),
            },
            "OneOf" => {
                let first_number = crate::index::is_number(args[0].name());
                let a = match value {
                    Json::Num(_) => first_number,
                    Json::Str(_) => args[0].name() == "String",
                    _ => true,
                };
                if a {
                    format!("OneOf::A({})", self.literal(&args[0], value))
                } else {
                    format!("OneOf::B({})", self.literal(&args[1], value))
                }
            }
            "Vector2" => format!(
                "Vec2::new({}, {})",
                obj_num(value, "x", "f32"),
                obj_num(value, "y", "f32")
            ),
            "Vector3" => format!(
                "Vec3::new({}, {}, {})",
                obj_num(value, "x", "f32"),
                obj_num(value, "y", "f32"),
                obj_num(value, "z", "f32")
            ),
            "Vector4" => format!(
                "Vec4::new({}, {}, {}, {})",
                obj_num(value, "x", "f32"),
                obj_num(value, "y", "f32"),
                obj_num(value, "z", "f32"),
                obj_num(value, "w", "f32")
            ),
            "Quaternion" => format!(
                "Quat::from_xyzw({}, {}, {}, {})",
                obj_num(value, "x", "f32"),
                obj_num(value, "y", "f32"),
                obj_num(value, "z", "f32"),
                obj_num(value, "w", "f32")
            ),
            "Matrix4" => match value {
                Json::Obj(pairs) => match pairs.iter().find(|(k, _)| k == "elements") {
                    Some((_, Json::Arr(e))) => format!(
                        "Mat4::from_cols_array(&[{}])",
                        e.iter()
                            .map(|v| float(v, "f32"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    _ => "Mat4::IDENTITY".into(),
                },
                _ => "Mat4::IDENTITY".into(),
            },
            _ if name == "Json" || crate::index::is_builtin(name) => match value {
                Json::Null => "Value::Null".into(),
                v => format!("serde_json::json!({})", v.compact()),
            },
            _ => match self.ix.decl(name) {
                Some(TypeDecl::Handle(_)) => num(value),
                Some(TypeDecl::Enum(e)) => match value {
                    Json::Str(s) if is_closed(e) => format!("{name}::{}", variant_name(s)),
                    Json::Str(s) => format!("String::from({s:?})"),
                    _ => "Default::default()".into(),
                },
                Some(TypeDecl::Struct(s)) => match value {
                    Json::Obj(pairs) if !pairs.is_empty() => {
                        let mut parts = Vec::new();
                        for (k, v) in pairs {
                            if let Some(f) = s.fields.iter().find(|f| &f.name == k) {
                                let lit = if f.optional && !matches!(v, Json::Null) {
                                    format!("Some({})", self.literal(&f.ty, v))
                                } else if f.optional {
                                    "None".into()
                                } else {
                                    self.literal(&f.ty, v)
                                };
                                parts.push(format!("{}: {lit}", field_name(&f.name)));
                            }
                        }
                        let rest = if parts.len() < s.fields.len() {
                            ", ..Default::default()"
                        } else {
                            ""
                        };
                        format!("{name} {{ {}{rest} }}", parts.join(", "))
                    }
                    _ => "Default::default()".into(),
                },
                _ => "Default::default()".into(),
            },
        }
    }

    fn field_default(&self, f: &Field) -> String {
        match &f.default {
            Some(values) => {
                let json = super::default_json(self.ix, &f.ty, Some(values)).unwrap_or(Json::Null);
                if f.optional && !matches!(json, Json::Null) {
                    format!("Some({})", self.literal(&f.ty, &json))
                } else if f.optional {
                    "None".into()
                } else {
                    self.literal(&f.ty, &json)
                }
            }
            None => "Default::default()".into(),
        }
    }

    fn field_ty(&self, f: &Field) -> String {
        let t = self.rust(None, &f.ty);
        if f.optional && f.ty.name() != "Option" {
            format!("Option<{t}>")
        } else {
            t
        }
    }

    fn structure(&self, out: &mut String, module: &str, s: &Struct) {
        self.fields_type(out, module, &s.name, &s.fields, true);
    }

    /// A struct of `fields` named `name`: its type, its IDL defaults, its
    /// reader and, with `writer`, its writer.
    fn fields_type(
        &self,
        out: &mut String,
        module: &str,
        name: &str,
        fields: &[Field],
        writer: bool,
    ) {
        let _ = writeln!(out, "/// `{}`: packages/knowledge/wiki/{module}.md.", name);
        let _ = writeln!(out, "#[derive(Clone, Debug, PartialEq)]");
        let _ = writeln!(out, "pub struct {} {{", name);
        for f in fields {
            let _ = writeln!(
                out,
                "    pub {}: {},",
                field_name(&f.name),
                self.field_ty(f)
            );
        }
        let _ = writeln!(out, "}}\n");
        let _ = writeln!(out, "impl Default for {} {{", name);
        let _ = writeln!(out, "    fn default() -> Self {{");
        if fields.is_empty() {
            let _ = writeln!(out, "        Self {{}}");
        } else {
            let _ = writeln!(out, "        Self {{");
            for f in fields {
                let _ = writeln!(
                    out,
                    "            {}: {},",
                    field_name(&f.name),
                    self.field_default(f)
                );
            }
            let _ = writeln!(out, "        }}");
        }
        let _ = writeln!(out, "    }}\n}}\n");
        let _ = writeln!(out, "impl FromJson for {} {{", name);
        let _ = writeln!(
            out,
            "    fn from_json(v: &Value) -> Result<Self, String> {{"
        );
        if fields.is_empty() {
            let _ = writeln!(out, "        object(v)?;");
            let _ = writeln!(out, "        Ok(Self {{}})");
        } else {
            let _ = writeln!(out, "        Self::from_json_over(v, Self::default())");
        }
        let _ = writeln!(out, "    }}");
        if !fields.is_empty() {
            let _ = writeln!(
                out,
                "    fn from_json_over(v: &Value, d: Self) -> Result<Self, String> {{"
            );
            let _ = writeln!(out, "        let o = object(v)?;");
            let _ = writeln!(out, "        Ok(Self {{");
            for f in fields {
                let n = field_name(&f.name);
                let _ = writeln!(out, "            {n}: field(o, {:?}, d.{n})?,", f.name);
            }
            let _ = writeln!(out, "        }})\n    }}");
        }
        let _ = writeln!(out, "}}\n");
        if !writer {
            return;
        }
        let _ = writeln!(out, "impl ToJson for {} {{", name);
        let _ = writeln!(out, "    fn to_json(&self) -> Value {{");
        if fields.is_empty() {
            let _ = writeln!(out, "        Value::Object(Map::new())\n    }}\n}}\n");
            return;
        }
        let _ = writeln!(out, "        let mut o = Map::new();");
        for f in fields {
            let n = field_name(&f.name);
            if f.optional && f.ty.name() != "Option" {
                let _ = writeln!(
                    out,
                    "        if let Some(x) = &self.{n} {{\n            o.insert({:?}.into(), x.to_json());\n        }}",
                    f.name
                );
            } else {
                let _ = writeln!(
                    out,
                    "        o.insert({:?}.into(), self.{n}.to_json());",
                    f.name
                );
            }
        }
        let _ = writeln!(out, "        Value::Object(o)\n    }}\n}}\n");
    }

    fn enumeration(&self, out: &mut String, module: &str, e: &Enum) {
        let _ = writeln!(
            out,
            "/// `{}`: packages/knowledge/wiki/{module}.md.",
            e.name
        );
        let _ = writeln!(
            out,
            "#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]"
        );
        let _ = writeln!(out, "pub enum {} {{", e.name);
        for (i, v) in e.variants.iter().enumerate() {
            if i == 0 {
                let _ = writeln!(out, "    #[default]");
            }
            let _ = writeln!(out, "    {},", variant_name(&v.name));
        }
        let _ = writeln!(out, "}}\n");
        let names: Vec<String> = e.variants.iter().map(|v| format!("{:?}", v.name)).collect();
        let _ = writeln!(out, "impl {} {{", e.name);
        let _ = writeln!(out, "    /// The variants' IDL names, in index order.");
        let _ = writeln!(
            out,
            "    pub const NAMES: &'static [&'static str] = &[{}];",
            names.join(", ")
        );
        let _ = writeln!(out, "    /// Every variant, in index order.");
        let _ = writeln!(
            out,
            "    pub const ALL: &'static [Self] = &[{}];",
            e.variants
                .iter()
                .map(|v| format!("Self::{}", variant_name(&v.name)))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let _ = writeln!(out, "    /// The IDL name.");
        let _ = writeln!(
            out,
            "    pub fn as_str(self) -> &'static str {{\n        Self::NAMES[self as usize]\n    }}"
        );
        let _ = writeln!(
            out,
            "    /// The variant named `s`, or a refusal naming them all."
        );
        let _ = writeln!(
            out,
            "    pub fn parse(s: &str) -> Result<Self, String> {{\n        match Self::NAMES.iter().position(|n| *n == s) {{\n            Some(i) => Ok(Self::ALL[i]),\n            None => Err(unknown_variant({:?}, s, Self::NAMES)),\n        }}\n    }}",
            e.name
        );
        let _ = writeln!(out, "}}\n");
        let _ = writeln!(out, "impl FromJson for {} {{", e.name);
        let _ = writeln!(
            out,
            "    fn from_json(v: &Value) -> Result<Self, String> {{\n        Self::parse(variant(v, {:?}, Self::NAMES)?)\n    }}\n}}\n",
            e.name
        );
        let _ = writeln!(
            out,
            "impl ToJson for {} {{\n    fn to_json(&self) -> Value {{\n        Value::String(self.as_str().into())\n    }}\n}}\n",
            e.name
        );
    }

    fn union(&self, out: &mut String, module: &str, u: &Union) {
        let _ = writeln!(
            out,
            "/// `{}`: packages/knowledge/wiki/{module}.md.",
            u.name
        );
        let _ = writeln!(out, "#[derive(Clone, Debug, PartialEq)]");
        let _ = writeln!(out, "pub enum {} {{", u.name);
        for c in &u.cases {
            match &c.payload {
                Some(p) => {
                    let _ = writeln!(out, "    {}({p}),", variant_name(&c.name));
                }
                None => {
                    let _ = writeln!(out, "    {},", variant_name(&c.name));
                }
            }
        }
        let _ = writeln!(out, "}}\n");
        let first = &u.cases[0];
        let _ = writeln!(out, "impl Default for {} {{", u.name);
        let _ = writeln!(
            out,
            "    fn default() -> Self {{\n        Self::{}{}\n    }}\n}}\n",
            variant_name(&first.name),
            if first.payload.is_some() {
                "(Default::default())"
            } else {
                ""
            }
        );
        let tags: Vec<String> = u.cases.iter().map(|c| format!("{:?}", c.name)).collect();
        let _ = writeln!(out, "impl {} {{", u.name);
        let _ = writeln!(out, "    /// The tag key and the cases' tags, in order.");
        let _ = writeln!(out, "    pub const TAG: &'static str = {:?};", u.tag);
        let _ = writeln!(
            out,
            "    pub const TAGS: &'static [&'static str] = &[{}];",
            tags.join(", ")
        );
        let _ = writeln!(out, "    /// This case's tag.");
        let _ = writeln!(out, "    pub fn tag(&self) -> &'static str {{");
        let _ = writeln!(out, "        match self {{");
        for c in &u.cases {
            let pat = if c.payload.is_some() { "(_)" } else { "" };
            let _ = writeln!(
                out,
                "            Self::{}{pat} => {:?},",
                variant_name(&c.name),
                c.name
            );
        }
        let _ = writeln!(out, "        }}\n    }}\n}}\n");
        let _ = writeln!(out, "impl FromJson for {} {{", u.name);
        let _ = writeln!(
            out,
            "    fn from_json(v: &Value) -> Result<Self, String> {{"
        );
        let _ = writeln!(out, "        let o = object(v)?;");
        let _ = writeln!(
            out,
            "        let tag = match o.get(Self::TAG) {{\n            Some(t) => text(t).map_err(|e| at(Self::TAG, e))?,\n            None => return Err(at(Self::TAG, format!(\"missing: one of {{}}\", Self::TAGS.join(\", \")))),\n        }};"
        );
        let _ = writeln!(out, "        Ok(match tag {{");
        for c in &u.cases {
            match &c.payload {
                Some(p) => {
                    let _ = writeln!(
                        out,
                        "            {:?} => Self::{}({p}::from_json(v)?),",
                        c.name,
                        variant_name(&c.name)
                    );
                }
                None => {
                    let _ = writeln!(
                        out,
                        "            {:?} => Self::{},",
                        c.name,
                        variant_name(&c.name)
                    );
                }
            }
        }
        let _ = writeln!(
            out,
            "            t => return Err(at(Self::TAG, unknown_variant({:?}, t, Self::TAGS))),",
            u.name
        );
        let _ = writeln!(out, "        }})\n    }}\n}}\n");
        let _ = writeln!(out, "impl ToJson for {} {{", u.name);
        let _ = writeln!(out, "    fn to_json(&self) -> Value {{");
        let _ = writeln!(out, "        let mut v = match self {{");
        for c in &u.cases {
            let name = variant_name(&c.name);
            if c.payload.is_some() {
                let _ = writeln!(out, "            Self::{name}(p) => p.to_json(),");
            } else {
                let _ = writeln!(
                    out,
                    "            Self::{name} => Value::Object(Map::new()),"
                );
            }
        }
        let _ = writeln!(out, "        }};");
        let _ = writeln!(
            out,
            "        if let Value::Object(o) = &mut v {{\n            o.insert(Self::TAG.into(), Value::String(self.tag().into()));\n        }}\n        v\n    }}\n}}\n"
        );
    }
}

/// `crates/og-runtime/src/host/generated/wire.rs`.
pub fn wire_rs(ix: &Index<'_>) -> String {
    let mut g = Gen {
        ix,
        wanted: BTreeSet::new(),
    };
    for b in bindings(ix) {
        if let Some(f) = b.function {
            for p in &f.params {
                g.want(Some(f), &p.ty);
            }
            g.want(Some(f), &f.returns);
        }
        if let Some(p) = b.prop {
            g.want(None, &p.ty);
        }
    }
    let mut out = String::new();
    let _ = writeln!(out, "// {GENERATED_BY}");
    let _ = writeln!(
        out,
        "//! The typed wire types, generated from the IDL: every struct, enum and\n//! union a binding takes or answers, with the IDL's defaults, a reader\n//! (`FromJson`, src/wire.rs) that refuses a value of the wrong type and a\n//! writer (`ToJson`) of the IDL's fields. See\n//! og-idl's `gen/wire.rs` for the rules. The IDL's prose is not repeated:\n//! each type names its wiki page.\n"
    );
    let _ = writeln!(
        out,
        "#![allow(clippy::derivable_impls, clippy::approx_constant, clippy::excessive_precision, clippy::large_enum_variant, clippy::unreadable_literal, clippy::type_complexity)]\n// Which imports are used depends on what the IDL declares.\n#![allow(unused_imports)]\n"
    );
    let _ = writeln!(
        out,
        "use super::{{FromJson, OneOf, ToJson, at, field, object, text, unknown_variant, variant}};"
    );
    let _ = writeln!(out, "use glam::{{Mat4, Quat, Vec2, Vec3, Vec4}};");
    let _ = writeln!(out, "use serde_json::{{Map, Value}};");
    let _ = writeln!(out, "use std::collections::BTreeMap;\n");
    // The engine's records, whose rows the runtime reads decoded
    // (`codec::Layout::read`): their field types are wanted too.
    for module in &ix.api.modules {
        for item in &module.items {
            if let Item::Record(r) = item {
                for f in &r.fields {
                    g.want(None, &f.ty);
                }
            }
        }
    }
    for module in &ix.api.modules {
        for item in &module.items {
            if !g.wanted.contains(item.name()) {
                continue;
            }
            match item {
                Item::Struct(s) => g.structure(&mut out, &module.name, s),
                Item::Enum(e) if is_closed(e) => g.enumeration(&mut out, &module.name, e),
                Item::Union(u) => g.union(&mut out, &module.name, u),
                _ => {}
            }
        }
    }
    let _ = writeln!(
        out,
        "/// The rows of the engine's records (IDL `component`, `resource` and\n/// `event`), as their layouts decode them: every field, with its IDL\n/// default, read through the same rules as the wire types.\npub mod record {{\n    use super::*;\n"
    );
    let mut rows = String::new();
    for module in &ix.api.modules {
        for item in &module.items {
            if let Item::Record(r) = item {
                g.fields_type(&mut rows, &module.name, &r.name, &r.fields, false);
            }
        }
    }
    for line in rows.lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            let _ = writeln!(out, "    {line}");
        }
    }
    let _ = writeln!(out, "}}");
    out
}
