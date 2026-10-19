//! Meaning checks over a loaded API. Every rule here is listed in
//! `crates/og-api/README.md` §Validation; a rule added here is added there.

use crate::Diagnostic;
use crate::index::{ARRAYS, GENERICS, Index, META, PRIMITIVES, TypeDecl};
use crate::model::*;
use crate::values::{as_f64, is_number, show as show_value};
use og_core::ogt::Value;
use std::collections::BTreeSet;

pub(crate) fn validate(api: &Api) -> Vec<Diagnostic> {
    let ix = Index::new(api);
    let mut v = Validator {
        ix: &ix,
        diags: Vec::new(),
    };
    v.run();
    let mut diags = v.diags;
    diags.extend(ix.duplicates.iter().map(|(line, message)| Diagnostic {
        line: *line,
        message: message.clone(),
    }));
    diags.sort_by_key(|d| d.line);
    diags
}

struct Validator<'a, 'b> {
    ix: &'b Index<'a>,
    diags: Vec<Diagnostic>,
}

#[derive(Clone, Copy, PartialEq)]
enum FieldSite {
    Struct { desc: bool },
    Record(RecordKind),
}

fn pascal(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(ch) if ch.is_ascii_uppercase())
        && c.all(|ch| ch.is_ascii_alphanumeric())
}

fn camel(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(ch) if ch.is_ascii_lowercase())
        && c.all(|ch| ch.is_ascii_alphanumeric())
}

fn lower(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(ch) if ch.is_ascii_lowercase())
        && c.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
}

fn screaming(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(ch) if ch.is_ascii_uppercase())
        && c.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

/// A variant is a string value everywhere it is generated (a TypeScript
/// literal, a JSON schema enum, a Rust string), so `-` may join words
/// (`vulkan-cpu`, as the reference tiers are named).
fn variant_name(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(ch) if ch.is_ascii_lowercase())
        && c.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        && !s.ends_with('-')
        && !s.contains("--")
}

