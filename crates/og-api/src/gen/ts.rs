//! The TypeScript declaration generator: `packages/engine/types/index.d.ts`.
//!
//! Mapping (README §TypeScript):
//! - top-placed module functions and column factories are named exports;
//!   root-placed functions, every component/resource/event token and every
//!   namespace-placed module live on the `og` namespace;
//! - structs, enums, unions, handles, interfaces, callbacks and record values
//!   are exported types named as in the IDL.

use super::{default_json, GENERATED_BY};
use crate::index::{Index, TypeDecl};
use crate::json::Json;
use crate::model::*;
use std::fmt::Write;

pub fn dts(ix: &Index<'_>) -> String {
    let mut g = Gen {
        ix,
        out: String::new(),
    };
    g.file();
    g.out
}

struct Gen<'a, 'b> {
    ix: &'b Index<'a>,
    out: String,
}

const PRELUDE: &str = r#"declare const valueType: unique symbol;
declare const handleType: unique symbol;

/**
 * Every failure in the API is a value, never a throw: check `ok` before
 * reading `value`. `reason` is a plain sentence that names the fix.
 */
export type Result<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly reason: string };

/** A column kind: what one field of a component or resource stores. */
export type ColumnKind = keyof ColumnKinds;

/**
 * One field of a reflected layout, made by a column factory
 * (`f32(100, { min: 0 })`, `Vector3()`). `default` is the value a new
 * component or resource starts with.
 */
export interface FieldSpec<K extends ColumnKind = ColumnKind> {
  readonly kind: K;
  readonly default: ColumnKinds[K]["value"];
  readonly options: FieldOptions;
}

/** A reflected layout: field names to field specs. */
export type Fields = { readonly [field: string]: FieldSpec };

/** The value of a layout: each field as its column's value type. */
export type FieldValues<F extends Fields> = {
  -readonly [K in keyof F]: ColumnKinds[F[K]["kind"]]["value"];
};

/** The typed views of a layout: each field as its column's typed array. */
export type ColumnViews<F extends Fields> = {
  readonly [K in keyof F]: ColumnKinds[F[K]["kind"]]["array"];
};

/** A registered component: the token systems, queries and the world take. */
export interface ComponentType<F extends Fields = Fields, V = FieldValues<F>> {
  readonly kind: "component";
  readonly name: string;
  readonly fields: F;
  readonly [valueType]: V;
}

/** A registered resource: a reflected singleton. */
export interface ResourceType<F extends Fields = Fields, V = FieldValues<F>> {
  readonly kind: "resource";
  readonly name: string;
  readonly fields: F;
  readonly [valueType]: V;
}

/** A registered event type. */
export interface EventType<V = unknown> {
  readonly kind: "event";
  readonly name: string;
  readonly [valueType]: V;
}

/** Any component token. */
export type AnyComponent = ComponentType<any, any>;
/** Any resource token. */
export type AnyResource = ResourceType<any, any>;
/** Any event token. */
export type AnyEvent = EventType<any>;
/** Any reflected token a system may declare: a component or a resource. */
export type AnyReflected = AnyComponent | AnyResource;

/** The value a token stands for: a component's fields, a resource object, an event payload. */
export type Value<T> = T extends { readonly [valueType]: infer V } ? V : never;

/** The typed column views of a component token, as `ArchetypeView.column` returns them. */
export type View<T> = T extends ComponentType<infer F, any> ? ColumnViews<F> : never;
"#;

