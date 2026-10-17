//! Name lookup over a loaded API: every type, column, unit and const by name.

use crate::model::*;
use std::collections::BTreeMap;

/// Scalar types with no arguments.
pub const PRIMITIVES: &[&str] = &[
    "Void",
    "bool",
    "i32",
    "u32",
    "f32",
    "f64",
    "String",
    "Json",
    "Bytes",
    "Float32Array",
    "Float64Array",
    "Uint32Array",
    "Int32Array",
    "Uint8Array",
];

/// Built-in generic types and their arity.
pub const GENERICS: &[(&str, usize)] = &[
    ("List", 1),
    ("Map", 1),
    ("Option", 1),
    ("Result", 1),
    ("Partial", 1),
    ("Value", 1),
    ("View", 1),
    ("Array", 2),
    ("OneOf", 2),
    ("ComponentType", 1),
    ("ResourceType", 1),
];

/// Built-in meta types: the reflection tokens game code passes around.
pub const META: &[&str] = &[
    "Fields",
    "AnyComponent",
    "AnyResource",
    "AnyEvent",
    "AnyReflected",
];

/// The typed arrays a column may use, with the scalar each element holds.
pub const ARRAYS: &[(&str, &str)] = &[
    ("Float32Array", "f32"),
    ("Float64Array", "f64"),
    ("Uint32Array", "u32"),
    ("Int32Array", "i32"),
    ("Uint8Array", "bool"),
];

pub fn is_builtin(name: &str) -> bool {
    PRIMITIVES.contains(&name) || META.contains(&name) || GENERICS.iter().any(|(n, _)| *n == name)
}

pub fn is_number(name: &str) -> bool {
    matches!(name, "i32" | "u32" | "f32" | "f64")
}

