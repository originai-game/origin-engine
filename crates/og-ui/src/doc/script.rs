//! A document's `<script>` (docs/ui.md §2): only what is data, never code.
//! `let { … } = $props()` declares its props with literal defaults, and
//! `$derived(expr)` / `$derived.by(() => expr)` name pure expressions. A
//! derived value and a `{@const}` are names for expressions: every read of
//! one is replaced by its expression when the document is parsed, so they
//! cost nothing at run time. Anything else is refused: game logic lives in
//! TypeScript systems that write the stores a document reads.

use super::syntax::{Error, identifier};
use super::*;
use std::collections::BTreeMap;

const ELSEWHERE: &str =
    "put state and logic in a TS system and pass the result to the document through a store";

/// What a script declares: its props and its derived values, in order.
type Declared = (Vec<ContractProp>, Vec<(String, Expr)>);

/// The props and derived values `text` (at `base`) declares.
pub(super) fn read(text: &str, base: usize) -> Result<Declared, Error> {
    let mut props = Vec::new();
    let mut derived: Vec<(String, Expr)> = Vec::new();
    for (stmt, at) in statements(text) {
        let at = base + at;
        let words: Vec<&str> = stmt.split_whitespace().collect();
        let first = words.first().copied().unwrap_or("");
        if matches!(first, "interface" | "type") {
            // Erased TypeScript: types of the props, nothing that runs.
            continue;
        }
        let refuse =
            |what: &str| -> Error { (at, format!("{what} in a document's <script>; {ELSEWHERE}")) };
        let Some(rest) = ["let ", "const "].iter().find_map(|k| stmt.strip_prefix(k)) else {
            return Err(refuse(match first {
                "import" | "export" => "imports and exports are not allowed",
                "function" | "async" | "class" => "functions are not allowed",
                _ if stmt.contains("$effect") => "$effect is not allowed",
                _ => "statements are not allowed (only let { … } = $props() and $derived)",
            }));
        };
        let Some((target, value)) = split_assign(rest) else {
            return Err((
                at,
                "declare `let { name = default } = $props()` or `const name = $derived(expr)`"
                    .into(),
            ));
        };
        let value = value.trim();
        if value.starts_with("$state") {
            return Err(refuse("$state is not allowed"));
        }
        if value.starts_with("$effect") {
            return Err(refuse("$effect is not allowed"));
        }
        if value == "$props()" {
            let pattern = strip_type(target.trim());
            let Some(inner) = pattern.strip_prefix('{').and_then(|p| p.strip_suffix('}')) else {
                return Err((
                    at,
                    "destructure the props: let { a, b = 1 } = $props()".into(),
                ));
            };
            for entry in split_top(inner, ',') {
                let entry = entry.trim();
                if entry.is_empty() {
                    continue;
                }
                if entry.starts_with("...") {
                    return Err((
                        at,
                        "rest props (...rest) are not supported; name each prop".into(),
                    ));
                }
                let (name, default) = match split_assign(entry) {
                    Some((n, d)) => (n.trim(), Some(d.trim())),
                    None => (entry, None),
                };
                if !identifier(name) {
                    return Err((
                        at,
                        format!("{name}: a prop is a plain name; renaming is not supported"),
                    ));
                }
                let default = match default {
                    None => None,
                    Some(d) => {
                        let d = match d
                            .strip_prefix("$bindable(")
                            .and_then(|d| d.strip_suffix(')'))
                        {
                            Some(inner) => inner.trim(),
                            None => d,
                        };
                        if d.is_empty() {
                            None
                        } else {
                            Some(literal(d).ok_or((
                                at,
                                format!("{name}'s default `{d}` is not a literal; a default is a number, string, boolean, null, array or object of them"),
                            ))?)
                        }
                    }
                };
                if props.iter().any(|p: &ContractProp| p.name == name) {
                    return Err((at, format!("duplicate prop {name}; declare it once")));
                }
                let ty = match &default {
                    Some(serde_json::Value::Number(_)) => "number",
                    Some(serde_json::Value::String(_)) => "string",
                    Some(serde_json::Value::Bool(_)) => "bool",
                    Some(serde_json::Value::Array(_)) => "list",
                    Some(serde_json::Value::Object(_)) => "object",
                    _ => "any",
                };
                props.push(ContractProp {
                    name: name.into(),
                    ty: ty.into(),
                    doc: String::new(),
                    default,
                });
            }
            continue;
        }
        let name = strip_type(target.trim());
        if !identifier(name) {
            return Err((at, format!("{name}: a derived value takes one plain name")));
        }
        let source = if let Some(e) = value
            .strip_prefix("$derived.by(")
            .and_then(|v| v.strip_suffix(')'))
        {
            let body = e
                .trim()
                .strip_prefix("()")
                .map(str::trim)
                .and_then(|b| b.strip_prefix("=>"));
            let Some(body) = body.map(str::trim) else {
                return Err((
                    at,
                    "$derived.by takes an arrow function: $derived.by(() => expr)".into(),
                ));
            };
            match body.strip_prefix('{').and_then(|b| b.strip_suffix('}')) {
                Some(block) => {
                    let block = block.trim().trim_end_matches(';').trim();
                    match block.strip_prefix("return ") {
                        Some(e) if !e.contains(';') => e.trim().to_string(),
                        _ => return Err(refuse("a $derived.by body with statements")),
                    }
                }
                None => body.to_string(),
            }
        } else if let Some(e) = value
            .strip_prefix("$derived(")
            .and_then(|v| v.strip_suffix(')'))
        {
            e.trim().to_string()
        } else {
            return Err(refuse(&format!(
                "`{name} = {value}` is a variable, which is not allowed"
            )));
        };
        if derived.iter().any(|(n, _)| n == name) || props.iter().any(|p| p.name == name) {
            return Err((at, format!("{name} is declared twice")));
        }
        let mut expr = super::svelte::expression(&source, at)?;
        let scope: BTreeMap<String, Expr> = derived.iter().cloned().collect();
        substitute(&mut expr, &scope).map_err(|e| (at, e))?;
        derived.push((name.into(), expr));
    }
    Ok((props, derived))
}