impl<'a> Gen<'a, '_> {
    fn line(&mut self, s: &str) {
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn file(&mut self) {
        self.line(&format!("// {GENERATED_BY}"));
        let api = self.ix.api;
        self.line(&format!("// {} API v{}.", api.package, api.version));
        self.line("");
        for d in &api.doc {
            self.line(&format!("// {d}"));
        }
        self.line("");
        self.out.push_str(PRELUDE);
        self.line("");
        self.column_kinds();
        for module in &api.modules {
            self.module(module);
        }
        self.root();
    }

    fn column_kinds(&mut self) {
        self.line("/** Every column kind, from the IDL's `column` declarations. */");
        self.line("export interface ColumnKinds {");
        let columns: Vec<&Column> = self
            .ix
            .api
            .modules
            .iter()
            .flat_map(|m| m.items.iter())
            .filter_map(|i| match i {
                Item::Column(c) => Some(c),
                _ => None,
            })
            .collect();
        for c in columns {
            self.jsdoc("  ", &c.doc, &[]);
            let value = self.ty(&c.value, &[]);
            self.line(&format!(
                "  {}: {{ readonly array: {}; readonly stride: {}; readonly value: {} }};",
                c.name, c.array, c.stride, value
            ));
        }
        self.line("}");
        self.line("");
    }

    fn module(&mut self, m: &Module) {
        let bar = "─".repeat(60usize.saturating_sub(m.name.len()));
        self.line(&format!("// ── {} {bar}", m.name));
        for d in &m.doc {
            self.line(&format!("// {d}"));
        }
        self.line("");
        for item in &m.items {
            match item {
                Item::Fn(f) if m.place == Place::Top => {
                    self.function("export declare function ", f);
                }
                Item::Fn(_) => {}
                Item::Const(c) => {
                    self.jsdoc("", &c.doc, &[]);
                    let lit = match &c.value {
                        crate::ogt::Value::Str(s) => {
                            Json::str(s.clone()).render().trim_end().to_string()
                        }
                        v => v.to_string().trim_start_matches('+').to_string(),
                    };
                    self.line(&format!("export declare const {}: {lit};", c.name));
                }
                Item::Handle(h) => {
                    self.jsdoc("", &h.doc, &["A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.".to_string()]);
                    self.line(&format!(
                        "export type {} = number & {{ readonly [handleType]: \"{}\" }};",
                        h.name, h.name
                    ));
                }
                Item::Struct(s) => self.structure(s),
                Item::Enum(e) => {
                    let mut doc = e.doc.clone();
                    for v in &e.variants {
                        doc.push(format!("- `\"{}\"`: {}", v.name, v.doc.join(" ")));
                    }
                    self.jsdoc("", &doc, &[]);
                    let variants: Vec<String> = e
                        .variants
                        .iter()
                        .map(|v| format!("\"{}\"", v.name))
                        .collect();
                    self.line(&format!(
                        "export type {} = {};",
                        e.name,
                        variants.join(" | ")
                    ));
                }
                Item::Union(u) => {
                    self.jsdoc("", &u.doc, &[]);
                    self.line(&format!("export type {} =", u.name));
                    for (i, c) in u.cases.iter().enumerate() {
                        let end = if i + 1 == u.cases.len() { ";" } else { "" };
                        let tag = format!("{{ readonly {}: \"{}\" }}", u.tag, c.name);
                        let body = match &c.payload {
                            Some(p) => format!("({tag} & {p})"),
                            None => tag,
                        };
                        self.line(&format!("  /** {} */", escape(&c.doc.join(" "))));
                        self.line(&format!("  | {body}{end}"));
                    }
                }
                Item::Interface(i) => self.interface(i),
                Item::Callback(c) => {
                    let tags = self.param_tags(&c.params);
                    self.jsdoc("", &c.doc, &tags);
                    let params = self.params(&c.params, &[], false);
                    let ret = self.ty(&c.returns, &[]);
                    self.line(&format!("export type {} = ({params}) => {ret};", c.name));
                }
                Item::Column(c) => {
                    let value = self.ty(&c.value, &[]);
                    let tags = vec![
                        format!(
                            "A `{}` field: stored in a `{}` with stride {}.",
                            c.name, c.array, c.stride
                        ),
                        "@param value The field's default; omitted, the column's zero value."
                            .to_string(),
                        "@param options Range, unit, label, docs and the tunable flag.".to_string(),
                    ];
                    self.jsdoc("", &c.doc, &tags);
                    self.line(&format!(
                        "export declare function {}(value?: {value}, options?: FieldOptions): FieldSpec<\"{}\">;",
                        c.name, c.name
                    ));
                }
                Item::Record(r) => self.record_value(r),
            }
            self.line("");
        }
    }

    fn structure(&mut self, s: &Struct) {
        let mutable = self.ix.is_column_value(&s.name);
        let extra: Vec<String> = if s.desc {
            vec!["A desc: every field may be omitted and then takes its documented default; `{}` is valid.".to_string()]
        } else {
            vec![]
        };
        self.jsdoc("", &s.doc, &extra);
        self.line(&format!("export interface {} {{", s.name));
        for f in &s.fields {
            let tags = self.field_tags(f, s.desc);
            self.jsdoc("  ", &f.doc, &tags);
            let opt = if s.desc || f.optional { "?" } else { "" };
            let ro = if mutable { "" } else { "readonly " };
            let ty = self.ty(&f.ty, &[]);
            self.line(&format!("  {ro}{}{opt}: {ty};", f.name));
        }
        self.line("}");
    }

    fn record_value(&mut self, r: &Record) {
        let what = match r.kind {
            RecordKind::Component => format!(
                "The value of the `{}` component (token `og.{}`).",
                r.name, r.name
            ),
            RecordKind::Resource => format!("The `{}` resource (token `og.{}`).", r.name, r.name),
            RecordKind::Event => format!(
                "The payload of the `{}` event (token `og.{}`).",
                r.name, r.name
            ),
        };
        self.jsdoc("", &r.doc, &[what]);
        self.line(&format!("export interface {} {{", r.name));
        for f in &r.fields {
            let tags = self.field_tags(f, false);
            self.jsdoc("  ", &f.doc, &tags);
            let ro = if f.readonly || r.kind == RecordKind::Event {
                "readonly "
            } else {
                ""
            };
            let ty = self.ty(&f.ty, &[]);
            self.line(&format!("  {ro}{}: {ty};", f.name));
        }
        for m in &r.methods {
            self.function("  ", m);
        }
        self.line("}");
    }

    fn interface(&mut self, i: &Interface) {
        self.jsdoc("", &i.doc, &[]);
        let ext = i
            .extends
            .as_ref()
            .map(|b| format!(" extends {b}"))
            .unwrap_or_default();
        self.line(&format!("export interface {}{ext} {{", i.name));
        for m in &i.members {
            match m {
                Member::Prop(p) => {
                    self.jsdoc("  ", &p.doc, &[]);
                    let ty = self.ty(&p.ty, &[]);
                    self.line(&format!("  readonly {}: {ty};", p.name));
                }
                Member::Fn(f) => self.function("  ", f),
            }
        }
        self.line("}");
    }

    fn root(&mut self) {
        self.line("// ── og ──────────────────────────────────────────────────────────");
        self.line("");
        self.line("/**");
        self.line(" * The engine's root object: root functions (`og.game`), every engine");
        self.line(" * component, resource and event token (`og.Transform`, `og.Time`), and one");
        self.line(" * namespace per module (`og.load`, `og.commands`).");
        self.line(" */");
        self.line("export declare namespace og {");
        let api = self.ix.api;
        for m in &api.modules {
            for item in &m.items {
                match item {
                    Item::Fn(f) if m.place == Place::Root => self.function("  function ", f),
                    Item::Record(r) => self.token(r),
                    _ => {}
                }
            }
            let fns: Vec<&Function> = m
                .items
                .iter()
                .filter_map(|i| match i {
                    Item::Fn(f) => Some(f),
                    _ => None,
                })
                .collect();
            if m.place == Place::Namespace && !fns.is_empty() {
                self.jsdoc("  ", &m.doc, &[]);
                self.line(&format!("  namespace {} {{", m.name));
                for f in fns {
                    self.function("    function ", f);
                }
                self.line("  }");
            }
        }
        self.line("}");
    }

    fn token(&mut self, r: &Record) {
        self.jsdoc("  ", &r.doc, &[]);
        let fields: Vec<String> = r
            .fields
            .iter()
            .map(|f| format!("readonly {}: FieldSpec<\"{}\">", f.name, f.ty))
            .collect();
        let layout = if fields.is_empty() {
            "{}".to_string()
        } else {
            format!("{{ {} }}", fields.join("; "))
        };
        let ty = match r.kind {
            RecordKind::Component => format!("ComponentType<{layout}, {}>", r.name),
            RecordKind::Resource => format!("ResourceType<{layout}, {}>", r.name),
            RecordKind::Event => format!("EventType<{}>", r.name),
        };
        self.line(&format!("  const {}: {ty};", r.name));
    }

    /// `prefix name<G>(params): R;` with its JSDoc. `prefix` carries the
    /// indentation and the declaration keyword.
    fn function(&mut self, prefix: &str, f: &Function) {
        let indent: String = prefix.chars().take_while(|c| *c == ' ').collect();
        let mut tags = self.param_tags(&f.params);
        if !f.returns_doc.is_empty() {
            tags.push(format!("@returns {}", f.returns_doc.join(" ")));
        }
        self.jsdoc(&indent, &f.doc, &tags);
        let generics = if f.generics.is_empty() {
            String::new()
        } else {
            let gs: Vec<String> = f
                .generics
                .iter()
                .map(|g| format!("{} extends {}", g.name, g.bound.name()))
                .collect();
            format!("<{}>", gs.join(", "))
        };
        let params = self.params(&f.params, &f.generics, true);
        let ret = self.ty(&f.returns, &f.generics);
        self.line(&format!("{prefix}{}{generics}({params}): {ret};", f.name));
    }

    fn params(&self, params: &[Param], generics: &[Generic], allow_optional: bool) -> String {
        let omit = crate::model::omittable(params, self.ix);
        params
            .iter()
            .zip(omit)
            .map(|(p, o)| {
                let opt = if allow_optional && o { "?" } else { "" };
                format!("{}{opt}: {}", p.name, self.ty(&p.ty, generics))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn param_tags(&self, params: &[Param]) -> Vec<String> {
        params
            .iter()
            .map(|p| {
                let mut s = format!("@param {} {}", p.name, p.doc.join(" "));
                if let Some(d) = &p.default {
                    if let Some(j) = default_json(self.ix, &p.ty, Some(d)) {
                        let _ = write!(s, " Default `{}`.", compact(&j));
                    }
                }
                s
            })
            .collect()
    }

    fn field_tags(&self, f: &Field, desc: bool) -> Vec<String> {
        let mut tags = Vec::new();
        if f.default.is_some() || desc {
            if let Some(j) = default_json(self.ix, &f.ty, f.default.as_deref()) {
                tags.push(format!("@default {}", compact(&j)));
            }
        }
        match (f.min, f.max) {
            (Some(lo), Some(hi)) => tags.push(format!("@range {lo}..{hi}")),
            (Some(lo), None) => tags.push(format!("@range {lo}..")),
            (None, Some(hi)) => tags.push(format!("@range ..{hi}")),
            _ => {}
        }
        if let Some(s) = f.step {
            tags.push(format!("@step {s}"));
        }
        if let Some(u) = &f.unit {
            let doc = self
                .ix
                .units
                .get(u.as_str())
                .map(|u| u.doc.join(" "))
                .unwrap_or_default();
            tags.push(format!("@unit {u} ({doc})"));
        }
        if let Some(l) = &f.label {
            tags.push(format!("@label {l}"));
        }
        if f.tunable {
            tags.push("@tunable".to_string());
        }
        if f.readonly {
            tags.push("@readonly written by the engine".to_string());
        }
        tags
    }

    fn jsdoc(&mut self, indent: &str, doc: &[String], tags: &[String]) {
        if doc.is_empty() && tags.is_empty() {
            return;
        }
        if tags.is_empty() && doc.len() == 1 && doc[0].len() + indent.len() < 90 {
            self.line(&format!("{indent}/** {} */", escape(&doc[0])));
            return;
        }
        self.line(&format!("{indent}/**"));
        for d in doc {
            self.line(format!("{indent} * {}", escape(d)).trim_end());
        }
        if !doc.is_empty() && !tags.is_empty() {
            self.line(&format!("{indent} *"));
        }
        for t in tags {
            self.line(&format!("{indent} * {}", escape(t)));
        }
        self.line(&format!("{indent} */"));
    }

    /// An IDL type expression in TypeScript.
    fn ty(&self, ty: &TypeRef, generics: &[Generic]) -> String {
        let (name, args) = match ty {
            TypeRef::Int(n) => return n.to_string(),
            TypeRef::Named { name, args } => (name.as_str(), args.as_slice()),
        };
        if generics.iter().any(|g| g.name == name) {
            return name.to_string();
        }
        let arg = |i: usize| self.ty(&args[i], generics);
        match name {
            "Void" => "void".into(),
            "bool" => "boolean".into(),
            "i32" | "u32" | "f32" | "f64" => "number".into(),
            "String" => "string".into(),
            "Json" => "unknown".into(),
            "Bytes" => "Uint8Array".into(),
            "List" => format!("readonly {}[]", paren(&arg(0))),
            "Map" => format!("Readonly<Record<string, {}>>", arg(0)),
            "Option" => format!("{} | null", arg(0)),
            "OneOf" => format!("{} | {}", arg(0), arg(1)),
            "Result" => format!("Result<{}>", arg(0)),
            "Partial" => format!("Partial<{}>", arg(0)),
            "Value" | "View" => {
                let a = args[0].name();
                let is_record = matches!(self.ix.decl(a), Some(TypeDecl::Record(_)));
                match (name, is_record) {
                    ("Value", true) => a.to_string(),
                    (_, true) => format!("View<typeof og.{a}>"),
                    _ => format!("{name}<{a}>"),
                }
            }
            "Array" => {
                let n = match args.get(1) {
                    Some(TypeRef::Int(n)) => *n as usize,
                    _ => 0,
                };
                let one = arg(0);
                format!("[{}]", vec![one; n].join(", "))
            }
            "ComponentType" | "ResourceType" => format!("{name}<{}>", arg(0)),
            other => other.to_string(),
        }
    }
}

fn paren(t: &str) -> String {
    if t.contains(" | ") || t.contains("=>") || t.contains(" & ") {
        format!("({t})")
    } else {
        t.to_string()
    }
}

fn escape(s: &str) -> String {
    s.replace("*/", "*\\/")
}

/// One-line JSON for doc tags.
fn compact(j: &Json) -> String {
    match j {
        Json::Null => "null".into(),
        Json::Bool(b) => b.to_string(),
        Json::Num(n) => n.clone(),
        Json::Str(_) => j.render().trim_end().to_string(),
        Json::Arr(items) => format!(
            "[{}]",
            items.iter().map(compact).collect::<Vec<_>>().join(", ")
        ),
        Json::Obj(pairs) if pairs.is_empty() => "{}".into(),
        Json::Obj(pairs) => format!(
            "{{ {} }}",
            pairs
                .iter()
                .map(|(k, v)| format!("{k}: {}", compact(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
