//! Reads the `.ogt` tree of `api.ogt` into the IDL model.
//!
//! Shape errors (an unknown key, a missing name, a malformed type) are
//! reported here; meaning errors (an unknown type, a duplicate, a missing doc)
//! are the validator's.

use crate::model::*;
use crate::values::{as_f64, show};
use crate::Diagnostic;
use og_core::ogt::{Entry as Node, Value};

/// The entries of a block, or none for a line without one.
fn children(node: &Node) -> &[Node] {
    node.block.as_ref().map_or(&[], |b| b.entries.as_slice())
}

pub(crate) struct Loader {
    pub diags: Vec<Diagnostic>,
}

impl Loader {
    fn error(&mut self, line: usize, message: impl Into<String>) {
        self.diags.push(Diagnostic {
            line,
            message: message.into(),
        });
    }

    pub fn api(&mut self, version: u32, nodes: &[Node]) -> Api {
        let mut api = Api {
            version,
            ..Api::default()
        };
        for node in nodes {
            match node.key.as_str() {
                "package" => api.package = self.string_value(node).unwrap_or_default(),
                "doc" => api.doc.extend(self.doc_line(node)),
                "unit" => {
                    if let Some(name) = self.name(node, "unit") {
                        let doc = self.doc(node, 1);
                        api.units.push(Unit {
                            name,
                            doc,
                            line: node.line,
                        });
                    }
                }
                "module" => {
                    if let Some(m) = self.module(node) {
                        api.modules.push(m);
                    }
                }
                other => self.error(
                    node.line,
                    format!(
                        "unknown top-level key `{other}`: expected package, doc, unit or module"
                    ),
                ),
            }
        }
        api
    }