/// Replaces each read of a name in `scope` by its expression.
pub(super) fn substitute(expr: &mut Expr, scope: &BTreeMap<String, Expr>) -> Result<(), String> {
    if scope.is_empty() {
        return Ok(());
    }
    match expr {
        Expr::Lit(_) | Expr::Interaction(_) | Expr::InteractionAt { .. } => {}
        Expr::Path(p) => {
            let (root, rest) = p.split_once('.').unwrap_or((p.as_str(), ""));
            if let Some(value) = scope.get(root) {
                *expr = match (value, rest) {
                    (v, "") => v.clone(),
                    (Expr::Path(base), rest) => Expr::Path(format!("{base}.{rest}")),
                    _ => {
                        return Err(format!(
                            "{p} reads a field of {root}, which is an expression; name the field's path instead"
                        ));
                    }
                };
            }
        }
        Expr::Translate(key, args) => {
            substitute(key, scope)?;
            for (_, v) in args {
                substitute(v, scope)?;
            }
        }
        Expr::Not(a)
        | Expr::Neg(a)
        | Expr::Format(_, a)
        | Expr::HasToken(a, _)
        | Expr::AttrTest(a, _) => substitute(a, scope)?,
        Expr::Lang(a, _) | Expr::Dir(a, _) => {
            if let Some(a) = a {
                substitute(a, scope)?;
            }
        }
        Expr::Bin(_, a, b) => {
            substitute(a, scope)?;
            substitute(b, scope)?;
        }
        Expr::If(c, a, b) => {
            substitute(c, scope)?;
            substitute(a, scope)?;
            substitute(b, scope)?;
        }
    }
    Ok(())
}