#[derive(Debug, Clone, Copy)]
pub enum TypeDecl<'a> {
    Handle(&'a Handle),
    Struct(&'a Struct),
    Enum(&'a Enum),
    Union(&'a Union),
    Interface(&'a Interface),
    Callback(&'a Callback),
    Record(&'a Record),
}

impl TypeDecl<'_> {
    pub fn kind(&self) -> &'static str {
        match self {
            TypeDecl::Handle(_) => "handle",
            TypeDecl::Struct(_) => "struct",
            TypeDecl::Enum(_) => "enum",
            TypeDecl::Union(_) => "union",
            TypeDecl::Interface(_) => "interface",
            TypeDecl::Callback(_) => "callback",
            TypeDecl::Record(r) => r.kind.as_str(),
        }
    }
}

/// Lookup tables. The first declaration of a name wins; later ones are
/// recorded in `duplicates` for the validator.
pub struct Index<'a> {
    pub api: &'a Api,
    pub types: BTreeMap<&'a str, (&'a Module, TypeDecl<'a>)>,
    pub columns: BTreeMap<&'a str, &'a Column>,
    pub consts: BTreeMap<&'a str, &'a Const>,
    pub units: BTreeMap<&'a str, &'a Unit>,
    pub duplicates: Vec<(usize, String)>,
}

impl<'a> Index<'a> {
    pub fn new(api: &'a Api) -> Index<'a> {
        let mut ix = Index {
            api,
            types: BTreeMap::new(),
            columns: BTreeMap::new(),
            consts: BTreeMap::new(),
            units: BTreeMap::new(),
            duplicates: Vec::new(),
        };
        for unit in &api.units {
            if ix.units.insert(&unit.name, unit).is_some() {
                ix.duplicates
                    .push((unit.line, format!("duplicate unit `{}`", unit.name)));
            }
        }
        for module in &api.modules {
            for item in &module.items {
                let decl = match item {
                    Item::Handle(x) => TypeDecl::Handle(x),
                    Item::Struct(x) => TypeDecl::Struct(x),
                    Item::Enum(x) => TypeDecl::Enum(x),
                    Item::Union(x) => TypeDecl::Union(x),
                    Item::Interface(x) => TypeDecl::Interface(x),
                    Item::Callback(x) => TypeDecl::Callback(x),
                    Item::Record(x) => TypeDecl::Record(x),
                    Item::Column(c) => {
                        if ix.columns.insert(&c.name, c).is_some() {
                            ix.duplicates
                                .push((c.line, format!("duplicate column `{}`", c.name)));
                        }
                        continue;
                    }
                    Item::Const(c) => {
                        if ix.consts.insert(&c.name, c).is_some() {
                            ix.duplicates
                                .push((c.line, format!("duplicate const `{}`", c.name)));
                        }
                        continue;
                    }
                    Item::Fn(_) => continue,
                };
                let name = item.name();
                if is_builtin(name) {
                    ix.duplicates.push((
                        item.line(),
                        format!("duplicate name `{name}`: it is a built-in type"),
                    ));
                } else if let Some((m, prev)) = ix.types.get(name) {
                    ix.duplicates.push((
                        item.line(),
                        format!(
                            "duplicate type name `{name}`: already declared as a {} in module `{}`",
                            prev.kind(),
                            m.name
                        ),
                    ));
                } else {
                    ix.types.insert(name, (module, decl));
                }
            }
        }
        ix
    }

    pub fn decl(&self, name: &str) -> Option<TypeDecl<'a>> {
        self.types.get(name).map(|(_, d)| *d)
    }

    pub fn module_of(&self, name: &str) -> Option<&'a Module> {
        self.types.get(name).map(|(m, _)| *m)
    }

    pub fn structure(&self, name: &str) -> Option<&'a Struct> {
        match self.decl(name) {
            Some(TypeDecl::Struct(s)) => Some(s),
            _ => None,
        }
    }

    pub fn interface(&self, name: &str) -> Option<&'a Interface> {
        match self.decl(name) {
            Some(TypeDecl::Interface(i)) => Some(i),
            _ => None,
        }
    }

    pub fn is_desc(&self, ty: &TypeRef) -> bool {
        ty.args().is_empty() && self.structure(ty.name()).is_some_and(|s| s.desc)
    }

    /// Whether an omitted value of this type has an obvious meaning:
    /// `0`, `false`, `""`, `null`, an empty list or map, the first enum
    /// variant, the null handle, or a struct whose fields all have one.
    pub fn has_zero(&self, ty: &TypeRef) -> bool {
        let name = ty.name();
        if PRIMITIVES.contains(&name) {
            return name != "Void";
        }
        match name {
            "Option" | "List" | "Map" | "Partial" | "Array" => return true,
            _ => {}
        }
        match self.decl(name) {
            Some(TypeDecl::Enum(_)) | Some(TypeDecl::Handle(_)) => true,
            Some(TypeDecl::Struct(s)) => {
                s.desc
                    || s.fields
                        .iter()
                        .all(|f| f.default.is_some() || f.optional || self.has_zero(&f.ty))
            }
            Some(TypeDecl::Record(r)) => r
                .fields
                .iter()
                .all(|f| f.default.is_some() || self.has_zero(&f.ty)),
            _ => false,
        }
    }

    /// The scalars a value of this type flattens to, in field order, when it
    /// is plain numeric data: `Vector3` is `[f32, f32, f32]`, a handle is two
    /// `u32` (index, generation). `None` for anything with strings, lists or
    /// references.
    pub fn flat(&self, ty: &TypeRef) -> Option<Vec<&'static str>> {
        let name = ty.name();
        for scalar in ["f32", "f64", "u32", "i32", "bool"] {
            if name == scalar {
                return Some(vec![scalar]);
            }
        }
        if name == "Array" {
            let [inner, TypeRef::Int(n)] = ty.args() else {
                return None;
            };
            let one = self.flat(inner)?;
            return Some(one.repeat(*n as usize));
        }
        match self.decl(name)? {
            TypeDecl::Handle(_) => Some(vec!["u32", "u32"]),
            TypeDecl::Struct(s) if !s.desc => {
                let mut out = Vec::new();
                for f in &s.fields {
                    out.extend(self.flat(&f.ty)?);
                }
                Some(out)
            }
            _ => None,
        }
    }

    /// Structs used as a column's value: plain mutable math values in TypeScript.
    pub fn is_column_value(&self, name: &str) -> bool {
        self.columns.values().any(|c| c.value.name() == name)
    }

    /// Every member of an interface, including the ones it extends, base first.
    pub fn members(&self, iface: &'a Interface) -> Vec<(&'a Interface, &'a Member)> {
        let mut chain = vec![iface];
        let mut cur = iface;
        while let Some(base) = cur.extends.as_deref().and_then(|b| self.interface(b)) {
            if chain.iter().any(|c| std::ptr::eq(*c, base)) {
                break;
            }
            chain.push(base);
            cur = base;
        }
        chain
            .into_iter()
            .rev()
            .flat_map(|i| i.members.iter().map(move |m| (i, m)))
            .collect()
    }
}
