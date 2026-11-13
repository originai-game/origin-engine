//! A document's TypeScript declaration (docs/ui.md §2.1): its props, the
//! events it can send and its slots, as one `.d.ts` that adds the document
//! to the engine's `UiScreens` or `UiComponents` registry. Then
//! `og.ui.push(world, "shop", vm)` type-checks `vm` against the contract
//! and og.ui.screenVm answers the contract's viewmodel type.
//!
//! A prop's type is its contract's (`number`, `string`, `bool`, `list`,
//! `object`, `any`); a list's items and an object's fields are read from
//! the document's fixtures, where it has any. The events are what the
//! builder sends: an `emit` trigger's name, `back emit`'s, the ids of its
//! elements for clicks and gestures, and those of the components it uses.
//! A component the lookup cannot find leaves those names as any string,
//! since its events would be missed.

use crate::doc::schema::Kind;
use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::{Act, Back, DocKind, NodeKind, UiDocument};

/// The names a document's events can carry, by kind; `None` for one
/// whose names are open (it uses a component the lookup does not have).
#[derive(Default)]
struct Names {
    emit: Option<BTreeSet<String>>,
    ids: Option<BTreeSet<String>>,
}

fn names<'a>(
    doc: &'a UiDocument,
    lookup: &dyn Fn(&str) -> Option<&'a UiDocument>,
    seen: &mut BTreeSet<String>,
) -> Names {
    let mut emit = BTreeSet::new();
    let mut ids = BTreeSet::from([String::new()]);
    let mut open = false;
    for n in &doc.nodes {
        ids.insert(n.id.clone());
        for t in &n.triggers {
            if let Act::Emit(e) | Act::Submit { action: e, .. } = &t.act {
                emit.insert(e.clone());
            }
        }
        if let NodeKind::Use(c) = &n.kind {
            if !seen.insert(c.clone()) {
                continue;
            }
            match lookup(c) {
                Some(cd) => {
                    let inner = names(cd, lookup, seen);
                    match (inner.emit, inner.ids) {
                        (Some(e), Some(i)) => {
                            emit.extend(e);
                            ids.extend(i);
                        }
                        _ => open = true,
                    }
                }
                None => open = true,
            }
        }
    }
    if open {
        Names::default()
    } else {
        Names {
            emit: Some(emit),
            ids: Some(ids),
        }
    }
}

/// `main-menu` → `MainMenu`, `rpg/shop` → `RpgShop`.
pub fn type_name(doc_name: &str) -> String {
    let mut out = String::new();
    for w in doc_name.split(|c: char| !c.is_ascii_alphanumeric()) {
        let mut cs = w.chars();
        if let Some(f) = cs.next() {
            out.extend(f.to_uppercase());
            out.push_str(cs.as_str());
        }
    }
    if out.is_empty() || out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'D');
    }
    out
}

fn lit(s: &str) -> String {
    serde_json::to_string(s).expect("a string")
}

fn union(names: &BTreeSet<String>) -> String {
    if names.is_empty() {
        return "never".into();
    }
    names.iter().map(|n| lit(n)).collect::<Vec<_>>().join(" | ")
}

fn ident_or_quoted(k: &str) -> String {
    let ident = k
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_' || c == '$')
        && k.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
    if ident { k.to_string() } else { lit(k) }
}

/// The TypeScript type of every value in `vs` (fixtures' values of one
/// prop, or one list's items): their kinds' union, an object's fields
/// optional where some value lacks them.
fn infer(vs: &[&Value], indent: &str) -> String {
    let mut kinds: Vec<String> = Vec::new();
    let mut push = |t: String| {
        if !kinds.contains(&t) {
            kinds.push(t);
        }
    };
    let mut objects: Vec<&serde_json::Map<String, Value>> = Vec::new();
    let mut items: Vec<&Value> = Vec::new();
    let mut arrays = false;
    for v in vs {
        match v {
            Value::Null => push("null".into()),
            Value::Bool(_) => push("boolean".into()),
            Value::Number(_) => push("number".into()),
            Value::String(_) => push("string".into()),
            Value::Array(a) => {
                arrays = true;
                items.extend(a.iter());
            }
            Value::Object(o) => objects.push(o),
        }
    }
    if arrays {
        let inner = if items.is_empty() {
            "unknown".to_string()
        } else {
            infer(&items, indent)
        };
        let wrapped = if inner.contains(' ') && !inner.starts_with('{') {
            format!("({inner})")
        } else {
            inner
        };
        push(format!("readonly {wrapped}[]"));
    }
    if !objects.is_empty() {
        let mut keys: BTreeMap<&str, Vec<&Value>> = BTreeMap::new();
        for o in &objects {
            for (k, v) in o.iter() {
                keys.entry(k).or_default().push(v);
            }
        }
        if keys.is_empty() {
            push("Readonly<Record<string, unknown>>".into());
        } else {
            let deeper = format!("{indent}  ");
            let mut s = String::from("{\n");
            for (k, vals) in &keys {
                let opt = if vals.len() < objects.len() { "?" } else { "" };
                s.push_str(&format!(
                    "{deeper}readonly {}{opt}: {};\n",
                    ident_or_quoted(k),
                    infer(vals, &deeper)
                ));
            }
            s.push_str(indent);
            s.push('}');
            push(s);
        }
    }
    if kinds.is_empty() {
        "unknown".into()
    } else {
        kinds.join(" | ")
    }
}

