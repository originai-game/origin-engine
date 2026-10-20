//! The IDL model: what `api.ogt` declares, after the `.ogt` tree is read.
//!
//! Every declaration keeps its source line so diagnostics name it.

use og_core::ogt::Value;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct Api {
    pub version: u32,
    pub package: String,
    pub doc: Vec<String>,
    pub units: Vec<Unit>,
    pub modules: Vec<Module>,
    /// The IDL text, one entry per line, so generators can quote a
    /// declaration exactly as written (the wiki). Empty when the model was
    /// not read from text.
    pub source: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub name: String,
    pub doc: Vec<String>,
    pub line: usize,
}

/// Where a module's functions appear in TypeScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// `og.<module>.<fn>` (the default).
    Namespace,
    /// `og.<fn>`: members of the root object itself.
    Root,
    /// Named exports of the package: `import { <fn> } from "@origingame/engine"`.
    Top,
}

impl Place {
    pub fn as_str(self) -> &'static str {
        match self {
            Place::Namespace => "namespace",
            Place::Root => "root",
            Place::Top => "top",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub doc: Vec<String>,
    pub place: Place,
    /// `false` drops the whole module from `publish` builds.
    pub publish: bool,
    /// The knowledge skill that teaches the module (`skill og-feel`: the
    /// engine repository's packages/knowledge/skills/og-feel); the wiki's
    /// index groups the modules by it and each page names it.
    pub skill: String,
    pub items: Vec<Item>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum Item {
    Fn(Function),
    Const(Const),
    Handle(Handle),
    Struct(Struct),
    Enum(Enum),
    Union(Union),
    Interface(Interface),
    Callback(Callback),
    Column(Column),
    Record(Record),
    Value(ValueDecl),
}

impl Item {
    pub fn name(&self) -> &str {
        match self {
            Item::Fn(x) => &x.name,
            Item::Const(x) => &x.name,
            Item::Handle(x) => &x.name,
            Item::Struct(x) => &x.name,
            Item::Enum(x) => &x.name,
            Item::Union(x) => &x.name,
            Item::Interface(x) => &x.name,
            Item::Callback(x) => &x.name,
            Item::Column(x) => &x.name,
            Item::Record(x) => &x.name,
            Item::Value(x) => &x.name,
        }
    }

    pub fn line(&self) -> usize {
        match self {
            Item::Fn(x) => x.line,
            Item::Const(x) => x.line,
            Item::Handle(x) => x.line,
            Item::Struct(x) => x.line,
            Item::Enum(x) => x.line,
            Item::Union(x) => x.line,
            Item::Interface(x) => x.line,
            Item::Callback(x) => x.line,
            Item::Column(x) => x.line,
            Item::Record(x) => x.line,
            Item::Value(x) => x.line,
        }
    }
}

/// A type expression: `Name` or `Name<Arg, ...>`; an argument may be an integer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    Named { name: String, args: Vec<TypeRef> },
    Int(u32),
}

impl TypeRef {
    pub fn named(name: &str) -> TypeRef {
        TypeRef::Named {
            name: name.to_string(),
            args: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            TypeRef::Named { name, .. } => name,
            TypeRef::Int(_) => "",
        }
    }

    pub fn args(&self) -> &[TypeRef] {
        match self {
            TypeRef::Named { args, .. } => args,
            TypeRef::Int(_) => &[],
        }
    }

    /// Parse the IDL's dotted prefix form: `Map.List.Option.Entity` is
    /// `Map<List<Option<Entity>>>`, `Array.f32.16` is `Array<f32,16>`.
    /// Built-in generics have fixed arities, so the form is unambiguous;
    /// every other name takes no arguments.
    pub fn parse(text: &str) -> Option<TypeRef> {
        let mut parts = text.split('.');
        let ty = Self::parse_parts(&mut parts)?;
        parts.next().is_none().then_some(ty)
    }

    fn parse_parts<'t>(parts: &mut impl Iterator<Item = &'t str>) -> Option<TypeRef> {
        let head = parts.next()?;
        if head.is_empty() {
            return None;
        }
        if head.bytes().all(|b| b.is_ascii_digit()) {
            return Some(TypeRef::Int(head.parse().ok()?));
        }
        if !head.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return None;
        }
        let arity = crate::index::GENERICS
            .iter()
            .find(|(n, _)| *n == head)
            .map_or(0, |(_, a)| *a);
        let mut args = Vec::with_capacity(arity);
        for _ in 0..arity {
            args.push(Self::parse_parts(parts)?);
        }
        Some(TypeRef::Named {
            name: head.to_string(),
            args,
        })
    }
}

