//! Scene write-back (`save_scene`, `docs/design.md` §3.4): a scene's `.ogt`
//! text rewritten line by line from the values a world holds now.
//!
//! The text is the source of truth for everything but values: every line
//! keeps its place, its indentation and its trailing `# comment`; a field
//! line is rewritten only when the value it reads as (after the f32
//! rounding the cook applies) differs from the current one. A tuned field
//! the text does not list gets a line before its component's closing `}`
//! (or a new component block before its entity's). The result therefore
//! cooks to the world it was written from, and writing it again changes
//! nothing.

/// A field's current value, as the scene text would give it.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    /// Numbers, one per lane (a vector's `x y z`, a scalar's one).
    Lanes(Vec<f64>),
    /// A flag: `true`/`false`.
    Bool(bool),
    /// A string field: one quoted string.
    Text(String),
}

/// One tuned field: `(entity, component, field)` as the scene names them.
pub type Tuned = (String, String, String);

/// What [`rewrite`] made.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rewrite {
    pub text: String,
    /// The 1-based lines of the new text that changed or were added.
    pub lines: Vec<usize>,
    /// Tuned fields that were not written, each with the reason.
    pub skipped: Vec<String>,
}

enum Frame {
    Entity {
        name: String,
        indent: String,
    },
    Component {
        entity: String,
        name: String,
        indent: String,
    },
    Other,
}

/// Splits a line into its code and its comment (from the first `#` outside
/// a quoted string, with the whitespace before it).
fn split_comment(line: &str) -> (&str, &str) {
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '#' if !quoted => {
                let code = line[..i].trim_end();
                return (code, &line[code.len()..]);
            }
            _ => {}
        }
    }
    (line, "")
}

/// The tokens of a line's code: words, numbers and quoted strings (with
/// their quotes).
fn tokens(code: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = code.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                i += if bytes[i] == b'\\' { 2 } else { 1 };
            }
            i = (i + 1).min(bytes.len());
        } else {
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
        }
        out.push(&code[start..i]);
    }
    out
}

fn unquote(token: &str) -> Option<String> {
    let inner = token.strip_prefix('"')?.strip_suffix('"')?;
    let mut s = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                'n' => s.push('\n'),
                't' => s.push('\t'),
                c => s.push(c),
            }
        } else {
            s.push(c);
        }
    }
    Some(s)
}

fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A number as the shortest text that cooks back to the same f32 (`0` for
/// either zero).
pub fn number(x: f64) -> String {
    let f = x as f32;
    if f == 0.0 { "0".into() } else { format!("{f}") }
}

/// A token's value as one lane: a number, or `true`/`false` as 1/0.
fn lane(token: &str) -> Option<f32> {
    match token {
        "true" => Some(1.0),
        "false" => Some(0.0),
        t => t.parse::<f64>().ok().map(|x| x as f32),
    }
}

/// The value tokens of `value`, with the old tokens' spelling kept where
/// they read the same (a `true` stays a word).
fn spell(value: &FieldValue, old: &[&str]) -> String {
    match value {
        FieldValue::Text(s) => quote(s),
        FieldValue::Bool(b) => b.to_string(),
        FieldValue::Lanes(lanes) => lanes
            .iter()
            .enumerate()
            .map(|(i, x)| match old.get(i) {
                Some(&t) if lane(t) == Some(*x as f32) => t.to_string(),
                Some(&"true" | &"false") if *x == 0.0 || *x == 1.0 => (*x == 1.0).to_string(),
                _ => number(*x),
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// Whether the old tokens already read as `value`.
fn same(value: &FieldValue, old: &[&str]) -> bool {
    match value {
        FieldValue::Text(s) => old.len() == 1 && unquote(old[0]).as_deref() == Some(s),
        FieldValue::Bool(b) => old.len() == 1 && lane(old[0]) == Some(if *b { 1.0 } else { 0.0 }),
        FieldValue::Lanes(lanes) => {
            old.len() == lanes.len()
                && old
                    .iter()
                    .zip(lanes)
                    .all(|(t, x)| lane(t) == Some(*x as f32))
        }
    }
}

/// Rewrites a scene's text from `current(entity, component, field)` (the
/// value now, `None` for one the world does not have: the line is kept),
/// adding a line for each of `tuned` the text does not list. Only the body
/// (after `---`) is read; lines it cannot read as fields are kept as they
/// are.
pub fn rewrite(
    text: &str,
    current: &dyn Fn(&str, &str, &str) -> Option<FieldValue>,
    tuned: &[Tuned],
) -> Rewrite {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + tuned.len());
    let mut changed: Vec<usize> = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();
    // Per (entity, component): the fields the text lists.
    let mut listed: std::collections::BTreeSet<(String, String, String)> = Default::default();
    let mut components: std::collections::BTreeSet<(String, String)> = Default::default();
    let mut entities: std::collections::BTreeSet<String> = Default::default();
    let mut placed: std::collections::BTreeSet<usize> = Default::default();
    let mut body = !lines.iter().any(|l| l.trim() == "---");
    let indent_of = |l: &str| l[..l.len() - l.trim_start().len()].to_string();
    let push = |out: &mut Vec<String>, changed: &mut Vec<usize>, line: String, new: bool| {
        out.push(line);
        if new {
            changed.push(out.len());
        }
    };
    for raw in &lines {
        let line = *raw;
        if !body {
            body = line.trim() == "---";
            out.push(line.to_string());
            continue;
        }
        let (code, comment) = split_comment(line);
        let trimmed = code.trim();
        let indent = indent_of(line);
        if let Some(head) = trimmed.strip_suffix('{') {
            let t = tokens(head);
            let frame = match (stack.last(), t.as_slice()) {
                (None | Some(Frame::Entity { .. }), ["entity", name]) => match unquote(name) {
                    Some(name) => {
                        entities.insert(name.clone());
                        Frame::Entity {
                            name,
                            indent: indent.clone(),
                        }
                    }
                    None => Frame::Other,
                },
                (Some(Frame::Entity { name, .. }), [comp]) => {
                    components.insert((name.clone(), comp.to_string()));
                    Frame::Component {
                        entity: name.clone(),
                        name: comp.to_string(),
                        indent: indent.clone(),
                    }
                }
                _ => Frame::Other,
            };
            stack.push(frame);
            out.push(line.to_string());
            continue;
        }
        if trimmed == "}" {
            match stack.pop() {
                Some(Frame::Component {
                    entity,
                    name,
                    indent: ci,
                }) => {
                    // Tuned fields of this component the text lacks.
                    for (i, (e, c, f)) in tuned.iter().enumerate() {
                        if *e == entity
                            && *c == name
                            && !listed.contains(&(e.clone(), c.clone(), f.clone()))
                            && let Some(v) = current(e, c, f)
                        {
                            let l = format!("{ci}  {f} {}", spell(&v, &[]));
                            push(&mut out, &mut changed, l, true);
                            listed.insert((e.clone(), c.clone(), f.clone()));
                            placed.insert(i);
                        }
                    }
                }
                Some(Frame::Entity { name, indent: ei }) => {
                    // Tuned fields of components the entity lacks: one
                    // block per component, in the order first tuned.
                    let mut blocks: Vec<(String, Vec<(usize, String)>)> = Vec::new();
                    for (i, (e, c, f)) in tuned.iter().enumerate() {
                        if *e != name
                            || components.contains(&(e.clone(), c.clone()))
                            || placed.contains(&i)
                        {
                            continue;
                        }
                        match blocks.iter_mut().find(|b| b.0 == *c) {
                            Some(b) => b.1.push((i, f.clone())),
                            None => blocks.push((c.clone(), vec![(i, f.clone())])),
                        }
                    }
                    for (c, fields) in blocks {
                        let values: Vec<(usize, String, FieldValue)> = fields
                            .into_iter()
                            .filter_map(|(i, f)| current(&name, &c, &f).map(|v| (i, f, v)))
                            .collect();
                        if values.is_empty() {
                            continue;
                        }
                        push(&mut out, &mut changed, format!("{ei}  {c} {{"), true);
                        for (i, f, v) in values {
                            push(
                                &mut out,
                                &mut changed,
                                format!("{ei}    {f} {}", spell(&v, &[])),
                                true,
                            );
                            placed.insert(i);
                        }
                        push(&mut out, &mut changed, format!("{ei}  }}"), true);
                        components.insert((name.clone(), c));
                    }
                }
                _ => {}
            }
            out.push(line.to_string());
            continue;
        }
        if let Some(Frame::Component { entity, name, .. }) = stack.last() {
            let t = tokens(trimmed);
            if let Some((key, old)) = t.split_first() {
                listed.insert((entity.clone(), name.clone(), key.to_string()));
                if let Some(v) = current(entity, name, key)
                    && !same(&v, old)
                {
                    let l = format!("{indent}{key} {}{comment}", spell(&v, old));
                    push(&mut out, &mut changed, l, true);
                    continue;
                }
            }
        }
        out.push(line.to_string());
    }
    let mut skipped = Vec::new();
    for (i, (e, c, f)) in tuned.iter().enumerate() {
        if placed.contains(&i) || listed.contains(&(e.clone(), c.clone(), f.clone())) {
            continue;
        }
        skipped.push(if entities.contains(e) {
            format!("{e}.{c}.{f}: the world has no such field")
        } else {
            format!("{e}.{c}.{f}: '{e}' is not an entity of the scene")
        });
    }
    Rewrite {
        text: out.join("\n"),
        lines: changed,
        skipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCENE: &str = "type scene
version 1
---
# the level
name \"Main\"
entity \"ship\" {
  model \"models/ship.glb\"   # the hull
  Transform {
    position 0 1.5 0 # spawn
    quaternion 0 0 0 1
    scale 1 1 1
  }
  entity \"light\" {
    Transform {
      position 0 3 0
    }
    Light {
      on true
      label \"key # light\"
    }
  }
}
";

    fn world(
        changes: &[(&str, &str, &str, FieldValue)],
    ) -> impl Fn(&str, &str, &str) -> Option<FieldValue> {
        let base: Vec<(String, String, String, FieldValue)> = vec![
            (
                "ship",
                "Transform",
                "position",
                FieldValue::Lanes(vec![0.0, 1.5, 0.0]),
            ),
            (
                "ship",
                "Transform",
                "quaternion",
                FieldValue::Lanes(vec![0.0, 0.0, 0.0, 1.0]),
            ),
            (
                "ship",
                "Transform",
                "scale",
                FieldValue::Lanes(vec![1.0, 1.0, 1.0]),
            ),
            (
                "light",
                "Transform",
                "position",
                FieldValue::Lanes(vec![0.0, 3.0, 0.0]),
            ),
            (
                "light",
                "Transform",
                "scale",
                FieldValue::Lanes(vec![1.0, 1.0, 1.0]),
            ),
            ("light", "Light", "on", FieldValue::Bool(true)),
            (
                "light",
                "Light",
                "label",
                FieldValue::Text("key # light".into()),
            ),
            ("light", "Light", "range", FieldValue::Lanes(vec![10.0])),
        ]
        .into_iter()
        .map(|(e, c, f, v)| (e.into(), c.into(), f.into(), v))
        .collect();
        let changes: Vec<(String, String, String, FieldValue)> = changes
            .iter()
            .map(|(e, c, f, v)| (e.to_string(), c.to_string(), f.to_string(), v.clone()))
            .collect();
        move |e, c, f| {
            changes
                .iter()
                .chain(base.iter())
                .find(|(a, b, d, _)| a == e && b == c && d == f)
                .map(|x| x.3.clone())
        }
    }

    #[test]
    fn unchanged_world_rewrites_nothing() {
        let r = rewrite(SCENE, &world(&[]), &[]);
        assert_eq!(r.text, SCENE);
        assert!(r.lines.is_empty());
    }

    #[test]
    fn values_rounding_to_the_same_f32_keep_their_line() {
        // 1.5000000001 is 1.5 as an f32: the line is not rewritten.
        let r = rewrite(
            SCENE,
            &world(&[(
                "ship",
                "Transform",
                "position",
                FieldValue::Lanes(vec![0.0, 1.500_000_000_1, -0.0]),
            )]),
            &[],
        );
        assert_eq!(r.text, SCENE);
    }

    #[test]
    fn changed_fields_rewrite_their_line_only_with_comment_and_spelling() {
        let r = rewrite(
            SCENE,
            &world(&[
                (
                    "ship",
                    "Transform",
                    "position",
                    FieldValue::Lanes(vec![-2.25, 1.5, 0.1]),
                ),
                ("light", "Light", "on", FieldValue::Bool(false)),
                (
                    "light",
                    "Light",
                    "label",
                    FieldValue::Text("fill \"b\"".into()),
                ),
            ]),
            &[],
        );
        let want = SCENE
            .replace("position 0 1.5 0 # spawn", "position -2.25 1.5 0.1 # spawn")
            .replace("on true", "on false")
            .replace("label \"key # light\"", "label \"fill \\\"b\\\"\"");
        assert_eq!(r.text, want);
        assert_eq!(r.lines, vec![9, 18, 19]);
    }

    #[test]
    fn tuned_fields_the_text_lacks_are_added_in_place() {
        let tuned: Vec<Tuned> = [
            ("light", "Transform", "scale"),
            ("light", "Light", "range"),
            ("ship", "Light", "range"),
            ("ghost", "Transform", "scale"),
        ]
        .iter()
        .map(|(e, c, f)| (e.to_string(), c.to_string(), f.to_string()))
        .collect();
        let r = rewrite(
            SCENE,
            &world(&[("ship", "Light", "range", FieldValue::Lanes(vec![2.5]))]),
            &tuned,
        );
        let want = SCENE
            .replace(
                "      position 0 3 0\n    }",
                "      position 0 3 0\n      scale 1 1 1\n    }",
            )
            .replace(
                "      label \"key # light\"\n    }",
                "      label \"key # light\"\n      range 10\n    }",
            )
            .replace(
                "    }\n  }\n}\n",
                "    }\n  }\n  Light {\n    range 2.5\n  }\n}\n",
            );
        assert_eq!(r.text, want);
        assert_eq!(r.lines, vec![16, 21, 24, 25, 26]);
        assert_eq!(
            r.skipped,
            vec!["ghost.Transform.scale: 'ghost' is not an entity of the scene"]
        );
        // Written again from the same world: nothing moves.
        let again = rewrite(
            &r.text,
            &world(&[("ship", "Light", "range", FieldValue::Lanes(vec![2.5]))]),
            &tuned,
        );
        assert_eq!(again.text, r.text);
        assert!(again.lines.is_empty());
    }

    #[test]
    fn numbers_are_the_shortest_f32_text() {
        assert_eq!(number(0.823_456_7), "0.8234567");
        assert_eq!(number(-0.0), "0");
        assert_eq!(number(2.0), "2");
        assert_eq!(number(0.1_f32 as f64), "0.1");
    }
}