/// After the stylesheet: the document's derived values everywhere, and
/// each `{@const}` (`consts`: its block, name and expression) in its block.
pub(super) fn apply(doc: &mut UiDocument, consts: &[(usize, String, Expr)]) -> Result<(), String> {
    if doc.derived.is_empty() && consts.is_empty() {
        return Ok(());
    }
    let base: BTreeMap<String, Expr> = doc.derived.iter().cloned().collect();
    for i in 0..doc.nodes.len() {
        // Its scope: the derived values, then each enclosing block's consts
        // (inner wins), less the each aliases that shadow them.
        let mut chain = Vec::new();
        let mut at = doc.nodes[i].parent;
        while at != NONE {
            chain.push(at);
            at = doc.nodes[at].parent;
        }
        let mut scope = base.clone();
        for &block in chain.iter().rev() {
            if let NodeKind::For { var, .. } = &doc.nodes[block].kind {
                scope.remove(var);
            }
            for (_, name, e) in consts.iter().filter(|(b, ..)| *b == block) {
                let mut e = e.clone();
                substitute(&mut e, &scope)?;
                scope.insert(name.clone(), e);
            }
        }
        let line = doc.nodes[i].line;
        let at_line = |e: String| format!("line {line}: {e}");
        let n = &mut doc.nodes[i];
        // The node's own `{#each}` key and list read the outer scope; its
        // alias shadows inside.
        match &mut n.kind {
            NodeKind::For { list, key, var, .. } => {
                substitute(list, &scope).map_err(at_line)?;
                let mut inner = scope.clone();
                inner.remove(var.as_str());
                substitute(key, &inner).map_err(at_line)?;
            }
            NodeKind::If(e) | NodeKind::Key(e) => substitute(e, &scope).map_err(at_line)?,
            _ => {}
        }
        let value = |v: &mut PropValue| -> Result<(), String> {
            match v {
                PropValue::Bind(e, _) => substitute(e, &scope),
                PropValue::Model(p) => {
                    let root = p.split('.').next().unwrap_or_default();
                    if scope.contains_key(root) {
                        Err(format!(
                            "bind:{{{p}}} writes {root}, which is derived; bind a store or row field"
                        ))
                    } else {
                        Ok(())
                    }
                }
                PropValue::Template(t) => {
                    let mut paths = Vec::new();
                    t.paths(&mut paths);
                    match paths
                        .iter()
                        .find(|p| scope.contains_key(p.split('.').next().unwrap_or_default()))
                    {
                        Some(p) => Err(format!("{p} is derived; a template reads stores")),
                        None => Ok(()),
                    }
                }
                PropValue::Lit(_) => Ok(()),
            }
        };
        for p in n
            .props
            .iter_mut()
            .chain(n.at.iter_mut().flat_map(|a| a.1.iter_mut()))
        {
            value(&mut p.value).map_err(at_line)?;
        }
        for rule in &mut n.rules {
            if let Some(c) = &mut rule.condition {
                substitute(c, &scope).map_err(at_line)?;
            }
            for p in &mut rule.props {
                value(&mut p.value).map_err(at_line)?;
            }
        }
        for (_, v, _) in &mut n.args {
            value(v).map_err(at_line)?;
        }
        for rule in &mut n.span_rules {
            if let Some(c) = &mut rule.condition {
                substitute(c, &scope).map_err(at_line)?;
            }
        }
        for g in &mut n.groups {
            if let Some(b) = &mut g.bind {
                substitute(b, &scope).map_err(at_line)?;
            }
        }
        for t in &mut n.triggers {
            if let When::Change(e) = &mut t.when {
                substitute(e, &scope).map_err(at_line)?;
            }
            if let Act::Set(_, e) = &mut t.act {
                substitute(e, &scope).map_err(at_line)?;
            }
        }
        for (_, e) in n.selector_attrs.iter_mut().chain(n.custom.iter_mut()) {
            substitute(e, &scope).map_err(at_line)?;
        }
        for run in &mut n.inline {
            substitute(&mut run.text, &scope).map_err(at_line)?;
        }
    }
    Ok(())
}

/// Statements at depth 0, comments removed: each with its offset.
fn statements(text: &str) -> Vec<(String, usize)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut start = None;
    let mut depth = 0i32;
    let mut i = 0;
    let flush = |cur: &mut String, start: &mut Option<usize>, out: &mut Vec<(String, usize)>| {
        let s = cur.trim().to_string();
        if !s.is_empty() {
            out.push((s, start.unwrap_or(0)));
        }
        cur.clear();
        *start = None;
    };
    while i < bytes.len() {
        let rest = &text[i..];
        if rest.starts_with("//") {
            i += rest.find('\n').unwrap_or(rest.len());
            continue;
        }
        if rest.starts_with("/*") {
            i += rest.find("*/").map_or(rest.len(), |e| e + 2);
            continue;
        }
        let c = rest.chars().next().unwrap();
        if matches!(c, '"' | '\'' | '`') {
            let mut j = 1;
            while let Some(d) = rest[j..].chars().next() {
                j += d.len_utf8();
                if d == '\\' {
                    j += rest[j..].chars().next().map_or(0, char::len_utf8);
                } else if d == c {
                    break;
                }
            }
            start.get_or_insert(i);
            cur.push_str(&rest[..j]);
            i += j;
            continue;
        }
        match c {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            _ => {}
        }
        if depth == 0 && c == ';' {
            flush(&mut cur, &mut start, &mut out);
        } else if depth == 0 && c == '\n' {
            // A line ends a statement unless it is plainly continued.
            let before = cur.trim_end();
            let after = text[i + 1..].trim_start();
            let continued = before.ends_with([
                '=', ',', '(', '+', '-', '*', '/', '?', ':', '&', '|', '>', '<',
            ]) || after
                .starts_with(['.', '?', ':', '+', '*', '/', '&', '|', '=', ')'])
                || before.is_empty();
            if continued {
                cur.push(' ');
            } else {
                flush(&mut cur, &mut start, &mut out);
            }
        } else {
            if !c.is_whitespace() {
                start.get_or_insert(i);
            }
            cur.push(c);
        }
        i += c.len_utf8();
    }
    flush(&mut cur, &mut start, &mut out);
    out
}