fn prop_type(doc: &UiDocument, name: &str, ty: &str) -> String {
    let vals: Vec<&Value> = doc
        .fixtures
        .iter()
        .filter_map(|(_, f)| f.get(name))
        .filter(|v| !v.is_null())
        .collect();
    let from = |want: fn(&Value) -> bool| -> Option<String> {
        let vs: Vec<&Value> = vals.iter().copied().filter(|v| want(v)).collect();
        (!vs.is_empty()).then(|| infer(&vs, "  "))
    };
    match ty {
        "number" => "number".into(),
        "string" => "string".into(),
        "bool" => "boolean".into(),
        "list" => from(Value::is_array).unwrap_or_else(|| "readonly unknown[]".into()),
        "object" => {
            from(Value::is_object).unwrap_or_else(|| "Readonly<Record<string, unknown>>".into())
        }
        _ => "unknown".into(),
    }
}

/// The `.d.ts` of `doc`; `lookup` finds the components it uses by name.
pub fn document_dts<'a>(
    doc: &'a UiDocument,
    lookup: &dyn Fn(&str) -> Option<&'a UiDocument>,
) -> String {
    let t = type_name(&doc.name);
    let (registry, what, key) = match doc.kind {
        DocKind::Screen => ("UiScreens", "screen", "Vm"),
        DocKind::Component => ("UiComponents", "component", "Props"),
    };
    let mut out = format!(
        "// Generated by `og ui types` from {}. Do not edit: it is rewritten from the document.\n\
         import type {{ UiDocEvent }} from \"@origingame/engine\";\n\n",
        if doc.source.is_empty() {
            format!("the {what} '{}'", doc.name)
        } else {
            format!("assets/{}", doc.source)
        }
    );
    // Props.
    out.push_str(&format!(
        "/** The {} of the {what} '{}'. */\nexport interface {t}{key} {{\n",
        if doc.kind == DocKind::Screen {
            "viewmodel"
        } else {
            "props"
        },
        doc.name
    ));
    for p in &doc.props {
        if !p.doc.is_empty() {
            out.push_str(&format!("  /** {} */\n", p.doc.replace("*/", "*\\/")));
        }
        out.push_str(&format!(
            "  readonly {}{}: {};\n",
            ident_or_quoted(&p.name),
            if p.default.is_some() { "?" } else { "" },
            prop_type(doc, &p.name, &p.ty)
        ));
    }
    out.push_str("}\n\n");
    // Events.
    let n = names(doc, lookup, &mut BTreeSet::from([doc.name.clone()]));
    let base = "Omit<UiDocEvent, \"kind\" | \"name\">";
    let mut arms = Vec::new();
    let emit = n.emit.as_ref().map_or("string".into(), union);
    if emit != "never" {
        arms.push(format!(
            "{base} & {{ readonly kind: \"emit\"; readonly name: {emit} }}"
        ));
    }
    if let Back::Emit(b) = &doc.layer.back {
        arms.push(format!(
            "{base} & {{ readonly kind: \"back\"; readonly name: {} }}",
            lit(b)
        ));
    }
    let ids = n.ids.as_ref().map_or("string".into(), union);
    arms.push(format!(
        "{base} & {{ readonly kind: \"click\" | \"tap\" | \"doubleTap\" | \"longPress\"; readonly name: {ids} }}"
    ));
    let mut rest = vec!["changed", "select", "action", "set", "use"];
    if doc.kind == DocKind::Screen {
        rest.extend(["activate", "deactivate"]);
    }
    arms.push(format!(
        "{base} & {{ readonly kind: {}; readonly name: string }}",
        rest.iter().map(|k| lit(k)).collect::<Vec<_>>().join(" | ")
    ));
    out.push_str(&format!(
        "/** An event the {what} '{}' can send (og.ui.{what}'s answer). */\nexport type {t}Event =\n",
        doc.name
    ));
    for a in &arms {
        out.push_str(&format!("  | ({a})\n"));
    }
    out.pop();
    out.push_str(";\n\n");
    // Slots.
    let slots: BTreeSet<String> = doc
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Element(Kind::Slot) && !n.id.is_empty())
        .map(|n| format!("{}/{}", doc.name, n.id))
        .collect();
    out.push_str(&format!(
        "/** Its slots, by path. */\nexport type {t}Slot = {};\n\n",
        union(&slots)
    ));
    out.push_str(&format!(
        "declare module \"@origingame/engine\" {{\n  interface {registry} {{\n    {}: {{ props: {t}{key}; events: {t}Event; slots: {t}Slot }};\n  }}\n}}\n",
        lit(&doc.name)
    ));
    out
}