    fn module(&mut self, node: &Node) -> Option<Module> {
        let name = self.name(node, "module")?;
        let mut module = Module {
            name,
            doc: self.inline(node, 1),
            place: Place::Namespace,
            publish: true,
            items: Vec::new(),
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => module.doc.extend(self.doc_line(child)),
                "place" => match child.values.first().and_then(Value::as_word) {
                    Some("namespace") => module.place = Place::Namespace,
                    Some("root") => module.place = Place::Root,
                    Some("top") => module.place = Place::Top,
                    _ => self.error(child.line, "`place` is one of namespace, root, top"),
                },
                "publish" => module.publish = self.bool_value(child).unwrap_or(true),
                _ => {
                    if let Some(item) = self.item(child) {
                        module.items.push(item);
                    }
                }
            }
        }
        Some(module)
    }

    fn item(&mut self, node: &Node) -> Option<Item> {
        Some(match node.key.as_str() {
            "fn" => Item::Fn(self.function(node)?),
            "const" => Item::Const(self.constant(node)?),
            "handle" => {
                let name = self.name(node, "handle")?;
                Item::Handle(Handle {
                    doc: self.doc(node, 1),
                    name,
                    line: node.line,
                })
            }
            "struct" => Item::Struct(self.structure(node)?),
            "enum" => Item::Enum(self.enumeration(node)?),
            "union" => Item::Union(self.union(node)?),
            "interface" => Item::Interface(self.interface(node)?),
            "callback" => Item::Callback(self.callback(node)?),
            "column" => Item::Column(self.column(node)?),
            "component" => Item::Record(self.record(node, RecordKind::Component)?),
            "resource" => Item::Record(self.record(node, RecordKind::Resource)?),
            "event" => Item::Record(self.record(node, RecordKind::Event)?),
            "value" => {
                let name = self.name(node, "value")?;
                let ty = self.decl_type(node, "value")?;
                Item::Value(ValueDecl {
                    doc: self.doc(node, 2),
                    name,
                    ty,
                    line: node.line,
                })
            }
            other => {
                self.error(
                    node.line,
                    format!(
                        "unknown key `{other}` in a module: expected doc, place, publish, fn, const, value, handle, struct, enum, union, interface, callback, column, component, resource or event"
                    ),
                );
                return None;
            }
        })
    }

    fn function(&mut self, node: &Node) -> Option<Function> {
        let name = self.name(node, "fn")?;
        let mut f = Function {
            name,
            doc: self.inline(node, 1),
            generics: Vec::new(),
            params: Vec::new(),
            returns: TypeRef::named("Void"),
            returns_doc: Vec::new(),
            publish: None,
            host: true,
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => f.doc.extend(self.doc_line(child)),
                "generic" => {
                    let (Some(n), Some(bound)) = (
                        child.values.first().and_then(Value::as_word),
                        child.values.get(1).and_then(|v| self.type_value(v, child.line)),
                    ) else {
                        self.error(child.line, "a generic is written `generic <Name> <Bound>`");
                        continue;
                    };
                    f.generics.push(Generic {
                        name: n.to_string(),
                        bound,
                        line: child.line,
                    });
                }
                "param" => {
                    if let Some(p) = self.param(child) {
                        f.params.push(p);
                    }
                }
                "returns" => {
                    if let Some(ty) = child.values.first().and_then(|v| self.type_value(v, child.line)) {
                        f.returns = ty;
                    } else {
                        self.error(child.line, "`returns` needs a type");
                    }
                    f.returns_doc = self.doc(child, 1);
                }
                "publish" => f.publish = self.bool_value(child),
                "host" => f.host = self.bool_value(child).unwrap_or(true),
                other => self.error(
                    child.line,
                    format!("unknown key `{other}` in fn `{}`: expected doc, generic, param, returns, publish or host", f.name),
                ),
            }
        }
        Some(f)
    }

    fn param(&mut self, node: &Node) -> Option<Param> {
        let name = self.name(node, "param")?;
        let ty = self.decl_type(node, "param")?;
        let mut p = Param {
            name,
            ty,
            doc: self.inline(node, 2),
            default: None,
            optional: false,
            flatten: false,
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => p.doc.extend(self.doc_line(child)),
                "default" => p.default = Some(child.values.clone()),
                "optional" => p.optional = self.bool_value(child).unwrap_or(false),
                "flatten" => p.flatten = self.bool_value(child).unwrap_or(false),
                other => self.error(
                    child.line,
                    format!("unknown key `{other}` in param `{}`: expected doc, default, optional or flatten", p.name),
                ),
            }
        }
        Some(p)
    }

    fn constant(&mut self, node: &Node) -> Option<Const> {
        let name = self.name(node, "const")?;
        let ty = self.decl_type(node, "const")?;
        let Some(value) = node.values.get(2).cloned() else {
            self.error(
                node.line,
                "a const is written `const <NAME> <Type> <value>`",
            );
            return None;
        };
        let doc = self.doc(node, 3);
        Some(Const {
            name,
            ty,
            value,
            doc,
            line: node.line,
        })
    }

    fn structure(&mut self, node: &Node) -> Option<Struct> {
        let name = self.name(node, "struct")?;
        let mut s = Struct {
            name,
            doc: self.inline(node, 1),
            desc: false,
            fields: Vec::new(),
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => s.doc.extend(self.doc_line(child)),
                "desc" => s.desc = self.bool_value(child).unwrap_or(false),
                "field" => {
                    if let Some(f) = self.field(child) {
                        s.fields.push(f);
                    }
                }
                other => self.error(
                    child.line,
                    format!(
                        "unknown key `{other}` in struct `{}`: expected doc, desc or field",
                        s.name
                    ),
                ),
            }
        }
        Some(s)
    }

    fn field(&mut self, node: &Node) -> Option<Field> {
        let name = self.name(node, "field")?;
        let ty = self.decl_type(node, "field")?;
        let mut f = Field {
            name,
            ty,
            doc: self.inline(node, 2),
            default: None,
            min: None,
            max: None,
            step: None,
            unit: None,
            label: None,
            tunable: false,
            readonly: false,
            optional: false,
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => f.doc.extend(self.doc_line(child)),
                "default" => f.default = Some(child.values.clone()),
                "min" => f.min = self.number_value(child),
                "max" => f.max = self.number_value(child),
                "step" => f.step = self.number_value(child),
                "unit" => {
                    f.unit = child.values.first().and_then(Value::as_word).map(str::to_string);
                    if f.unit.is_none() {
                        self.error(child.line, "`unit` names a declared unit, written bare");
                    }
                }
                "label" => f.label = self.string_value(child),
                "tunable" => f.tunable = self.bool_value(child).unwrap_or(false),
                "readonly" => f.readonly = self.bool_value(child).unwrap_or(false),
                "optional" => f.optional = self.bool_value(child).unwrap_or(false),
                other => self.error(
                    child.line,
                    format!(
                        "unknown key `{other}` in field `{}`: expected doc, default, min, max, step, unit, label, tunable, readonly or optional",
                        f.name
                    ),
                ),
            }
        }
        Some(f)
    }

    fn enumeration(&mut self, node: &Node) -> Option<Enum> {
        let name = self.name(node, "enum")?;
        let mut e = Enum {
            name,
            doc: self.inline(node, 1),
            variants: Vec::new(),
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => e.doc.extend(self.doc_line(child)),
                "variant" => {
                    if let Some(name) = self.name(child, "variant") {
                        let doc = self.doc(child, 1);
                        e.variants.push(Variant {
                            name,
                            doc,
                            line: child.line,
                        });
                    }
                }
                other => self.error(
                    child.line,
                    format!(
                        "unknown key `{other}` in enum `{}`: expected doc or variant",
                        e.name
                    ),
                ),
            }
        }
        Some(e)
    }

    fn union(&mut self, node: &Node) -> Option<Union> {
        let name = self.name(node, "union")?;
        let mut u = Union {
            name,
            doc: self.inline(node, 1),
            tag: String::new(),
            cases: Vec::new(),
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => u.doc.extend(self.doc_line(child)),
                "tag" => {
                    u.tag = child
                        .values
                        .first()
                        .and_then(Value::as_word)
                        .map(str::to_string)
                        .unwrap_or_default()
                }
                "case" => {
                    let Some(name) = self.name(child, "case") else {
                        continue;
                    };
                    let payload = child
                        .values
                        .get(1)
                        .and_then(Value::as_word)
                        .map(str::to_string);
                    let doc = self.doc(child, if payload.is_some() { 2 } else { 1 });
                    u.cases.push(Case {
                        name,
                        payload,
                        doc,
                        line: child.line,
                    });
                }
                other => self.error(
                    child.line,
                    format!(
                        "unknown key `{other}` in union `{}`: expected doc, tag or case",
                        u.name
                    ),
                ),
            }
        }
        if u.tag.is_empty() {
            self.error(u.line, format!("union `{}` needs `tag <field>`", u.name));
        }
        Some(u)
    }

    fn interface(&mut self, node: &Node) -> Option<Interface> {
        let name = self.name(node, "interface")?;
        let mut i = Interface {
            name,
            doc: self.inline(node, 1),
            extends: None,
            publish: None,
            members: Vec::new(),
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => i.doc.extend(self.doc_line(child)),
                "extends" => i.extends = child.values.first().and_then(Value::as_word).map(str::to_string),
                "publish" => i.publish = self.bool_value(child),
                "prop" => {
                    let Some(name) = self.name(child, "prop") else {
                        continue;
                    };
                    let Some(ty) = self.decl_type(child, "prop") else {
                        continue;
                    };
                    let mut prop = Prop {
                        name,
                        ty,
                        doc: self.inline(child, 2),
                        publish: None,
                        line: child.line,
                    };
                    for g in children(child) {
                        match g.key.as_str() {
                            "doc" => prop.doc.extend(self.doc_line(g)),
                            "publish" => prop.publish = self.bool_value(g),
                            other => self.error(g.line, format!("unknown key `{other}` in prop: expected doc or publish")),
                        }
                    }
                    i.members.push(Member::Prop(prop));
                }
                "fn" => {
                    if let Some(f) = self.function(child) {
                        i.members.push(Member::Fn(f));
                    }
                }
                other => self.error(
                    child.line,
                    format!("unknown key `{other}` in interface `{}`: expected doc, extends, publish, prop or fn", i.name),
                ),
            }
        }
        Some(i)
    }

    fn callback(&mut self, node: &Node) -> Option<Callback> {
        let name = self.name(node, "callback")?;
        let mut c = Callback {
            name,
            doc: self.inline(node, 1),
            params: Vec::new(),
            returns: TypeRef::named("Void"),
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => c.doc.extend(self.doc_line(child)),
                "param" => {
                    if let Some(p) = self.param(child) {
                        c.params.push(p);
                    }
                }
                "returns" => {
                    if let Some(ty) = child
                        .values
                        .first()
                        .and_then(|v| self.type_value(v, child.line))
                    {
                        c.returns = ty;
                    }
                }
                other => self.error(
                    child.line,
                    format!(
                        "unknown key `{other}` in callback `{}`: expected doc, param or returns",
                        c.name
                    ),
                ),
            }
        }
        Some(c)
    }

    fn column(&mut self, node: &Node) -> Option<Column> {
        let name = self.name(node, "column")?;
        let mut c = Column {
            name,
            doc: self.inline(node, 1),
            array: String::new(),
            stride: 0,
            value: TypeRef::named(""),
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => c.doc.extend(self.doc_line(child)),
                "array" => {
                    c.array = child
                        .values
                        .first()
                        .and_then(Value::as_word)
                        .map(str::to_string)
                        .unwrap_or_default()
                }
                "stride" => c.stride = self.number_value(child).map(|n| n as u32).unwrap_or(0),
                "value" => {
                    if let Some(ty) = child.values.first().and_then(|v| self.type_value(v, child.line)) {
                        c.value = ty;
                    }
                }
                other => self.error(
                    child.line,
                    format!("unknown key `{other}` in column `{}`: expected doc, array, stride or value", c.name),
                ),
            }
        }
        Some(c)
    }

    fn record(&mut self, node: &Node, kind: RecordKind) -> Option<Record> {
        let name = self.name(node, kind.as_str())?;
        let mut r = Record {
            kind,
            name,
            doc: self.inline(node, 1),
            fields: Vec::new(),
            methods: Vec::new(),
            line: node.line,
        };
        for child in children(node) {
            match child.key.as_str() {
                "doc" => r.doc.extend(self.doc_line(child)),
                "field" => {
                    if let Some(f) = self.field(child) {
                        r.fields.push(f);
                    }
                }
                "fn" if kind == RecordKind::Resource => {
                    if let Some(f) = self.function(child) {
                        r.methods.push(f);
                    }
                }
                other => self.error(
                    child.line,
                    format!(
                        "unknown key `{other}` in {} `{}`: expected doc, field{}",
                        kind.as_str(),
                        r.name,
                        if kind == RecordKind::Resource {
                            " or fn"
                        } else {
                            ""
                        }
                    ),
                ),
            }
        }
        Some(r)
    }

    // ── value helpers ────────────────────────────────────────────────────

    fn name(&mut self, node: &Node, what: &str) -> Option<String> {
        match node.values.first().and_then(Value::as_word) {
            Some(n) => Some(n.to_string()),
            None => {
                self.error(node.line, format!("`{what}` needs a bare name"));
                None
            }
        }
    }

    fn decl_type(&mut self, node: &Node, what: &str) -> Option<TypeRef> {
        match node.values.get(1) {
            Some(v) => self.type_value(v, node.line),
            None => {
                self.error(node.line, format!("`{what}` needs a type after its name"));
                None
            }
        }
    }

    fn type_value(&mut self, value: &Value, line: usize) -> Option<TypeRef> {
        let parsed = value.as_word().and_then(TypeRef::parse);
        if parsed.is_none() {
            self.error(
                line,
                format!("`{}` is not a type expression: write `Name`, or a built-in generic with its arguments after dots, as `List.Entity`, `Map.Option.f32` or `Array.f32.16`", show(value)),
            );
        }
        parsed
    }

    fn doc_line(&mut self, node: &Node) -> Option<String> {
        match (node.values.as_slice(), node.block.is_some()) {
            ([Value::String(s)], false) => Some(s.clone()),
            _ => {
                self.error(node.line, "`doc` takes exactly one quoted string");
                None
            }
        }
    }

    /// Inline trailing doc strings after `skip` values, plus `doc` children.
    fn doc(&mut self, node: &Node, skip: usize) -> Vec<String> {
        let mut doc = self.inline(node, skip);
        for child in children(node) {
            if child.key == "doc" {
                doc.extend(self.doc_line(child));
            } else {
                self.error(
                    child.line,
                    format!("unknown key `{}` in {}: expected doc", child.key, node.key),
                );
            }
        }
        doc
    }

    /// Quoted strings on the declaration line after the first `skip` values;
    /// anything else there is a mistake, not a silently ignored extra.
    fn inline(&mut self, node: &Node, skip: usize) -> Vec<String> {
        let mut doc = Vec::new();
        for v in node.values.iter().skip(skip) {
            match v {
                Value::String(s) => doc.push(s.clone()),
                other => self.error(
                    node.line,
                    format!(
                        "unexpected `{}` on a `{}` line: only a quoted doc may follow",
                        show(other),
                        node.key
                    ),
                ),
            }
        }
        doc
    }

    fn string_value(&mut self, node: &Node) -> Option<String> {
        match node.values.as_slice() {
            [Value::String(s)] => Some(s.clone()),
            _ => {
                self.error(node.line, format!("`{}` takes one quoted string", node.key));
                None
            }
        }
    }

    fn bool_value(&mut self, node: &Node) -> Option<bool> {
        match node.values.as_slice() {
            [v] if v.as_bool().is_some() => v.as_bool(),
            _ => {
                self.error(node.line, format!("`{}` takes true or false", node.key));
                None
            }
        }
    }

    fn number_value(&mut self, node: &Node) -> Option<f64> {
        match node.values.as_slice() {
            [v @ Value::Number(_)] => as_f64(v),
            _ => {
                self.error(node.line, format!("`{}` takes one bare number", node.key));
                None
            }
        }
    }
}