/// `a = b` split at its first top-level `=` that is not `==`, `=>`, `<=`…
fn split_assign(s: &str) -> Option<(&str, &str)> {
    let b = s.as_bytes();
    let mut depth = 0;
    let mut quote = None;
    for (i, &c) in b.iter().enumerate() {
        if let Some(q) = quote {
            if c == q && b[i - 1] != b'\\' {
                quote = None;
            }
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => quote = Some(c),
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => depth -= 1,
            b'=' if depth == 0 => {
                let prev = i.checked_sub(1).map(|k| b[k]);
                let next = b.get(i + 1).copied();
                if !matches!(prev, Some(b'=' | b'!' | b'<' | b'>'))
                    && !matches!(next, Some(b'=' | b'>'))
                {
                    return Some((&s[..i], &s[i + 1..]));
                }
            }
            _ => {}
        }
    }
    None
}

fn split_top(s: &str, sep: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0;
    let mut quote = None;
    let mut last = 0;
    for (i, c) in s.char_indices() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' | '`' => quote = Some(c),
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            c if c == sep && depth == 0 => {
                out.push(&s[last..i]);
                last = i + c.len_utf8();
            }
            _ => {}
        }
    }
    out.push(&s[last..]);
    out
}

/// `{ a } : Props` or `name: number` without its type annotation.
fn strip_type(s: &str) -> &str {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            ':' if depth == 0 => return s[..i].trim(),
            _ => {}
        }
    }
    s.trim()
}

/// A JavaScript literal (numbers, strings in either quote, booleans,
/// null, arrays and objects of them, keys bare or quoted) as JSON.
pub(super) fn literal(s: &str) -> Option<serde_json::Value> {
    fn value(s: &str) -> Option<serde_json::Value> {
        let s = s.trim();
        match s {
            "true" => return Some(true.into()),
            "false" => return Some(false.into()),
            "null" | "undefined" => return Some(serde_json::Value::Null),
            _ => {}
        }
        if let Some(q @ ('"' | '\'' | '`')) = s.chars().next() {
            let inner = s.strip_prefix(q)?.strip_suffix(q)?;
            if q == '`' && inner.contains("${") {
                return None;
            }
            return Some(inner.replace(&format!("\\{q}"), &q.to_string()).into());
        }
        if let Some(inner) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            return split_top(inner, ',')
                .into_iter()
                .filter(|e| !e.trim().is_empty())
                .map(value)
                .collect::<Option<Vec<_>>>()
                .map(serde_json::Value::Array);
        }
        if let Some(inner) = s.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            let mut map = serde_json::Map::new();
            for e in split_top(inner, ',')
                .into_iter()
                .filter(|e| !e.trim().is_empty())
            {
                let parts = split_top(e, ':');
                let [k, v] = parts[..] else { return None };
                let k = k.trim();
                let k = match value(k) {
                    Some(serde_json::Value::String(k)) => k,
                    _ if identifier(k) => k.to_string(),
                    _ => return None,
                };
                map.insert(k, value(v)?);
            }
            return Some(serde_json::Value::Object(map));
        }
        let n: f64 = s.replace('_', "").parse().ok()?;
        serde_json::Number::from_f64(n).map(|n| {
            if n.as_f64()
                .is_some_and(|f| f.fract() == 0.0 && f.abs() < 9e15)
            {
                serde_json::Value::from(n.as_f64().unwrap() as i64)
            } else {
                serde_json::Value::Number(n)
            }
        })
    }
    value(s)
}