impl fmt::Display for TypeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeRef::Int(n) => write!(f, "{n}"),
            TypeRef::Named { name, args } => {
                f.write_str(name)?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    for (i, a) in args.iter().enumerate() {
                        if i > 0 {
                            f.write_str(",")?;
                        }
                        write!(f, "{a}")?;
                    }
                    f.write_str(">")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Generic {
    pub name: String,
    pub bound: TypeRef,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: TypeRef,
    pub doc: Vec<String>,
    pub default: Option<Vec<Value>>,
    pub optional: bool,
    pub flatten: bool,
    pub line: usize,
}

impl Param {
    /// Declared optional or given a default.
    pub fn declared_optional(&self) -> bool {
        self.optional || self.default.is_some()
    }
}

/// Which parameters a caller may omit: the declared-optional ones, and a desc
/// struct when everything after it may be omitted too (a desc in the middle
/// is passed, even if as `{}`).
pub fn omittable(params: &[Param], ix: &crate::Index<'_>) -> Vec<bool> {
    let mut out = vec![false; params.len()];
    let mut tail = true;
    for (i, p) in params.iter().enumerate().rev() {
        let o = p.declared_optional() || (tail && ix.is_desc(&p.ty));
        out[i] = o;
        tail = tail && o;
    }
    out
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub doc: Vec<String>,
    pub generics: Vec<Generic>,
    pub params: Vec<Param>,
    pub returns: TypeRef,
    pub returns_doc: Vec<String>,
    /// `None` inherits the container's publish flag.
    pub publish: Option<bool>,
    /// `false`: implemented by the TypeScript runtime over other members
    /// (sugar such as `og.ui.panel`), so it has no binding symbol and no
    /// host trait method.
    pub host: bool,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Const {
    pub name: String,
    pub ty: TypeRef,
    pub value: Value,
    pub doc: Vec<String>,
    pub line: usize,
}

/// A value the TypeScript runtime provides under the module's path
/// (`og.systems.cameras`): data or tokens, never a host call.
#[derive(Debug, Clone)]
pub struct ValueDecl {
    pub name: String,
    pub ty: TypeRef,
    pub doc: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Handle {
    pub name: String,
    pub doc: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: TypeRef,
    pub doc: Vec<String>,
    pub default: Option<Vec<Value>>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub unit: Option<String>,
    pub label: Option<String>,
    pub tunable: bool,
    pub readonly: bool,
    pub optional: bool,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub doc: Vec<String>,
    /// A desc struct: every field may be omitted and takes its default.
    pub desc: bool,
    pub fields: Vec<Field>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub doc: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub doc: Vec<String>,
    pub variants: Vec<Variant>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub name: String,
    pub payload: Option<String>,
    pub doc: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Union {
    pub name: String,
    pub doc: Vec<String>,
    pub tag: String,
    pub cases: Vec<Case>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Prop {
    pub name: String,
    pub ty: TypeRef,
    pub doc: Vec<String>,
    pub publish: Option<bool>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum Member {
    Prop(Prop),
    Fn(Function),
}

impl Member {
    pub fn name(&self) -> &str {
        match self {
            Member::Prop(p) => &p.name,
            Member::Fn(f) => &f.name,
        }
    }
    pub fn line(&self) -> usize {
        match self {
            Member::Prop(p) => p.line,
            Member::Fn(f) => f.line,
        }
    }
    pub fn publish(&self) -> Option<bool> {
        match self {
            Member::Prop(p) => p.publish,
            Member::Fn(f) => f.publish,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Interface {
    pub name: String,
    pub doc: Vec<String>,
    pub extends: Option<String>,
    pub publish: Option<bool>,
    pub members: Vec<Member>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Callback {
    pub name: String,
    pub doc: Vec<String>,
    pub params: Vec<Param>,
    pub returns: TypeRef,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub doc: Vec<String>,
    pub array: String,
    pub stride: u32,
    /// The factory takes a byte capacity first, and the stride is that
    /// capacity (`str(32)`: a `Uint8Array` of stride 32 holding text).
    pub capacity: bool,
    pub value: TypeRef,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    Component,
    Resource,
    Event,
}

impl RecordKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RecordKind::Component => "component",
            RecordKind::Resource => "resource",
            RecordKind::Event => "event",
        }
    }
}

/// A component, resource or event: a reflected record registered by name.
#[derive(Debug, Clone)]
pub struct Record {
    pub kind: RecordKind,
    pub name: String,
    pub doc: Vec<String>,
    pub fields: Vec<Field>,
    /// Resources only: methods on the resource value.
    pub methods: Vec<Function>,
    pub line: usize,
}
