//! The description budget `og api check` reports: an IDL description says
//! what an entry is, its units and defaults, when it errors and its
//! behavioural contract; workflow, sizes, timings and cross-member caveats
//! live in the guides (`guide "saves §2"`).
//!
//! The measuring rule: an entry's description is its `doc` text (the
//! trailing doc string and every `doc` line), counted in Unicode scalar
//! values after unescaping, its paragraphs summed. Its parameters, fields,
//! props, variants, cases and its `returns` doc are entries of their own,
//! and a `guide` line costs nothing. Modules, functions, methods and types
//! (struct, enum, union, interface, component, resource, event, handle,
//! callback, column) have [`ITEM`] characters; everything else (fields,
//! params, props, variants, cases, returns, consts, values, units) [`FIELD`].

use crate::model::*;

/// The budget of a module's, function's, method's or type's description.
pub const ITEM: usize = 600;
/// The budget of a field's, parameter's, prop's, variant's, case's,
/// return's, constant's, value's or unit's description.
pub const FIELD: usize = 300;

/// Descriptions over budget that predate it. `og api check` fails when the
/// count grows past this; each one fixed lowers it, and at zero any
/// description over budget fails.
pub const BACKLOG: usize = 17;

/// One description over its budget.
#[derive(Debug, Clone, PartialEq)]
pub struct Over {
    /// `fn physics.castRay`, `field RenderDesc.bloom`, ...
    pub entry: String,
    pub line: usize,
    pub chars: usize,
    pub budget: usize,
}

fn len(doc: &[String]) -> usize {
    doc.iter().map(|d| d.chars().count()).sum()
}

/// Every description over its budget, in declaration order.
pub fn over(api: &Api) -> Vec<Over> {
    let mut out = Vec::new();
    let mut check = |entry: String, doc: &[String], line: usize, budget: usize| {
        let chars = len(doc);
        if chars > budget {
            out.push(Over {
                entry,
                line,
                chars,
                budget,
            });
        }
    };
    for u in &api.units {
        check(format!("unit {}", u.name), &u.doc, u.line, FIELD);
    }
    fn function(
        check: &mut impl FnMut(String, &[String], usize, usize),
        owner: &str,
        f: &Function,
    ) {
        let name = format!("{owner}.{}", f.name);
        check(format!("fn {name}"), &f.doc, f.line, ITEM);
        for p in &f.params {
            check(format!("param {name}({})", p.name), &p.doc, p.line, FIELD);
        }
        check(format!("returns {name}"), &f.returns_doc, f.line, FIELD);
    }
    fn fields(check: &mut impl FnMut(String, &[String], usize, usize), owner: &str, fs: &[Field]) {
        for f in fs {
            check(format!("field {owner}.{}", f.name), &f.doc, f.line, FIELD);
        }
    }
    for m in &api.modules {
        check(format!("module {}", m.name), &m.doc, m.line, ITEM);
        for item in &m.items {
            match item {
                Item::Fn(f) => function(&mut check, &m.name, f),
                Item::Const(c) => check(format!("const {}", c.name), &c.doc, c.line, FIELD),
                Item::Value(v) => check(
                    format!("value {}.{}", m.name, v.name),
                    &v.doc,
                    v.line,
                    FIELD,
                ),
                Item::Handle(h) => check(format!("handle {}", h.name), &h.doc, h.line, ITEM),
                Item::Struct(s) => {
                    check(format!("struct {}", s.name), &s.doc, s.line, ITEM);
                    fields(&mut check, &s.name, &s.fields);
                }
                Item::Enum(e) => {
                    check(format!("enum {}", e.name), &e.doc, e.line, ITEM);
                    for v in e.variants.iter().chain(&e.patterns) {
                        check(
                            format!("variant {}.{}", e.name, v.name),
                            &v.doc,
                            v.line,
                            FIELD,
                        );
                    }
                }
                Item::Union(u) => {
                    check(format!("union {}", u.name), &u.doc, u.line, ITEM);
                    for c in &u.cases {
                        check(format!("case {}.{}", u.name, c.name), &c.doc, c.line, FIELD);
                    }
                }
                Item::Interface(i) => {
                    check(format!("interface {}", i.name), &i.doc, i.line, ITEM);
                    for mem in &i.members {
                        match mem {
                            Member::Fn(f) => function(&mut check, &i.name, f),
                            Member::Prop(p) => {
                                check(format!("prop {}.{}", i.name, p.name), &p.doc, p.line, FIELD)
                            }
                        }
                    }
                }
                Item::Callback(c) => {
                    check(format!("callback {}", c.name), &c.doc, c.line, ITEM);
                    for p in &c.params {
                        check(
                            format!("param {}({})", c.name, p.name),
                            &p.doc,
                            p.line,
                            FIELD,
                        );
                    }
                }
                Item::Column(c) => check(format!("column {}", c.name), &c.doc, c.line, ITEM),
                Item::Record(r) => {
                    check(
                        format!("{} {}", r.kind.as_str(), r.name),
                        &r.doc,
                        r.line,
                        ITEM,
                    );
                    fields(&mut check, &r.name, &r.fields);
                    for f in &r.methods {
                        function(&mut check, &r.name, f);
                    }
                }
            }
        }
    }
    out
}