impl<'a> Validator<'a, '_> {
    fn error(&mut self, line: usize, message: impl Into<String>) {
        self.diags.push(Diagnostic {
            line,
            message: message.into(),
        });
    }

    fn doc(&mut self, doc: &[String], line: usize, what: impl FnOnce() -> String) {
        if doc.iter().all(|d| d.trim().is_empty()) {
            self.error(line, format!("missing doc on {}", what()));
        }
    }

    fn name(&mut self, ok: fn(&str) -> bool, name: &str, line: usize, what: &str, rule: &str) {
        if !ok(name) {
            self.error(line, format!("{what} `{name}` must be {rule}"));
        }
    }

    fn run(&mut self) {
        let api = self.ix.api;
        if api.version != 1 {
            self.error(
                2,
                format!(
                    "api.ogt version {} is not supported: this og-api reads version 1",
                    api.version
                ),
            );
        }
        if api.package.is_empty() {
            self.error(4, "missing `package \"<npm name>\"`");
        }
        self.doc(&api.doc, 4, || "the api".into());
        for unit in &api.units {
            self.name(camel, &unit.name, unit.line, "unit", "lowerCamelCase");
            self.doc(&unit.doc, unit.line, || format!("unit `{}`", unit.name));
        }
        if self.ix.structure("FieldOptions").is_none() {
            self.error(4, "the TypeScript generator needs a struct `FieldOptions` (the options every field factory takes)");
        }

        let mut symbols = std::collections::BTreeMap::new();
        for b in crate::r#gen::bindings(self.ix) {
            let line = b
                .function
                .map(|f| f.line)
                .or(b.prop.map(|p| p.line))
                .unwrap_or(0);
            if let Some(prev) = symbols.insert(b.symbol.clone(), b.path.clone()) {
                self.error(
                    line,
                    format!(
                        "`{}` and `{prev}` would both bind the host symbol `{}`; rename one",
                        b.path, b.symbol
                    ),
                );
            }
        }

        let mut modules = BTreeSet::new();
        let mut top_values: BTreeSet<String> =
            self.ix.columns.keys().map(|s| s.to_string()).collect();
        top_values.extend(self.ix.consts.keys().map(|s| s.to_string()));
        let mut og_members = BTreeSet::new();
        for module in &api.modules {
            if !modules.insert(module.name.as_str()) {
                self.error(module.line, format!("duplicate module `{}`", module.name));
            }
            self.name(
                lower,
                &module.name,
                module.line,
                "module",
                "lowercase letters and digits",
            );
            self.doc(&module.doc, module.line, || {
                format!("module `{}`", module.name)
            });
            if module.place == Place::Namespace
                && module
                    .items
                    .iter()
                    .any(|i| matches!(i, Item::Fn(_) | Item::Value(_)))
                && !og_members.insert(module.name.clone())
            {
                self.error(module.line, format!("duplicate name `og.{}`", module.name));
            }
            let mut fns = BTreeSet::new();
            for item in &module.items {
                match item {
                    Item::Fn(f) => {
                        if !fns.insert(f.name.as_str()) {
                            self.error(
                                f.line,
                                format!("duplicate fn `{}` in module `{}`", f.name, module.name),
                            );
                        }
                        let clash = match module.place {
                            Place::Top => !top_values.insert(f.name.clone()),
                            Place::Root => !og_members.insert(f.name.clone()),
                            Place::Namespace => false,
                        };
                        if clash {
                            self.error(
                                f.line,
                                format!(
                                    "duplicate name `{}`: another export already uses it",
                                    f.name
                                ),
                            );
                        }
                        self.function(f, &format!("fn `{}.{}`", module.name, f.name));
                    }
                    Item::Value(v) => {
                        if !fns.insert(v.name.as_str()) {
                            self.error(
                                v.line,
                                format!("duplicate name `{}` in module `{}`", v.name, module.name),
                            );
                        }
                        let clash = match module.place {
                            Place::Top => !top_values.insert(v.name.clone()),
                            Place::Root => !og_members.insert(v.name.clone()),
                            Place::Namespace => false,
                        };
                        if clash {
                            self.error(
                                v.line,
                                format!(
                                    "duplicate name `{}`: another export already uses it",
                                    v.name
                                ),
                            );
                        }
                        self.name(camel, &v.name, v.line, "value", "lowerCamelCase");
                        self.doc(&v.doc, v.line, || {
                            format!("value `{}.{}`", module.name, v.name)
                        });
                        self.ty(&v.ty, &[], v.line);
                    }
                    Item::Const(c) => self.constant(c),
                    Item::Handle(h) => {
                        self.name(pascal, &h.name, h.line, "handle", "PascalCase");
                        self.doc(&h.doc, h.line, || format!("handle `{}`", h.name));
                    }
                    Item::Struct(s) => self.structure(s),
                    Item::Enum(e) => self.enumeration(e),
                    Item::Union(u) => self.union(u),
                    Item::Interface(i) => self.interface(i),
                    Item::Callback(c) => {
                        self.name(pascal, &c.name, c.line, "callback", "PascalCase");
                        self.doc(&c.doc, c.line, || format!("callback `{}`", c.name));
                        self.params(&c.params, &[], &format!("callback `{}`", c.name));
                        self.ty(&c.returns, &[], c.line);
                    }
                    Item::Column(c) => self.column(c),
                    Item::Record(r) => {
                        if !og_members.insert(r.name.clone()) {
                            self.error(r.line, format!("duplicate name `og.{}`", r.name));
                        }
                        self.record(r);
                    }
                }
            }
        }
    }

    fn function(&mut self, f: &Function, what: &str) {
        self.name(camel, &f.name, f.line, "fn", "lowerCamelCase");
        self.doc(&f.doc, f.line, || what.to_string());
        let mut seen = BTreeSet::new();
        for g in &f.generics {
            self.name(pascal, &g.name, g.line, "generic", "PascalCase");
            if !seen.insert(g.name.as_str()) {
                self.error(g.line, format!("duplicate generic `{}` in {what}", g.name));
            }
            match g.bound.name() {
                "Fields" | "AnyComponent" | "AnyResource" | "AnyEvent" | "AnyReflected" => {}
                other => self.error(
                    g.line,
                    format!(
                        "generic `{}` is bounded by `{other}`: a bound is one of {}",
                        g.name,
                        META.join(", ")
                    ),
                ),
            }
        }
        self.params(&f.params, &f.generics, what);
        self.ty(&f.returns, &f.generics, f.line);
    }

    fn params(&mut self, params: &[Param], generics: &[Generic], what: &str) {
        let mut seen = BTreeSet::new();
        let mut optional_seen = false;
        for p in params.iter() {
            self.name(camel, &p.name, p.line, "param", "lowerCamelCase");
            if !seen.insert(p.name.as_str()) {
                self.error(p.line, format!("duplicate param `{}` in {what}", p.name));
            }
            self.doc(&p.doc, p.line, || format!("param `{}` of {what}", p.name));
            self.ty(&p.ty, generics, p.line);
            if let Some(d) = &p.default
                && let Err(e) = self.value(&p.ty, d)
            {
                self.error(p.line, format!("default of param `{}`: {e}", p.name));
            }
            if p.flatten && !self.ix.is_desc(&p.ty) {
                self.error(
                    p.line,
                    format!(
                        "param `{}` is flattened but `{}` is not a desc struct",
                        p.name, p.ty
                    ),
                );
            }
            if p.declared_optional() {
                optional_seen = true;
            } else if optional_seen {
                self.error(
                    p.line,
                    format!(
                        "required param `{}` follows an optional one in {what}",
                        p.name
                    ),
                );
            }
        }
    }

    fn constant(&mut self, c: &Const) {
        self.name(screaming, &c.name, c.line, "const", "SCREAMING_SNAKE_CASE");
        self.doc(&c.doc, c.line, || format!("const `{}`", c.name));
        if !matches!(
            c.ty.name(),
            "u32" | "i32" | "f32" | "f64" | "String" | "bool"
        ) || !c.ty.args().is_empty()
        {
            self.error(
                c.line,
                format!(
                    "const `{}` has type `{}`: a const is a u32, i32, f32, f64, String or bool",
                    c.name, c.ty
                ),
            );
        } else if let Err(e) = self.value(&c.ty, std::slice::from_ref(&c.value)) {
            self.error(c.line, format!("const `{}`: {e}", c.name));
        }
    }

    fn structure(&mut self, s: &Struct) {
        self.name(pascal, &s.name, s.line, "struct", "PascalCase");
        self.doc(&s.doc, s.line, || format!("struct `{}`", s.name));
        self.fields(&s.fields, FieldSite::Struct { desc: s.desc }, &s.name);
    }

    fn enumeration(&mut self, e: &Enum) {
        self.name(pascal, &e.name, e.line, "enum", "PascalCase");
        self.doc(&e.doc, e.line, || format!("enum `{}`", e.name));
        if e.variants.is_empty() {
            self.error(e.line, format!("enum `{}` has no variants", e.name));
        }
        let mut seen = BTreeSet::new();
        for v in &e.variants {
            self.name(
                variant_name,
                &v.name,
                v.line,
                "variant",
                "lowercase first, then letters, digits, `_` or single `-` between them",
            );
            if !seen.insert(v.name.as_str()) {
                self.error(
                    v.line,
                    format!("duplicate variant `{}` in enum `{}`", v.name, e.name),
                );
            }
            self.doc(&v.doc, v.line, || {
                format!("variant `{}.{}`", e.name, v.name)
            });
        }
    }

    fn union(&mut self, u: &Union) {
        self.name(pascal, &u.name, u.line, "union", "PascalCase");
        self.doc(&u.doc, u.line, || format!("union `{}`", u.name));
        if !u.tag.is_empty() {
            self.name(camel, &u.tag, u.line, "tag", "lowerCamelCase");
        }
        let mut seen = BTreeSet::new();
        for c in &u.cases {
            self.name(
                variant_name,
                &c.name,
                c.line,
                "case",
                "lowercase first, then letters, digits, `_` or single `-` between them",
            );
            if !seen.insert(c.name.as_str()) {
                self.error(
                    c.line,
                    format!("duplicate case `{}` in union `{}`", c.name, u.name),
                );
            }
            self.doc(&c.doc, c.line, || format!("case `{}.{}`", u.name, c.name));
            if let Some(p) = &c.payload {
                match self.ix.structure(p) {
                    None if self.ix.decl(p).is_none() => {
                        self.error(c.line, format!("unknown type `{p}`"))
                    }
                    None => self.error(
                        c.line,
                        format!("case `{}` carries `{p}`, which is not a struct", c.name),
                    ),
                    Some(s) if s.fields.iter().any(|f| f.name == u.tag) => self.error(
                        c.line,
                        format!(
                            "case `{}` carries `{p}`, which has a field named after the tag `{}`",
                            c.name, u.tag
                        ),
                    ),
                    Some(_) => {}
                }
            }
        }
    }

    fn interface(&mut self, i: &'a Interface) {
        self.name(pascal, &i.name, i.line, "interface", "PascalCase");
        self.doc(&i.doc, i.line, || format!("interface `{}`", i.name));
        if let Some(base) = &i.extends
            && self.ix.interface(base).is_none()
        {
            self.error(
                i.line,
                format!(
                    "interface `{}` extends `{base}`, which is not an interface",
                    i.name
                ),
            );
        }
        let mut seen = BTreeSet::new();
        for (owner, m) in self.ix.members(i) {
            if !seen.insert(m.name()) && std::ptr::eq(owner, i) {
                self.error(
                    m.line(),
                    format!("duplicate member `{}` in interface `{}`", m.name(), i.name),
                );
            }
        }
        for m in &i.members {
            match m {
                Member::Prop(p) => {
                    self.name(camel, &p.name, p.line, "prop", "lowerCamelCase");
                    self.doc(&p.doc, p.line, || format!("prop `{}.{}`", i.name, p.name));
                    self.ty(&p.ty, &[], p.line);
                }
                Member::Fn(f) => self.function(f, &format!("method `{}.{}`", i.name, f.name)),
            }
        }
    }

    fn column(&mut self, c: &Column) {
        self.doc(&c.doc, c.line, || format!("column `{}`", c.name));
        self.ty(&c.value, &[], c.line);
        if c.name != c.value.to_string() {
            self.error(
                c.line,
                format!(
                    "column `{}` holds `{}`: a column is named after its value type",
                    c.name, c.value
                ),
            );
        }
        let Some(elem) = ARRAYS.iter().find(|(a, _)| *a == c.array).map(|(_, e)| *e) else {
            let names: Vec<_> = ARRAYS.iter().map(|(a, _)| *a).collect();
            self.error(
                c.line,
                format!(
                    "column `{}` uses array `{}`: one of {}",
                    c.name,
                    c.array,
                    names.join(", ")
                ),
            );
            return;
        };
        match self.ix.flat(&c.value) {
            None => self.error(
                c.line,
                format!(
                    "column `{}` holds `{}`, which is not plain numeric data",
                    c.name, c.value
                ),
            ),
            Some(flat) => {
                if let Some(bad) = flat.iter().find(|s| **s != elem) {
                    self.error(
                        c.line,
                        format!(
                            "column `{}` stores {} in a {}, which holds {}",
                            c.name, bad, c.array, elem
                        ),
                    );
                }
                if flat.len() as u32 != c.stride {
                    self.error(
                        c.line,
                        format!(
                            "column `{}` declares stride {} but `{}` is {} scalars",
                            c.name,
                            c.stride,
                            c.value,
                            flat.len()
                        ),
                    );
                }
            }
        }
    }

    fn record(&mut self, r: &Record) {
        let kind = r.kind.as_str();
        self.name(pascal, &r.name, r.line, kind, "PascalCase");
        self.doc(&r.doc, r.line, || format!("{kind} `{}`", r.name));
        self.fields(&r.fields, FieldSite::Record(r.kind), &r.name);
        for m in &r.methods {
            if r.fields.iter().any(|f| f.name == m.name) {
                self.error(
                    m.line,
                    format!("duplicate member `{}` in resource `{}`", m.name, r.name),
                );
            }
            self.function(m, &format!("method `{}.{}`", r.name, m.name));
        }
        let mut seen = BTreeSet::new();
        for m in &r.methods {
            if !seen.insert(m.name.as_str()) {
                self.error(
                    m.line,
                    format!("duplicate method `{}` in resource `{}`", m.name, r.name),
                );
            }
        }
    }

    fn fields(&mut self, fields: &[Field], site: FieldSite, owner: &str) {
        let mut seen = BTreeSet::new();
        for f in fields {
            self.name(camel, &f.name, f.line, "field", "lowerCamelCase");
            if !seen.insert(f.name.as_str()) {
                self.error(f.line, format!("duplicate field `{}` in `{owner}`", f.name));
            }
            self.doc(&f.doc, f.line, || format!("field `{owner}.{}`", f.name));
            self.ty(&f.ty, &[], f.line);

            let reflected = matches!(
                site,
                FieldSite::Record(RecordKind::Component | RecordKind::Resource)
            );
            if reflected && !self.ix.columns.contains_key(f.ty.to_string().as_str()) {
                self.error(
                    f.line,
                    format!("field `{owner}.{}` has type `{}`, which is not a column kind: component and resource fields are columns", f.name, f.ty),
                );
            }
            if let FieldSite::Record(RecordKind::Event) = site
                && (matches!(
                    self.ix.decl(f.ty.name()),
                    Some(TypeDecl::Interface(_) | TypeDecl::Callback(_))
                ) || META.contains(&f.ty.name()))
            {
                self.error(
                    f.line,
                    format!(
                        "event field `{owner}.{}` must be data, not `{}`",
                        f.name, f.ty
                    ),
                );
            }
            if (f.tunable || f.readonly) && !reflected {
                self.error(f.line, format!("field `{owner}.{}`: `tunable` and `readonly` belong to component and resource fields", f.name));
            }
            if f.optional && !matches!(site, FieldSite::Struct { desc: false }) {
                self.error(f.line, format!("field `{owner}.{}`: `optional` belongs to plain struct fields (desc fields are all optional)", f.name));
            }
            if let Some(unit) = &f.unit
                && !self.ix.units.contains_key(unit.as_str())
            {
                self.error(
                    f.line,
                    format!("field `{owner}.{}` uses unknown unit `{unit}`", f.name),
                );
            }
            let numeric = !matches!(self.ix.decl(f.ty.name()), Some(TypeDecl::Enum(_)))
                && self
                    .ix
                    .flat(&f.ty)
                    .is_some_and(|fl| fl.iter().all(|s| *s != "bool"));
            if (f.min.is_some() || f.max.is_some() || f.step.is_some() || f.tunable) && !numeric {
                self.error(
                    f.line,
                    format!(
                        "field `{owner}.{}` has a range or is tunable, but `{}` is not numeric",
                        f.name, f.ty
                    ),
                );
            }
            if let (Some(lo), Some(hi)) = (f.min, f.max)
                && lo > hi
            {
                self.error(
                    f.line,
                    format!("field `{owner}.{}`: min {lo} is above max {hi}", f.name),
                );
            }
            if f.step.is_some_and(|s| s <= 0.0) {
                self.error(
                    f.line,
                    format!("field `{owner}.{}`: step must be positive", f.name),
                );
            }
            match &f.default {
                Some(d) => {
                    match self.value(&f.ty, d) {
                        Err(e) => self.error(
                            f.line,
                            format!("default of field `{owner}.{}`: {e}", f.name),
                        ),
                        Ok(()) => {
                            for n in d.iter().filter_map(as_f64) {
                                if f.min.is_some_and(|lo| n < lo) || f.max.is_some_and(|hi| n > hi)
                                {
                                    self.error(f.line, format!("default of field `{owner}.{}` ({n}) is outside its range", f.name));
                                }
                            }
                        }
                    }
                }
                None => {
                    let needs = matches!(
                        site,
                        FieldSite::Struct { desc: true } | FieldSite::Record(_)
                    );
                    if needs && !self.ix.has_zero(&f.ty) {
                        self.error(
                            f.line,
                            format!("field `{owner}.{}` of type `{}` has no zero value: give it a `default` or make it Option<{}>", f.name, f.ty, f.ty),
                        );
                    } else if needs
                        && crate::index::is_number(f.ty.name())
                        && (f.min.is_some_and(|lo| lo > 0.0) || f.max.is_some_and(|hi| hi < 0.0))
                    {
                        self.error(
                            f.line,
                            format!("field `{owner}.{}` defaults to 0, which is outside its range: declare a `default`", f.name),
                        );
                    }
                }
            }
        }
    }

    /// Type expressions: every name resolves, arities match, special forms
    /// take what they mean.
    fn ty(&mut self, ty: &TypeRef, generics: &[Generic], line: usize) {
        let (name, args) = match ty {
            TypeRef::Int(n) => {
                self.error(line, format!("`{n}` is a number where a type is expected"));
                return;
            }
            TypeRef::Named { name, args } => (name.as_str(), args.as_slice()),
        };
        let generic = |n: &str| generics.iter().find(|g| g.name == n);
        if let Some(g) = generic(name) {
            if !args.is_empty() {
                self.error(line, format!("generic `{}` takes no arguments", g.name));
            }
            return;
        }
        if PRIMITIVES.contains(&name) || META.contains(&name) {
            if !args.is_empty() {
                self.error(line, format!("`{name}` takes no type arguments"));
            }
            return;
        }
        if let Some((_, arity)) = GENERICS.iter().find(|(n, _)| *n == name) {
            if args.len() != *arity {
                self.error(
                    line,
                    format!(
                        "`{name}` takes {arity} type argument(s), got {}",
                        args.len()
                    ),
                );
                return;
            }
            match name {
                "Array" => {
                    if !matches!(args[1], TypeRef::Int(n) if n > 0) {
                        self.error(line, "`Array<T,N>` needs a positive length N");
                    }
                    self.ty(&args[0], generics, line);
                    if self.ix.flat(&args[0]).is_none() {
                        self.error(
                            line,
                            format!("`Array<{},..>` must hold plain numeric data", args[0]),
                        );
                    }
                }
                "Value" | "View" => {
                    let a = args[0].name();
                    let ok = match generic(a) {
                        Some(g) => match name {
                            "View" => g.bound.name() == "AnyComponent",
                            _ => matches!(
                                g.bound.name(),
                                "AnyComponent" | "AnyResource" | "AnyEvent" | "AnyReflected"
                            ),
                        },
                        None => match self.ix.decl(a) {
                            Some(TypeDecl::Record(r)) => {
                                name == "Value" || r.kind == RecordKind::Component
                            }
                            _ => false,
                        },
                    };
                    if !ok {
                        let want = if name == "View" {
                            "a component"
                        } else {
                            "a component, resource or event"
                        };
                        self.error(
                            line,
                            format!("`{name}<{a}>` needs {want} token or a generic bounded by one"),
                        );
                    }
                }
                "ComponentType" | "ResourceType" => {
                    let a = args[0].name();
                    if !generic(a).is_some_and(|g| g.bound.name() == "Fields") {
                        self.error(
                            line,
                            format!("`{name}<{a}>` needs a generic bounded by Fields"),
                        );
                    }
                }
                _ => {
                    for a in args {
                        self.ty(a, generics, line);
                    }
                }
            }
            return;
        }
        if self.ix.decl(name).is_some() {
            if !args.is_empty() {
                self.error(line, format!("`{name}` takes no type arguments"));
            }
            return;
        }
        self.error(line, format!("unknown type `{name}`"));
    }

    /// A literal default against its type.
    fn value(&self, ty: &TypeRef, values: &[Value]) -> Result<(), String> {
        let name = ty.name();
        if let [Value::Word(w)] = values
            && w == "null"
        {
            return match name {
                "Option" | "Json" => Ok(()),
                _ if matches!(self.ix.decl(name), Some(TypeDecl::Handle(_))) => Ok(()),
                _ => Err(format!("`null` is not a `{ty}`")),
            };
        }
        match name {
            "Option" => return self.value(&ty.args()[0], values),
            "Json" => {
                return if values.len() == 1 {
                    Ok(())
                } else {
                    Err("a Json default is one value".into())
                };
            }
            "String" => {
                return match values {
                    [Value::String(_)] => Ok(()),
                    _ => Err(format!(
                        "a String default is one quoted string, got {}",
                        show(values)
                    )),
                };
            }
            _ => {}
        }
        if let Some(TypeDecl::Enum(e)) = self.ix.decl(name) {
            return match values {
                [Value::Word(w)] if e.variants.iter().any(|v| &v.name == w) => Ok(()),
                _ => Err(format!(
                    "an `{name}` default is one of its variants, got {}",
                    show(values)
                )),
            };
        }
        let Some(flat) = self.ix.flat(ty) else {
            return Err(format!(
                "`{ty}` takes no explicit default: its zero value is the default"
            ));
        };
        if matches!(self.ix.decl(name), Some(TypeDecl::Handle(_))) {
            return Err(format!("a `{name}` default is `null`"));
        }
        if flat.len() != values.len() {
            return Err(format!(
                "`{ty}` is {} value(s), got {}",
                flat.len(),
                values.len()
            ));
        }
        for (scalar, v) in flat.iter().zip(values) {
            match (*scalar, v) {
                ("bool", v) if v.as_bool().is_some() => {}
                ("bool", other) => {
                    return Err(format!(
                        "expected true or false, got `{}`",
                        show_value(other)
                    ));
                }
                (s, v) if is_number(v) => {
                    let n = as_f64(v).unwrap_or(f64::NAN);
                    let int_ok = |lo: f64, hi: f64| n.fract() == 0.0 && n >= lo && n <= hi;
                    let ok = match s {
                        "u32" => int_ok(0.0, u32::MAX as f64),
                        "i32" => int_ok(i32::MIN as f64, i32::MAX as f64),
                        _ => n.is_finite(),
                    };
                    if !ok {
                        return Err(format!("`{}` is not a valid {s}", show_value(v)));
                    }
                }
                (s, other) => {
                    return Err(format!(
                        "expected a bare {s} number, got `{}`",
                        show_value(other)
                    ));
                }
            }
        }
        Ok(())
    }
}

fn show(values: &[Value]) -> String {
    if values.is_empty() {
        return "nothing".into();
    }
    values
        .iter()
        .map(|v| format!("`{}`", show_value(v)))
        .collect::<Vec<_>>()
        .join(" ")
}
